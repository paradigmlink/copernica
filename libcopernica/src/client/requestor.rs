// @implement: listen_for_requests
use {
    crate::{
        packets::{Packet as CopernicaPacket, mk_request_packet},
        sdri::{Sdri},
        response_store::{Response},
    },
    bincode::{serialize, deserialize},
    std::{
        net::{SocketAddr},
        sync::{Arc, RwLock},
        time::{Duration},
        collections::{HashMap, BTreeMap},
        thread,
        path::Path,
    },
    crossbeam_channel::{
            unbounded,
            Sender,
            Receiver,
            select,
            after,
            never
    },
    log::{trace},
    laminar::{
        Packet as LaminarPacket, Socket, SocketEvent
    },
};

#[derive(Clone)]
pub struct CopernicaRequestor {
    remote_addr: SocketAddr,
    sender: Option<Sender<LaminarPacket>>,
    receiver: Option<Receiver<SocketEvent>>,
    sdri_binding_to_packet: HashMap<Sdri, BTreeMap<u64, CopernicaPacket>>,
    sdri_binding_to_name: HashMap<Sdri, String>,}

impl CopernicaRequestor {
    pub fn new(remote_addr: String) -> CopernicaRequestor {
        CopernicaRequestor {
            remote_addr: remote_addr.parse().unwrap(),
            sender: None,
            receiver: None,
            sdri_binding_to_packet: HashMap::new(),
            sdri_binding_to_name: HashMap::new(),
        }
    }
    pub fn start_polling(&mut self) {
        let mut socket = Socket::bind_any().unwrap();
        let (sender, receiver) = (socket.get_packet_sender(), socket.get_event_receiver());
        self.sender = Some(sender.clone());
        self.receiver = Some(receiver.clone());
        thread::spawn(move || socket.start_polling());
    }

    pub fn request(&mut self, name: String, timeout: u64) -> Response {
        let response: Arc<RwLock<BTreeMap<u64, CopernicaPacket>>> = Arc::new(RwLock::new(BTreeMap::new()));
        let response_write_ref = response.clone();
        let response_read_ref  = response.clone();
        let expected_sdri = Sdri::new(name.clone());
        if let Some(sender) =  &self.sender {
                let sender = sender.clone();
                let packet = serialize(&mk_request_packet(name.clone())).unwrap();
                let packet = LaminarPacket::reliable_unordered(self.remote_addr, packet);
                //let packet = LaminarPacket::unreliable(self.remote_addr, packet);
                sender.send(packet).unwrap()
        }
        let (completed_s, completed_r) = unbounded();
        if let Some(receiver) = &self.receiver {
            let receiver = receiver.clone();
            thread::spawn(move || {
                loop {
                    let packet: SocketEvent = receiver.recv().unwrap();
                    match packet {
                        SocketEvent::Packet(packet) => {
                            let packet: CopernicaPacket = deserialize(&packet.payload()).unwrap();
                            match packet.clone() {
                                CopernicaPacket::Request { sdri } => {
                                    trace!("REQUEST ARRIVED: {:?}", sdri);
                                    continue
                                },
                                CopernicaPacket::Response { sdri, numerator, denominator, .. } => {
                                    trace!("RESPONSE ARRIVED: {:?} {}/{}", sdri, numerator, denominator-1);
                                    if expected_sdri == sdri {
                                        let mut response_guard = response_write_ref.write().unwrap();
                                        response_guard.insert(numerator, packet);
                                    }
                                    if numerator == denominator - 1 {
                                        completed_s.send(true).unwrap();
                                        break
                                    }
                                    // @missing: need a self.looking_for so valid responses are not thrown away
                                },
                            }
                        }
                        SocketEvent::Timeout(address) => {
                            trace!("Client timed out: {}", address);
                        }
                        SocketEvent::Connect(address) => {
                            trace!("New connection from: {:?}", address);
                        }
                    }
                }
            });
        }  // end loop
        let duration = Some(Duration::from_millis(timeout));
        let timeout = duration.map(|d| after(d)).unwrap_or_else(never);
        select! {
            recv(completed_r) -> _msg => {trace!("COMPLETED") },
            recv(timeout) -> _ => { println!("TIME OUT") },
        };
        let response_guard = response_read_ref.read().unwrap();
        let mut result: BTreeMap<u64, CopernicaPacket> = BTreeMap::new();
        for (count, packet) in response_guard.iter() {
            result.insert(*count as u64, packet.clone());
        }
        Response::from_name_and_btreemap(name, result)
    }
}

pub fn load_named_responses(dir: &Path) -> HashMap<String, CopernicaPacket> {
    let mut resps: HashMap<String, CopernicaPacket> = HashMap::new();
    for entry in std::fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            continue
        } else {
            let contents = std::fs::read(path.clone()).unwrap();
            let packet: CopernicaPacket = bincode::deserialize(&contents).unwrap();
            let name = &path.file_stem().unwrap();
            resps.insert(name.to_os_string().into_string().unwrap(), packet);
        }
    }
    resps
}

#[cfg(test)]
mod requestor {
    use super::*;

    #[test]
    fn test_polling() {
        let mut cr = CopernicaRequestor::new("127.0.0.1:8089".to_string());
        cr.start_polling();
        cr.request("hello0".to_string(), 100);
    }
}

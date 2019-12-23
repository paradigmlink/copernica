// @implement: listen_for_requests
use {
    crate::{
        packets::{NarrowWaist, mk_request_packet},
        sdri::{Sdri},
        response_store::{Response, ResponseStore},
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
    response_store: Arc<RwLock<ResponseStore>>
}

impl CopernicaRequestor {
    pub fn new(remote_addr: String) -> CopernicaRequestor {
        CopernicaRequestor {
            remote_addr: remote_addr.parse().unwrap(),
            sender: None,
            receiver: None,
            response_store: Arc::new(RwLock::new(ResponseStore::new(1000))),
        }
    }
    pub fn start_polling(&mut self) {
        let mut socket = Socket::bind_any().unwrap();
        let (sender, receiver) = (socket.get_packet_sender(), socket.get_event_receiver());
        self.sender = Some(sender.clone());
        self.receiver = Some(receiver.clone());
        thread::spawn(move || socket.start_polling());
    }

    pub fn request(&mut self, name: String, timeout: u64) -> Option<Response> {
        let response: Arc<RwLock<BTreeMap<u64, NarrowWaist>>> = Arc::new(RwLock::new(BTreeMap::new()));
        let response_write_ref = self.response_store.clone();
        let response_read_ref  = self.response_store.clone();
        let expected_sdri_p1 = Sdri::new(name.clone());
        let expected_sdri_p2 = expected_sdri_p1.clone();
        if let Some(sender) =  &self.sender {
            let sender = sender.clone();
            let packet = serialize(&mk_request_packet(name.clone())).unwrap();
            let packet = LaminarPacket::unreliable(self.remote_addr, packet);
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
                            let packet: NarrowWaist = deserialize(&packet.payload()).unwrap();
                            match packet.clone() {
                                NarrowWaist::Request { sdri } => {
                                    trace!("REQUEST ARRIVED: {:?}", sdri);
                                    continue
                                },
                                NarrowWaist::Response { sdri, count, total, .. } => {
                                    trace!("RESPONSE PACKET ARRIVED: {:?} {}/{}", sdri, count+1, total);
                                    if expected_sdri_p1 == sdri {
                                        let mut response_guard = response_write_ref.write().unwrap();
                                        response_guard.insert_packet(packet);
                                    }
                                    if count == total - 1 {
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
        response_guard.get_response(&expected_sdri_p2)
    }
}

pub fn load_named_responses(dir: &Path) -> HashMap<String, NarrowWaist> {
    let mut resps: HashMap<String, NarrowWaist> = HashMap::new();
    for entry in std::fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            continue
        } else {
            let contents = std::fs::read(path.clone()).unwrap();
            let packet: NarrowWaist = bincode::deserialize(&contents).unwrap();
            let name = &path.file_stem().unwrap();
            resps.insert(name.to_os_string().into_string().unwrap(), packet);
        }
    }
    resps
}

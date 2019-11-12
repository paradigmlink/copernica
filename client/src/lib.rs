// @implement: listen_for_requests
use {
    packets::{Packet as CopernicaPacket, Sdri, Data, generate_sdr_index, response, request},
    bincode::{serialize, deserialize},
    std::{
        net::{SocketAddr},
        sync::{Arc, Mutex},
        time::{Duration, Instant},
        collections::{HashMap as StdHashMap},
        thread,
    },
    crossbeam_channel::{
            Sender,
            Receiver,
            unbounded,
            select,
            after,
            never
    },
    log::{trace},
    im::{HashMap},
    laminar::{
        ErrorKind, Packet as LaminarPacket, Socket, SocketEvent
    },
};

#[derive(Clone)]
pub struct CopernicaRequestor {
    remote_addr: SocketAddr,
    sdri_binding_to_packet: Arc<Mutex<HashMap<Sdri, CopernicaPacket>>>,
    sdri_binding_to_name: Arc<Mutex<HashMap<Sdri, String>>>,}

impl CopernicaRequestor {
    pub fn new(remote_addr: String) -> CopernicaRequestor {
        CopernicaRequestor {
            remote_addr: remote_addr.parse().unwrap(),
            sdri_binding_to_packet: Arc::new(Mutex::new(HashMap::new())),
            sdri_binding_to_name: Arc::new(Mutex::new(HashMap::new())),
        }
    }
/*    pub fn listen_for_requests(&mut self, names: Vec<String>) -> Receiver<String> {
        for name in names {
            self.listen_for.lock().unwrap().insert(generate_sdr_index(name.clone()), name);
        }
        self.inbound_request_receiver.clone()
    }
    */
    pub fn request_many(&mut self, names: Vec<String>) -> StdHashMap<String, Option<CopernicaPacket>> {
        let mut look_for_these: Arc<Mutex<HashMap<Sdri, String>>> = Arc::new(Mutex::new(HashMap::new()));
        let mut results: Arc<Mutex<StdHashMap<String, Option<CopernicaPacket>>>> = Arc::new(Mutex::new(StdHashMap::new()));
        let sdri_binding_to_packet_phase1_ref = self.sdri_binding_to_packet.clone();
        let sdri_binding_to_packet_phase2_ref = self.sdri_binding_to_packet.clone();
        let sdri_binding_to_name_phase2_ref = self.sdri_binding_to_name.clone();
        let results_phase1_ref = results.clone();
        let results_phase2_ref = results.clone();
        let results_phase3_ref = results.clone();
        let look_for_these_phase1_ref = look_for_these.clone();
        let look_for_these_phase2_ref = look_for_these.clone();
        let look_for_these_phase3_ref = look_for_these.clone();

        let res = crossbeam::scope(|scope| {
            println!("request many");
            for name in names {
                let sdri = generate_sdr_index(name.clone());
                let mut sdri_binding_to_packet_guard = sdri_binding_to_packet_phase1_ref.lock().unwrap();
                if let Some(p) = sdri_binding_to_packet_guard.get(&sdri) {
                    let mut results_guard = results_phase1_ref.lock().unwrap();
                    results_guard.insert(name.clone(), Some(p.clone()));
                } else {
                    let mut look_for_these_guard = look_for_these_phase1_ref.lock().unwrap();
                    look_for_these_guard.insert(sdri, name.clone());
                }
            }
            let mut socket = Socket::bind_any().unwrap();
            let (sender, receiver) = (socket.get_packet_sender(), socket.get_event_receiver());
            let poller_thread = scope.spawn(move |_| socket.start_polling());
            let (completed_s, completed_r) = unbounded();
            let sync_thread = scope.spawn(move |_| {
                let mut remove_from_look_for_these: usize = 0;
                let mut results_guard = results_phase2_ref.lock().unwrap();
                let mut look_for_these_guard = look_for_these_phase2_ref.lock().unwrap();
                let mut sdri_binding_to_packet_guard = sdri_binding_to_packet_phase2_ref.lock().unwrap();
                let mut sdri_binding_to_name_guard = sdri_binding_to_name_phase2_ref.lock().unwrap();
                for (index, name) in look_for_these_guard.iter() {
                    let packet = serialize(&request(name.clone())).unwrap();
                    let packet = LaminarPacket::reliable_unordered(self.remote_addr, packet);
                    sender.send(packet.clone());
                    println!("sending packet: {:?}", packet);
                    loop {
                        println!("before recv");
                        let packet: SocketEvent = receiver.recv().unwrap();
                        println!("after recv");
                        trace!("got packet: {:?}", packet);
                        match packet {
                            SocketEvent::Packet(packet) => {
                                let packet: CopernicaPacket = deserialize(&packet.payload()).unwrap();
                                match packet.clone() {
                                    CopernicaPacket::Request { sdri } => {
                                        trace!("got a request: {:?}", sdri);
                                        continue
                                    },
                                    CopernicaPacket::Response { sdri, data } => {
                                        if look_for_these_guard.contains_key(&sdri) {
                                            remove_from_look_for_these += 1;
                                            sdri_binding_to_packet_guard.insert(sdri.clone(), packet.clone());
                                            sdri_binding_to_name_guard.insert(sdri.clone(), name.clone());
                                            results_guard.insert(name.to_string(), Some(packet));
                                        }
                                        // @missing: need a self.looking_for so valid responses are not thrown away
                                    },
                                }
                            }
                            SocketEvent::Timeout(address) => {
                                println!("Client timed out: {}", address);
                            }
                            SocketEvent::Connect(address) => {
                                trace!("New connection from: {:?}", address);
                            }
                        }
                        trace!("before breakout");
                        if look_for_these_guard.len() == remove_from_look_for_these {
                            completed_s.send(results_guard.clone()).unwrap();
                            trace!("length of looking 1: {} 2: {}", look_for_these_guard.len(), remove_from_look_for_these);
                            trace!("CONTENTS OF RESULTS: {:?}", results_guard);
                            break
                        }
                    }
                }
            });
            let duration = Some(Duration::from_millis(5000));
            let timeout = duration.map(|d| after(d)).unwrap_or(never());
            let res = select! {
                recv(completed_r) -> msg => {trace!("COMPLETED"); },
                recv(timeout) -> _ => {
                    println!("timed out");
                    let mut results_guard = results_phase3_ref.lock().unwrap();
                    let mut look_for_these_guard = look_for_these_phase3_ref.lock().unwrap();
                    for (sdri, name) in look_for_these_guard.iter() {
                        results_guard.insert(name.to_string(), None);
                    }
                },
            };
            res
        }).unwrap();

        let mut results_guard = results_phase3_ref.lock().unwrap();
        let mut res : StdHashMap<String, Option<CopernicaPacket>> = StdHashMap::new();
        for (name, packet) in results_guard.iter() {
            res.insert(name.to_string(), packet.clone());
        }
        res
    }
}

#[cfg(test)]
mod client {
    use {
        super::*,
    };
    #[test]
    fn laminar_setup() {
        let mut cr = CopernicaRequestor::new("127.0.0.1:8091".into());
        let actual = cr.request_many(vec!["hello1".to_string()]);
        trace!("I'M OUT {:?}", response1);
        let expected: StdHashMap<String, Option<CoperniaPacket>> = StdHashMap::new();
        expected.insert("hello1".to_string(), Some(response("hello1".to_string(),"world".to_string().as_bytes().to_vec())));
        assert!(keys_match(actual, expected));
    }
}

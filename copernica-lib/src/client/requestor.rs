// @implement: listen_for_requests
use {
    packets::{Packet as CopernicaPacket, Sdri, Data, generate_sdr_index, request},
    bincode::{serialize, deserialize},
    std::{
        net::{SocketAddr},
        sync::{Arc, Mutex, RwLock},
        time::{Duration},
        collections::{HashMap as StdHashMap},
        thread,
        path::Path,
    },
    crossbeam_channel::{
            unbounded,
            select,
            after,
            never
    },
    log::{trace},
    im::{HashMap},
    laminar::{
        Packet as LaminarPacket, Socket, SocketEvent
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
    pub fn request(&mut self, names: Vec<String>, timeout: u64) -> StdHashMap<String, Option<CopernicaPacket>> {
        let look_for_these: Arc<RwLock<HashMap<Sdri, String>>> = Arc::new(RwLock::new(HashMap::new()));
        let mut results : StdHashMap<String, Option<CopernicaPacket>> = StdHashMap::new();
        let found: Arc<RwLock<StdHashMap<String, Option<CopernicaPacket>>>> = Arc::new(RwLock::new(StdHashMap::new()));
        let sdri_binding_to_packet_phase1_ref = self.sdri_binding_to_packet.clone();
        let sdri_binding_to_packet_phase2_ref = self.sdri_binding_to_packet.clone();
        let sdri_binding_to_name_phase2_ref = self.sdri_binding_to_name.clone();
        let look_for_these_phase1_ref = look_for_these.clone();
        let look_for_these_phase2_ref = look_for_these.clone();
        let look_for_these_phase3_ref = look_for_these;
        let found_phase2_ref = found.clone();
        let found_phase3_ref = found;
        let mut socket = Socket::bind_any().unwrap();
        let (sender, receiver) = (socket.get_packet_sender(), socket.get_event_receiver());
        thread::spawn(move || socket.start_polling());
        for name in names {
            let sdri = generate_sdr_index(name.clone());
            let sdri_binding_to_packet_guard = sdri_binding_to_packet_phase1_ref.lock().unwrap();
            if let Some(p) = sdri_binding_to_packet_guard.get(&sdri) {
                results.insert(name.clone(), Some(p.clone()));
            } else {
                let mut look_for_these_guard = look_for_these_phase1_ref.write().unwrap();
                look_for_these_guard.insert(sdri, name.clone());
                let packet = serialize(&request(name.clone())).unwrap();
                let packet = LaminarPacket::reliable_unordered(self.remote_addr, packet);
                sender.send(packet.clone()).unwrap();
            }
        }
        let (completed_s, completed_r) = unbounded();
        thread::spawn(move || {
            let look_for_these_guard = look_for_these_phase2_ref.read().unwrap();
            let mut sdri_binding_to_packet_guard = sdri_binding_to_packet_phase2_ref.lock().unwrap();
            let mut sdri_binding_to_name_guard = sdri_binding_to_name_phase2_ref.lock().unwrap();
            loop {
                let packet: SocketEvent = receiver.recv().unwrap();
                match packet {
                    SocketEvent::Packet(packet) => {
                        let mut found_guard = found_phase2_ref.write().unwrap();
                        let packet: CopernicaPacket = deserialize(&packet.payload()).unwrap();
                        match packet.clone() {
                            CopernicaPacket::Request { sdri } => {
                                trace!("REQUEST ARRIVED: {:?}", sdri);
                                continue
                            },
                            CopernicaPacket::Response { sdri, .. } => {
                                trace!("RESPONSE ARRIVED: {:?}", sdri);
                                if let Some(name)= look_for_these_guard.get(&sdri) {
                                    sdri_binding_to_packet_guard.insert(sdri.clone(), packet.clone());
                                    sdri_binding_to_name_guard.insert(sdri.clone(), name.clone());
                                    found_guard.insert(name.to_string(), Some(packet));
                                }
                                // @missing: need a self.looking_for so valid responses are not thrown away
                            },
                        }
                        if look_for_these_guard.len() == found_guard.len() {
                            completed_s.send(true).unwrap();
                            break
                        }
                    }
                    SocketEvent::Timeout(address) => {
                        trace!("Client timed out: {}", address);
                    }
                    SocketEvent::Connect(address) => {
                        trace!("New connection from: {:?}", address);
                    }
                }
            } // end loop
        });
        let duration = Some(Duration::from_millis(timeout));
        let timeout = duration.map(|d| after(d)).unwrap_or_else(never);
        select! {
            recv(completed_r) -> _msg => {trace!("COMPLETED") },
            recv(timeout) -> _ => { trace!("TIME OUT") },
        };
        let found_guard = found_phase3_ref.read().unwrap();
        let look_for_these_guard = look_for_these_phase3_ref.read().unwrap();
        for (_sdri, name) in look_for_these_guard.iter() {
            if let Some(packet) = found_guard.get(name) {
                results.insert(name.to_string(), packet.clone());
            } else {
                results.insert(name.to_string(), None);
            }
        }
        results
    }

/*    pub fn resolve(name: String, timeout: u64) -> Data {
        let actual = self.request(vec![name]);
        match bincode::deserialize(&actual.1) {
            Packet::Response { sdri, data } => {
            }
        }
        let out
    }
*/
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


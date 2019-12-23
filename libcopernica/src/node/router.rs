use {
    crate::{
        node::{
            faces::{Face},
        },
        narrow_waist::{NarrowWaist},
        transport::{TransportPacket, InterFace},
        serdeser::{serialize, deserialize},
        response_store::{Response, ResponseStore},
        sdri::{Sdri},
    },
    bincode,
    laminar::{Packet as LaminarPacket, Socket, SocketEvent},
    log::{trace},
    rand::Rng,
    std::{
        path::{
            Path,
            PathBuf,
        },
        net::SocketAddr,
        collections::{HashMap, HashSet},
        fs,
        error::Error,
        io::BufReader,
    },
    serde_derive::Deserialize,
};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub listen_addr: SocketAddr,
    pub content_store_size: u64,
    pub peers: Option<Vec<String>>,
    pub data_dir: String,
}

impl Config {
    pub fn new() -> Config {
        let mut data_dir = dirs::home_dir().unwrap();
        data_dir.push(".copernica");
        Config {
            listen_addr: "127.0.0.1:8089".parse().unwrap(),
            content_store_size: 500,
            peers: Some(vec!["127.0.0.1:8090".into()]),
            data_dir: data_dir.to_string_lossy().to_string(),
        }
    }
}


pub fn read_config_file<P: AsRef<Path>>(path: P) -> Result<Config, Box<dyn Error>> {
    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);
    let confs= serde_json::from_reader(reader)?;
    Ok(confs)
}

#[derive(Clone)]
pub struct Router {
    listen_addr: SocketAddr,
    faces: HashMap<InterFace, Face>,
    id: u8,
    response_store:  ResponseStore,
}

impl Router {
    pub fn new() -> Router {
        let config: Config = Config::new();
        Router {
            listen_addr: config.listen_addr,
            faces: HashMap::new(),
            id: rand::thread_rng().gen_range(0, 255),
            response_store:  ResponseStore::new(config.content_store_size),
        }
    }

    pub fn new_with_config(config: Config) -> Router {
        let id = rand::thread_rng().gen_range(0,255);
        let mut faces: HashMap<InterFace, Face> = HashMap::new();
        if let Some(peer_addresses) = config.peers {
            for address in peer_addresses {
                trace!("[SETUP] router {}: adding peer: {:?}", id, address);
                let socket_addr: SocketAddr = address.parse().unwrap();
                let inter_face: InterFace = InterFace::SocketAddr(socket_addr);
                faces.insert(inter_face, Face::new());
            }
        }
        let mut response_store = ResponseStore::new(config.content_store_size);
        let content_store: PathBuf = [config.data_dir.clone()].iter().collect();
        let identity: PathBuf = [config.data_dir.clone(), "identity".to_string()].iter().collect();
        let trusted_connections: PathBuf = [config.data_dir.clone(), "trusted_connections".to_string()].iter().collect();
        let cs_dirs: Vec<PathBuf> = vec![content_store, identity, trusted_connections];
        for dir in cs_dirs {
            fs::create_dir_all(dir.clone()).unwrap();
            for entry in std::fs::read_dir(dir.clone()).expect("directory not found") {
                let entry = entry.unwrap();
                let path = entry.path();
                if path.is_dir() {
                    continue
                } else {
                    let contents = std::fs::read(path.clone()).expect("file not found");
                    let response: Response = bincode::deserialize(&contents).unwrap();
                    trace!("[SETUP] router {} using {:?}: adding to content store", id, dir);
                    response_store.insert_response(response);
                }
            }
        }
        Router {
            listen_addr: config.listen_addr,
            faces,
            id,
            response_store,
        }
    }

    pub fn run(&mut self) {
        trace!("listening on {:?}", self.listen_addr);
        let mut socket = Socket::bind(self.listen_addr).unwrap();
        let (sender, receiver) = (socket.get_packet_sender(), socket.get_event_receiver());
        let _thread = std::thread::spawn(move || socket.start_polling());
        let mut active_connections: HashSet<InterFace> = HashSet::new();
        loop {
            match receiver.recv() {
                Ok(event) => {
                    let mut handled_packets: Vec<LaminarPacket> = vec![];
                    match event {
                        SocketEvent::Packet(packet) => {
                            let transport_packet: TransportPacket = deserialize(&packet.payload());
                            if self.faces.contains_key(&transport_packet.clone().reply_to()) {
                                self.handle_packet(transport_packet.clone(), &mut handled_packets);
                                for p in handled_packets {
                                    sender.send(p).expect("Failed to send");
                                }
                            }
                        }
                        SocketEvent::Timeout(address) => {
                            trace!("Client timed out: {}", address);
                        }
                        SocketEvent::Connect(address) => {
                            let inter_face: InterFace = InterFace::SocketAddr(address);
                            if !active_connections.contains(&inter_face) {
                                trace!("Adding {:?} to faces", inter_face);
                                active_connections.insert(inter_face.clone());
                                self.faces.insert(inter_face, Face::new());
                            }
                        }
                    }
                },
                Err(event) => { panic!("Err {:?}", event) },
            }
        };
    }

    fn handle_packet(&mut self,  transport_packet: TransportPacket, handle_packets: &mut Vec<LaminarPacket>) {
        let thin_waist_packet: NarrowWaist = transport_packet.payload();
        let packet_from: InterFace = transport_packet.reply_to();
        if let Some(this_face) = self.faces.get_mut(&packet_from) {
            match thin_waist_packet.clone() {
                NarrowWaist::Request { sdri } => {
                    match self.response_store.get_response(&sdri) {
                        Some(response) => {
                            //this_face.create_pending_request(&sdri);
                            trace!("[RESUP] *** response found ***");
                            for (_seq, packet) in response.iter() {
                                prepare_packet(packet_from.clone(), self.listen_addr, packet.clone(), handle_packets);
                            }
                            return
                        },
                        None => {
                            let mut is_forwarded = false;
                            let mut broadcast = Vec::new();
                            this_face.create_pending_request(&sdri);
                            trace!("[REQDN {}] left breadcrumb pending request",
                                face_stats(self.id, "IN",  this_face, &sdri));
                            for (address, that_face) in self.faces.iter_mut() {
                                if *address == packet_from { continue }
                                if that_face.contains_forwarded_request(&sdri) > 51 {
                                    trace!("[REQDN {}] don't send request downstream again",
                                        face_stats(self.id, "OUT",  that_face, &sdri));
                                    continue
                                }
                                if that_face.contains_pending_request(&sdri)   > 51 {
                                    trace!("[REQDN {}] don't send request upstream",
                                        face_stats(self.id, "OUT",  that_face, &sdri));
                                    continue
                                }
                                if that_face.contains_forwarding_hint(&sdri)   > 90 {
                                    that_face.create_forwarded_request(&sdri);
                                    trace!("[REQDN {}] sending request downstream based on forwarding hint",
                                        face_stats(self.id, "OUT",  that_face, &sdri));
                                    prepare_packet(address.clone(), self.listen_addr, thin_waist_packet.clone(), handle_packets);
                                    is_forwarded = true;
                                    continue
                                }
                                broadcast.push(address.clone())

                            }
                            if !is_forwarded {
                                for address in broadcast {
                                    if let Some(face) = self.faces.get_mut(&address) {
                                        face.create_forwarded_request(&sdri);
                                        trace!("[REQDN {}] bursting on face",
                                            face_stats(self.id, "BURST",  face, &sdri));
                                        prepare_packet(address, self.listen_addr, thin_waist_packet.clone(), handle_packets);
                                    }
                                }
                            }
                        },
                    }
                },
                NarrowWaist::Response { sdri, count, total, .. } => {
                    if this_face.contains_forwarded_request(&sdri) > 15 {
                        trace!("[RESUP {}] response matched pending request",
                            face_stats(self.id, "IN",  this_face, &sdri));
                        self.response_store.insert_packet(thin_waist_packet.clone());
                        if this_face.forwarding_hint_decoherence() > 80 {
                            this_face.partially_forget_forwarding_hint();
                        }
                        if count == total - 1 {
                            this_face.delete_forwarded_request(&sdri);
                            this_face.create_forwarding_hint(&sdri);
                        }
                        for (address, that_face) in self.faces.iter_mut() {
                            if *address == packet_from { continue }
                            if that_face.contains_pending_request(&sdri) > 50 {
                                trace!("[RESUP {}] send response upstream",
                                    face_stats(self.id, "OUT",  that_face, &sdri));
                                // store-and-forward
                                if count == total - 1 {
                                    if let Some(response) = self.response_store.get_response(&sdri) {
                                        for (_seq, packet) in response.iter() {
                                            prepare_packet(address.clone(), self.listen_addr, packet.clone(), handle_packets);
                                        }
                                        that_face.delete_forwarded_request(&sdri);
                                        that_face.delete_pending_request(&sdri);
                                    }
                                }
                            }
                        }
                    }
                },
            }
        }
    }
}

fn prepare_packet(remote_addr: InterFace, local_addr: SocketAddr, packet: NarrowWaist, handle_packets: &mut Vec<LaminarPacket>) {
    match remote_addr {
        InterFace::SocketAddr(raddress) => {
            let reply_to: InterFace = InterFace::SocketAddr(local_addr);
            let transport_packet = TransportPacket::new(reply_to, packet);
            let laminar_packet = LaminarPacket::reliable_unordered(
                raddress, serialize(&transport_packet));
            handle_packets.push(laminar_packet);
        },
        InterFace::Sdr(_hz) => {},
    }
}
fn face_stats(router_id: u8, direction: &str, face: &mut Face, sdri: &Sdri) -> String {
    format!(
    "r{0:<3} f{1: <5} {2: <5} pr{3: <3}d{4: <3}fr{5: <3}d{6: <3}fh{7: <3}d{8: <0}",
        router_id,
        face.get_id(),
        direction,
        face.contains_pending_request(&sdri),
        face.pending_request_decoherence(),
        face.contains_forwarded_request(&sdri),
        face.forwarded_request_decoherence(),
        face.contains_forwarding_hint(&sdri),
        face.forwarding_hint_decoherence())
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

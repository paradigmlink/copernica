use {
    crate::{
        node::{
            faces::{Face},
        },
        narrow_waist::{NarrowWaist},
        transport::{
            TransportPacket, TransportResponse, ReplyTo,
            relay_transport_packet, send_transport_packet, send_transport_response, receive_transport_packet,
        },
        response_store::{Response, ResponseStore, Got},
        sdri::{Sdri},
    },
    bincode,
    log::{trace},
    crossbeam_channel::{Sender, unbounded},
    std::{
        path::{
            Path,
            PathBuf,
        },
        net::SocketAddr,
        collections::{HashMap},
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
    listen_addr: ReplyTo,
    faces: HashMap<ReplyTo, Face>,
    response_store:  ResponseStore,
}

impl Router {
    pub fn new() -> Router {
        let config: Config = Config::new();
        let listen_addr: ReplyTo = ReplyTo::Udp(config.listen_addr);
        Router {
            listen_addr: listen_addr,
            faces: HashMap::new(),
            response_store:  ResponseStore::new(config.content_store_size),
        }
    }

    pub fn new_with_config(config: Config) -> Router {
        let mut faces: HashMap<ReplyTo, Face> = HashMap::new();
        let listen_addr: ReplyTo = ReplyTo::Udp(config.listen_addr);
        if let Some(peer_addresses) = config.peers {
            for address in peer_addresses {
                trace!("[SETUP] router {:?}: adding peer: {:?}", listen_addr, address);
                let socket_addr: SocketAddr = address.parse().unwrap();
                let face_id: ReplyTo = ReplyTo::Udp(socket_addr);
                faces.insert(face_id.clone(), Face::new(face_id));
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
                    trace!("[SETUP] router {:?} using {:?}: adding to content store", listen_addr, dir);
                    response_store.insert_response(response);
                }
            }
        }
        Router {
            listen_addr,
            faces,
            response_store,
        }
    }

    pub async fn run(&mut self) {
        trace!("{:?} IS LISTENING", self.listen_addr);
        let listen_addr_1 = self.listen_addr.clone();
        let listen_addr_2 = self.listen_addr.clone();
        let listen_addr_3 = self.listen_addr.clone();
        let listen_addr_4 = self.listen_addr.clone();
        let (receive_tp_sender, receive_tp_receiver) = unbounded::<TransportPacket>();
        let (send_tr_sender, send_tr_receiver) = unbounded::<TransportResponse>();
        let (relay_tp_sender, relay_tp_receiver) = unbounded::<(ReplyTo, TransportPacket)>();
        let (send_tp_sender, send_tp_receiver) = unbounded::<TransportPacket>();
        std::thread::spawn(move || receive_transport_packet(listen_addr_1, receive_tp_sender));
        std::thread::spawn(move || send_transport_response(listen_addr_2, send_tr_receiver));
        std::thread::spawn(move || relay_transport_packet(listen_addr_3, relay_tp_receiver));
        std::thread::spawn(move || send_transport_packet(listen_addr_4, send_tp_receiver));
        loop {
            match receive_tp_receiver.recv() {
                Ok(tp) => {
                    let reply_to: ReplyTo = tp.reply_to();
                    if !self.faces.contains_key(&reply_to) {
                        trace!("ADDING {:?} to NODE {:?} FACES", reply_to, self.listen_addr.clone());
                        self.faces.insert(reply_to.clone(), Face::new(reply_to));
                    }
                    //all_faces_stats(&self.faces, &tp, &format!("ALL FACES STATS ON INBOUND for {:?}", self.listen_addr.clone()));
                    self.handle_packet(&tp, send_tr_sender.clone(), relay_tp_sender.clone(), send_tp_sender.clone()).await;
                    //all_faces_stats(&self.faces, &tp, &format!("ALL FACES STATS ON OUTBOUND for {:?}", self.listen_addr.clone()));
                },
                _ => {},
                //Err(error) => { println!("Transport Packet Receive Error {}", error) },
            }
        };
    }

    async fn handle_packet(&mut self, transport_packet: &TransportPacket,
            send_transport_response: Sender<TransportResponse>,
            relay_transport_packet: Sender<(ReplyTo, TransportPacket)>,
            send_transport_packet: Sender<TransportPacket>) {
        let thin_waist_packet: NarrowWaist = transport_packet.payload();
        let packet_from: ReplyTo = transport_packet.reply_to();
        if let Some(this_face) = self.faces.get_mut(&packet_from) {
            match thin_waist_packet.clone() {
                NarrowWaist::Request { sdri } => {
                    match self.response_store.get(&sdri).await {
                        Some(response) => {
                            match response {
                                Got::Response(response) => {
                                    let tr = TransportResponse::new(transport_packet.reply_to(), response);
                                    outbound_stats(&transport_packet, &self.listen_addr,
                                        this_face, "********* RESPONSE FOUND *********");
                                    send_transport_response.send(tr).unwrap();
                                },
                                Got::NarrowWaist(narrow_waist) => {
                                    let tp = TransportPacket::new(transport_packet.reply_to(), narrow_waist);
                                    outbound_stats(&transport_packet, &self.listen_addr,
                                        this_face, "********* RESPONSE PACKET FOUND *********");
                                    send_transport_packet.send(tp).unwrap();
                                },
                            }
                            return
                        },
                        None => {
                            let mut is_forwarded = false;
                            let mut broadcast = Vec::new();
                            this_face.create_pending_request(&sdri);
                            inbound_stats(&transport_packet, &self.listen_addr,
                                this_face, "Inserting pending request");
                            for (address, that_face) in self.faces.iter_mut() {
                                if *address == packet_from {
                                    continue
                                }
                                if that_face.contains_forwarded_request(&sdri) > 51 {
                                    outbound_stats(&transport_packet, &self.listen_addr,
                                        that_face, "Don't send request upstream again");
                                    continue
                                }
                                if that_face.contains_pending_request(&sdri)   > 51 {
                                    outbound_stats(&transport_packet, &self.listen_addr,
                                        that_face, "Don't send request downstream");
                                    continue
                                }
                                if that_face.contains_forwarding_hint(&sdri)   > 90 {
                                    that_face.create_forwarded_request(&sdri);
                                    outbound_stats(&transport_packet, &self.listen_addr,
                                        that_face, "Sending request downstream based on forwarding hint");
                                    relay_transport_packet.send((address.clone(),
                                        transport_packet.clone())).unwrap();
                                    is_forwarded = true;
                                    continue
                                }
                                broadcast.push(address.clone())

                            }
                            if !is_forwarded {
                                for address in broadcast {
                                    if let Some(burst_face) = self.faces.get_mut(&address.clone()) {
                                        burst_face.create_forwarded_request(&sdri);
                                        outbound_stats(&transport_packet, &self.listen_addr,
                                            burst_face, "Bursting on face");
                                        relay_transport_packet.send((address.clone(),
                                            transport_packet.clone())).unwrap();
                                    }
                                }
                            }
                        },
                    }
                },
                NarrowWaist::Response { sdri, .. } => {
                    if this_face.contains_forwarded_request(&sdri) > 15 {
                        self.response_store.insert_packet(thin_waist_packet.clone());
                        if this_face.forwarding_hint_decoherence() > 80 {
                            this_face.partially_forget_forwarding_hint();
                        }
                        if self.response_store.complete(&sdri) {
                            this_face.delete_forwarded_request(&sdri);
                            this_face.create_forwarding_hint(&sdri);
                        }
                        for (address, that_face) in self.faces.iter_mut() {
                            if *address == packet_from { continue }
                            if that_face.contains_pending_request(&sdri) > 50 {
                                outbound_stats(&transport_packet, &self.listen_addr,
                                    that_face, "Send response upstream");
                                relay_transport_packet.send((address.clone(),
                                        transport_packet.clone())).unwrap();
                            }
                        }
                    }
                }
            }
        }
    }
}

fn inbound_stats(packet: &TransportPacket, router_id: &ReplyTo, face: &Face, message: &str) {
    let print = format!(
        "INBOUND PACKET for {:?}\n\t{:?}\n\tFrom {:?} => To {:?}\n\t{}\n\t\t{}",
        router_id,
        packet,
        face.id(),
        router_id,
        face_stats(face, packet),
        message,
    );
    trace!("{}", print);
}

fn outbound_stats(packet: &TransportPacket, router_id: &ReplyTo, face: &Face, message: &str) {
    let print = format!(
        "OUTBOUND PACKET for {:?}\n\t{:?}\n\tFrom {:?} => To {:?}\n\t{}\n\t\t{}",
        router_id,
        packet,
        router_id,
        face.id(),
        face_stats(face, packet),
        message,
    );
    trace!("{}", print);
}

#[allow(dead_code)]
fn all_faces_stats(faces: &HashMap<ReplyTo, Face>, packet: &TransportPacket, message: &str) {
    let mut s: String = message.to_string();
    for (_address, face) in faces {
        s.push_str(&format!("\n\t"));
        s.push_str(&face_stats(face, packet));
    }
    trace!("{}",s);
}

fn face_stats(face: &Face, packet: &TransportPacket) -> String {
    let sdri: Sdri = match packet.payload() {
        NarrowWaist::Request{sdri} => sdri,
        NarrowWaist::Response{sdri,..} => sdri,
    };
    format!(
    "[pr{0: <3}d{1: <3}fr{2: <3}d{3: <3}fh{4: <3}d{5: <0}] faceid {6:?} sdri {7:?}",
        face.contains_pending_request(&sdri),
        face.pending_request_decoherence(),
        face.contains_forwarded_request(&sdri),
        face.forwarded_request_decoherence(),
        face.contains_forwarding_hint(&sdri),
        face.forwarding_hint_decoherence(),
        face.id(),
        sdri)
}


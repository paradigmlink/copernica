use {
    crate::{
        node::{
            faces::{Face},
        },
        narrow_waist::{NarrowWaist},
        transport::{
            TransportPacket, ReplyTo,
            relay_transport_packet, send_transport_packet, receive_transport_packet,
        },
        hbfi::{HBFI},
        borsh::{BorshDeserialize, BorshSerialize},
    },
    anyhow::{Result, anyhow},
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
    pub data_dir: PathBuf,
}

impl Config {
    pub fn new() -> Config {
        let mut data_dir = dirs::home_dir().unwrap();
        data_dir.push(".copernica");
        Config {
            listen_addr: "127.0.0.1:8089".parse().unwrap(),
            content_store_size: 500,
            peers: Some(vec!["127.0.0.1:8090".into()]),
            data_dir,
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
    data_dir: PathBuf,
}

impl Router {
    pub fn new() -> Result<Router> {
        let config: Config = Config::new();
        let listen_addr: ReplyTo = ReplyTo::Udp(config.listen_addr);
        let data_dir: PathBuf = [config.data_dir.clone()].iter().collect();
        Ok(Router {
            listen_addr: listen_addr,
            faces: HashMap::new(),
            data_dir,
        })
    }

    pub fn new_with_config(config: Config) -> Result<Router> {
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
        let data_dir: PathBuf = [config.data_dir.clone()].iter().collect();
        Ok(Router {
            listen_addr,
            faces,
            data_dir,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        trace!("{:?} IS LISTENING", self.listen_addr);
        let listen_addr_1 = self.listen_addr.clone();
        let listen_addr_2 = self.listen_addr.clone();
        let listen_addr_3 = self.listen_addr.clone();
        let (receive_tp_sender, receive_tp_receiver) = unbounded::<TransportPacket>();
        let (relay_tp_sender, relay_tp_receiver) = unbounded::<(ReplyTo, TransportPacket)>();
        let (send_tp_sender, send_tp_receiver) = unbounded::<TransportPacket>();
        let db = sled::open(self.data_dir.clone())?;
        std::thread::spawn(move || receive_transport_packet(listen_addr_1, receive_tp_sender));
        std::thread::spawn(move || relay_transport_packet(listen_addr_2, relay_tp_receiver));
        std::thread::spawn(move || send_transport_packet(listen_addr_3, send_tp_receiver));
        loop {
            match receive_tp_receiver.recv() {
                Ok(tp) => {
                    let reply_to: ReplyTo = tp.reply_to();
                    if !self.faces.contains_key(&reply_to) {
                        trace!("ADDING {:?} to NODE {:?} FACES", reply_to, self.listen_addr.clone());
                        self.faces.insert(reply_to.clone(), Face::new(reply_to));
                    }
                    //all_faces_stats(&self.faces, &tp, &format!("ALL FACES STATS ON INBOUND for {:?}", self.listen_addr.clone()));
                    self.handle_packet(&tp,
                        relay_tp_sender.clone(),
                        send_tp_sender.clone(),
                        db.clone(),
                        ).await?;
                    //all_faces_stats(&self.faces, &tp, &format!("ALL FACES STATS ON OUTBOUND for {:?}", self.listen_addr.clone()));
                },
                Err(error) => return Err(anyhow!("{}", error))
            }
        };
    }

    async fn handle_packet(&mut self, transport_packet: &TransportPacket,
            relay_transport_packet: Sender<(ReplyTo, TransportPacket)>,
            send_transport_packet: Sender<TransportPacket>,
            response_store: sled::Db,
            ) -> Result<()> {
        let narrow_waist_packet: NarrowWaist = transport_packet.payload();
        let packet_from: ReplyTo = transport_packet.reply_to();
        if let Some(this_face) = self.faces.get_mut(&packet_from) {
            match narrow_waist_packet.clone() {
                NarrowWaist::Request { hbfi } => {
                    match response_store.get(&hbfi.try_to_vec()?)? {
                        Some(response) => {
                            let narrow_waist = NarrowWaist::try_from_slice(&response)?;
                            outbound_stats(&transport_packet, &self.listen_addr,
                                this_face, "********* RESPONSE PACKET FOUND *********");
                            let tp = TransportPacket::new(transport_packet.reply_to(), narrow_waist);
                            send_transport_packet.send(tp).unwrap();
                            return Ok(())
                        },
                        None => {
                            let mut is_forwarded = false;
                            let mut broadcast = Vec::new();
                            this_face.create_pending_request(&hbfi);
                            inbound_stats(&transport_packet, &self.listen_addr,
                                this_face, "Inserting pending request");
                            for (address, that_face) in self.faces.iter_mut() {
                                if *address == packet_from {
                                    continue
                                }
                                if that_face.contains_forwarded_request(&hbfi) > 51 {
                                    outbound_stats(&transport_packet, &self.listen_addr,
                                        that_face, "Don't send request upstream again");
                                    continue
                                }
                                if that_face.contains_pending_request(&hbfi)   > 51 {
                                    outbound_stats(&transport_packet, &self.listen_addr,
                                        that_face, "Don't send request downstream");
                                    continue
                                }
                                if that_face.contains_forwarding_hint(&hbfi)   > 90 {
                                    that_face.create_forwarded_request(&hbfi);
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
                                        burst_face.create_forwarded_request(&hbfi);
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
                NarrowWaist::Response { hbfi, .. } => {
                    if this_face.contains_forwarded_request(&hbfi) > 15 {
                        response_store.insert(hbfi.try_to_vec()?, narrow_waist_packet.clone().try_to_vec()?)?;
                        if this_face.forwarding_hint_decoherence() > 80 {
                            this_face.partially_forget_forwarding_hint();
                        }
                        /*
                        if response_store.complete(&hbfi) {
                            this_face.delete_forwarded_request(&hbfi);
                            this_face.create_forwarding_hint(&hbfi);
                        }
                        */
                        for (address, that_face) in self.faces.iter_mut() {
                            if *address == packet_from { continue }
                            if that_face.contains_pending_request(&hbfi) > 50 {
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
        Ok(())
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
    let hbfi: HBFI = match packet.payload() {
        NarrowWaist::Request{hbfi} => hbfi,
        NarrowWaist::Response{hbfi,..} => hbfi,
    };
    format!(
    "[pr{0: <3}d{1: <3}fr{2: <3}d{3: <3}fh{4: <3}d{5: <0}] faceid {6:?} hbfi {7:?}",
        face.contains_pending_request(&hbfi),
        face.pending_request_decoherence(),
        face.contains_forwarded_request(&hbfi),
        face.forwarded_request_decoherence(),
        face.contains_forwarding_hint(&hbfi),
        face.forwarding_hint_decoherence(),
        face.id(),
        hbfi)
}


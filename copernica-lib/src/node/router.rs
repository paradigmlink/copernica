use {
    packets::{Packet as CopernicaPacket, response, Sdri},
    crate::{
        node::content_store::{ContentStore},
        node::faces::{Face},
    },
    bincode::{serialize, deserialize},
    laminar::{Packet as LaminarPacket, Socket, SocketEvent},
    log::{trace},
    rand::Rng,
    std::{
        net::SocketAddr,
        collections::{HashMap, HashSet},
    },
    serde_derive::Deserialize,
};

#[derive(Debug, PartialEq, Deserialize)]
pub struct NamedData {
    pub name: String,
    pub data: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub listen_addr: String,
    pub content_store_size: u64,
    pub peers: Option<Vec<String>>,
    pub data: Option<Vec<NamedData>>,
}

#[derive(Clone)]
pub struct Router {
    listen_addr: SocketAddr,
    faces: HashMap<SocketAddr, Face>,
    id: u8,
    cs:  ContentStore,
}

impl Router {
    pub fn new() -> Router {
        Router {
            listen_addr: "127.0.0.1:8089".parse().unwrap(),
            faces: HashMap::new(),
            id: rand::thread_rng().gen_range(0, 255),
            cs:  ContentStore::new(20),
        }
    }

    pub fn new_with_config(config: Config) -> Router {
        let mut faces: HashMap<SocketAddr, Face> = HashMap::new();
        if let Some(peer_addresses) = config.peers {
            for address in peer_addresses {
                trace!("[SETUP] adding peer: {:?}", address);
                let socket_addr: SocketAddr = address.parse().unwrap();
                faces.insert(socket_addr, Face::new(socket_addr.port()));
            }
        }
        let mut cs = ContentStore::new(config.content_store_size);
        if let Some(data) = config.data {
            for named_data in data {
                trace!("[SETUP] adding data: name: {} data: {}", named_data.name.to_string(), named_data.data.to_string());
                cs.put_data(response(named_data.name.to_string(), named_data.data.to_string().as_bytes().to_vec()));
            }
        }
        Router {
            listen_addr: config.listen_addr.parse().unwrap(),
            faces,
            id: rand::thread_rng().gen_range(0,255),
            cs,
        }
    }

    pub fn run(&mut self) {
        let mut socket = Socket::bind(self.listen_addr).unwrap();
        let (sender, receiver) = (socket.get_packet_sender(), socket.get_event_receiver());
        let _thread = std::thread::spawn(move || socket.start_polling());
        let mut active_connections: HashSet<SocketAddr> = HashSet::new();
        loop {
            if let Ok(event) = receiver.recv() {
                let mut handled_packets: Vec<LaminarPacket> = vec![];
                match event {
                    SocketEvent::Packet(packet) => {
                        if self.faces.contains_key(&packet.clone().addr()) {
                            self.handle_packet(packet.clone(), &mut handled_packets);
                            for p in handled_packets {
                                sender.send(p).expect("Failed to send");
                            }
                        }
                    }
                    SocketEvent::Timeout(address) => {
                        trace!("Client timed out: {}", address);
                    }
                    SocketEvent::Connect(address) => {
                        trace!("Adding {:?} to faces", address);
                        if !active_connections.contains(&address) {
                            active_connections.insert(address.clone());
                            self.faces.insert(address, Face::new(address.port()));
                        }
                    }
                }
            }
        };
    }

    fn handle_packet(&mut self,  laminar_packet: LaminarPacket, handle_packets: &mut Vec<LaminarPacket>) {
        let payload = laminar_packet.payload();
        let copernica_packet: CopernicaPacket = deserialize(&payload).unwrap();
        let packet_from: SocketAddr = laminar_packet.addr();
        if let Some(this_face) = self.faces.get_mut(&packet_from) {
            match copernica_packet.clone() {
                CopernicaPacket::Request { sdri } => {
                    match self.cs.has_data(&sdri) {
                        Some(data) => {
                            this_face.create_pending_request(&sdri);
                            trace!("[RESUP] *** response found *** {:?}", data);
                            handle_packets.push(mk_laminar_packet(packet_from, data));
                            return
                        },
                        None => {
                            let mut is_forwarded = false;
                            let mut broadcast = Vec::new();
                            this_face.create_pending_request(&sdri);
                            trace!("[REQDN {}] left breadcrumb pending request", face_stats(self.id, "IN",  this_face, &sdri));
                            for (address, that_face) in self.faces.iter_mut() {
                                if *address == packet_from { continue }
                                if that_face.contains_forwarded_request(&sdri) > 10 {
                                    trace!("[REQDN {}] don't send request downstream again", face_stats(self.id, "OUT",  that_face, &sdri));
                                    continue
                                }
                                if that_face.contains_pending_request(&sdri)   > 10 {
                                    trace!("[REQDN {}] don't send request upstream", face_stats(self.id, "OUT",  that_face, &sdri));
                                    continue
                                }
                                if that_face.contains_forwarding_hint(&sdri)   > 90 {
                                    that_face.create_forwarded_request(&sdri);
                                    trace!("[REQDN {}] sending request downstream based on forwarding hint", face_stats(self.id, "OUT",  that_face, &sdri));
                                    handle_packets.push(mk_laminar_packet(*address, copernica_packet.clone()));
                                    is_forwarded = true;
                                    continue
                                }
                                broadcast.push(address.clone())

                            }
                            if !is_forwarded {
                                for address in broadcast {
                                    if let Some(face) = self.faces.get_mut(&address) {
                                        face.create_forwarded_request(&sdri);
                                        trace!("[REQDN {}] bursting on face", face_stats(self.id, "BURST",  face, &sdri));
                                        handle_packets.push(mk_laminar_packet(address, copernica_packet.clone()));
                                    }
                                }
                            }
                        },
                    }
                },
                CopernicaPacket::Response { sdri, .. } => {
                    if this_face.contains_forwarded_request(&sdri) > 15 {
                        this_face.delete_forwarded_request(&sdri);
                        if this_face.forwarding_hint_decoherence() > 80 {
                            this_face.partially_forget_forwarding_hint();
                        }
                        this_face.create_forwarding_hint(&sdri);
                        trace!("[RESUP {}] response matched pending request", face_stats(self.id, "IN",  this_face, &sdri));
                        self.cs.put_data(copernica_packet.clone());
                        for (address, that_face) in self.faces.iter_mut() {
                            if *address == packet_from { continue }
                            that_face.delete_forwarded_request(&sdri);
                            if that_face.contains_pending_request(&sdri) > 50 {
                                trace!("[RESUP {}] send response upstream", face_stats(self.id, "OUT",  that_face, &sdri));
                                handle_packets.push(mk_laminar_packet(*address, copernica_packet.clone()));
                                that_face.delete_pending_request(&sdri);
                            }
                        }
                    }
                },
            }
        }
    }
}

fn mk_laminar_packet(address: SocketAddr, packet: CopernicaPacket) -> LaminarPacket {
    LaminarPacket::reliable_unordered(address, serialize(&packet).unwrap().to_vec())
}

fn face_stats(router_id: u8, direction: &str, face: &mut Face, sdri: &Sdri) -> String {
    format!(
    "r{0:<3}f{1: <5} {2: <5} pr{3: <3}d{4: <3}fr{5: <3}d{6: <3}fh{7: <3}d{8: <0}",
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

#![cfg(unix)]
#![warn(rust_2018_idioms)]
use rand::Rng;
use crate::Face;
use std::collections::VecDeque;

use std::net::{UdpSocket, SocketAddr, SocketAddrV4, Ipv4Addr};
use std::time::{Duration, Instant};
use std::io;
use std::thread;
use bincode::{serialize, deserialize};

use packets::{Packet, mk_data, mk_interest};
use crate::sparse_distributed_representation::{SparseDistributedRepresentation};


#[derive(Debug, Clone)]
pub struct Udp {
    listen_addr: SocketAddrV4,
    send_addr: SocketAddrV4,
    pub id: u32,
    pending_interest: SparseDistributedRepresentation,
    forwarding_hint: SparseDistributedRepresentation,
    interest_inbound: VecDeque<Packet>,
    interest_outbound: VecDeque<Packet>,
    data_inbound: VecDeque<Packet>,
    data_outbound: VecDeque<Packet>,
}

impl Udp {
    pub fn new(listen_addr: String, send_addr: String) -> Box<Udp> {
        let mut rng = rand::thread_rng();
        Box::new(Udp {
            id: rng.gen(),
            listen_addr: listen_addr.parse().unwrap(),
            send_addr: send_addr.parse().unwrap(),
            interest_inbound: VecDeque::new(),
            interest_outbound: VecDeque::new(),
            data_inbound: VecDeque::new(),
            data_outbound: VecDeque::new(),
            pending_interest: SparseDistributedRepresentation::new(),
            forwarding_hint: SparseDistributedRepresentation::new(),
        })
    }
}

impl Face for Udp {

    fn id(&self) -> u32 {
        self.id
    }

    // Basic Send and Receive Operations

    fn send_interest_downstream(&mut self, interest: Packet) {
        self.interest_outbound.push_back(interest);
        self.send();
    }
    fn receive_upstream_interest(&mut self) -> Option<Packet> {
        self.receive();
        self.interest_inbound.pop_front()
    }
    fn send_data_upstream(&mut self, data: Packet) {
        self.data_outbound.push_back(data);
        self.send();
    }
    fn receive_downstream_data(&mut self) -> Option<Packet> {
        self.receive();
        self.data_inbound.pop_front()
    }

    // Pending Interest Sparse Distributed Representation

    fn create_pending_interest(&mut self, packet: Packet) {
        self.pending_interest.insert(packet);
    }
    fn contains_pending_interest(&mut self, interest: Packet) -> u8 {
        self.pending_interest.contains(interest)
    }
    fn delete_pending_interest(&mut self, interest: Packet) {
        self.pending_interest.delete(interest);
    }

    // Forwarding Hint Sparse Distributed Representation
    fn create_forwarding_hint(&mut self, data: Packet) {
        self.forwarding_hint.insert(data);
    }
    fn contains_forwarding_hint(&mut self, interest: Packet) -> u8 {
        self.forwarding_hint.contains(interest)
    }
    fn forwarding_hint_decoherence(&mut self) -> u8 {
        self.forwarding_hint.decoherence()
    }
    fn restore_forwarding_hint(&mut self) {
        self.forwarding_hint.restore();
    }

    // @boilerplate: can't find a way to enable this witout polluting api
    fn box_clone(&self) -> Box<dyn Face> {
        Box::new((*self).clone())
    }

    fn print_pi(&self) {
        println!("pending interest on face {}:\n{:?}", self.id, self.pending_interest);
    }

    fn print_fh(&self) {
        println!("forwarding hint on face {}:\n{:?}",self.id, self.forwarding_hint);
    }
    // @Optimisation: keeping send and recv as part of the API cause maybe I want to
    // batch send this during the router main loop after an interval
    fn send(&mut self) {
        match self.interest_outbound.pop_front() {
            Some(interest) => send(self.listen_addr, self.send_addr, interest),
            None => {},
        }
        match self.data_outbound.pop_front() {
            Some(data) => send(self.listen_addr, self.send_addr, data),
            None => {},
        }
    }

    fn receive(&mut self) {
        let mut socket = UdpSocket::bind(SocketAddr::V4(self.listen_addr)).unwrap();
        socket.set_nonblocking(true).unwrap();
        let start = Instant::now();

        let mut buf: [u8; 200] = [0; 200];

        let (amt, _) =
            loop {
                match socket.recv_from(&mut buf) {
                    Ok(n) => break n,
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        if start.elapsed() > Duration::from_millis(10) {
                            //println!("{}", start.elapsed().as_millis());
                            return
                        } else {
                            continue
                        }
                    }
                    Err(e) => panic!("encountered IO error: {}", e),
                    _ => { return }
                };
            };
        let buf = &mut buf[..amt];
        let received_mesg: Packet = deserialize(&buf).unwrap();
        println!("Got {:?}", received_mesg);
        match received_mesg {
            Packet::Interest{ sdri: _ } => {
                self.interest_inbound.push_back(received_mesg)
            },
            Packet::Data{ sdri: _ } => {
                self.data_inbound.push_back(received_mesg)
            },
        };

    }
}

fn send(listen_addr: SocketAddrV4, send_addr: SocketAddrV4, packet: Packet) {
    let mut socket = UdpSocket::bind(SocketAddr::V4(listen_addr)).unwrap();
    let target = SocketAddr::V4(send_addr);
    socket.send_to(&serialize(&packet).unwrap(), target);
}


#[cfg(test)]
mod face {
    use super::*;

    #[test]
    fn vector_of_faces_and_calls_trait_methods() {
        // trait methods never return `Self`!
        let mut f1 = Udp::new();
        let mut f2 = Udp::new();
        let faces: Vec<Box<dyn Face>> = vec![f1, f2];
        let mut id = 0;
        for face in &faces {
            id = face.id();
        }
        assert!(id >= 0 as u32);
    }
}

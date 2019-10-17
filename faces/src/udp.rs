#![cfg(unix)]
#![warn(rust_2018_idioms)]
use rand::Rng;
use crate::Face;
use std::collections::VecDeque;

use async_std::io;
use async_std::net::UdpSocket;
use async_std::task;
use async_task;
use crossbeam_channel;

use std::net::{SocketAddr, SocketAddrV4, Ipv4Addr};
use bincode::{serialize, deserialize};

use packets::{Packet, mk_data, mk_interest};
use crate::sparse_distributed_representation::{SparseDistributedRepresentation};

use std::future::Future;
use std::sync::Arc;
use std::thread;

use futures::executor;


#[derive(Debug, Clone)]
pub struct Udp {
    pub id: u32,
    listen_addr: SocketAddrV4,
    send_addr: SocketAddrV4,
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
    }
    fn receive_upstream_interest(&mut self) -> Option<Packet> {
        self.interest_inbound.pop_front()
    }
    fn send_data_upstream(&mut self, data: Packet) {
        self.data_outbound.push_back(data);
    }
    fn receive_downstream_data(&mut self) -> Option<Packet> {
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

    fn run(&mut self) -> async_task::JoinHandle<(), ()> {
        let addr = self.listen_addr.clone();
        let send_addr = self.send_addr.clone();
        let mut interest_inbound = self.interest_inbound.clone();
        let mut interest_outbound = self.interest_outbound.clone();
        let mut data_inbound = self.data_inbound.clone();
        let mut data_outbound = self.data_outbound.clone();
        let future = async move {
            let socket = UdpSocket::bind(addr).await.unwrap();
            println!("Hello >");
            let mut buf = vec![0u8; 1024];
            println!("Listening on {}", socket.local_addr().unwrap());
            let (n, peer) = socket.recv_from(&mut buf).await.unwrap();
            let packet: Packet = deserialize(&buf[..n]).unwrap();
            match packet {
                Packet::Interest{ sdri: _ } => {
                    println!("{:?}", packet);
                    interest_inbound.push_back(packet)
                },
                Packet::Data{ sdri: _ } => {
                    data_inbound.push_back(packet)
                },
            };
            match interest_outbound.pop_front() {
                Some(interest) => {
                    let interest = serialize(&interest).unwrap();
                    socket.send_to(&interest, send_addr);
                },
                None => {},
            }
            match data_outbound.pop_front() {
                Some(data) => {
                    let data = serialize(&data).unwrap();
                    socket.send_to(&data, send_addr);
                },
                None => {},
            }
        };

        let (sender, receiver) = crossbeam_channel::unbounded();
        let sender = Arc::new(sender);
        let s = Arc::downgrade(&sender);
        let future = async move {
            let _sender = sender;
            future.await
        };
        let schedule = move |t| s.upgrade().unwrap().send(t).unwrap();
        let (task, handle) = async_task::spawn(future, schedule, ());
        task.schedule();
        thread::spawn(move || {
            for task in receiver {
                task.run();
            }
        });
        handle
    }
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

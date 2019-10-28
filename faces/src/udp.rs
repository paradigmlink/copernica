use crate::{Face};

use async_std::net::UdpSocket;
use async_std::task;
use crossbeam_channel::{Sender};

use std::net::{SocketAddrV4};
use bincode::{serialize, deserialize};

use packets::{Packet};
use crate::sparse_distributed_representation::{SparseDistributedRepresentation};

use log::{info};
use futures::executor::ThreadPool;


#[derive(Debug, Clone)]
pub struct Udp {
    pub id: usize,
    listen_addr: SocketAddrV4,
    send_addr: SocketAddrV4,
    pending_request: SparseDistributedRepresentation,
    forwarding_hint: SparseDistributedRepresentation,
}

impl Udp {
    pub fn new(listen_addr: String, send_addr: String) -> Box<Udp> {
        Box::new(Udp {
            id: 0,
            listen_addr: listen_addr.parse().unwrap(),
            send_addr: send_addr.parse().unwrap(),
            pending_request: SparseDistributedRepresentation::new(),
            forwarding_hint: SparseDistributedRepresentation::new(),
        })
    }
}

impl Face for Udp {

    fn set_id(&mut self, face_id: usize) {
        self.id = face_id;
    }

    // Basic Send and Receive Operations

    fn send_request_downstream(&mut self, interest: Packet) {
        send_request_downstream_or_response_upstream(self.send_addr, interest);
    }
    fn send_response_upstream(&mut self, data: Packet) {
        send_request_downstream_or_response_upstream(self.send_addr, data);
    }

    // Pending Interest Sparse Distributed Representation

    fn create_pending_request(&mut self, packet_sdri: &Vec<Vec<u16>>) {
        self.pending_request.insert(&packet_sdri);
    }
    fn contains_pending_request(&mut self, request_sdri: &Vec<Vec<u16>>) -> u8 {
        self.pending_request.contains(request_sdri)
    }
    fn delete_pending_request(&mut self, request_sdri: &Vec<Vec<u16>>) {
        self.pending_request.delete(request_sdri);
    }
    fn pending_request_decoherence(&mut self) -> u8 {
        self.pending_request.decoherence()
    }
    fn partially_forget_pending_requests(&mut self) {
        self.pending_request.partially_forget();
    }

    // Forwarding Hint Sparse Distributed Representation
    fn create_forwarding_hint(&mut self, data_sdri: &Vec<Vec<u16>>) {
        self.forwarding_hint.insert(&data_sdri);
    }
    fn contains_forwarding_hint(&mut self, request_sdri: &Vec<Vec<u16>>) -> u8 {
        self.forwarding_hint.contains(request_sdri)
    }
    fn forwarding_hint_decoherence(&mut self) -> u8 {
        self.forwarding_hint.decoherence()
    }
    fn partially_forget_forwarding_hints(&mut self) {
        self.forwarding_hint.partially_forget();
    }

    // @boilerplate: can't find a way to enable this witout polluting api
    fn box_clone(&self) -> Box<dyn Face> {
        Box::new((*self).clone())
    }

    fn print_pi(&self) {
        println!("pending request on face {}:\n{:?}", self.id, self.pending_request);
    }

    fn print_fh(&self) {
        println!("forwarding hint on face {}:\n{:?}",self.id, self.forwarding_hint);
    }

    fn receive_upstream_request_or_downstream_response(&mut self, face_id: usize, spawner: ThreadPool , packet_sender: Sender<(usize, Packet)>) {
        let addr = self.listen_addr.clone();
        self.set_id(face_id);
        spawner.spawn_ok(async move {
            let socket = UdpSocket::bind(addr).await.unwrap();
            loop {
                let mut buf = vec![0u8; 1024];
                let (n, peer) = socket.recv_from(&mut buf).await.unwrap();
                let packet: Packet = deserialize(&buf[..n]).unwrap();
                info!("RECV from {} => to {}: {:?}", peer, socket.local_addr().unwrap(), packet);
                let _r = packet_sender.send((face_id, packet));
            }
        });
    }
}

fn send_request_downstream_or_response_upstream(
    send_addr: SocketAddrV4,
    packet: Packet) {
    task::block_on( async move {
        let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let packet_ser = serialize(&packet).unwrap();
        let _r = socket.send_to(&packet_ser, send_addr).await;
        info!("SENT from {} => to {}: {:?}",socket.local_addr().unwrap(), send_addr, packet);
    });
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

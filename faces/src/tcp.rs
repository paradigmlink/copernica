use rand::Rng;
use crate::Face;

use bitvec::prelude::*;
use packets::{Packet, mk_data, mk_interest};
use crate::sparse_distributed_representation::{SparseDistributedRepresentation};

#[derive(Debug, Clone)]
pub struct Tcp {
    pub id: u32,
    pending_interest: SparseDistributedRepresentation,
    forwarding_hint: SparseDistributedRepresentation,
    interest_inbound: Vec<Packet>,
    interest_outbound: Vec<Packet>,
    data_inbound: Vec<Packet>,
    data_outbound: Vec<Packet>,
}

impl Tcp {
    pub fn new() -> Box<Tcp> {
        let mut rng = rand::thread_rng();
        Box::new(Tcp {
            id: rng.gen(),
            interest_inbound: Vec::new(),
            interest_outbound: Vec::new(),
            data_inbound: Vec::new(),
            data_outbound: Vec::new(),
            pending_interest: SparseDistributedRepresentation::new(),
            forwarding_hint: SparseDistributedRepresentation::new(),
        })
    }
}

impl Face for Tcp {

    fn id(&self) -> u32 {
        self.id
    }

    // Basic Send and Receive Operations

    fn send_interest_downstream(&mut self, interest: Packet) {
        self.interest_outbound.push(interest);
    }
    fn receive_upstream_interest(&mut self) -> Option<Packet> {
        self.interest_inbound.pop()
    }
    fn send_data_upstream(&mut self, data: Packet) {
        self.data_outbound.push(data);
    }
    fn receive_downstream_data(&mut self) -> Option<Packet> {
        self.data_inbound.pop()
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

    fn contains_forwarding_hint(&mut self, interest: Packet) -> u8 {
        self.forwarding_hint.contains(interest)
    }
    fn create_forwarding_hint(&mut self, data: Packet) {
        self.forwarding_hint.insert(data);
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
}

#[cfg(test)]
mod face {
    use super::*;

    #[test]
    fn vector_of_faces_and_calls_trait_methods() {
        // trait methods never return `Self`!
        let mut f1 = Tcp::new();
        let mut f2 = Tcp::new();
        let faces: Vec<Box<dyn Face>> = vec![f1, f2];
        let mut id = 0;
        for face in &faces {
            id = face.id();
        }
        assert!(id >= 0 as u32);
    }
}

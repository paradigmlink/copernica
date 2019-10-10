extern crate rand;

use rand::Rng;
//use crate::Face;

use bitvec::prelude::*;
use packets::{Packet, mk_data, mk_interest};
use crate::sparse_distributed_representation::{SparseDistributedRepresentation};

#[derive(Debug, Clone, PartialEq)]
pub struct Mock {
    pub id: u8,
    pending_interest_sdr: SparseDistributedRepresentation,
    forwarding_hint_sdr: SparseDistributedRepresentation,
    interest_inbound: Vec<Packet>,
    interest_outbound: Vec<Packet>,
    data_inbound: Vec<Packet>,
    data_outbound: Vec<Packet>,
}

impl Mock { // Face for Mock {
    pub fn new() -> Mock {
        let mut rng = rand::thread_rng();
        Mock {
            id: rng.gen(),
            interest_inbound: Vec::new(),
            interest_outbound: Vec::new(),
            data_inbound: Vec::new(),
            data_outbound: Vec::new(),
            pending_interest_sdr: SparseDistributedRepresentation::new(),
            forwarding_hint_sdr: SparseDistributedRepresentation::new(),
        }
    }

    pub fn id(&self) -> u8 {
        self.id
    }

    pub fn send_interest_downstream(&mut self, interest: Packet) {
        self.interest_outbound.push(interest);
    }
    pub fn receive_upstream_interest(&mut self) -> Option<Packet> {
        self.interest_inbound.pop()
    }
    pub fn send_data_upstream(&mut self, data: Packet) {
        self.data_outbound.push(data);
    }
    pub fn receive_downstream_data(&mut self) -> Option<Packet> {
        self.data_inbound.pop()
    }

    pub fn create_pending_interest(&mut self, packet: Packet) {
        self.pending_interest_sdr.insert(packet);
    }

    pub fn create_forwarding_hint(&mut self, data: Packet) {
        self.forwarding_hint_sdr.insert(data);
    }

    pub fn contains_forwarding_hint(&mut self, interest: Packet) -> u8 {
        self.forwarding_hint_sdr.contains(interest)
    }

    pub fn contains_pending_interest(&mut self, interest: Packet) -> u8 {
        self.pending_interest_sdr.contains(interest)
    }

    pub fn delete_pending_interest(&mut self, interest: Packet) {
        self.pending_interest_sdr.delete(interest);
    }
}


//impl PartialEq for dyn Face {
//    fn eq(&self, other: &dyn Face) -> bool {
//        self.id() == other.id()
//    }
//}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_face_send_and_receive_interest() {
        let mut mock_face: Mock = Mock::new();
        let interest = mk_interest("blah".to_string());
        mock_face.send_interest_downstream(interest.clone());
        let out = match mock_face.receive_upstream_interest() {
            Some(i) => i,
            None => mk_interest("".to_string()),
        };
        assert_eq!(interest, out);
    }

    #[test]
    fn mock_face_send_and_receive_data() {
        let mut mock_face: Mock = Mock::new();
        let data = mk_data("blah".to_string());
        mock_face.send_data_upstream(data.clone());
        let out = match mock_face.receive_downstream_data() {
            Some(d) => d,
            None => mk_data("".to_string()),
        };
        assert_eq!(data, out);
    }
}

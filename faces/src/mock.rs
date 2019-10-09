extern crate rand;

use rand::Rng;
use crate::Face;

use bitvec::prelude::*;
use packets::{Interest, Data};
use crate::sparse_distributed_representation::{SparseDistributedRepresentation};

#[derive(Debug, Clone, PartialEq)]
pub struct Mock {
    pub id: u8,
    pending_interest_sdr: SparseDistributedRepresentation,
    breadcrumb_trail_sdr: SparseDistributedRepresentation,
    forwarding_hint_sdr: SparseDistributedRepresentation,
    interest_inbound: Vec<Interest>,
    interest_outbound: Vec<Interest>,
    data_inbound: Vec<Data>,
    data_outbound: Vec<Data>,
}

impl Face for Mock {
    fn new() -> Mock {
        let mut rng = rand::thread_rng();
        Mock {
            id: rng.gen(),
            interest_inbound: Vec::new(),
            interest_outbound: Vec::new(),
            data_inbound: Vec::new(),
            data_outbound: Vec::new(),
            pending_interest_sdr: SparseDistributedRepresentation::new(),
            breadcrumb_trail_sdr: SparseDistributedRepresentation::new(),
            forwarding_hint_sdr: SparseDistributedRepresentation::new(),
        }
    }

    fn id(&self) -> u8 {
        self.id
    }

    fn send_interest_downstream(&mut self, i: Interest) {
        self.interest_outbound.push(i);
    }
    fn receive_upstream_interest(&mut self) -> Option<Interest> {
        self.interest_inbound.pop()
    }
    fn send_data_upstream(&mut self, d: Data) {
        self.data_outbound.push(d);
    }
    fn receive_downstream_data(&mut self) -> Option<Data> {
        self.data_inbound.pop()
    }

    fn create_pending_interest(&mut self, interest: Interest) {
        self.pending_interest_sdr.insert(interest);
    }

    fn create_breadcrumb_trail(&mut self, interest: Interest) {
        self.breadcrumb_trail_sdr.insert(interest);
    }

    fn contains_forwarding_hint(&mut self, interest: Interest) -> u8 {
        self.forwarding_hint_sdr.contains(interest)
    }

    fn contains_pending_interest(&mut self, interest: Interest) -> u8 {
        self.pending_interest_sdr.contains(interest)
    }
}


impl PartialEq for dyn Face {
    fn eq(&self, other: &dyn Face) -> bool {
        self.id() == other.id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_face_send_and_receive_interest() {
        let mock_face: Mock = Face::new();
        let interest = Interest::new("blah".to_string());
        mock_face.send_interest_downstream(interest.clone());
        let out = match mock_face.receive_upstream_interest() {
            Some(i) => i,
            None => Interest::new("".to_string()),
        };
        assert_eq!(interest, out);
    }

    #[test]
    fn mock_face_send_and_receive_data() {
        let mock_face: Mock = Face::new();
        let data = Data::new("blah".to_string());
        mock_face.send_data_upstream(data.clone());
        let out = match mock_face.receive_downstream_data() {
            Some(d) => d,
            None => Data::new("".to_string()),
        };
        assert_eq!(data, out);
    }
}

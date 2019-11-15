mod sparse_distributed_representation;
use {
    sparse_distributed_representation::{SparseDistributedRepresentation},
};

#[derive(Debug, Clone)]
pub struct Face {
    id:                u16,
    pending_request:   SparseDistributedRepresentation,
    forwarding_hint:   SparseDistributedRepresentation,
    forwarded_request: SparseDistributedRepresentation,

}

impl Face {
    pub fn new(id: u16) -> Face {
        Face {
            id:                 id,
            pending_request:    SparseDistributedRepresentation::new(),
            forwarding_hint:    SparseDistributedRepresentation::new(),
            forwarded_request:  SparseDistributedRepresentation::new(),
        }
    }

    pub fn get_id(&self) -> u16 {
        self.id.clone()
    }
    // Pending Request Sparse Distributed Representation

    pub fn create_pending_request(&mut self, packet_sdri: &Vec<Vec<u16>>) {
        self.pending_request.insert(&packet_sdri);
    }
    pub fn contains_pending_request(&mut self, request_sdri: &Vec<Vec<u16>>) -> u8 {
        self.pending_request.contains(request_sdri)
    }
    pub fn delete_pending_request(&mut self, request_sdri: &Vec<Vec<u16>>) {
        self.pending_request.delete(request_sdri);
    }
    pub fn pending_request_decoherence(&mut self) -> u8 {
        self.pending_request.decoherence()
    }
    pub fn partially_forget_pending_request(&mut self) {
        self.pending_request.partially_forget();
    }


    // Forwarded Request Sparse Distributed Representation

    pub fn create_forwarded_request(&mut self, packet_sdri: &Vec<Vec<u16>>) {
        self.forwarded_request.insert(&packet_sdri);
    }
    pub fn contains_forwarded_request(&mut self, request_sdri: &Vec<Vec<u16>>) -> u8 {
        self.forwarded_request.contains(request_sdri)
    }
    pub fn delete_forwarded_request(&mut self, request_sdri: &Vec<Vec<u16>>) {
        self.forwarded_request.delete(request_sdri);
    }
    pub fn forwarded_request_decoherence(&mut self) -> u8 {
        self.forwarded_request.decoherence()
    }
    pub fn partially_forget_forwarded_request(&mut self) {
        self.forwarded_request.partially_forget();
    }


    // Forwarding Hint Sparse Distributed Representation
    pub fn create_forwarding_hint(&mut self, data_sdri: &Vec<Vec<u16>>) {
        self.forwarding_hint.insert(&data_sdri);
    }
    pub fn contains_forwarding_hint(&mut self, request_sdri: &Vec<Vec<u16>>) -> u8 {
        self.forwarding_hint.contains(request_sdri)
    }
    pub fn forwarding_hint_decoherence(&mut self) -> u8 {
        self.forwarding_hint.decoherence()
    }
    pub fn partially_forget_forwarding_hint(&mut self) {
        self.forwarding_hint.partially_forget();
    }


}

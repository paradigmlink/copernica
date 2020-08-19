use {
    crate::{
        node::sparse_distributed_representation::{
            SparseDistributedRepresentation
        },
        transport::{ReplyTo},
        hbfi::{HBFI},
    },
};

#[derive(Debug, Clone)]
pub struct Face {
    id:                ReplyTo,
    pending_request:   SparseDistributedRepresentation,
    forwarding_hint:   SparseDistributedRepresentation,
    forwarded_request: SparseDistributedRepresentation,

}

impl Face {
    pub fn new(id: ReplyTo) -> Face {
        Face {
            id,
            pending_request:    SparseDistributedRepresentation::new(),
            forwarding_hint:    SparseDistributedRepresentation::new(),
            forwarded_request:  SparseDistributedRepresentation::new(),
        }
    }

    pub fn id(&self) -> ReplyTo {
        self.id.clone()
    }
    // Pending Request Sparse Distributed Representation
    // Used to determine the direction of upstream and shouldn't be conflated
    // with Forwarded Request which determines which faces are downstream nodes,
    // specifically which nodes to not forward to again.

    pub fn create_pending_request(&mut self, packet_hbfi: &HBFI) {
        self.pending_request.insert(&packet_hbfi);
    }
    pub fn contains_pending_request(&self, request_hbfi: &HBFI) -> u8 {
        self.pending_request.contains(request_hbfi)
    }
    #[allow(dead_code)]
    pub fn delete_pending_request(&mut self, request_hbfi: &HBFI) {
        self.pending_request.delete(request_hbfi);
    }
    pub fn pending_request_decoherence(&self) -> u8 {
        self.pending_request.decoherence()
    }
    #[allow(dead_code)]
    pub fn partially_forget_pending_request(&mut self) {
        self.pending_request.partially_forget();
    }


    // Forwarded Request Sparse Distributed Representation
    // Used to determine if a request has been forwarded on this face so as
    // not to forward the request on the face again. It's easy to get
    // this mixed up with Pending Requests, which has the specific purpose
    // of determining which faces are upstream nodes
    pub fn create_forwarded_request(&mut self, packet_hbfi: &HBFI) {
        self.forwarded_request.insert(&packet_hbfi);
    }
    pub fn contains_forwarded_request(&self, request_hbfi: &HBFI) -> u8 {
        self.forwarded_request.contains(request_hbfi)
    }
    #[allow(dead_code)]
    pub fn delete_forwarded_request(&mut self, request_hbfi: &HBFI) {
        self.forwarded_request.delete(request_hbfi);
    }
    pub fn forwarded_request_decoherence(&self) -> u8 {
        self.forwarded_request.decoherence()
    }
    #[allow(dead_code)]
    pub fn partially_forget_forwarded_request(&mut self) {
        self.forwarded_request.partially_forget();
    }


    // Forwarding Hint Sparse Distributed Representation
    // Used to determine if a request can be satisfied on this face.
    // There's a subtle difference between Pending Request
    #[allow(dead_code)]
    pub fn create_forwarding_hint(&mut self, data_hbfi: &HBFI) {
        self.forwarding_hint.insert(&data_hbfi);
    }
    pub fn contains_forwarding_hint(&self, request_hbfi: &HBFI) -> u8 {
        self.forwarding_hint.contains(request_hbfi)
    }
    pub fn forwarding_hint_decoherence(&self) -> u8 {
        self.forwarding_hint.decoherence()
    }
    pub fn partially_forget_forwarding_hint(&mut self) {
        self.forwarding_hint.partially_forget();
    }


}

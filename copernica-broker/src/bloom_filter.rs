use {
    copernica_common::{HBFI, BFI},
    std::{ collections::HashMap, },
};

#[derive(Clone)]
pub struct Blooms {
    pending_request: HashMap<Vec<BFI>, u64>,
    forwarded_request: HashMap<Vec<BFI>, u64>,
}

impl Blooms {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            pending_request: HashMap::new(),
            forwarded_request: HashMap::new(),
        }
    }

    // Pending Request Sparse Distributed Representation
    // Used to determine the direction of upstream and shouldn't be conflated
    // with Forwarded Request which determines which faces are downstream nodes,
    // specifically which nodes to not forward to again.

    pub fn create_pending_request(&mut self, hbfi: &HBFI) {
        *self.pending_request.entry(hbfi.to_vec()).or_insert(0) += 1;
    }
    pub fn contains_pending_request(&self, hbfi: &HBFI) -> bool {
        if let Some(contains) = self.pending_request.get(&hbfi.to_vec()) {
            if contains > &0 {
                return true
            } else {
                return false
            }
        } else {
            return false
        }
    }
    #[allow(dead_code)]
    pub fn delete_pending_request(&mut self, hbfi: &HBFI) {
        *self.pending_request.entry(hbfi.to_vec()).or_insert(0) -= 1;
    }

    // Forwarded Request Sparse Distributed Representation
    // Used to determine if a request has been forwarded on this face so as
    // not to forward the request on the face again. It's easy to get
    // this mixed up with Pending Requests, which has the specific purpose
    // of determining which faces are upstream nodes
    pub fn create_forwarded_request(&mut self, hbfi: &HBFI) {
        *self.forwarded_request.entry(hbfi.to_vec()).or_insert(0) += 1;
    }
    pub fn contains_forwarded_request(&self, hbfi: &HBFI) -> bool {
        if let Some(contains) = self.forwarded_request.get(&hbfi.to_vec()) {
            if contains > &0 {
                return true
            } else {
                return false
            }
        } else {
            return false
        }
    }
    #[allow(dead_code)]
    pub fn delete_forwarded_request(&mut self, hbfi: &HBFI) {
        *self.forwarded_request.entry(hbfi.to_vec()).or_insert(0) -= 1;
    }
}

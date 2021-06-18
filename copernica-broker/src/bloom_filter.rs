use {
    copernica_common::{HBFI, HBFIExcludeFrame, constants},
    uluru::LRUCache,
};
pub type Pheromone = LRUCache<HBFIExcludeFrame, { constants::RESPONSE_STORE_SIZE }>;
#[derive(Clone)]
pub struct Blooms {
    pending_request: Pheromone,
    forwarded_request: Pheromone,
}
impl Blooms {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            pending_request: Pheromone::default(),
            forwarded_request: Pheromone::default(),
        }
    }
    // Pending Request Sparse Distributed Representation
    // Used to determine the direction of upstream and shouldn't be conflated
    // with Forwarded Request which determines which faces are downstream nodes,
    // specifically which nodes to not forward to again.
    pub fn create_pending_request(&mut self, hbfi: HBFI) {
        self.pending_request.insert(HBFIExcludeFrame(hbfi));
    }
    pub fn contains_pending_request(&mut self, hbfi: HBFI) -> bool {
        self.pending_request.touch(|n| n == &HBFIExcludeFrame(hbfi.clone()))
    }
    // Forwarded Request Sparse Distributed Representation
    // Used to determine if a request has been forwarded on this face so as
    // not to forward the request on the face again. It's easy to get
    // this mixed up with Pending Requests, which has the specific purpose
    // of determining which faces are upstream nodes
    pub fn create_forwarded_request(&mut self, hbfi: HBFI) {
        self.forwarded_request.insert(HBFIExcludeFrame(hbfi));
    }
    pub fn contains_forwarded_request(&mut self, hbfi: HBFI) -> bool {
        self.forwarded_request.touch(|n| n == &HBFIExcludeFrame(hbfi.clone()))
    }
}

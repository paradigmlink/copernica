use {
    copernica_packets::{HBFI, HBFIOnlyKeys},
    copernica_common::{ constants },
    uluru::LRUCache,
};
pub type Pheromone = LRUCache<HBFIOnlyKeys, { constants::PACKET_PHEROMONE_TRAIL_COUNT }>;
#[derive(Clone)]
pub struct Blooms {
    pending_request: Pheromone,
    forwarded_request: Pheromone,
}
impl Blooms {
    pub fn new() -> Self {
        Self {
            pending_request: Pheromone::default(),
            forwarded_request: Pheromone::default(),
        }
    }
    // A pending request is used to determine the direction of downstream
    // (from where the Request originated, NOT where the Response might be).
    pub fn create_pending_request(&mut self, hbfi: HBFI) {
        self.pending_request.insert(HBFIOnlyKeys(hbfi));
    }
    pub fn contains_pending_request(&mut self, hbfi: HBFI) -> bool {
        self.pending_request.touch(|n| n == &HBFIOnlyKeys(hbfi.clone()))
    }
    // A forwarded request is used to determine the direction or a potential
    // direction of upstream (where the Response might be). If a link has a
    // pending_request on it, it means that link is facing DOWNSTREAM (towards
    // the Request) hence we will not forward the Request on that link.
    pub fn create_forwarded_request(&mut self, hbfi: HBFI) {
        self.forwarded_request.insert(HBFIOnlyKeys(hbfi));
    }
    pub fn contains_forwarded_request(&mut self, hbfi: HBFI) -> bool {
        self.forwarded_request.touch(|n| n == &HBFIOnlyKeys(hbfi.clone()))
    }
}

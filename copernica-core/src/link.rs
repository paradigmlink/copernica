use {
    crate::{
        bloom_filter::BloomFilter,
        borsh::{BorshDeserialize, BorshSerialize},
        hbfi::HBFI,
    },
    rand::Rng,
    std::{net::SocketAddr},
};

pub type Hertz = u32;
pub type Nonce = u64;

#[derive(
    Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize, BorshSerialize, BorshDeserialize,
)]
pub enum ReplyTo {
    UdpIp(SocketAddr),
    Rf(Hertz),
    Mpsc,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct LinkId {
    nonce: Nonce,
    reply_to: ReplyTo,
}

impl LinkId {
    pub fn new(nonce: Nonce, reply_to: ReplyTo) -> Self {
        Self { nonce, reply_to }
    }
    pub fn listen(reply_to: ReplyTo) -> Self {
        let mut rng = rand::thread_rng();
        let nonce: u64 = rng.gen();
        Self { nonce, reply_to }
    }
    pub fn remote(&self, reply_to: ReplyTo) -> Self {
        Self { nonce: self.nonce.clone(), reply_to }
    }
    pub fn reply_to(&self) -> ReplyTo {
        self.reply_to.clone()
    }
    pub fn nonce(&self) -> Nonce {
        self.nonce.clone()
    }
}

#[derive(Clone)]
pub struct Blooms {
    pending_request: BloomFilter,
    forwarded_request: BloomFilter,
}

impl Blooms {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            pending_request: BloomFilter::new(),
            forwarded_request: BloomFilter::new(),
        }
    }

    // Pending Request Sparse Distributed Representation
    // Used to determine the direction of upstream and shouldn't be conflated
    // with Forwarded Request which determines which faces are downstream nodes,
    // specifically which nodes to not forward to again.

    pub fn create_pending_request(&mut self, packet_hbfi: &HBFI) {
        self.pending_request.insert(&packet_hbfi);
    }
    pub fn contains_pending_request(&self, request_hbfi: &HBFI) -> bool {
        self.pending_request.contains(request_hbfi)
    }
    #[allow(dead_code)]
    pub fn delete_pending_request(&mut self, request_hbfi: &HBFI) {
        self.pending_request.delete(request_hbfi);
    }

    // Forwarded Request Sparse Distributed Representation
    // Used to determine if a request has been forwarded on this face so as
    // not to forward the request on the face again. It's easy to get
    // this mixed up with Pending Requests, which has the specific purpose
    // of determining which faces are upstream nodes
    pub fn create_forwarded_request(&mut self, packet_hbfi: &HBFI) {
        self.forwarded_request.insert(&packet_hbfi);
    }
    pub fn contains_forwarded_request(&self, request_hbfi: &HBFI) -> bool {
        self.forwarded_request.contains(request_hbfi)
    }
    #[allow(dead_code)]
    pub fn delete_forwarded_request(&mut self, request_hbfi: &HBFI) {
        self.forwarded_request.delete(request_hbfi);
    }
}

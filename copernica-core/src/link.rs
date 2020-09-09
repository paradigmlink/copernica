use {
    crate::{
        bloom_filter::BloomFilter,
        borsh::{BorshDeserialize, BorshSerialize},
        hbfi::HBFI,
    },
    rand::Rng,
    std::{fmt, net::SocketAddr},
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
    forwarding_hint: BloomFilter,
    forwarded_request: BloomFilter,
}

impl fmt::Debug for Blooms {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "(pr:{}, fh:{}, fr:{})",
            self.pending_request.decoherence(),
            self.forwarding_hint.decoherence(),
            self.forwarded_request.decoherence()
        )
    }
}

impl Blooms {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            pending_request: BloomFilter::new(),
            forwarding_hint: BloomFilter::new(),
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
    pub fn contains_pending_request(&self, request_hbfi: &HBFI) -> u8 {
        self.pending_request.contains(request_hbfi)
    }
    #[allow(dead_code)]
    pub fn delete_pending_request(&mut self, request_hbfi: &HBFI) {
        self.pending_request.delete(request_hbfi);
    }
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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

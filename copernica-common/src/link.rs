use {
    borsh::{BorshDeserialize, BorshSerialize},
    rand::Rng,
    std::{net::SocketAddr},
};

pub type Hertz = u32;
pub type Nonce = u64;

#[derive(
    Clone, Debug, Eq, Hash, PartialEq, BorshSerialize, BorshDeserialize,
)]
pub enum ReplyTo {
    UdpIp(SocketAddr),
    Rf(Hertz),
    Mpsc,
    DeepSix,
    //Release, // think about how to release the constriction
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
        let mut nonce = 0;
        while nonce == 0 {
            nonce = rng.gen(); // 0 is reserved for DeepSix
        }
        Self { nonce, reply_to }
    }
    pub fn deep_six() -> Self {
        Self { nonce: 0, reply_to: ReplyTo::DeepSix }
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


use {
    crate::{
        borsh::{BorshSerialize, BorshDeserialize},
    },
    std::net::{SocketAddr},
    rand::Rng,
};

pub type Hertz = u32;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum ReplyTo {
    UdpIp(SocketAddr),
    Rf(Hertz),
    Mpsc,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct LinkId {
    reply_to: ReplyTo,
    nonce: u16
}

impl LinkId {
    #[allow(dead_code)]
    pub fn new(reply_to: ReplyTo, mut nonce: u16) -> Self {
        if nonce == 0 {
            let mut rng = rand::thread_rng();
            nonce = rng.gen();
        }
        Self { reply_to, nonce }
    }
    #[allow(dead_code)]
    pub fn nonce(&self) -> u16 {
        self.nonce.clone()
    }
    #[allow(dead_code)]
    pub fn reply_to(&self) -> ReplyTo {
        self.reply_to.clone()
    }
}

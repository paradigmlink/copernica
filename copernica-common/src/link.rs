use {
    serde::{Deserialize, Serialize},
    rand::Rng,
    std::{fmt, net::SocketAddr},
    keynesis::{PublicKey},
};

pub type Hertz = u32;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Identity {
    PublicKey(u64),
    Choke,
    Pk(PublicKey),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum ReplyTo {
    UdpIp(SocketAddr),
    Rf(Hertz),
    Mpsc,
    Choke,
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct LinkId {
    identity: Identity,
    reply_to: ReplyTo,
}

impl LinkId {
    pub fn new(identity: Identity, reply_to: ReplyTo) -> Self {
        Self { identity, reply_to }
    }
    pub fn listen(reply_to: ReplyTo) -> Self {
        let mut rng = rand::thread_rng();
        Self { identity: Identity::PublicKey(rng.gen()), reply_to }
    }
    pub fn choke() -> Self {
        Self { identity: Identity::Choke, reply_to: ReplyTo::Choke }
    }
    pub fn remote(&self, reply_to: ReplyTo) -> Self {
        Self { identity: self.identity.clone(), reply_to }
    }
    pub fn reply_to(&self) -> ReplyTo {
        self.reply_to.clone()
    }
    pub fn identity(&self) -> Identity {
        self.identity.clone()
    }
}

impl fmt::Debug for LinkId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Link:({:?}, {:?})", self.identity(), self.reply_to())
    }
}

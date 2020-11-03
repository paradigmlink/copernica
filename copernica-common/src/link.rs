use {
    serde::{Deserialize, Serialize},
    rand::Rng,
    std::{fmt, net::SocketAddr},
};

pub type Hertz = u32;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum IdentityState {
    PublicKey(u64),
    Choke,
    Unchoke,
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Identity {
    name: String,
    id_state: IdentityState,
}

impl Identity {
    pub fn new(name: String, id_state: IdentityState) -> Self {
        Self {
            name,
            id_state,
        }
    }
}

impl fmt::Debug for Identity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\"{}\"", self.name)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum ReplyTo {
    UdpIp(SocketAddr),
    Rf(Hertz),
    Mpsc,
    Choke,
    Unchoke,
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
    pub fn listen(name: String, reply_to: ReplyTo) -> Self {
        let mut rng = rand::thread_rng();
        let identity = Identity { name, id_state: IdentityState::PublicKey(rng.gen()) };
        Self { identity, reply_to }
    }
    pub fn choke() -> Self {
        let identity = Identity { name: "choke".into(), id_state: IdentityState::Choke };
        Self { identity, reply_to: ReplyTo::Choke }
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

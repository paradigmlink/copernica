use {
    serde::{Deserialize, Serialize},
    rand::Rng,
    std::{fmt, net::SocketAddr},
    keynesis::{PrivateIdentity, Seed},
    anyhow::{Result, anyhow},
};

pub type Hertz = u32;
/*
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Identity {
    PublicIdentity(PublicIdentity),
    Choke,
}
*/
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum ReplyTo {
    UdpIp(SocketAddr),
    Rf(Hertz),
    Mpsc,
}
/*
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct LinkId {
    identity: Identity,
    reply_to: ReplyTo,
}
*/

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum LinkId {
    Identity {
        private_identity: PrivateIdentity,
        reply_to: ReplyTo,
    },
    Choke,
}


impl LinkId {
    pub fn new(private_identity: PrivateIdentity, reply_to: ReplyTo) -> Self {
        LinkId::Identity { private_identity, reply_to }
    }
    pub fn listen(private_identity: PrivateIdentity, reply_to: ReplyTo) -> Self {
        LinkId::Identity { private_identity, reply_to }
    }
    pub fn choke() -> Self {
        LinkId::Choke
    }
    pub fn remote(&self, reply_to: ReplyTo) -> Result<Self> {
        match self {
            LinkId::Identity { private_identity, .. } => {
                Ok(LinkId::Identity { private_identity: private_identity.clone(), reply_to })
            },
            Choke => {
                Err(anyhow!("Requesting a ReplyTo when in state Choke. Not going to happen buddy"))
            }
        }
    }
    pub fn reply_to(&self) -> Result<ReplyTo> {
        match self {
            LinkId::Identity { private_identity, reply_to } => {
                Ok(reply_to.clone())
            },
            Choke => {
                Err(anyhow!("Requesting a ReplyTo when in state Choke. Not going to happen buddy"))
            }
        }
    }
    pub fn private_identity(&self) -> Result<PrivateIdentity> {
        match self {
            LinkId::Identity { private_identity, reply_to } => {
                Ok(private_identity.clone())
            },
            Choke => {
                Err(anyhow!("Requesting a PrivateIdentity when in state Choke. Not going to happen buddy"))
            }
        }
    }
}

impl fmt::Debug for LinkId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LinkId::Identity { private_identity, reply_to } => {
                write!(f, "LinkId:({:?}, {:?})", private_identity, reply_to)
            },
            Choke => {
                write!(f, "LinkId: CHOKED")
            }
        }
    }
}

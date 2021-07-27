use {
    std::{fmt},
    crate::{
        PrivateIdentityInterface, PublicIdentity, PublicIdentityInterface, SharedSecret, Nonce, ReplyTo,
    },
    anyhow::{Result, anyhow},
    rand::Rng,
};
#[derive(Clone, Eq, Hash, PartialEq)]
pub enum LinkId {
    Identity {
        lookup_id: u32,
        link_sid: PrivateIdentityInterface,
        remote_link_pid: PublicIdentityInterface,
        reply_to: ReplyTo,
    },
    Choke,
}
impl LinkId {
    pub fn new(lookup_id: u32, link_sid: PrivateIdentityInterface, remote_link_pid: PublicIdentityInterface, reply_to: ReplyTo) -> Self {
        LinkId::Identity { lookup_id, link_sid, remote_link_pid, reply_to }
    }
    pub fn link_with_type(link_sid: PrivateIdentityInterface, remote_link_pid: PublicIdentityInterface, reply_to: ReplyTo) -> Self {
        let mut rng = rand::thread_rng();
        let i: u32 = rng.gen();
        LinkId::Identity { lookup_id: i,  link_sid, remote_link_pid, reply_to }
    }
    pub fn lookup_id(&self) -> Result<u32> {
        match self {
            LinkId::Identity { lookup_id, .. } => Ok(*lookup_id),
            LinkId::Choke => {
                Err(anyhow!("Requesting a ReplyTo when in state Choke. Not going to happen buddy"))
            }
        }
    }
    pub fn shared_secret(&self, nonce: Nonce, remote_link_pid: PublicIdentity) -> Result<SharedSecret> {
        match self {
            LinkId::Identity { link_sid, .. } => {
                Ok(link_sid.shared_secret(nonce, remote_link_pid))
            },
            LinkId::Choke => {
                Err(anyhow!("Requesting a ReplyTo when in state Choke. Not going to happen buddy"))
            }
        }
    }
    pub fn choke() -> Self {
        LinkId::Choke
    }
    pub fn remote(&self, reply_to: ReplyTo) -> Result<Self> {
        match self {
            LinkId::Identity { lookup_id, link_sid, remote_link_pid, .. } => {
                Ok(LinkId::Identity { lookup_id: lookup_id.clone(),  link_sid: link_sid.clone(), remote_link_pid: remote_link_pid.clone(), reply_to })
            },
            LinkId::Choke => {
                Err(anyhow!("Requesting a ReplyTo when in state Choke. Not going to happen buddy"))
            }
        }
    }
    pub fn reply_to(&self) -> Result<ReplyTo> {
        match self {
            LinkId::Identity { reply_to, .. } => {
                Ok(reply_to.clone())
            },
            LinkId::Choke => {
                Err(anyhow!("Requesting a ReplyTo when in state Choke. Not going to happen buddy"))
            }
        }
    }
    pub fn link_sid(&self) -> Result<PrivateIdentityInterface> {
        match self {
            LinkId::Identity { link_sid, ..} => {
                Ok(link_sid.clone())
            },
            LinkId::Choke => {
                Err(anyhow!("Requesting a PrivateIdentityInterface when in state Choke. Not going to happen buddy"))
            }
        }
    }
    pub fn link_pid(&self) -> Result<PublicIdentity> {
        match self {
            LinkId::Identity { link_sid, .. } => {
                Ok(link_sid.public_id())
            },
            LinkId::Choke => {
                Err(anyhow!("Requesting a PrivateIdentityInterface when in state Choke. Not going to happen buddy"))
            }
        }
    }
    pub fn remote_link_pid(&self) -> Result<PublicIdentityInterface> {
        match self {
            LinkId::Identity { remote_link_pid, ..} => {
                Ok(remote_link_pid.clone())
            },
            LinkId::Choke => {
                Err(anyhow!("Requesting a PublicIdentity when in state Choke. Not going to happen buddy"))
            }
        }
    }
}
impl fmt::Debug for LinkId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LinkId::Identity { lookup_id, link_sid, remote_link_pid, reply_to} => {
                write!(f, "LinkId:({}, {:?}, {:?}, {:?})", lookup_id, link_sid, remote_link_pid, reply_to)
            },
            LinkId::Choke => {
                write!(f, "LinkId: CHOKED")
            }
        }
    }
}
impl fmt::Debug for ReplyTo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ReplyTo::UdpIp(addr) => {
                write!(f, "ReplyTo::UdpIp({})", addr)
            },
            ReplyTo::Mpsc => {
                write!(f, "ReplyTo::Mpsc")
            },
            ReplyTo::Rf(hertz) => {
                write!(f, "ReplyTo::Rf({})", hertz)
            },
        }
    }
}

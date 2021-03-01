use {
    serde::{Deserialize, Serialize},
    std::{fmt, net::SocketAddr},
    copernica_identity::{PrivateIdentity, PublicIdentity, SharedSecret},
    anyhow::{Result, anyhow},
    rand::Rng,
    crate::{Nonce},
};
pub type Hertz = u32;
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum ReplyTo {
    Mpsc,
    UdpIp(SocketAddr),
    Rf(Hertz),
}
#[derive(Clone, Eq, Hash, PartialEq)]
pub enum LinkId {
    Identity {
        lookup_id: u32,
        sid: PrivateIdentity,
        rx_pid: Option<PublicIdentity>,
        reply_to: ReplyTo,
    },
    Choke,
}
impl LinkId {
    pub fn new(lookup_id: u32, sid: PrivateIdentity, rx_pid: Option<PublicIdentity>, reply_to: ReplyTo) -> Self {
        LinkId::Identity { lookup_id, sid, rx_pid, reply_to }
    }
    pub fn listen(sid: PrivateIdentity, rx_pid: Option<PublicIdentity>, reply_to: ReplyTo) -> Self {
        let mut rng = rand::thread_rng();
        let i: u32 = rng.gen();
        LinkId::Identity { lookup_id: i,  sid, rx_pid, reply_to }
    }
    pub fn lookup_id(&self) -> Result<u32> {
        match self {
            LinkId::Identity { lookup_id, .. } => Ok(*lookup_id),
            LinkId::Choke => {
                Err(anyhow!("Choke state shouldn't do anything, thus doesn't require a lookup_id"))
            }
        }
    }
    pub fn shared_secret(&self, nonce: Nonce, lnk_rx_pid: PublicIdentity) -> Result<SharedSecret> {
        let lnk_rx_pk = lnk_rx_pid.derive(&nonce);
        let lnk_tx_sk = self.sid()?.derive(&nonce);
        let shared_secret = lnk_tx_sk.exchange(&lnk_rx_pk);
        Ok(shared_secret)
    }
    pub fn choke() -> Self {
        LinkId::Choke
    }
    pub fn remote(&self, reply_to: ReplyTo) -> Result<Self> {
        match self {
            LinkId::Identity { lookup_id, sid, rx_pid, .. } => {
                Ok(LinkId::Identity { lookup_id: lookup_id.clone(),  sid: sid.clone(), rx_pid: rx_pid.clone(), reply_to })
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
    pub fn sid(&self) -> Result<PrivateIdentity> {
        match self {
            LinkId::Identity { sid, ..} => {
                Ok(sid.clone())
            },
            LinkId::Choke => {
                Err(anyhow!("Requesting a PrivateIdentity when in state Choke. Not going to happen buddy"))
            }
        }
    }
    pub fn tx_pid(&self) -> Result<PublicIdentity> {
        match self {
            LinkId::Identity { sid, .. } => {
                Ok(sid.public_id())
            },
            LinkId::Choke => {
                Err(anyhow!("Requesting a PrivateIdentity when in state Choke. Not going to happen buddy"))
            }
        }
    }
    pub fn rx_pid(&self) -> Result<Option<PublicIdentity>> {
        match self {
            LinkId::Identity { rx_pid, ..} => {
                Ok(rx_pid.clone())
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
            LinkId::Identity { lookup_id, sid, rx_pid, reply_to} => {
                write!(f, "LinkId:({}, {:?}, {:?}, {:?})", lookup_id, sid, rx_pid, reply_to)
            },
            LinkId::Choke => {
                write!(f, "LinkId: CHOKED")
            }
        }
    }
}

use {
    crate::{
        narrow_waist::{NarrowWaist},
    },
    std::{
        net::{SocketAddr},
    },
};

pub type Hertz = u32;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum InterFace {
    SocketAddr(SocketAddr),
    Sdr(Hertz), // Software Defined Radio (not Sparse Distributed Representation)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TransportPacket {
    pub reply_to: InterFace,
    pub payload: NarrowWaist,
}

impl TransportPacket {
    pub fn new(reply_to: InterFace, payload: NarrowWaist) -> TransportPacket {
        TransportPacket {
            reply_to,
            payload,
        }
    }
    pub fn payload(&self) -> NarrowWaist {
        self.payload.clone()
    }
    pub fn reply_to(&self) -> InterFace {
        self.reply_to.clone()
    }
}

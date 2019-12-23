use {
    crate::{
        narrow_waist::{NarrowWaist},
    },
    std::{
        net::{SocketAddr},
    },
};

pub type Hertz = f32;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum InterFace {
    SocketAddr(SocketAddr),
    Sdr(Hertz), // Software Defined Radio (not Sparse Distributed Representation)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TransportPacket {
    reply_to: InterFace,
    payload: NarrowWaist,
}

impl TransportPacket {
    pub fn new(reply_to: InterFace, payload: NarrowWaist) -> TransportPacket {
        TransportPacket {
            reply_to,
            payload,
        }
    }
}

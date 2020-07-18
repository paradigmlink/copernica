use {
    crate::{
        narrow_waist::{NarrowWaist},
        response_store::{Response},
        borsh::{BorshSerialize, BorshDeserialize},
    },
    std::{
        net::{SocketAddr},
    },
};

pub type Hertz = u32;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum ReplyTo {
    Udp(SocketAddr),
    Sdr(Hertz), // Software Defined Radio (not Sparse Distributed Representation)
}

// Does go over the wire
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct TransportPacket {
    pub reply_to: ReplyTo,
    pub payload: NarrowWaist,
}

impl TransportPacket {
    pub fn new(reply_to: ReplyTo, payload: NarrowWaist) -> TransportPacket {
        TransportPacket {
            reply_to,
            payload,
        }
    }
    pub fn payload(&self) -> NarrowWaist {
        self.payload.clone()
    }
    pub fn reply_to(&self) -> ReplyTo {
        self.reply_to.clone()
    }
}

/// Doesn't go over the wire
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct TransportResponse {
    pub reply_to: ReplyTo,
    pub payload: Response,
}

impl TransportResponse {
    pub fn new(reply_to: ReplyTo, payload: Response) -> TransportResponse {
        TransportResponse {
            reply_to,
            payload,
        }
    }
    pub fn payload(&self) -> Response {
        self.payload.clone()
    }
    pub fn reply_to(&self) -> ReplyTo {
        self.reply_to.clone()
    }
}


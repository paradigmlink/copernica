use {
    crate::{
        borsh::{BorshSerialize, BorshDeserialize},
        channel::{ReplyTo},
        copernica_constants,
        hbfi::{HBFI},
    },
    std::{
        fmt,
    },
};

#[derive(Clone, BorshSerialize, BorshDeserialize)]
pub struct Data {
    pub len: u16,
    pub data: [u8; copernica_constants::FRAGMENT_SIZE as usize],
}

#[derive(Clone, BorshSerialize, BorshDeserialize)]
pub enum NarrowWaist {
    Request     { hbfi: HBFI },
    Response    { hbfi: HBFI, data: Data, offset: u64, total: u64 },
}

impl fmt::Debug for NarrowWaist {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            NarrowWaist::Request { hbfi } => write!(f, "REQ{:?}", hbfi),
            NarrowWaist::Response { hbfi, offset, total, .. } =>
                write!(f, "RES{:?} {}/{}", hbfi, offset, total)
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
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

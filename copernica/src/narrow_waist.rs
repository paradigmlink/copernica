use {
    std::{
        fmt,
    },
    borsh::{BorshSerialize, BorshDeserialize},
    anyhow::{Result},
    crate::{
        sdri::{Sdri},
    }
};

pub type Bytes = Vec<u8>;

#[derive(Clone, PartialEq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum NarrowWaist {
    Request     { sdri: Sdri },
    Response    { sdri: Sdri, data: Bytes, count: u64, total: u64 },
}

pub fn mk_request_packet(name: String) -> Result<NarrowWaist> {
    Ok(NarrowWaist::Request {
        sdri: Sdri::new(name)?
    })
}

pub fn mk_response_packet(name: String, data: Bytes, count: u64, total: u64) -> Result<NarrowWaist> {
    Ok(NarrowWaist::Response {
        sdri: Sdri::new(name)?,
        data,
        count,
        total,
    })
}

impl fmt::Debug for NarrowWaist {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            NarrowWaist::Request{sdri} => write!(f, "REQ{:?}", sdri),
            NarrowWaist::Response{sdri, count, total, ..} =>
                write!(f, "RES{:?} {}/{}", sdri, count, total)
        }
    }
}


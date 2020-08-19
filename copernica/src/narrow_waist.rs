use {
    std::{
        convert::{TryInto, TryFrom},
        fmt,
    },
    borsh::{BorshSerialize, BorshDeserialize},
    anyhow::{Result},
    crate::{
        constants,
        hbfi::{HBFI},
    }
};

#[derive(Clone, BorshSerialize, BorshDeserialize)]
pub struct Data {
    pub len: u16,
    pub data: [u8; constants::FRAGMENT_SIZE as usize],
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


use {
    crate::{
        borsh::{BorshDeserialize, BorshSerialize},
        copernica_constants,
        hbfi::HBFI,
        link::{Link, ReplyTo},
    },
    std::fmt,
};

#[derive(Clone, BorshSerialize, BorshDeserialize)]
pub struct Data {
    pub len: u16,
    pub data: [u8; copernica_constants::FRAGMENT_SIZE as usize],
}

#[derive(Clone, BorshSerialize, BorshDeserialize)]
pub enum NarrowWaist {
    Request {
        hbfi: HBFI,
    },
    Response {
        hbfi: HBFI,
        data: Data,
        offset: u64,
        total: u64,
    },
}

impl fmt::Debug for NarrowWaist {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            NarrowWaist::Request { hbfi } => write!(f, "REQ{:?}", hbfi),
            NarrowWaist::Response {
                hbfi,
                offset,
                total,
                ..
            } => write!(f, "RES{:?} {}/{}", hbfi, offset, total),
        }
    }
}

#[derive(Debug, Clone)]
pub struct InterLinkPacket {
    pub link: Link,
    pub nw: NarrowWaist,
}

impl InterLinkPacket {
    pub fn new(link: Link, nw: NarrowWaist) -> Self {
        Self { link, nw }
    }
    pub fn link(&self) -> Link {
        self.link.clone()
    }
    pub fn change_destination(&self, link: Link) -> Self {
        Self { link, nw: self.nw.clone() }
    }
    pub fn reply_to(&self) -> ReplyTo {
        self.link.reply_to()
    }
    pub fn narrow_waist(&self) -> NarrowWaist {
        self.nw.clone()
    }
}

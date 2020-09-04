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

#[derive(BorshSerialize, Debug, BorshDeserialize, Clone)]
pub struct WirePacket {
    reply_to: ReplyTo,
    nw: NarrowWaist,
}

impl WirePacket {
    pub fn new(reply_to: ReplyTo, nw: NarrowWaist) -> Self {
        Self { reply_to, nw }
    }
    pub fn reply_to(&self) -> ReplyTo {
        self.reply_to.clone()
    }
    pub fn narrow_waist(&self) -> NarrowWaist {
        self.nw.clone()
    }
    pub fn change_reply_to(&self, reply_to: ReplyTo) -> Self {
        Self {
            reply_to,
            nw: self.nw.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct InterLinkPacket {
    pub link: Link,
    pub wp: WirePacket,
}

impl InterLinkPacket {
    pub fn new(link: Link, wp: WirePacket) -> Self {
        Self { link, wp }
    }
    pub fn link(&self) -> Link {
        self.link.clone()
    }
    pub fn wire_packet(&self) -> WirePacket {
        self.wp.clone()
    }
    pub fn reply_to(&self) -> ReplyTo {
        self.wp.reply_to()
    }
    pub fn narrow_waist(&self) -> NarrowWaist {
        self.wp.narrow_waist()
    }
    pub fn change_destination(&self, link: Link) -> Self {
        Self {
            link: link.clone(),
            wp: self.wp.change_reply_to(link.reply_to()),
        }
    }
    pub fn change_origination(&self, link: Link) -> Self {
        Self {
            link: link.clone(),
            wp: self.wp.change_reply_to(link.reply_to()),
        }
    }
}

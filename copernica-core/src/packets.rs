use {
    crate::{
        borsh::{BorshDeserialize, BorshSerialize},
        copernica_constants,
        hbfi::HBFI,
        link::{LinkId, ReplyTo},
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
    pub link_id: LinkId,
    pub wp: WirePacket,
}

impl InterLinkPacket {
    pub fn new(link_id: LinkId, wp: WirePacket) -> Self {
        Self { link_id, wp }
    }
    pub fn link_id(&self) -> LinkId {
        self.link_id.clone()
    }
    pub fn change_destination(&self, link_id: LinkId) -> Self {
        Self { link_id, wp: self.wp.clone() }
    }
    pub fn reply_to(&self) -> ReplyTo {
        self.link_id.reply_to()
    }
    pub fn narrow_waist(&self) -> NarrowWaist {
        self.wp.narrow_waist().clone()
    }
    pub fn wire_packet(&self) -> WirePacket {
        self.wp.clone()
    }
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize)]
pub struct WirePacket {
    pub reply_to: ReplyTo,
    pub nw: NarrowWaist,
}

impl WirePacket {
    pub fn new(reply_to: ReplyTo, nw: NarrowWaist) -> Self {
        Self { reply_to , nw }
    }
    pub fn narrow_waist(&self) -> NarrowWaist {
        self.nw.clone()
    }
    pub fn reply_to(&self) -> ReplyTo {
        self.reply_to.clone()
    }
    pub fn change_origination(&self, reply_to: ReplyTo) -> Self {
        Self { reply_to, nw: self.nw.clone() }
    }
}


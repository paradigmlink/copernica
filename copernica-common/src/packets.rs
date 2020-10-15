use {
    crate::{
        constants,
        hbfi::HBFI,
        link::{LinkId, ReplyTo},
    },
    borsh::{BorshDeserialize, BorshSerialize},
    std::fmt,
};

#[derive(Clone, BorshSerialize, BorshDeserialize)]
pub struct Data {
    pub len: u16,
    pub data: [u8; constants::FRAGMENT_SIZE as usize],
}

#[derive(Clone, BorshSerialize, BorshDeserialize)]
pub enum NarrowWaistPacket {
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

impl fmt::Debug for NarrowWaistPacket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            NarrowWaistPacket::Request { hbfi } => write!(f, "REQ{:?}", hbfi),
            NarrowWaistPacket::Response {
                hbfi,
                offset,
                total,
                ..
            } => write!(f, "RES{:?} {}/{}", hbfi, offset, total),
        }
    }
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize)]
pub struct LinkPacket {
    pub reply_to: ReplyTo,
    pub nw: NarrowWaistPacket,
}

impl LinkPacket {
    pub fn new(reply_to: ReplyTo, nw: NarrowWaistPacket) -> Self {
        Self { reply_to , nw }
    }
    pub fn narrow_waist(&self) -> NarrowWaistPacket {
        self.nw.clone()
    }
    pub fn reply_to(&self) -> ReplyTo {
        self.reply_to.clone()
    }
    pub fn change_origination(&self, reply_to: ReplyTo) -> Self {
        Self { reply_to, nw: self.nw.clone() }
    }
}

#[derive(Debug, Clone)]
pub struct InterLinkPacket {
    pub link_id: LinkId,
    pub lp: LinkPacket,
}

impl InterLinkPacket {
    pub fn new(link_id: LinkId, lp: LinkPacket) -> Self {
        Self { link_id, lp }
    }
    pub fn link_id(&self) -> LinkId {
        self.link_id.clone()
    }
    pub fn change_destination(&self, link_id: LinkId) -> Self {
        Self { link_id, lp: self.lp.clone() }
    }
    pub fn reply_to(&self) -> ReplyTo {
        self.link_id.reply_to()
    }
    pub fn narrow_waist(&self) -> NarrowWaistPacket {
        self.lp.narrow_waist().clone()
    }
    pub fn wire_packet(&self) -> LinkPacket {
        self.lp.clone()
    }
}

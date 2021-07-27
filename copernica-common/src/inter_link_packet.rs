use {
    crate::{
        link::{LinkId},
        reply_to::{ReplyTo},
        LinkPacket,
        NarrowWaistPacket,
    },
    anyhow::{Result},
};

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
    pub fn reply_to(&self) -> Result<ReplyTo> {
        self.link_id.reply_to()
    }
    pub fn narrow_waist(&self) -> NarrowWaistPacket {
        self.lp.narrow_waist().clone()
    }
    pub fn link_packet(&self) -> LinkPacket {
        self.lp.clone()
    }
}

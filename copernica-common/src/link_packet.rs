use {
    crate::{
        link::{ReplyTo},
        NarrowWaistPacket,
    },
};

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct LinkPacket {
    reply_to: ReplyTo,
    nw: NarrowWaistPacket,
}

impl LinkPacket {
    pub fn new(reply_to: ReplyTo, nw: NarrowWaistPacket) -> Self {
        LinkPacket { reply_to, nw }
    }
    pub fn narrow_waist(&self) -> NarrowWaistPacket {
        self.nw.clone()
    }
    pub fn reply_to(&self) -> ReplyTo {
        self.reply_to.clone()
    }
    pub fn change_origination(&self, reply_to: ReplyTo) -> Self {
        LinkPacket {
            reply_to: reply_to,
            nw: self.nw.clone(),
        }
    }
}

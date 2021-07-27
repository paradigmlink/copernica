use {
    crate::{
        reply_to::{ReplyTo},
        LinkId, Nonce, Tag,
        NarrowWaistPacket, PublicIdentity, PublicIdentityInterface,
        constants::*,
        serialization::*,
    },
    cryptoxide::{chacha20poly1305::{ChaCha20Poly1305}},
    anyhow::{Result},
    //log::{error, debug},
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
    pub fn from_bytes(data: &[u8], link_id: LinkId) -> Result<(PublicIdentity, Self)> {
        match link_id.remote_link_pid()? {
            PublicIdentityInterface::Present { .. } => {
                let mut link_tx_pk_with_cc = [0u8; ID_SIZE + CC_SIZE];
                link_tx_pk_with_cc.clone_from_slice(&data[CYPHERTEXT_LINK_TX_PK_START..CYPHERTEXT_LINK_TX_PK_END]);
                let lnk_tx_pid: PublicIdentity = PublicIdentity::from(link_tx_pk_with_cc);
                let mut link_nonce = Nonce([0u8; NONCE_SIZE]);
                link_nonce.0.clone_from_slice(&data[CYPHERTEXT_LINK_NONCE_START..CYPHERTEXT_LINK_NONCE_END]);
                let mut link_tag = [0u8; TAG_SIZE];
                link_tag.clone_from_slice(&data[CYPHERTEXT_LINK_TAG_START..CYPHERTEXT_LINK_TAG_END]);
                let reply_to_size = &data[CYPHERTEXT_LINK_REPLY_TO_SIZE_START..CYPHERTEXT_LINK_REPLY_TO_SIZE_END];
                let mut nw_size = [0u8; 2];
                nw_size.clone_from_slice(&data[CYPHERTEXT_LINK_NARROW_WAIST_SIZE_START..CYPHERTEXT_LINK_NARROW_WAIST_SIZE_END]);
                let nw_size: usize = u8_to_u16(nw_size) as usize;
                let reply_to = ReplyTo::from_bytes(&data[CYPHERTEXT_LINK_NARROW_WAIST_SIZE_END..CYPHERTEXT_LINK_NARROW_WAIST_SIZE_END + reply_to_size[0] as usize].to_vec())?;
                let nw_start = CYPHERTEXT_LINK_NARROW_WAIST_SIZE_END + reply_to_size[0] as usize;
                let shared_secret = link_id.shared_secret(link_nonce.clone(), lnk_tx_pid.clone())?;
                let ctx = ChaCha20Poly1305::new(&shared_secret.as_ref(), &link_nonce.0, &[]);
                drop(shared_secret);
                let nw = NarrowWaistPacket::from_cyphertext_bytes(&data[nw_start..nw_start + nw_size], ctx.clone(), Tag(link_tag.clone()))?;
                Ok((lnk_tx_pid, LinkPacket::new(reply_to, nw)))
            },
            PublicIdentityInterface::Absent => {
                let mut link_tx_pk = [0u8; ID_SIZE + CC_SIZE];
                link_tx_pk.clone_from_slice(&data[CLEARTEXT_LINK_TX_PK_START..CLEARTEXT_LINK_TX_PK_END]);
                let lnk_tx_pid: PublicIdentity = PublicIdentity::from(link_tx_pk);
                let reply_to_size = &data[CLEARTEXT_LINK_REPLY_TO_SIZE_START..CLEARTEXT_LINK_REPLY_TO_SIZE_END];
                let mut nw_size = [0u8; 2];
                nw_size.clone_from_slice(&data[CLEARTEXT_LINK_NARROW_WAIST_SIZE_START..CLEARTEXT_LINK_NARROW_WAIST_SIZE_END]);
                let nw_size: usize = u8_to_u16(nw_size) as usize;
                let reply_to = ReplyTo::from_bytes(&data[CLEARTEXT_LINK_NARROW_WAIST_SIZE_END..CLEARTEXT_LINK_NARROW_WAIST_SIZE_END + reply_to_size[0] as usize].to_vec())?;
                let nw_start = CLEARTEXT_LINK_NARROW_WAIST_SIZE_END + reply_to_size[0] as usize;
                let nw = NarrowWaistPacket::from_cleartext_bytes(&data[nw_start..nw_start + nw_size])?;
                Ok((lnk_tx_pid, LinkPacket::new(reply_to, nw)))
            },
        }
    }
}

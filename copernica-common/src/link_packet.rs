use {
    crate::{
        reply_to::{ReplyTo},
        LinkId, Nonce, Tag, generate_nonce,
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
    pub fn as_bytes(&self, link_id: LinkId) -> Result<Vec<u8>> {
        let mut buf: Vec<u8> = vec![];
        let lnk_tx_pid = link_id.link_pid()?;
        match link_id.remote_link_pid()? {
            PublicIdentityInterface::Absent => {
                match self {
                    LinkPacket { reply_to, nw } => {
                        buf.extend_from_slice(lnk_tx_pid.key().as_ref());
                        buf.extend_from_slice(lnk_tx_pid.chain_code().as_ref());
                        let reply_to_s = reply_to.as_bytes()?;
                        let nws = nw.as_bytes();
                        buf.extend_from_slice(&[reply_to_s.len() as u8]);
                        buf.extend_from_slice(&u16_to_u8(nws.len() as u16));
                        buf.extend_from_slice(&reply_to_s);
                        buf.extend_from_slice(&nws);
                    }
                }
            },
            PublicIdentityInterface::Present { public_identity: lnk_rx_pid } => {
                match self {
                    LinkPacket { reply_to, nw } => {
                        buf.extend_from_slice(lnk_tx_pid.key().as_ref());
                        buf.extend_from_slice(lnk_tx_pid.chain_code().as_ref());
                        let mut rng = rand::thread_rng();
                        let nonce: Nonce = generate_nonce(&mut rng);
                        buf.extend_from_slice(&nonce.0);
                        let mut tag = Tag([0; TAG_SIZE]);
                        let shared_secret = link_id.shared_secret(nonce.clone(), lnk_rx_pid)?;
                        let mut ctx = ChaCha20Poly1305::new(&shared_secret.as_ref(), &nonce.0, &[]);
                        drop(shared_secret);
                        let mut nws = nw.as_bytes();
                        let mut encrypted = vec![0u8; nws.len()];
                        ctx.encrypt(&nws, &mut encrypted[..], &mut tag.0);
                        nws.copy_from_slice(&encrypted[..]);
                        buf.extend_from_slice(&tag.0);
                        let reply_to_s = reply_to.as_bytes()?;
                        buf.extend_from_slice(&[reply_to_s.len() as u8]);
                        buf.extend_from_slice(&u16_to_u8(nws.len() as u16));
                        buf.extend_from_slice(&reply_to_s);
                        buf.extend_from_slice(&nws);
                    }
                }
            },
        }
        Ok(buf)
    }
    pub fn from_bytes(data: &[u8], link_id: LinkId) -> Result<(PublicIdentity, Self)> {
        match link_id.remote_link_pid()? {
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
                let nw = NarrowWaistPacket::from_cyphertext_bytes(&data[nw_start..nw_start + nw_size], link_id, link_nonce, lnk_tx_pid.clone(), Tag(link_tag.clone()))?;
                Ok((lnk_tx_pid, LinkPacket::new(reply_to, nw)))
            },
        }
    }
}

use {
    crate::{
        reply_to::{ReplyTo},
        LinkId, Nonce, Tag,
        NarrowWaistPacket, PublicIdentity, PublicIdentityInterface,
        constants::*,
    },
    cryptoxide::{chacha20poly1305::{ChaCha20Poly1305}},
    anyhow::{Result, anyhow},
    log::{error},
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
                        buf.extend_from_slice(&reply_to.as_bytes()?);
                        buf.extend_from_slice(&nw.as_bytes());
                    }
                }
            },
            PublicIdentityInterface::Present { public_identity: lnk_rx_pid } => {
                match self {
                    LinkPacket { reply_to, nw } => {
                        buf.extend_from_slice(lnk_tx_pid.key().as_ref());
                        buf.extend_from_slice(lnk_tx_pid.chain_code().as_ref());
                        let nonce = Nonce::new_nonce();
                        buf.extend_from_slice(&nonce.0);
                        let mut tag = Tag::new_empty_tag();
                        let shared_secret = link_id.shared_secret(nonce.clone(), lnk_rx_pid)?;
                        let mut ctx = ChaCha20Poly1305::new(&shared_secret.as_ref(), &nonce.0, &[]);
                        drop(shared_secret);
                        let mut nws = nw.as_bytes();
                        let mut encrypted = vec![0u8; nws.len()];
                        ctx.encrypt(&nws, &mut encrypted[..], &mut tag.0);
                        nws.copy_from_slice(&encrypted[..]);
                        buf.extend_from_slice(&tag.0);
                        buf.extend_from_slice(&reply_to.as_bytes()?);
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
                let mut lnk_tx_pid = [0u8; ID_SIZE + CC_SIZE];
                lnk_tx_pid.clone_from_slice(&data[CLEARTEXT_LINK_TX_PK_START..CLEARTEXT_LINK_TX_PK_END]);
                let lnk_tx_pid: PublicIdentity = PublicIdentity::from(lnk_tx_pid);
                let reply_to = ReplyTo::from_bytes(&data[CLEARTEXT_LINK_REPLY_TO_START..CLEARTEXT_LINK_REPLY_TO_END])?;
                let nw = NarrowWaistPacket::from_bytes(&data[CLEARTEXT_LINK_NARROW_WAIST_PACKET_START..])?;
                Ok((lnk_tx_pid, LinkPacket::new(reply_to, nw)))
            },
            PublicIdentityInterface::Present { .. } => {
                let mut link_tx_pid = [0u8; ID_SIZE + CC_SIZE];
                link_tx_pid.clone_from_slice(&data[CYPHERTEXT_LINK_TX_PK_START..CYPHERTEXT_LINK_TX_PK_END]);
                let lnk_tx_pid: PublicIdentity = PublicIdentity::from(link_tx_pid);
                let link_nonce = Nonce::from_bytes(&data[CYPHERTEXT_LINK_NONCE_START..CYPHERTEXT_LINK_NONCE_END]);
                let link_tag = Tag::from_bytes(&data[CYPHERTEXT_LINK_TAG_START..CYPHERTEXT_LINK_TAG_END]);
                let reply_to = ReplyTo::from_bytes(&data[CYPHERTEXT_LINK_REPLY_TO_START..CYPHERTEXT_LINK_REPLY_TO_END])?;
                let shared_secret = link_id.shared_secret(link_nonce.clone(), lnk_tx_pid.clone())?;
                let mut ctx = ChaCha20Poly1305::new(&shared_secret.as_ref(), &link_nonce.0, &[]);
                drop(shared_secret);
                let encrypted = &data[CYPHERTEXT_LINK_NARROW_WAIST_PACKET_START..];
                let mut decrypted = vec![0u8; encrypted.len()];
                if !ctx.decrypt(encrypted, &mut decrypted, &link_tag.0) {
                    let err_msg = "Failed to decrypt NarrowWaistPacket";
                    error!("{}", err_msg);
                    return Err(anyhow!(err_msg))
                };
                let nw = NarrowWaistPacket::from_bytes(&decrypted)?;
                Ok((lnk_tx_pid, LinkPacket::new(reply_to, nw)))
            },
        }
    }
}

mod udp;
mod mpsc_channel;
mod mpsc_corruptor;
pub use {
    udp::{UdpIp},
    mpsc_channel::{MpscChannel},
    mpsc_corruptor::{MpscCorruptor},
};
use {
    copernica_common::{
        InterLinkPacket, LinkId,
        constants::*,
        HBFI, ReplyTo,
        NarrowWaistPacket, ResponseData, LinkPacket, BFI,
        generate_nonce, Nonce, Tag, Data,
        serialization::*,
    },
    cryptoxide::{chacha20poly1305::{ChaCha20Poly1305}},
    copernica_identity::{ PrivateIdentity, PublicIdentity, Signature },
    log::{trace},
    crossbeam_channel::{Sender, Receiver},
    anyhow::{anyhow, Result},
    reed_solomon::{Buffer, Encoder, Decoder},
};
pub fn decode(msg: Vec<u8>, lnk_rx_sid: Option<PrivateIdentity>) -> Result<(PublicIdentity, LinkPacket)> {
    let dec = Decoder::new(6);
    let reconstituted: Vec<_> = msg.chunks(255).map(|c| Buffer::from_slice(c, c.len())).map(|d| dec.correct(&d,None).unwrap()).collect();
    let reconstituted: Vec<_> = reconstituted.iter().map(|d| d.data()).collect::<Vec<_>>().concat();
    Ok(deserialize_link_packet(&reconstituted, lnk_rx_sid)?)
}
pub fn encode(lp: LinkPacket, lnk_tx_sid: PrivateIdentity, lnk_rx_pid: Option<PublicIdentity>) -> Result<Vec<u8>> {
    let mut merged = vec![];
    let enc = Encoder::new(6);
    let nw: Vec<u8> = serialize_link_packet(&lp, lnk_tx_sid, lnk_rx_pid)?;
    let cs = nw.chunks(255-6);
    for c in cs {
        let c = enc.encode(&c[..]);
        merged.extend(&**c);
    }
    Ok(merged)
}
pub trait Link<'a> {
    fn run(&self) -> Result<()>;
    fn new(name: String, link: LinkId, router_in_and_out: ( Sender<InterLinkPacket> , Receiver<InterLinkPacket>)) -> Result<Self> where Self: Sized;
}

mod udp;
mod mpsc_channel;
mod mpsc_corruptor;
pub use {
    udp::{UdpIp},
    mpsc_channel::{MpscChannel},
    mpsc_corruptor::{MpscCorruptor, Corruption},
};
use {
    copernica_common::{
        InterLinkPacket, LinkId, LinkPacket, PublicIdentity,
        Operations, serialization::*
    },
    crossbeam_channel::{Receiver, Sender},
    anyhow::{Result},
    reed_solomon::{Buffer, Encoder, Decoder},
};
pub fn decode(msg: Vec<u8>, link_id: LinkId) -> Result<(PublicIdentity, LinkPacket)> {
    let dec = Decoder::new(6);
    let reconstituted: Vec<_> = msg.chunks(255).map(|c| Buffer::from_slice(c, c.len())).map(|d| dec.correct(&d,None).unwrap()).collect();
    let reconstituted: Vec<_> = reconstituted.iter().map(|d| d.data()).collect::<Vec<_>>().concat();
    Ok(deserialize_link_packet(&reconstituted, link_id)?)
}
pub fn encode(lp: LinkPacket, link_id: LinkId) -> Result<Vec<u8>> {
    let mut merged = vec![];
    let enc = Encoder::new(6);
    let nw: Vec<u8> = serialize_link_packet(&lp, link_id)?;
    let cs = nw.chunks(255-6);
    for c in cs {
        let c = enc.encode(&c[..]);
        merged.extend(&**c);
    }
    Ok(merged)
}
pub trait Link {
    fn run(&mut self) -> Result<()>;
    fn new(link: LinkId, ops: (String, Operations), router_in_and_out: ( Sender<InterLinkPacket> , Receiver<InterLinkPacket>)) -> Result<Self> where Self: Sized;
}

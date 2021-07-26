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
    anyhow::{anyhow, Result},
    reed_solomon::{Buffer, Encoder, Decoder},
    //log::{debug},
};
pub fn decode(msg: Vec<u8>, link_id: LinkId) -> Result<(PublicIdentity, LinkPacket)> {
    let dec = Decoder::new(6);
    let mut buffers: Vec<Buffer> = vec![];
    for chunk in msg.chunks(255) {
        buffers.push(Buffer::from_slice(chunk, chunk.len()));
    }
    let mut reconstituted: Vec<Buffer> = vec![];
    for buffer in buffers {
        let buf = match dec.correct(&buffer, None) {
            Ok(b) => b,
            Err(e) => {
                return Err(anyhow!("Packet corrupted beyond recovery, dropping it (error: {:?})", e));
            },
        };
        reconstituted.push(buf);
    }
    let reconstituted: Vec<u8> = reconstituted.iter().map(|d| d.data()).collect::<Vec<_>>().concat();
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

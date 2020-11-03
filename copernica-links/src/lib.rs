mod udp;
mod mpsc_channel;
mod mpsc_corruptor;
pub use {
    udp::{UdpIp},
    mpsc_channel::{MpscChannel},
    mpsc_corruptor::{MpscCorruptor},
};

use {
    copernica_common::{InterLinkPacket, LinkId, LinkPacket},
    bincode,
    crossbeam_channel::{Sender, Receiver},
    anyhow::{Result},
    reed_solomon::{Buffer, Encoder, Decoder},
};

pub fn decode(msg: Vec<u8>) -> Result<LinkPacket> {
    let dec = Decoder::new(12);
    let reconstituted: Vec<_> = msg.chunks(255).map(|c| Buffer::from_slice(c, c.len())).map(|d| dec.correct(&d,None).unwrap()).collect();
    let reconstituted: Vec<_> = reconstituted.iter().map(|d| d.data()).collect::<Vec<_>>().concat();
    let wp: LinkPacket = bincode::deserialize(&reconstituted[..])?;
    Ok(wp)
}

pub fn encode(wp: LinkPacket) -> Result<Vec<u8>> {
    let mut merged = vec![];
    let enc = Encoder::new(12);
    let nw: Vec<u8> = bincode::serialize(&wp)?;
    let cs = nw.chunks(256-13);
    for c in cs {
        let c = enc.encode(&c[..]);
        merged.extend(&**c);
    }
    Ok(merged)
}

pub trait Link<'a> {
    fn run(&self) -> Result<()>;
    fn new(link: LinkId, router_in_and_out: ( Sender<InterLinkPacket> , Receiver<InterLinkPacket> ) ) -> Result<Self> where Self: Sized;
}

use {
    copernica_core::{
        InterLinkPacket, Link, WirePacket
    },
    borsh::{BorshDeserialize, BorshSerialize},
    crossbeam_channel::{Sender, Receiver},
    anyhow::{Result},
    reed_solomon::{Buffer, Encoder, Decoder},
};

pub fn decode(msg: Vec<u8>) -> Result<WirePacket> {
    let dec = Decoder::new(12);
    let reconstituted: Vec<_> = msg.chunks(255).map(|c| Buffer::from_slice(c, c.len())).map(|d| dec.correct(&d,None).unwrap()).collect();
    let reconstituted: Vec<_> = reconstituted.iter().map(|d| d.data()).collect::<Vec<_>>().concat();
    let wp = WirePacket::try_from_slice(&reconstituted[..])?;
    Ok(wp)
}

pub fn encode(wp: WirePacket) -> Result<Vec<u8>> {
    let mut merged = vec![];
    let enc = Encoder::new(12);
    let nw = wp.try_to_vec()?;
    let cs = nw.chunks(256-13);
    for c in cs {
        let c = enc.encode(&c[..]);
        merged.extend(&**c);
    }
    Ok(merged)
}

pub trait Transport<'a> {
    fn run(&self) -> Result<()>;
    fn new(link: Link, router_in_and_out: ( Sender<InterLinkPacket> , Receiver<InterLinkPacket> ) ) -> Result<Self> where Self: Sized;
}

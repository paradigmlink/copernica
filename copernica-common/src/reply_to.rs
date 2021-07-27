use {
    crate::{
        constants::*,
    },
    std::{net::SocketAddr},
    anyhow::{Result, anyhow},
    //log::{debug},
    bincode,
};
pub type Hertz = u32;
#[derive(Clone, Eq, Hash, PartialEq)]
pub enum ReplyTo {
    Mpsc,
    UdpIp(SocketAddr),
    Rf(Hertz),
}
impl ReplyTo {
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        let rt = match data.len() as usize {
            TO_REPLY_TO_MPSC => {
                ReplyTo::Mpsc
            },
            TO_REPLY_TO_UDPIP4 => {
                let address = bincode::deserialize(&data)?;
                ReplyTo::UdpIp(address)
            },
            TO_REPLY_TO_UDPIP6 => {
                let address = bincode::deserialize(&data)?;
                ReplyTo::UdpIp(address)
            },
            TO_REPLY_TO_RF => {
                let address = bincode::deserialize(&data)?;
                ReplyTo::Rf(address)
            },
            _ => return Err(anyhow!("Deserializing ReplyTo hit an unrecognised type or variation"))
        };
        Ok(rt)
    }
}

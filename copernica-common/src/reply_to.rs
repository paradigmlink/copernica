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
    pub fn as_bytes(&self) -> Result<Vec<u8>> {
        let mut buf: Vec<u8> = vec![];
        match self {
            ReplyTo::Mpsc => {
            },
            ReplyTo::UdpIp(addr) => {
                let addr_s = bincode::serialize(&addr)?;
                buf.extend_from_slice(addr_s.as_ref());
            }
            ReplyTo::Rf(hz) => {
                let hz = bincode::serialize(&hz)?;
                buf.extend_from_slice(hz.as_ref());
            }
        }
        Ok(buf)
    }
}

use {
    copernica_common::{u8_to_u16, u16_to_u8, constants::*},
    std::{fmt, net::{SocketAddrV4, SocketAddrV6}},
    anyhow::{Result, anyhow},
    bincode,
};
pub type Hertz = u32;
#[derive(Clone, Eq, Hash, PartialEq)]
pub enum ReplyTo {
    Mpsc,
    UdpIpV4(SocketAddrV4),
    UdpIpV6(SocketAddrV6),
    Rf(Hertz),
}
impl ReplyTo {
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        let mut reply_to_index = [0u8; 2];
        reply_to_index.clone_from_slice(&[data[REPLY_TO_INDEX_START], data[REPLY_TO_INDEX_END]]);
        let reply_to_index: u16 = u8_to_u16(reply_to_index);
        let rt = match reply_to_index {
            REPLY_TO_MPSC_INDEX => {
                ReplyTo::Mpsc
            },
            REPLY_TO_RF_INDEX => {
                let address = &data[REPLY_TO_START..REPLY_TO_MPSC_SIZE];
                let address = bincode::deserialize(address)?;
                ReplyTo::Rf(address)
            },
            REPLY_TO_UDPIPV4_INDEX => {
                let address = &data[REPLY_TO_START..REPLY_TO_UDPIPV4_SIZE];
                let address = bincode::deserialize(address)?;
                ReplyTo::UdpIpV4(address)
            },
            REPLY_TO_UDPIPV6_INDEX => {
                let address = &data[REPLY_TO_START..REPLY_TO_UDPIPV6_SIZE];
                let address = bincode::deserialize(address)?;
                ReplyTo::UdpIpV6(address)
            },
            i => return Err(anyhow!("Deserializing ReplyTo hit an unrecognised type or variation: {}", i))
        };
        Ok(rt)
    }
    pub fn as_bytes(&self) -> Result<Vec<u8>> {
        let mut buf: Vec<u8> = vec![];
        let padding: &[u8; REPLY_TO_SIZE] = &[0u8; REPLY_TO_SIZE];
        match self {
            ReplyTo::Mpsc => {
                buf.extend_from_slice(&u16_to_u8(REPLY_TO_MPSC_INDEX));
                buf.extend_from_slice(&padding[..]);
            },
            ReplyTo::Rf(hz) => {
                buf.extend_from_slice(&u16_to_u8(REPLY_TO_RF_INDEX));
                let address = bincode::serialize(&hz)?;
                buf.extend_from_slice(&address[..]);
                buf.extend_from_slice(&padding[address.len()..]);
            }
            ReplyTo::UdpIpV4(addr) => {
                buf.extend_from_slice(&u16_to_u8(REPLY_TO_UDPIPV4_INDEX));
                let address = bincode::serialize(&addr)?;
                buf.extend_from_slice(&address[..]);
                buf.extend_from_slice(&padding[address.len()..]);
            }
            ReplyTo::UdpIpV6(addr) => {
                buf.extend_from_slice(&u16_to_u8(REPLY_TO_UDPIPV6_INDEX));
                let address = bincode::serialize(&addr)?;
                buf.extend_from_slice(&address[..]);
                buf.extend_from_slice(&padding[address.len()..]);
            }
        }
        Ok(buf)
    }
}
impl fmt::Debug for ReplyTo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ReplyTo::UdpIpV4(addr) => {
                write!(f, "ReplyTo::UdpIpV4({})", addr)
            },
            ReplyTo::UdpIpV6(addr) => {
                write!(f, "ReplyTo::UdpIpV6({})", addr)
            },
            ReplyTo::Mpsc => {
                write!(f, "ReplyTo::Mpsc")
            },
            ReplyTo::Rf(hertz) => {
                write!(f, "ReplyTo::Rf({})", hertz)
            },
        }
    }
}

use smoltcp;

use smoltcp::wire::{IpAddress, Ipv6Address, IpCidr, Ipv4Cidr, Ipv6Cidr, EthernetAddress};
use nix::ifaddrs::{InterfaceAddress, getifaddrs};
use nix::net::if_::InterfaceFlags;
use nix::sys::socket::SockAddr;

pub type Flags = InterfaceFlags;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Interface {
    name : String,
    index: u32,
    flags: Flags,
    mtu  : u32,
    hwaddr: Option<EthernetAddress>,
    dstaddr: Option<IpCidr>,
    addrs: Vec<IpCidr>,
}

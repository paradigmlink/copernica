use {
    chain_addr,
};

pub const DISCRIMINATION: chain_addr::Discrimination = chain_addr::Discrimination::Production;
pub const ADDRESS_PREFIX: &str = "ceo";
pub const HEX : [&str; 16] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "a", "b", "c", "d", "e", "f"];
/// Maximum transmission unit of the payload.
///
/// Derived from ethernet_mtu - ipv6_header_size - udp_header_size - packet header size
///       1452 = 1500         - 40               - 8               - 8
///
/// This is not strictly guaranteed -- there may be less room in an ethernet frame than this due to
/// variability in ipv6 header size.
/// Now copernica structs take up 58
/// 1394 = 1452 - 58
/// The maximum size we can break up a chunk of data is 1394
pub const FRAGMENT_SIZE: usize = 1394;

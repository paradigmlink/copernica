/// Maximum transmission unit of the payload.
///
/// Derived from ethernet_mtu - ipv6_header_size - udp_header_size
///       1452 = 1500         - 40               - 8
///
/// This is not strictly guaranteed -- there may be less room in an ethernet frame than this due to
/// variability in ipv6 header size.
/// Now copernica structs take up 72 bytes (for the moment) also sdri is variable, to be fixed
/// 1444 = 1452 - 72
/// The maximum size we can break up a chunk of data is 1394
/// Was set at 1428 but I'm setting it to 1024 so that things don't blow up
/// using datatye u16 because this information is communicated in the Response Manifest.
pub const FRAGMENT_SIZE: u16 = 1024;
pub const BLOOM_FILTER_LENGTH: u64 = u16::MAX as u64;
pub const BLOOM_FILTER_INDEX_ELEMENT_LENGTH: u16 = 4;

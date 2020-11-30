pub const PARSE_INFO_POS: usize= 0;

pub const FRAGMENT_SIZE: usize= 1024;

pub const LENGTH_OF_DATA_STARTING_POSITION: usize = FRAGMENT_SIZE-2;
pub const LENGTH_OF_DATA_ENDING_POSITION: usize = FRAGMENT_SIZE-1;
pub const DATA_SIZE: usize = FRAGMENT_SIZE-3;

pub const BLOOM_FILTER_LENGTH: usize = u16::MAX as usize;
pub const BLOOM_FILTER_INDEX_ELEMENT_LENGTH: usize = 4;
pub const NONCE_SIZE: usize = 8;
pub const TAG_SIZE: usize = 16;
pub const ID_SIZE: usize = 32;
pub const CC_SIZE: usize = 32;
pub const SIG_SIZE: usize = 64;
pub const BFI_BYTE_SIZE: usize = BLOOM_FILTER_INDEX_ELEMENT_LENGTH * 2;
pub const BFI_COUNT: usize = 6; // RES, REQ, APP, MOD, FUN, ARG
pub const U64_SIZE: usize = 8;
pub const CYPHERTEXT_HBFI_SIZE: usize = ((BLOOM_FILTER_INDEX_ELEMENT_LENGTH * 2) * 6) + U64_SIZE + (ID_SIZE * 2) + (CC_SIZE * 2);
pub const CLEARTEXT_HBFI_SIZE: usize = ((BLOOM_FILTER_INDEX_ELEMENT_LENGTH * 2) * 6) + U64_SIZE + ID_SIZE + CC_SIZE;
pub const CYPTERTEXT_RESPONSE_LENGTH: usize = 2; // a u16 encoded with 2 bytes to tell the deserializer why type of packet we're dealing with
pub const CYPHERTEXT_RESPONSE_DATA_SIZE: usize = FRAGMENT_SIZE + TAG_SIZE;
pub const CLEARTEXT_RESPONSE_DATA_SIZE: usize = FRAGMENT_SIZE;
pub const NARROW_WAIST_PACKET_ENCRYPTED_RESPONSE_SIZE: usize = CYPHERTEXT_RESPONSE_DATA_SIZE + CYPHERTEXT_HBFI_SIZE + NONCE_SIZE + (U64_SIZE*2) + SIG_SIZE + CYPTERTEXT_RESPONSE_LENGTH;
pub const TO_REPLY_TO_MPSC: usize = 0;
pub const TO_REPLY_TO_UDPIP4: usize = 10;
pub const TO_REPLY_TO_UDPIP6: usize = 22;
pub const TO_REPLY_TO_RF: usize = 4;

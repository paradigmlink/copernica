pub const FRAGMENT_SIZE: usize= 1024;

pub const LENGTH_OF_DATA_STARTING_POSITION: usize = FRAGMENT_SIZE-2;
pub const LENGTH_OF_DATA_ENDING_POSITION: usize = FRAGMENT_SIZE-1;
pub const DATA_SIZE: usize = FRAGMENT_SIZE-3;

pub const NONCE_SIZE: usize = 8;
pub const TAG_SIZE: usize = 16;

pub const BLOOM_FILTER_LENGTH: usize = u16::MAX as usize;
pub const BLOOM_FILTER_INDEX_ELEMENT_LENGTH: usize = 4;

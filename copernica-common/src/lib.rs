mod operations;
mod serialization;
pub mod log;
pub mod constants;
pub use crate::{
    operations::{Operations, LogEntry},
    log::setup_logging,
    serialization::{u16_to_u8, u8_to_u16,
    //bfi_to_u8, u8_to_bfi,
    u8_to_u64, u64_to_u8},
};

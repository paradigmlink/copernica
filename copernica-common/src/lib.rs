#![no_std]
extern crate crossbeam_channel;
extern crate log;
mod operations;
mod serialization;
pub mod constants;
pub use crate::{
    operations::{Operations, LogEntry},
    serialization::{u16_to_u8, u8_to_u16, u8_to_u64, u64_to_u8},
};

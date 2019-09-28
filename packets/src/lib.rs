#[macro_use]
extern crate serde_derive;
extern crate bincode;

pub mod packets;
pub use self::packets::{Interest, Data};

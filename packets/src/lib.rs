#[macro_use]
extern crate serde_derive;
extern crate bincode;
extern crate base64;
extern crate sha3;

#[cfg(test)]
extern crate tar;
#[cfg(test)]
extern crate flate2;
#[cfg(test)]
extern crate bitvec;

pub mod interest;
pub mod data;
mod index;

use crate::{index::forwarding_hint};

#[derive(Debug, Clone, PartialEq)]
pub enum Packet {
    Interest { name: String, sdri: Vec<Vec<u16>> },
    Data     { name: String, sdri: Vec<Vec<u16>> },
}

pub fn mk_interest(name: String) -> Packet {
    Packet::Interest {
        name: name.clone(),
        sdri: forwarding_hint(name)
        // more to come
    }
}

pub fn mk_data(name: String) -> Packet {
    Packet::Data {
        name: name.clone(),
        sdri: forwarding_hint(name)
        // more to come
    }
}

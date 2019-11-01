#[macro_use]
extern crate serde_derive;
extern crate sha3;

#[cfg(test)]
extern crate tar;
#[cfg(test)]
extern crate flate2;
#[cfg(test)]
extern crate bitvec;

mod index;

pub use crate::{index::generate_sdr_index};
use std::fmt;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum Packet {
    Request     { sdri: Vec<Vec<u16>> },
    Response    { sdri: Vec<Vec<u16>>, data: Vec<u8> },
}

pub fn request(name: String) -> Packet {
    Packet::Request {
        sdri: generate_sdr_index(name)
        // more to come
    }
}

pub fn response(name: String, data: Vec<u8>) -> Packet {
    Packet::Response {
        sdri: generate_sdr_index(name),
        data: data,
    }
}

impl fmt::Debug for Packet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            Packet::Request{sdri: sdri} => write!(f, "REQ{:?}", sdri),
            Packet::Response{sdri: sdri, data:_} => write!(f, "RES{:?}", sdri),
        }
    }
}

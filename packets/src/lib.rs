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
use std::collections::HashMap;

pub type Sdri = Vec<Vec<u16>>;
pub type Data = Vec<u8>;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum Packet {
    Request     { sdri: Sdri },
    Response    { sdri: Sdri, data: Data },
}

pub fn request(name: String) -> Packet {
    Packet::Request {
        sdri: generate_sdr_index(name)
        // more to come
    }
}

pub fn response(name: String, data: Data) -> Packet {
    Packet::Response {
        sdri: generate_sdr_index(name),
        data,
    }
}

pub fn mk_response(name: String, data: Data) -> HashMap<String, Packet> {
    let safe_mtu: usize = 1024;
    let mut out: HashMap<String, Packet> = HashMap::new();
    if data.len() <= safe_mtu {
        out.insert(name.clone(), response(name, data));
    } else {
        let chunks = data.chunks(safe_mtu);
        let sequence: String = format!("{}\n{}", name.clone(), chunks.len() - 1);
        let mut count: usize = 0;
        out.insert(name.clone(), response(name.clone(), sequence.as_bytes().to_vec()));
        for chunk in chunks {
            let chunk_name: String = format!("{}-{}", name.clone(), count);
            out.insert(chunk_name.clone(), response(chunk_name, chunk.to_vec()));
            count += 1;
        }
    }
    out
}

impl fmt::Debug for Packet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            Packet::Request{sdri} => write!(f, "REQ{:?}", sdri),
            Packet::Response{sdri, data} => write!(f, "RES name: {:?} data: {:?}", sdri, data),
        }
    }
}

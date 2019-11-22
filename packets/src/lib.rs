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

pub fn mk_response(name: String, data: Data) -> Vec<(String, Packet)> {
    let chunks = data.chunks(1024);
    let mut out: Vec<(String, Packet)> = vec![];
    let sequence: String = format!("{}\n{}", name.clone(), chunks.len() - 1);
    //println!("sequence: {}", sequence.clone());
    let mut count: usize = 0;
    out.push((name.clone(), response(name.clone(), sequence.as_bytes().to_vec())));
    for chunk in chunks {
        let chunk_name: String = format!("{}/{}", name.clone(), count);
        //println!("chunk name: {}", chunk_name.clone());
        out.push((chunk_name.clone(), response(chunk_name, chunk.to_vec())));
        count += 1;
    }
    out
}

impl fmt::Debug for Packet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            Packet::Request{sdri} => write!(f, "REQ{:?}", sdri),
            Packet::Response{sdri, ..} => write!(f, "RES{:?}", sdri),
        }
    }
}

#[cfg(test)]
mod responses {
    use super::*;

    #[test]
    fn make_chunks() {
        let len = 1025;
        let zero_vec = vec![0; len];
        let responses = mk_response("ThePiratesOfTheCaribbean".to_string(), zero_vec);
        assert_eq!( vec![
            ("ThePiratesOfTheCaribbean".to_string(), Packet::Response {
                sdri: vec![vec![806, 1462, 1766]],
                data: "ThePiratesOfTheCaribbean\n1".as_bytes().to_vec(),
            }),
            ("ThePiratesOfTheCaribbean/0".to_string(), Packet::Response {
                sdri: vec![vec![806, 1462, 1766], vec![2, 482, 1250]],
                data: vec![0; 1024],
            }),
            ("ThePiratesOfTheCaribbean/1".to_string(), Packet::Response {
                sdri: vec![vec![806, 1462, 1766], vec![71, 1415, 1687]],
                data: vec![0; 1],
            }),
            ] , responses);
    }
}


use {
    std::{
        fmt,
    },
    crate::{
        sdri::{Sdri},
    }
};

pub type Bytes = Vec<u8>;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum Packet {
    Request     { sdri: Sdri },
    Response    { sdri: Sdri, data: Bytes, numerator: u64, denominator: u64 },
}

pub fn mk_request_packet(name: String) -> Packet {
    Packet::Request {
        sdri: Sdri::new(name)
    }
}

pub fn mk_response_packet(name: String, data: Bytes, numerator: u64, denominator: u64) -> Packet {
    Packet::Response {
        sdri: Sdri::new(name),
        data,
        numerator,
        denominator,
    }
}

impl fmt::Debug for Packet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            Packet::Request{sdri} => write!(f, "REQ{:?}", sdri),
            Packet::Response{sdri, numerator, denominator, ..} =>
                write!(f, "RES{:?} {}/{}", sdri, numerator+1, denominator)
        }
    }
}


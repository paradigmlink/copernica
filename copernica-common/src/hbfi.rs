use {
    crate::{constants},
    anyhow::Result,
    serde::{Deserialize, Serialize},
    sha3::{Digest, Sha3_512},
    std::fmt,
    keynesis::{PublicIdentity},
};

pub type BFI = [u16; constants::BLOOM_FILTER_INDEX_ELEMENT_LENGTH as usize]; // Bloom Filter Index

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HBFI {
    // Hierarchical Bloom Filter Index
    pub response_pid: PublicIdentity,
    pub request_pid: Option<PublicIdentity>,
    pub res: BFI, // response PublicIdentity
    pub req: BFI, // request PublicIdentity, when set indicates Response will be encrypted.
    pub app: BFI, // Application
    pub m0d: BFI, // Module
    pub fun: BFI, // Function
    pub arg: BFI, // Argument
    pub ost: u64,
}

impl HBFI {
    pub fn new(response_pid: PublicIdentity
        , request_pid: Option<PublicIdentity>
        , app: &str
        , m0d: &str
        , fun: &str
        , arg: &str
    ) -> Result<HBFI> {
        let req = match request_pid.clone() {
            Some(request_pid) => {
                bloom_filter_index(&format!("{}", request_pid))?
            },
            None => [0; constants::BLOOM_FILTER_INDEX_ELEMENT_LENGTH],
        };
        Ok(HBFI {
            response_pid: response_pid.clone(),
            request_pid,
            res: bloom_filter_index(&format!("{}", response_pid))?,
            req,
            app: bloom_filter_index(app)?,
            m0d: bloom_filter_index(m0d)?,
            fun: bloom_filter_index(fun)?,
            arg: bloom_filter_index(arg)?,
            ost: 0,
        })
    }
    pub fn to_vec(&self) -> Vec<BFI> {
        vec![ self.req.clone()
            , self.res.clone()
            , self.app.clone()
            , self.m0d.clone()
            , self.fun.clone()
            , self.arg.clone()
        ]
    }
    pub fn offset(mut self, ost: u64) -> Self {
        self.ost = ost;
        self
    }
}

impl fmt::Debug for HBFI {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            HBFI { request_pid, response_pid, res, req, app, m0d, fun, arg, ost } =>
            write!(f, "req_pid:{:?},res_pid:{:?},req:{:?},res:{:?},app:{:?},m0d:{:?},fun:{:?},arg:{:?},ost:{:?}", request_pid, response_pid, req, res, app, m0d, fun, arg, ost),
        }
    }
}

impl fmt::Display for HBFI {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            HBFI { request_pid, response_pid, res, req, app, m0d, fun, arg, ost } =>
            write!(f, "req_pid:{:?},res_pid:{:?},req:{:?},res:{:?},app:{:?},m0d:{:?},fun:{:?},arg:{:?},ost:{:?}", request_pid, response_pid, req, res, app, m0d, fun, arg, ost),
        }
    }
}

pub fn bloom_filter_index(
    s: &str,
) -> Result<[u16; constants::BLOOM_FILTER_INDEX_ELEMENT_LENGTH as usize]> {
    use std::str;
    let mut hasher = Sha3_512::new();
    hasher.input(s.as_bytes());
    let hash = hasher.result();
    let mut bloom_filter_index_array: BFI =
        [0; constants::BLOOM_FILTER_INDEX_ELEMENT_LENGTH as usize];
    let mut count = 0;
    for n in 0..constants::BLOOM_FILTER_INDEX_ELEMENT_LENGTH {
        let mut hasher = Sha3_512::new();
        hasher.input(format!("{:x}{}", hash, n));
        let hs = format!("{:x}", hasher.result());
        let subs = hs
            .as_bytes()
            .chunks(16)
            .map(str::from_utf8)
            .collect::<Result<Vec<&str>, _>>()?;
        let mut index: u64 = 0;
        for sub in subs {
            let o = u64::from_str_radix(&sub, 16)?;
            index = (index + o) % constants::BLOOM_FILTER_LENGTH as u64;
        }
        bloom_filter_index_array[count] = index as u16;
        count += 1;
    }
    Ok(bloom_filter_index_array)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        packets::{NarrowWaistPacket, LinkPacket},
        link::{ReplyTo},
        generate_nonce,
    };
    use keynesis::{PrivateIdentity, Seed};

    #[test]
    fn test_bloom_filter_index() {
        let actual = bloom_filter_index("9".into()).unwrap();
        let expected: [u16; constants::BLOOM_FILTER_INDEX_ELEMENT_LENGTH as usize] =
            [4804, 63297, 3290, 20147];
        assert_eq!(actual, expected);
    }

    #[test]
    fn less_than_mtu() {
        // https://gafferongames.com/post/packet_fragmentation_and_reassembly
        let mut rng = rand::thread_rng();
        let response_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));
        let response_pid = response_sid.public_id();
        let hbfi = HBFI::new(response_pid, None, "app", "m0d", "fun", "arg").unwrap();
        let data = vec![0; 600];
        let offset = u64::MAX;
        let total = u64::MAX;
        let nw: NarrowWaistPacket = NarrowWaistPacket::response(response_sid, hbfi, data, offset, total).unwrap();
        let reply_to: ReplyTo = ReplyTo::UdpIp("127.0.0.1:50000".parse().unwrap());
        let wp: LinkPacket = LinkPacket { reply_to, nw };
        let wp_ser = bincode::serialize(&wp).unwrap();
        let wp_ser_len = wp_ser.len();
        println!("must be less than 1472, current length: {}", wp_ser_len);
        let lt1472 = if wp_ser_len <= 1472 { true } else { false };
        assert_eq!(true, lt1472);
    }
}

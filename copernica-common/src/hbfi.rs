use {
    crate::{constants, PublicIdentity},
    anyhow::{Result},
    serde::{Deserialize, Serialize},
    std::fmt,
    core::hash::{Hash, Hasher}
};

pub type BFI = [u16; constants::BLOOM_FILTER_INDEX_ELEMENT_LENGTH]; // Bloom Filter Index
pub type BFIS = [BFI; constants::BFI_COUNT]; // Bloom Filter Index

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct HBFI {
    // Hierarchical Bloom Filter Index
    pub request_pid: Option<PublicIdentity>,
    pub response_pid: PublicIdentity,
    pub req: BFI, // request PublicIdentity, when set indicates Response will be encrypted.
    pub res: BFI, // response PublicIdentity
    pub app: BFI, // Application
    pub m0d: BFI, // Module
    pub fun: BFI, // Function
    pub arg: BFI, // Argument
    pub ost: u64, // Offset: current 1024 byte chunk of data in a range.
}
pub struct HBFIWithoutFrame(HBFI);
impl HBFIWithoutFrame {
    pub fn new(hbfi: HBFI) -> Self {
        Self(hbfi)
    }
}
impl Hash for HBFIWithoutFrame {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.request_pid.hash(state);
        self.0.response_pid.hash(state);
        self.0.req.hash(state);
        self.0.res.hash(state);
        self.0.app.hash(state);
        self.0.m0d.hash(state);
        self.0.fun.hash(state);
        self.0.arg.hash(state)
    }
}
impl PartialEq for HBFIWithoutFrame {
    fn eq(&self, other: &Self) -> bool {
        (self.0.request_pid == other.0.request_pid)
        && (self.0.response_pid == other.0.response_pid)
        && (self.0.req == other.0.req)
        && (self.0.res == other.0.res)
        && (self.0.app == other.0.app)
        && (self.0.m0d == other.0.m0d)
        && (self.0.fun == other.0.fun)
        && (self.0.arg == other.0.arg)
    }
}
impl Eq for HBFIWithoutFrame {}
impl HBFI {
    pub fn new(request_pid: Option<PublicIdentity>
        ,response_pid: PublicIdentity
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
            request_pid,
            response_pid: response_pid.clone(),
            req,
            res: bloom_filter_index(&format!("{}", response_pid))?,
            app: bloom_filter_index(app)?,
            m0d: bloom_filter_index(m0d)?,
            fun: bloom_filter_index(fun)?,
            arg: bloom_filter_index(arg)?,
            ost: 0,
        })
    }
/*    pub fn to_vec(&self) -> Vec<BFI> {
        vec![ self.req.clone()
            , self.res.clone()
            , self.app.clone()
            , self.m0d.clone()
            , self.fun.clone()
            , self.arg.clone()
        ]
    }*/
    pub fn to_bfis(&self) -> BFIS {
        [ self.req.clone()
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

impl HBFI {
    pub fn encrypt_for(&self, request_pid: PublicIdentity) -> Result<Self> {
        let req = bloom_filter_index(&format!("{}", request_pid))?;
        Ok(HBFI { request_pid: Some(request_pid)
            , response_pid: self.response_pid.clone()
            , req
            , res: self.res.clone()
            , app: self.app.clone()
            , m0d: self.m0d.clone()
            , fun: self.fun.clone()
            , arg: self.arg.clone()
            , ost: self.ost.clone()
        })
    }
    pub fn cleartext_repr(&self) -> Self {
        HBFI { request_pid: None
            , response_pid: self.response_pid.clone()
            , req: [0u16; constants::BLOOM_FILTER_INDEX_ELEMENT_LENGTH]
            , res: self.res.clone()
            , app: self.app.clone()
            , m0d: self.m0d.clone()
            , fun: self.fun.clone()
            , arg: self.arg.clone()
            , ost: self.ost.clone()
        }
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
    use cryptoxide::digest::Digest as _;
    let mut hash_orig = [0; 32];
    let mut b = cryptoxide::blake2b::Blake2b::new(32);
    b.input(&s.as_bytes());
    b.result(&mut hash_orig);
    let mut bloom_filter_index_array: BFI =
        [0; constants::BLOOM_FILTER_INDEX_ELEMENT_LENGTH as usize];
    let mut count = 0;
    for n in 0..constants::BLOOM_FILTER_INDEX_ELEMENT_LENGTH {
        let mut hash_derived = [0; 32];
        let mut b = cryptoxide::blake2b::Blake2b::new(32);
        let mut s: String = "".into();
        for byte in hash_orig.iter() {
            s.push_str(format!("{:x}", byte).as_str());
        }
        s.push_str(format!("{}", n).as_str());
        b.input(&s.as_bytes());
        b.result(&mut hash_derived);
        s = "".into();
        for byte in hash_derived.iter() {
            s.push_str(format!("{:x}", byte).as_str());
        }
        let subs = s
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

    #[test]
    fn test_bloom_filter_index() {
        let actual = bloom_filter_index("9".into()).unwrap();
        let expected: [u16; constants::BLOOM_FILTER_INDEX_ELEMENT_LENGTH as usize] =
            [19283, 50425, 20212, 47266];
        assert_eq!(actual, expected);
    }
}

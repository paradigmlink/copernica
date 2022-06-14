use {
    crate::{ PublicIdentity, PublicIdentityInterface },
    copernica_common::{constants::*, u8_to_u64, u16_to_u8, u64_to_u8 },
    anyhow::{Result, anyhow},
    std::fmt,
    core::hash::{Hash}
};
pub fn bloom_filter_index(
    s: &str,
) -> Result<BFI> {
    use std::str;
    use cryptoxide::digest::Digest as _;
    let mut hash_orig = [0; 32];
    let mut b = cryptoxide::blake2b::Blake2b::new(32);
    b.input(&s.as_bytes());
    b.result(&mut hash_orig);
    let mut bloom_filter_index_array: BFI = BFI::new();
    let mut count = 0;
    for n in 0..BLOOM_FILTER_INDEX_ELEMENT_LENGTH {
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
            index = (index + o) % BLOOM_FILTER_LENGTH as u64;
        }
        bloom_filter_index_array.0[count] = index as u16;
        count += 1;
    }
    Ok(bloom_filter_index_array)
}
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
pub struct BFI(pub [u16; BLOOM_FILTER_INDEX_ELEMENT_LENGTH]);
impl BFI {
    pub fn new() -> Self {
        Self([0; BLOOM_FILTER_INDEX_ELEMENT_LENGTH as usize])
    }
    pub fn to_bytes(&self) -> [u8; BFI_BYTE_SIZE] {
        let mut bbfi: [u8; BFI_BYTE_SIZE] = [0; BFI_BYTE_SIZE];
        let mut count = 0;
        for i in self.0.iter() {
            let two_u8 = u16_to_u8(*i);
            bbfi[count]   = two_u8[0];
            bbfi[count+1] = two_u8[1];
            count+=2;
        }
        bbfi
    }
    pub fn from_bytes(bbfi: [u8; BFI_BYTE_SIZE]) -> Self {
        Self([((bbfi[0] as u16) << 8) | bbfi[1] as u16,
        ((bbfi[2]  as u16) << 8) | bbfi[3] as u16,
        ((bbfi[4]  as u16) << 8) | bbfi[5] as u16,
        ((bbfi[6]  as u16) << 8) | bbfi[7] as u16])
    }
}
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
pub struct BFIS(pub [BFI; BFI_COUNT]);
impl BFIS {
    pub fn new(bfis: [BFI; BFI_COUNT]) -> Self {
        Self(bfis)
    }
}
#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct HBFI {
    // Hierarchical Bloom Filter Index
    pub request_pid: PublicIdentityInterface,
    pub response_pid: PublicIdentity,
    pub req: BFI, // request PublicIdentity, when set indicates Response will be encrypted.
    pub res: BFI, // response PublicIdentity
    pub app: BFI, // Application
    pub m0d: BFI, // Module
    pub fun: BFI, // Function
    pub arg: BFI, // Argument
    pub frm: u64, // Frame Count: current 1024 byte chunk of data in a range.
}
impl HBFI {
    pub fn new(request_pid: PublicIdentityInterface
        ,response_pid: PublicIdentity
        , app: &str
        , m0d: &str
        , fun: &str
        , arg: &str
    ) -> Result<HBFI> {
        Ok(HBFI {
            request_pid: request_pid.clone(),
            response_pid: response_pid.clone(),
            req: request_pid.bloom_filter_index()?,
            res: bloom_filter_index(&format!("{}", response_pid))?,
            app: bloom_filter_index(app)?,
            m0d: bloom_filter_index(m0d)?,
            fun: bloom_filter_index(fun)?,
            arg: bloom_filter_index(arg)?,
            frm: 0,
        })
    }
    pub fn to_bfis(&self) -> BFIS {
        BFIS::new([ self.req.clone()
        , self.res.clone()
        , self.app.clone()
        , self.m0d.clone()
        , self.fun.clone()
        , self.arg.clone()
        ])
    }
    pub fn offset(mut self, frm: u64) -> Self {
        self.frm = frm;
        self
    }
    pub fn encrypt_for(&self, request_pid: PublicIdentityInterface) -> Result<Self> {
        Ok(HBFI { request_pid: request_pid.clone()
            , response_pid: self.response_pid.clone()
            , req: request_pid.bloom_filter_index()?
            , res: self.res.clone()
            , app: self.app.clone()
            , m0d: self.m0d.clone()
            , fun: self.fun.clone()
            , arg: self.arg.clone()
            , frm: self.frm.clone()
        })
    }
    pub fn cleartext_repr(&self) -> Result<Self> {
        let absent_request_pid = PublicIdentityInterface::Absent;
        Ok(HBFI { request_pid: absent_request_pid.clone()
            , response_pid: self.response_pid.clone()
            , req: absent_request_pid.bloom_filter_index()?
            , res: self.res.clone()
            , app: self.app.clone()
            , m0d: self.m0d.clone()
            , fun: self.fun.clone()
            , arg: self.arg.clone()
            , frm: self.frm.clone()
        })
    }
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        match data.len() {
          CYPHERTEXT_HBFI_SIZE => {
              let mut bfis: Vec<BFI> = Vec::with_capacity(BFI_COUNT);
              let mut count = 0;
              for _ in 0..BFI_COUNT {
                  let mut bbfi = [0u8; BFI_BYTE_SIZE];
                  bbfi.clone_from_slice(&data[count..count+BFI_BYTE_SIZE]);
                  bfis.push(BFI::from_bytes(bbfi));
                  count += BFI_BYTE_SIZE;
              }
              let mut frm = [0u8; FRAME_SIZE];
              frm.clone_from_slice(&data[HBFI_FRAME_START..HBFI_FRAME_END]);
              let frm: u64 = u8_to_u64(frm);
              let mut res_key = [0u8; ID_SIZE + CC_SIZE];
              res_key.clone_from_slice(&data[HBFI_RESPONSE_KEY_START..HBFI_RESPONSE_KEY_END]);
              let mut req_key = [0u8; ID_SIZE + CC_SIZE];
              req_key.clone_from_slice(&data[HBFI_REQUEST_KEY_START..HBFI_REQUEST_KEY_END]);
              Ok(HBFI { response_pid: PublicIdentity::from(res_key)
                      , request_pid: PublicIdentityInterface::new(PublicIdentity::from(req_key))
                      , res: bfis[0].clone()
                      , req: bfis[1].clone()
                      , app: bfis[2].clone()
                      , m0d: bfis[3].clone()
                      , fun: bfis[4].clone()
                      , arg: bfis[5].clone()
                      , frm})
          },
          CLEARTEXT_HBFI_SIZE => {
              let mut bfis: Vec<BFI> = Vec::with_capacity(BFI_COUNT);
              let mut count = 0;
              for _ in 0..BFI_COUNT {
                  let mut bbfi = [0u8; BFI_BYTE_SIZE];
                  bbfi.clone_from_slice(&data[count..count+BFI_BYTE_SIZE]);
                  bfis.push(BFI::from_bytes(bbfi));
                  count += BFI_BYTE_SIZE;
              }
              let mut frm = [0u8; FRAME_SIZE];
              frm.clone_from_slice(&data[HBFI_FRAME_START..HBFI_FRAME_END]);
              let frm: u64 = u8_to_u64(frm);
              let mut res_key = [0u8; ID_SIZE + CC_SIZE];
              res_key.clone_from_slice(&data[HBFI_RESPONSE_KEY_START..HBFI_RESPONSE_KEY_END]);
              Ok(HBFI { response_pid: PublicIdentity::from(res_key)
                      , request_pid: PublicIdentityInterface::Absent
                      , res: bfis[0].clone()
                      , req: bfis[1].clone()
                      , app: bfis[2].clone()
                      , m0d: bfis[3].clone()
                      , fun: bfis[4].clone()
                      , arg: bfis[5].clone()
                      , frm})
          }
            _ => Err(anyhow!("Length of data used to reconstruct a HBFI is unrecognised")),
        }

    }
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = vec![];
        let res = self.res.to_bytes();
        let req = self.req.to_bytes();
        let app = self.app.to_bytes();
        let m0d = self.m0d.to_bytes();
        let fun = self.fun.to_bytes();
        let arg = self.arg.to_bytes();
        let frm = &u64_to_u8(self.frm);
        let mut ids_buf: Vec<u8> = vec![];
        match &self.request_pid {
            PublicIdentityInterface::Present { public_identity } => {
                ids_buf.extend_from_slice(self.response_pid.key().as_ref());
                ids_buf.extend_from_slice(self.response_pid.chain_code().as_ref());
                ids_buf.extend_from_slice(public_identity.key().as_ref());
                ids_buf.extend_from_slice(public_identity.chain_code().as_ref());
            },
            PublicIdentityInterface::Absent => {
                ids_buf.extend_from_slice(self.response_pid.key().as_ref());
                ids_buf.extend_from_slice(self.response_pid.chain_code().as_ref());
            },
        }
        buf.extend_from_slice(&res);
        buf.extend_from_slice(&req);
        buf.extend_from_slice(&app);
        buf.extend_from_slice(&m0d);
        buf.extend_from_slice(&fun);
        buf.extend_from_slice(&arg);
        buf.extend_from_slice(frm);
        buf.extend_from_slice(&ids_buf);
        buf
    }
}
impl fmt::Debug for HBFI {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            HBFI { request_pid, response_pid, res, req, app, m0d, fun, arg, frm } =>
            write!(f, "req_pid:{:?},res_pid:{:?},req:{:?},res:{:?},app:{:?},m0d:{:?},fun:{:?},arg:{:?},frm:{:?}", request_pid, response_pid, req, res, app, m0d, fun, arg, frm),
        }
    }
}
impl fmt::Display for HBFI {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            HBFI { request_pid, response_pid, res, req, app, m0d, fun, arg, frm } =>
            write!(f, "req_pid:{:?},res_pid:{:?},req:{:?},res:{:?},app:{:?},m0d:{:?},fun:{:?},arg:{:?},frm:{:?}", request_pid, response_pid, req, res, app, m0d, fun, arg, frm),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bloom_filter_index() {
        let actual = bloom_filter_index("9".into()).unwrap();
        let expected = BFI([19283, 50425, 20212, 47266]);
        assert_eq!(actual, expected);
    }
}

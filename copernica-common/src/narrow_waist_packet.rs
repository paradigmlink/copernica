use {
    crate::{
        hbfi::HBFI,
        manifest, generate_nonce,
        ResponseData, Nonce,
        PrivateIdentityInterface,
        Signature
    },
    core::hash::{Hash, Hasher},
    std::{
        cmp::Ordering,
        fmt,
    },
    anyhow::{anyhow, Result},
    log::{error},
};
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum NarrowWaistPacket {
    Request {
        hbfi: HBFI,
        nonce: Nonce,
    },
    Response {
        hbfi: HBFI,
        nonce: Nonce,
        signature: Signature,
        data: ResponseData,
    },
}
impl NarrowWaistPacket {
    pub fn request(hbfi: HBFI) -> Result<Self> {
        let mut rng = rand::thread_rng();
        let nonce: Nonce = generate_nonce(&mut rng);
        Ok(NarrowWaistPacket::Request { hbfi, nonce })
    }
    pub fn response(response_sid: PrivateIdentityInterface, hbfi: HBFI, data: Vec<u8>) -> Result<Self> {
        if hbfi.response_pid != response_sid.public_id() {
            let msg = "The Request's Response Public Identity doesn't match the Public Identity used to sign or encypt the Response";
            error!("{}", msg);
            return Err(anyhow!(msg));
        }
        let mut rng = rand::thread_rng();
        let hbfi = hbfi.clone();
        let nonce: Nonce = generate_nonce(&mut rng);
        let data = ResponseData::insert(response_sid.clone(), hbfi.request_pid.clone(), data, nonce.clone())?;
        let manifest = manifest(data.manifest_data(), &hbfi, &nonce)?;
        let response_signkey = response_sid.signing_key();
        let signature = response_signkey.sign(manifest);
        Ok(NarrowWaistPacket::Response { hbfi, nonce, data, signature })
    }
    pub fn verify(&self) -> Result<bool> {
        match self {
            NarrowWaistPacket::Request {..} => {
                return Ok(true)
            },
            NarrowWaistPacket::Response { data, hbfi, signature, nonce} => {
                let manifest = manifest(data.manifest_data(), hbfi, nonce)?;
                let verify_key = hbfi.response_pid.verify_key()?;
                let verified = verify_key.verify(&signature, manifest);
                return Ok(verified);
            },
        }
    }
    pub fn data(&self, request_sid: PrivateIdentityInterface) -> Result<Vec<u8>> {
        match self {
            NarrowWaistPacket::Request {..} => {
                let err_msg = "No data in a NarrowWaistPacket::Request";
                error!("{}", err_msg);
                return Err(anyhow!(err_msg))
            },
            NarrowWaistPacket::Response { data, hbfi, nonce, ..}=> {
                return Ok(data.extract(request_sid, hbfi.request_pid.clone(), hbfi.response_pid.clone(), nonce.clone())?)

            },
        }
    }
}
#[derive(Clone)]
pub struct NarrowWaistPacketReqEqRes(pub NarrowWaistPacket);
impl Hash for NarrowWaistPacketReqEqRes {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match &self.0 {
            NarrowWaistPacket::Request { hbfi, .. } => { hbfi.hash(state) },
            NarrowWaistPacket::Response { hbfi, .. } => { hbfi.hash(state) }
        }
    }
}
impl PartialOrd for NarrowWaistPacketReqEqRes {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let self_hbfi = match &self.0 {
            NarrowWaistPacket::Request { hbfi, .. } => { hbfi },
            NarrowWaistPacket::Response { hbfi, .. } => { hbfi }
        };
        let other_hbfi = match &other.0 {
            NarrowWaistPacket::Request { hbfi, .. } => { hbfi },
            NarrowWaistPacket::Response { hbfi, .. } => { hbfi }
        };
        Some(self_hbfi.cmp(other_hbfi))
    }
}

impl Ord for NarrowWaistPacketReqEqRes {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_hbfi = match &self.0 {
            NarrowWaistPacket::Request { hbfi, .. } => { hbfi },
            NarrowWaistPacket::Response { hbfi, .. } => { hbfi }
        };
        let other_hbfi = match &other.0 {
            NarrowWaistPacket::Request { hbfi, .. } => { hbfi },
            NarrowWaistPacket::Response { hbfi, .. } => { hbfi }
        };
        self_hbfi.frm.cmp(&other_hbfi.frm)
    }
}
impl PartialEq for NarrowWaistPacketReqEqRes {
    fn eq(&self, other: &Self) -> bool {
        let self_hbfi = match &self.0 {
            NarrowWaistPacket::Request { hbfi, .. } => { hbfi },
            NarrowWaistPacket::Response { hbfi, .. } => { hbfi }
        };
        let other_hbfi = match &other.0 {
            NarrowWaistPacket::Request { hbfi, .. } => { hbfi },
            NarrowWaistPacket::Response { hbfi, .. } => { hbfi }
        };
        self_hbfi == other_hbfi
    }
}
impl Eq for NarrowWaistPacketReqEqRes {}
impl fmt::Debug for NarrowWaistPacketReqEqRes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.0 {
            NarrowWaistPacket::Request  { hbfi, .. } => write!(f, "NWEQ REQ {:?}", hbfi),
            NarrowWaistPacket::Response { hbfi, .. } => write!(f, "NWEQ RES {:?}", hbfi),
        }
    }
}
impl fmt::Debug for NarrowWaistPacket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            NarrowWaistPacket::Request { hbfi, .. } => write!(f, "NW REQ {:?}", hbfi),
            NarrowWaistPacket::Response {
                hbfi,
                signature,
                nonce,
                ..
            } => write!(f, "NW RES {:?} {} {:?}", hbfi, signature, nonce),
        }
    }
}

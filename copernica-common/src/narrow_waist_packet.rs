use {
    crate::{
        hbfi::HBFI,
        ResponseData, Nonce,
        PrivateIdentityInterface,
        PublicIdentityInterface,
        Signature,
        constants::*,
    },
    core::hash::{Hash},
    std::{
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
        let nonce: Nonce = Nonce::new();
        Ok(NarrowWaistPacket::Request { hbfi, nonce })
    }
    pub fn response(response_sid: PrivateIdentityInterface, hbfi: HBFI, data: Vec<u8>) -> Result<Self> {
        if hbfi.response_pid != response_sid.public_id() {
            let msg = "The Request's Response Public Identity doesn't match the Public Identity used to sign or encypt the Response";
            error!("{}", msg);
            return Err(anyhow!(msg));
        }
        let hbfi = hbfi.clone();
        let nonce: Nonce = Nonce::new();
        let data = ResponseData::insert(response_sid.clone(), hbfi.request_pid.clone(), data, nonce.clone())?;
        let manifest = manifest(&data, &hbfi, &nonce);
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
                let manifest = manifest(&data, &hbfi, &nonce);
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
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = vec![];
        match self {
            NarrowWaistPacket::Request { hbfi, nonce } => {
                match hbfi.request_pid {
                    PublicIdentityInterface::Absent => {
                        buf.extend_from_slice(&[CLEARTEXT_NARROW_WAIST_PACKET_REQUEST_INDEX]);
                    },
                    PublicIdentityInterface::Present { .. } => {
                        buf.extend_from_slice(&[CYPHERTEXT_NARROW_WAIST_PACKET_REQUEST_INDEX]);
                    }
                }
                buf.extend_from_slice(&nonce.as_bytes());
                buf.extend_from_slice(&hbfi.as_bytes());
            },
            NarrowWaistPacket::Response { hbfi, signature, nonce, data } => {
                match hbfi.request_pid {
                    PublicIdentityInterface::Absent => {
                        buf.extend_from_slice(&[CLEARTEXT_NARROW_WAIST_PACKET_RESPONSE_INDEX]);
                    },
                    PublicIdentityInterface::Present { .. } => {
                        buf.extend_from_slice(&[CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_INDEX]);
                    }
                }
                buf.extend_from_slice(signature.as_ref());
                buf.extend_from_slice(&nonce.as_bytes());
                buf.extend_from_slice(&hbfi.as_bytes());
                buf.extend_from_slice(&data.as_bytes());
            },
        }
        buf
    }
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        let nw_index = data[0];
        let nw: NarrowWaistPacket = match nw_index {
            CLEARTEXT_NARROW_WAIST_PACKET_REQUEST_INDEX => {
                let nonce = Nonce::from_bytes(&data[CLEARTEXT_NARROW_WAIST_PACKET_REQUEST_NONCE_START..CLEARTEXT_NARROW_WAIST_PACKET_REQUEST_NONCE_END]);
                let hbfi: HBFI = HBFI::from_bytes(&data[CLEARTEXT_NARROW_WAIST_PACKET_REQUEST_HBFI_START..CLEARTEXT_NARROW_WAIST_PACKET_REQUEST_HBFI_END])?;
                NarrowWaistPacket::Request { hbfi, nonce }
            },
            CYPHERTEXT_NARROW_WAIST_PACKET_REQUEST_INDEX => {
                let nonce = Nonce::from_bytes(&data[CYPHERTEXT_NARROW_WAIST_PACKET_REQUEST_NONCE_START..CYPHERTEXT_NARROW_WAIST_PACKET_REQUEST_NONCE_END]);
                let hbfi: HBFI = HBFI::from_bytes(&data[CYPHERTEXT_NARROW_WAIST_PACKET_REQUEST_HBFI_START..CYPHERTEXT_NARROW_WAIST_PACKET_REQUEST_HBFI_END])?;
                NarrowWaistPacket::Request { hbfi, nonce }
            },
            CLEARTEXT_NARROW_WAIST_PACKET_RESPONSE_INDEX => {
                let mut signature = [0u8; Signature::SIZE];
                signature.clone_from_slice(&data[CLEARTEXT_NARROW_WAIST_PACKET_RESPONSE_SIG_START..CLEARTEXT_NARROW_WAIST_PACKET_RESPONSE_SIG_END]);
                let signature: Signature = Signature::from(signature);
                let nonce = Nonce::from_bytes(&data[CLEARTEXT_NARROW_WAIST_PACKET_RESPONSE_NONCE_START..CLEARTEXT_NARROW_WAIST_PACKET_RESPONSE_NONCE_END]);
                let hbfi: HBFI = HBFI::from_bytes(&data[CLEARTEXT_NARROW_WAIST_PACKET_RESPONSE_HBFI_START..CLEARTEXT_NARROW_WAIST_PACKET_RESPONSE_HBFI_END])?;
                let data: ResponseData = ResponseData::from_bytes(&data[CLEARTEXT_NARROW_WAIST_PACKET_RESPONSE_DATA_START..CLEARTEXT_NARROW_WAIST_PACKET_RESPONSE_DATA_END])?;
                NarrowWaistPacket::Response { hbfi, signature, nonce, data }
            },
            CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_INDEX => {
                let mut signature = [0u8; Signature::SIZE];
                signature.clone_from_slice(&data[CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_SIG_START..CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_SIG_END]);
                let signature: Signature = Signature::from(signature);
                let nonce = Nonce::from_bytes(&data[CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_NONCE_START..CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_NONCE_END]);
                let hbfi: HBFI = HBFI::from_bytes(&data[CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_HBFI_START..CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_HBFI_END])?;
                let data: ResponseData = ResponseData::from_bytes(&data[CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_DATA_START..CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_DATA_END])?;
                NarrowWaistPacket::Response { hbfi, signature, nonce, data }
            },
            _ => {
                let msg = format!("Index used in the NarrowWaistPacket is unrecognized");
                error!("{}", msg);
                return Err(anyhow!(msg));
            },
        };
        if !nw.verify()? {
            let err_msg = "NarrowWaistPacket::from_cleartext_bytes signature check failed";
            error!("{}", err_msg);
            return Err(anyhow!(err_msg))
        }
        Ok(nw)
    }
}
fn manifest(data: &ResponseData, hbfi: &HBFI, nonce: &Nonce) ->  Vec<u8> {
    [data.as_bytes(), hbfi.as_bytes(), nonce.as_bytes()].concat()
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

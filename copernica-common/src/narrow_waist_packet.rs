use {
    crate::{
        hbfi::HBFI,
        manifest, generate_nonce,
        ResponseData, Nonce,
        PrivateIdentityInterface, PublicIdentity, Signature
    },
    core::hash::{Hash, Hasher},
    std::{
        cmp::Ordering,
        fmt,
    },
    anyhow::{anyhow, Result},
    log::{debug, error},
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
    pub fn transmute(&self, response_sid: PrivateIdentityInterface, data: Vec<u8>) -> Result<Self> {
        match self {
            NarrowWaistPacket::Request { hbfi, .. } => {
                if hbfi.response_pid != response_sid.public_id() {
                    return Err(anyhow!("The Request's Response Public Identity doesn't match the Public Identity used to sign or encypt the Response"));
                }
                let mut rng = rand::thread_rng();
                let nonce: Nonce = generate_nonce(&mut rng);
                match hbfi.request_pid.clone() {
                    Some(request_pid) => {
                        let hbfi = hbfi.clone();
                        let nonce = nonce.clone();
                        let data = ResponseData::cypher_text(response_sid.clone(), request_pid, data, nonce.clone())?;
                        let manifest = manifest(data.manifest_data(), &hbfi, &nonce)?;
                        let response_signkey = response_sid.signing_key();
                        let signature = response_signkey.sign(manifest);
                        Ok(NarrowWaistPacket::Response { hbfi, nonce, data, signature })
                    },
                    None => {
                        let hbfi = hbfi.clone();
                        let nonce = nonce.clone();
                        let data = ResponseData::clear_text(data)?;
                        let manifest = manifest(data.manifest_data(), &hbfi, &nonce)?;
                        let response_signkey = response_sid.signing_key();
                        let signature = response_signkey.sign(manifest);
                        Ok(NarrowWaistPacket::Response { hbfi, nonce, data, signature })
                    }
                }

            },
            NarrowWaistPacket::Response { .. } => {
                let err_msg = "A NarrowWaistPacket::Response cannot become a NarrowWaistPacket::Response; it already is a Response.";
                error!("{}", err_msg);
                return Err(anyhow!(err_msg))
            },
        }
    }
    pub fn response(response_sid: PrivateIdentityInterface, hbfi: HBFI, data: Vec<u8>) -> Result<Self> {
        // consider returning Result<Vec<Self>>
        if hbfi.response_pid != response_sid.public_id() {
            let msg = "The Request's Response Public Identity doesn't match the Public Identity used to sign or encypt the Response";
            error!("{}", msg);
            return Err(anyhow!(msg));
        }
        let mut rng = rand::thread_rng();
        let nonce: Nonce = generate_nonce(&mut rng);
        match hbfi.request_pid.clone() {
            Some(request_pid) => {
                let data = ResponseData::cypher_text(response_sid.clone(), request_pid, data, nonce.clone())?;
                let manifest = manifest(data.manifest_data(), &hbfi, &nonce)?;
                let response_signkey = response_sid.signing_key();
                let signature = response_signkey.sign(manifest);
                Ok(NarrowWaistPacket::Response { hbfi, nonce, data, signature })
            },
            None => {
                let data = ResponseData::clear_text(data)?;
                let manifest = manifest(data.manifest_data(), &hbfi, &nonce)?;
                let response_signkey = response_sid.signing_key();
                let signature = response_signkey.sign(manifest);
                Ok(NarrowWaistPacket::Response { hbfi, nonce, data, signature })
            }
        }
    }
    pub fn encrypt(&self, response_sid: PrivateIdentityInterface, hbfi: HBFI) -> Result<Self> {
        match self {
            NarrowWaistPacket::Response { data, .. } => {
                if let Some(request_pid) = hbfi.request_pid.clone() {
                    match data {
                        ResponseData::ClearText { data } => {
                            if !self.verify()? {
                                let err_msg = "When encrypting a packet the cleartext manifest signature failed";
                                error!("{}", err_msg);
                                return Err(anyhow!(err_msg))
                            }
                            let mut rng = rand::thread_rng();
                            let nonce: Nonce = generate_nonce(&mut rng);
                            let data = ResponseData::cypher_text(response_sid.clone(), request_pid, data.data()?, nonce.clone())?;
                            let manifest = manifest(data.manifest_data(), &hbfi, &nonce)?;
                            let response_signk = response_sid.signing_key();
                            let signature = response_signk.sign(manifest);
                            Ok(NarrowWaistPacket::Response{ data, signature, hbfi, nonce})
                        },
                        ResponseData::CypherText { .. } => {
                            let err_msg = "No point in encrypting an already encrypted packet";
                            error!("{}", err_msg);
                            return Err(anyhow!(err_msg))
                        },
                    }
                } else {
                    let err_msg = "The HBFI doesn't contain a Request Public Identity to use in the encryption process of a Narrow Waist";
                    error!("{}", err_msg);
                    return Err(anyhow!(err_msg))
                }
            },
            NarrowWaistPacket::Request { .. } => {
                let err_msg = "Requests shouldn't be encrypted";
                error!("{}", err_msg);
                return Err(anyhow!(err_msg))
            },
        }
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
    pub fn encrypt_for(&self, request_pid: PublicIdentity, response_sid: PrivateIdentityInterface) -> Result<Self> {
        match self {
            NarrowWaistPacket::Request {..} => {
                return Err(anyhow!("The Request's Response Public Identity doesn't match the Public Identity used to sign or encypt the Response"));
            },
            NarrowWaistPacket::Response { data, hbfi, .. } => {
                match data {
                    ResponseData::ClearText { data } => {
                        if response_sid.public_id() != hbfi.response_pid {
                            debug!("\n{:?}\n{:?}", response_sid.public_id(), hbfi.response_pid);
                            let err_msg = "Cannot encrypt data one doesn't control";
                            error!("{}", err_msg);
                            return Err(anyhow!(err_msg))
                        }
                        if !self.verify()? {
                            let err_msg = "When encrypting a packet the cleartext manifest signature failed";
                            error!("{}", err_msg);
                            return Err(anyhow!(err_msg))
                        }
                        let mut rng = rand::thread_rng();
                        let nonce: Nonce = generate_nonce(&mut rng);
                        let hbfi = hbfi.encrypt_for(request_pid.clone())?;
                        let data = ResponseData::cypher_text(response_sid.clone(), request_pid, data.data()?, nonce.clone())?;
                        let manifest = manifest(data.manifest_data(), &hbfi, &nonce)?;
                        let response_signk = response_sid.signing_key();
                        let signature = response_signk.sign(manifest.clone());
                        let nw = NarrowWaistPacket::Response{ data, signature, hbfi: hbfi.clone(), nonce};
                        if !nw.verify()? {
                            let err_msg = "Encrypting for a public_id failed a signature check";
                            error!("{}", err_msg);
                            return Err(anyhow!(err_msg))
                        }
                        return Ok(nw);
                    },
                    ResponseData::CypherText { .. } => {
                        let err_msg = "No point in encrypting an already encrypted packet";
                        error!("{}", err_msg);
                        return Err(anyhow!(err_msg))
                    },
                }
            },
        }
    }
    pub fn data(&self, request_sid: Option<PrivateIdentityInterface>) -> Result<Vec<u8>> {
        match self {
            NarrowWaistPacket::Request {..} => {
                let err_msg = "No data in a NarrowWaistPacket::Request";
                error!("{}", err_msg);
                return Err(anyhow!(err_msg))
            },
            NarrowWaistPacket::Response { data, hbfi, nonce, ..}=> {
                if !self.verify()? {
                    let err_msg = format!("The manifest signature check failed when extracting the data from a NarrowWaistPacket::Response hbfi.response_pid = {:?}", hbfi.response_pid);
                    error!("{}", err_msg);
                    return Err(anyhow!(err_msg))
                }
                match data.clone() {
                    ResponseData::ClearText { .. } => {
                        return Ok(data.cleartext_data()?)
                    },
                    ResponseData::CypherText { .. } => {
                        match request_sid {
                            Some(request_sid) => {
                                match hbfi.request_pid.clone() {
                                    Some(request_pid) => {
                                        if request_pid != request_sid.public_id() {
                                            let err_msg = "The Response's Request_PublicIdentity doesn't match the Public Identity used to sign or decypt the Response";
                                            error!("{}", err_msg);
                                            return Err(anyhow!(err_msg));
                                        }
                                        //debug!("{:?}", data.decrypt_data(request_sid.clone(), hbfi.response_pid.clone(), *nonce)?);
                                        return Ok(data.decrypt_data(request_sid, hbfi.response_pid.clone(), nonce.clone())?)
                                    },
                                    None => {
                                        let err_msg = "Decrypting an encrypted data packet requires a PublicIdentity to do so";
                                        error!("{}", err_msg);
                                        return Err(anyhow!(err_msg))
                                    },
                                }
                            },
                            None => {
                                let err_msg = "Decrypting an encrypted data packet requires a PrivateIdentityInterface to do so";
                                error!("{}", err_msg);
                                return Err(anyhow!(err_msg))
                            },
                        }
                    },
                };
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
        self_hbfi.ost.cmp(&other_hbfi.ost)
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
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        PrivateIdentityInterface,
        narrow_waist_packet::{NarrowWaistPacket},
    };
    #[test]
    fn request_transmute_and_decrypt() {
        let response_sid = PrivateIdentityInterface::new_key();
        let response_pid = response_sid.public_id();
        let request_sid = PrivateIdentityInterface::new_key();
        let request_pid = request_sid.public_id();
        let hbfi = HBFI::new(Some(request_pid), response_pid.clone(), "app", "m0d", "fun", "arg").unwrap();
        let nw: NarrowWaistPacket = NarrowWaistPacket::request(hbfi.clone()).unwrap();
        let expected_data = vec![0; 600];
        let offset = 0;
        let total = 1;
        let nw: NarrowWaistPacket = nw.transmute(response_sid.clone(), expected_data.clone(), offset, total).unwrap();
        let actual_data = nw.data(Some(request_sid)).unwrap();
        assert_eq!(actual_data, expected_data);
    }
}

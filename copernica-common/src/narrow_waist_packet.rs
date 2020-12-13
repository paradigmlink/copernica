use {
    crate::{
        hbfi::HBFI,
        manifest, generate_nonce,
        ResponseData, Nonce,
        constants::*,
    },
    std::fmt,
    serde::{Deserialize, Serialize},
    copernica_identity::{PrivateIdentity, PublicIdentity, Signature},
    anyhow::{anyhow, Result},
    log::{debug},
};

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
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
        offset: u64,
        total: u64,
    },
}

impl NarrowWaistPacket {
    pub fn request(hbfi: HBFI) -> Result<Self> {
        let mut rng = rand::thread_rng();
        let nonce: Nonce = generate_nonce(&mut rng);
        Ok(NarrowWaistPacket::Request { hbfi, nonce })
    }
    pub fn transmute(&self, response_sid: PrivateIdentity, data: Vec<u8>, offset: u64, total: u64) -> Result<Self> {
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
                        let data = ResponseData::cypher_text(response_sid.clone(), request_pid, data, nonce)?;
                        let manifest = manifest(data.manifest_data(), &hbfi, &offset, &total, &nonce)?;
                        let response_signkey = response_sid.signing_key();
                        let signature = response_signkey.sign(manifest);
                        Ok(NarrowWaistPacket::Response { hbfi, nonce, offset, total, data, signature })
                    },
                    None => {
                        let hbfi = hbfi.clone();
                        let nonce = nonce.clone();
                        let data = ResponseData::clear_text(data)?;
                        let manifest = manifest(data.manifest_data(), &hbfi, &offset, &total, &nonce)?;
                        let response_signkey = response_sid.signing_key();
                        let signature = response_signkey.sign(manifest);
                        Ok(NarrowWaistPacket::Response { hbfi, nonce, offset, total, data, signature })
                    }
                }

            },
            NarrowWaistPacket::Response { .. } => {
                return Err(anyhow!("A NarrowWaistPacket::Response cannot become a NarrowWaistPacket::Response; it already is a Response."))
            },
        }
    }
    pub fn encrypt_for(&self, response_sid: PrivateIdentity, request_pid: PublicIdentity) -> Result<Self> {
        match self {
            NarrowWaistPacket::Request {..} => {
                return Err(anyhow!("The Request's Response Public Identity doesn't match the Public Identity used to sign or encypt the Response"));
            },
            NarrowWaistPacket::Response { data, hbfi, nonce, offset, total, .. } => {
                match data {
                    ResponseData::ClearText { data } => {
                        if !self.verify()? {
                            return Err(anyhow!("When encrypting a packet the cleartext manifest signature failed"))
                        }
                        let mut rng = rand::thread_rng();
                        let nonce: Nonce = generate_nonce(&mut rng);
                        let hbfi = hbfi.encrypt_for(request_pid.clone())?;

                        let data = ResponseData::cypher_text(response_sid.clone(), request_pid, data.data()?, nonce)?;
                        let manifest = manifest(data.manifest_data(), &hbfi, offset, total, &nonce)?;

                        let response_signk = response_sid.signing_key();
                        let signature = response_signk.sign(manifest);
                        let nw = NarrowWaistPacket::Response{ data, signature, hbfi, offset: *offset, total: *total, nonce};
                        debug!("PROBLEM IS IN THIS FUNCTION -> should be true but it's: {}", nw.verify()?);
                        Ok(nw)
                    },
                    ResponseData::CypherText { .. } => {
                        return Err(anyhow!("No point in encrypting an already encrypted packet"))
                    },
                }
            },
        }
    }
    pub fn response(response_sid: PrivateIdentity, hbfi: HBFI, data: Vec<u8>, offset: u64, total: u64) -> Result<Self> {
        if hbfi.response_pid != response_sid.public_id() {
            return Err(anyhow!("The Request's Response Public Identity doesn't match the Public Identity used to sign or encypt the Response"));
        }
        let mut rng = rand::thread_rng();
        let nonce: Nonce = generate_nonce(&mut rng);
        match hbfi.request_pid.clone() {
            Some(_request_pid) => {
                return Err(anyhow!("Initial creation of a NarrowWaistPacket::Response should be clear text (at least for now). Your service application should call NarrowWaistPacket::encrypt() using the nonce from the inbound NarrowWaistPacket::Request packets as an argument."))
            },
            None => {
                let data = ResponseData::clear_text(data)?;
                let manifest = manifest(data.manifest_data(), &hbfi, &offset, &total, &nonce)?;
                let response_signkey = response_sid.signing_key();
                let signature = response_signkey.sign(manifest);
                Ok(NarrowWaistPacket::Response { hbfi, nonce, offset, total, data, signature })
            }
        }

    }
    pub fn encrypt(&self, response_sid: PrivateIdentity, hbfi: HBFI) -> Result<Self> {
        match self {
            NarrowWaistPacket::Response { data, offset, total, .. } => {
                if let Some(request_pid) = hbfi.request_pid.clone() {
                    match data {
                        ResponseData::ClearText { data } => {
                            if !self.verify()? {
                                return Err(anyhow!("When encrypting a packet the cleartext manifest signature failed"))
                            }
                            let mut rng = rand::thread_rng();
                            let nonce: Nonce = generate_nonce(&mut rng);
                            let data = ResponseData::cypher_text(response_sid.clone(), request_pid, data.data()?, nonce)?;
                            let manifest = manifest(data.manifest_data(), &hbfi, offset, total, &nonce)?;
                            let response_signk = response_sid.signing_key();
                            let signature = response_signk.sign(manifest);
                            Ok(NarrowWaistPacket::Response{ data, signature, hbfi, offset: *offset, total: *total, nonce})
                        },
                        ResponseData::CypherText { .. } => {
                            return Err(anyhow!("No point in encrypting an already encrypted packet"))
                        },
                    }
                } else {
                    return Err(anyhow!("The HBFI doesn't contain a Request Public Identity to use in the encryption process of a Narrow Waist"))
                }
            },
            NarrowWaistPacket::Request { .. } => {
                return Err(anyhow!("Requests shouldn't be encrypted"))
            },
        }
    }
    pub fn verify(&self) -> Result<bool> {
        match self {
            NarrowWaistPacket::Request {..} => {
                return Err(anyhow!("No point in verifying a NarrowWaistPacket::Request"))
            },
            NarrowWaistPacket::Response { data, hbfi, offset, total, signature, nonce} => {
                let manifest = manifest(data.manifest_data(), hbfi, offset, total, nonce)?;
                let verify_key = hbfi.response_pid.verify_key()?;
                let verified = verify_key.verify(&signature, manifest);
                debug!("{:?}", verified);
                return Ok(verified);
            },
        }
    }
    pub fn data(&self, request_sid: Option<PrivateIdentity>) -> Result<Vec<u8>> {
        match self {
            NarrowWaistPacket::Request {..} => {
                return Err(anyhow!("No data in a NarrowWaistPacket::Request"))
            },
            NarrowWaistPacket::Response { data, hbfi, offset, total, signature, nonce}=> {
                if !self.verify()? {
                    return Err(anyhow!("The manifest signature When decrypting a NarrowWaistPacket::Response"))
                }
                match data.clone() {
                    ResponseData::ClearText { .. } => {
                        return Ok(data.cleartext_data()?)
                    },
                    ResponseData::CypherText { .. } => {
                        if let Some(request_sid) = request_sid {
                            if let Some(request_pid) = hbfi.request_pid.clone() {
                                if request_pid != request_sid.public_id() {
                                    return Err(anyhow!("The Response's Request_PublicIdentity doesn't match the Public Identity used to sign or decypt the Response"));
                                }
                                return Ok(data.decrypt_data(request_sid, hbfi.response_pid.clone(), *nonce)?)
                            }
                        }
                        return return Err(anyhow!("Decrypting an encrypted data packet requires a PrivateIdentity to do so"))
                    },
                };
            },
        }
    }
}

impl fmt::Debug for NarrowWaistPacket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            NarrowWaistPacket::Request { hbfi, .. } => write!(f, "REQ{:?}", hbfi),
            NarrowWaistPacket::Response {
                hbfi,
                offset,
                total,
                signature,
                nonce,
                ..
            } => write!(f, "RES {:?} {}/{} {} {:?}", hbfi, offset, total, signature, nonce),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        narrow_waist_packet::{NarrowWaistPacket},
    };
    use copernica_identity::{PrivateIdentity, Seed};

    #[test]
    fn request_transmute_and_decrypt() {
        let mut rng = rand::thread_rng();
        let response_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));
        let response_pid = response_sid.public_id();
        let request_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));
        let request_pid = request_sid.public_id();

        let hbfi = HBFI::new(Some(request_pid), response_pid.clone(), "app", "m0d", "fun", "arg").unwrap();
        let nw: NarrowWaistPacket = NarrowWaistPacket::request(hbfi.clone()).unwrap();
        let expected_data = vec![0; 600];
        let offset = 0;
        let total = 1;
        let nw: NarrowWaistPacket = nw.transmute(response_sid.clone(), expected_data.clone(), offset, total).unwrap();
        let actual_data = nw.decrypt(request_sid).unwrap();

        assert_eq!(actual_data, expected_data);
    }
}

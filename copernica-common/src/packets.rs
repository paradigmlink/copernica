use {
    crate::{
        constants,
        hbfi::HBFI,
        link::{LinkId, ReplyTo},
        data_length, manifest, generate_nonce,
        PublicIdentity, PrivateIdentity, Signature
    },
    serde::{Deserialize, Serialize},
    std::fmt,
    serde_big_array::{big_array},
    anyhow::{anyhow, Result},
    rand_core::{CryptoRng, RngCore},
    rand::Rng,
    log::{debug},
    cryptoxide::{chacha20poly1305::{ChaCha20Poly1305}},
};

#[derive(Clone, Serialize, Deserialize)]
pub enum NarrowWaistPacket {
    Request {
        hbfi: HBFI,
        request_nonce: Nonce,
    },
    Response {
        hbfi: HBFI,
        request_nonce: Nonce,
        response_nonce: Nonce,
        signature: Signature,
        data: ResponseData,
        offset: u64,
        total: u64,
    },
}

impl NarrowWaistPacket {
    pub fn request(hbfi: HBFI) -> Result<Self> {
        let mut rng = rand::thread_rng();
        let request_nonce: Nonce = generate_nonce(&mut rng);
        Ok(NarrowWaistPacket::Request { hbfi, request_nonce })
    }
    pub fn transmute(&self, response_sid: PrivateIdentity, data: Vec<u8>, offset: u64, total: u64) -> Result<Self> {
        match self {
            NarrowWaistPacket::Request { hbfi, request_nonce } => {
                if hbfi.response_pid != response_sid.public_id() {
                    return Err(anyhow!("The Request's Response Public Identity doesn't match the Public Identity used to sign or encypt the Response"));
                }
                let mut rng = rand::thread_rng();
                let response_nonce: Nonce = generate_nonce(&mut rng);
                match hbfi.request_pid.clone() {
                    Some(request_pid) => {
                        let hbfi = hbfi.clone();
                        let request_nonce = request_nonce.clone();
                        let data = ResponseData::cypher_text(response_sid.clone(), request_pid, data, request_nonce, response_nonce)?;
                        let manifest = manifest(data.manifest_data()?, &hbfi, &offset, &total, &request_nonce, &response_nonce);
                        let response_signkey = response_sid.signing_key();
                        let signature = response_signkey.sign(manifest);
                        Ok(NarrowWaistPacket::Response { hbfi, request_nonce, response_nonce, offset, total, data, signature })
                    },
                    None => {
                        let hbfi = hbfi.clone();
                        let request_nonce = request_nonce.clone();
                        let data = ResponseData::clear_text(data)?;
                        let manifest = manifest(data.manifest_data()?, &hbfi, &offset, &total, &request_nonce, &response_nonce);
                        let response_signkey = response_sid.signing_key();
                        let signature = response_signkey.sign(manifest);
                        Ok(NarrowWaistPacket::Response { hbfi, request_nonce, response_nonce, offset, total, data, signature })
                    }
                }

            },
            NarrowWaistPacket::Response { .. } => {
                return Err(anyhow!("A NarrowWaistPacket::Response cannot become a NarrowWaistPacket::Response; it already is a Response."))
            },
        }
    }
    pub fn response(response_sid: PrivateIdentity, hbfi: HBFI, data: Vec<u8>, offset: u64, total: u64) -> Result<Self> {
        if hbfi.response_pid != response_sid.public_id() {
            return Err(anyhow!("The Request's Response Public Identity doesn't match the Public Identity used to sign or encypt the Response"));
        }
        let mut rng = rand::thread_rng();
        let response_nonce: Nonce = generate_nonce(&mut rng);
        let request_nonce: Nonce = generate_nonce(&mut rng);
        match hbfi.request_pid.clone() {
            Some(_request_pid) => {
                return Err(anyhow!("Initial creation of a NarrowWaistPacket::Response should be clear text (at least for now). Your service application should call NarrowWaistPacket::encrypt() using the nonce from the inbound NarrowWaistPacket::Request packets as an argument."))
            },
            None => {
                let data = ResponseData::clear_text(data)?;
                let manifest = manifest(data.manifest_data()?, &hbfi, &offset, &total, &request_nonce, &response_nonce);
                let response_signkey = response_sid.signing_key();
                let signature = response_signkey.sign(manifest);
                Ok(NarrowWaistPacket::Response { hbfi, request_nonce, response_nonce, offset, total, data, signature })
            }
        }

    }
    pub fn decrypt(&self, request_sid: PrivateIdentity) -> Result<Vec<u8>> {
        match self {
            NarrowWaistPacket::Response { data, hbfi, request_nonce, response_nonce, .. } => {
                if let Some(request_pid) = hbfi.request_pid.clone() {
                    if request_pid != request_sid.public_id() {
                        return Err(anyhow!("The Response's Request_PublicIdentity doesn't match the Public Identity used to sign or decypt the Response"));
                    }
                    if !self.verify()? {
                        return Err(anyhow!("When decrypting a NarrowWaistPacket, the manifest signature failed"))
                    }
                    match data.decrypt_data(request_sid, hbfi.response_pid.clone(), *request_nonce, *response_nonce)? {
                        Some(data) => {
                            return Ok(data)
                        },
                        None => { return Err(anyhow!("Couldn't decrypt")) },
                    };
                } else {
                    return Err(anyhow!("The HBFI doesn't contain a Request Public Identity to use in the decryption process of a Narrow Waist"))
                }
            },
            NarrowWaistPacket::Request { .. } => {
                return Err(anyhow!("Requests shouldn't be decrypted"))
            },
        }
    }
    pub fn encrypt(&self, response_sid: PrivateIdentity, hbfi: HBFI) -> Result<Self> {
        match self {
            NarrowWaistPacket::Response { data, offset, total, request_nonce, .. } => {
                if let Some(request_pid) = hbfi.request_pid.clone() {
                    match data {
                        ResponseData::ClearText { data } => {
                            if data.len() != constants::FRAGMENT_SIZE {
                                return Err(anyhow!("Ensure data.len() passed into ResponseData::cypher_text() is not greater than {}", constants::FRAGMENT_SIZE))
                            }
                            if !self.verify()? {
                                return Err(anyhow!("When encrypting a packet the cleartext manifest signature failed"))
                            }
                            let mut rng = rand::thread_rng();
                            let response_nonce: Nonce = generate_nonce(&mut rng);
                            let length = data_length(&data)?;
                            let (data, _) = data.split_at(length);

                            let data = ResponseData::cypher_text(response_sid.clone(), request_pid, data.to_vec(), *request_nonce, response_nonce)?;
                            let manifest = manifest(data.manifest_data()?, &hbfi, offset, total, &request_nonce, &response_nonce);
                            let response_signk = response_sid.signing_key();
                            let signature = response_signk.sign(manifest);
                            let request_nonce = request_nonce.clone();
                            Ok(NarrowWaistPacket::Response{ data, signature, hbfi, offset: *offset, total: *total, request_nonce, response_nonce})
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
            NarrowWaistPacket::Response { data, hbfi, offset, total, signature, request_nonce, response_nonce}=> {
                let manifest = manifest(data.manifest_data()?, hbfi, offset, total, request_nonce, response_nonce);
                let verify_key = hbfi.response_pid.verify_key();
                return Ok(verify_key.verify(&signature, manifest))
            },
        }
    }
    pub fn data(&self) -> Result<Vec<u8>> {
        match self {
            NarrowWaistPacket::Request {..} => {
                return Err(anyhow!("No data in a NarrowWaistPacket::Request"))
            },
            NarrowWaistPacket::Response { data, hbfi, offset, total, signature, request_nonce, response_nonce}=> {
                let manifest = manifest(data.manifest_data()?, &hbfi, &offset, &total, &request_nonce, &response_nonce);
                match data {
                    ResponseData::ClearText { data } => {
                        let verify_key = hbfi.response_pid.verify_key();
                        match verify_key.verify(&signature, manifest) {
                            false => {
                                debug!("Verification Fail for hbfi: {}", hbfi);
                                return Err(anyhow!("Signature check didn't succeed when extracting a NarrowWaistPacket::Response"))
                            },
                            true => {
                                let length = data_length(&data)?;
                                let (chunk, _) = data.split_at(length);
                                return Ok(chunk.to_vec())
                            },
                        };
                    },
                    ResponseData::CypherText { data, .. } => {
                        let verify_key = hbfi.response_pid.verify_key();
                        match verify_key.verify(&signature, manifest) {
                            false => { return Err(anyhow!("Signature check didn't succeed when extracting a NarrowWaistPacket::Response")) },
                            true => {

                                let length = data_length(&data)?;
                                let (chunk, _) = data.split_at(length);
                                return Ok(chunk.to_vec())
                            },
                        };
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
                request_nonce,
                response_nonce,
                ..
            } => write!(f, "RES {:?} {}/{} {} {:?} {:?}", hbfi, offset, total, signature, request_nonce, response_nonce),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        packets::{NarrowWaistPacket},
        identity::{PrivateIdentity, Seed},
    };
    #[test]
    fn request_transmute_and_decrypt() {
        let mut rng = rand::thread_rng();
        let response_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));
        let response_pid = response_sid.public_id();
        let request_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));
        let request_pid = request_sid.public_id();

        let hbfi = HBFI::new(response_pid.clone(), Some(request_pid), "app", "m0d", "fun", "arg").unwrap();
        let nw: NarrowWaistPacket = NarrowWaistPacket::request(hbfi.clone()).unwrap();
        let expected_data = vec![0; 600];
        let offset = 0;
        let total = 1;
        let nw: NarrowWaistPacket = nw.transmute(response_sid.clone(), expected_data.clone(), offset, total).unwrap();
        let actual_data = nw.decrypt(request_sid).unwrap();

        assert_eq!(actual_data, expected_data);
    }
}

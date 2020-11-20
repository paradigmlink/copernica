use {
    crate::{
        constants,
        hbfi::HBFI,
        link::{LinkId, ReplyTo},
    },
    serde::{Deserialize, Serialize},
    std::fmt,
    serde_big_array::{big_array},
    keynesis::{PublicIdentity, PrivateIdentity, Signature},
    anyhow::{anyhow, Result},
    rand_core::{CryptoRng, RngCore},
    rand::Rng,
    log::{debug},
    cryptoxide::{chacha20poly1305::{ChaCha20Poly1305}},
};

big_array! { BigArray; }

pub type Nonce = [u8; constants::NONCE_SIZE];
pub type Tag = [u8; constants::TAG_SIZE];
pub type Data = [u8; constants::FRAGMENT_SIZE];

#[derive(Clone, Serialize, Deserialize)]
pub enum ResponseData {
    ClearText {
        #[serde(with = "BigArray")]
        data: Data,
    },
    CypherText {
        #[serde(with = "BigArray")]
        data: Data,
        tag: Tag,
    },
}

impl fmt::Display for ResponseData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            ResponseData::ClearText { data } => write!(f, "RD::ClearText: {:?}", data.as_ref()),
            ResponseData::CypherText { data, tag } => write!(f, "RD::CypherText: {:?} Tag: {:?}", data.as_ref(), tag),
        }
    }
}

impl fmt::Debug for ResponseData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            ResponseData::ClearText { data } => write!(f, "RD::ClearText: {:?}", data.as_ref()),
            ResponseData::CypherText { data, tag } => write!(f, "RD::CypherText: {:?} Tag: {:?}", data.as_ref(), tag),
        }
    }
}

impl ResponseData {
    pub fn clear_text(data: Vec<u8>) -> Result<Self> {
        if data.len() > constants::DATA_SIZE {
            return Err(anyhow!("Ensure data.len()({}) passed into ResponseData::clear_text() is not greater than {}", data.len(), constants::DATA_SIZE))
        }
        let mut rng = rand::thread_rng();
        let length = data.len();
        let padding: Vec<u8> = (0..(constants::DATA_SIZE - length)).map(|_| rng.gen()).collect();
        let b1 = length as u8;
        let b2 = (length >> 8) as u8;
        let u8_nonce: u8 = rng.gen();
        let metadata = vec![0, u8_nonce, b2, b1];
        let data = vec![data, padding, metadata];
        let flattened = data.into_iter().flatten().collect::<Vec<u8>>();
        let mut data: [u8; constants::FRAGMENT_SIZE] = [0; constants::FRAGMENT_SIZE];
        data.copy_from_slice(&flattened[0..constants::FRAGMENT_SIZE]);
        Ok(ResponseData::ClearText { data })
    }
    pub fn cypher_text(response_sid: PrivateIdentity, request_pid: PublicIdentity, data: Vec<u8>, request_nonce: Nonce, response_nonce: Nonce) -> Result<Self> {
        if data.len() > constants::DATA_SIZE {
            return Err(anyhow!("Ensure data.len() passed into ResponseData::cypher_text() is not greater than {}", constants::DATA_SIZE))
        }
        let mut rng = rand::thread_rng();
        let length = data.len();
        let padding: Vec<u8> = (0..(constants::DATA_SIZE - length)).map(|_| rng.gen()).collect();
        let b1 = length as u8;
        let b2 = (length >> 8) as u8;
        let u8_nonce: u8 = rng.gen();
        let metadata = vec![0, u8_nonce, b2, b1];

        let data = vec![data, padding, metadata];
        let flattened = data.into_iter().flatten().collect::<Vec<u8>>();
        let mut data: [u8; constants::FRAGMENT_SIZE] = [0; constants::FRAGMENT_SIZE];
        data.copy_from_slice(&flattened[0..constants::FRAGMENT_SIZE]);
        let request_pk = request_pid.derive(&request_nonce);
        let response_sk = response_sid.derive(&response_nonce);
        let shared_secret = response_sk.exchange(&request_pk);
        let mut ctx = ChaCha20Poly1305::new(&shared_secret.as_ref(), &response_nonce, &[]);
        let mut encrypted: Data = [0; constants::FRAGMENT_SIZE];
        let mut tag: Tag = [0; constants::TAG_SIZE];
        ctx.encrypt(&data, &mut encrypted[..], &mut tag);
        data.copy_from_slice(&encrypted[..]);
        Ok(ResponseData::CypherText { data, tag })
    }
    pub fn cleartext_data(&self) -> Result<Vec<u8>> {
        let data = match self {
            ResponseData::ClearText { data } => {
                let length = data_length(&data)?;
                let (data, _) = data.split_at(length);
                data
            },
            ResponseData::CypherText { .. } => {
                return Err(anyhow!("Cannot obtain the cleartext for encrypted data, use the decrypt_data method instead"))
            },
        };
        Ok(data.to_vec())
    }
    pub fn decrypt_data(&self, request_sid: PrivateIdentity, response_pid: PublicIdentity, request_nonce: Nonce, response_nonce: Nonce) -> Result<Option<Vec<u8>>> {
        let data = match self {
            ResponseData::CypherText { data, tag } => {
                if data.len() != constants::FRAGMENT_SIZE {
                    return Err(anyhow!("Ensure data.len() passed into ResponseData.decrypt_data() is equal to {}", constants::FRAGMENT_SIZE))
                }
                let response_pk = response_pid.derive(&response_nonce);
                let request_sk = request_sid.derive(&request_nonce);
                let shared_secret = request_sk.exchange(&response_pk);
                let mut ctx = ChaCha20Poly1305::new(&shared_secret.as_ref(), &response_nonce, &[]);
                let mut decrypted: Data = [0; constants::FRAGMENT_SIZE];
                if ctx.decrypt(&data[..], &mut decrypted[..], &tag[..]) {
                    let length = data_length(&decrypted)?;
                    let (data, _) = decrypted.split_at(length);
                    Ok(Some(data.to_vec()))
                } else {
                    Ok(None)
                }
            },
            ResponseData::ClearText { data } => {
                if data.len() != constants::FRAGMENT_SIZE {
                    return Err(anyhow!("Ensure data.len() passed into ResponseData.decrypt_data() is equal to {}", constants::FRAGMENT_SIZE))
                }
                let length = data_length(&data)?;
                let (data, _) = data.split_at(length);
                Ok(Some(data.to_vec()))
            },
        };
        data
    }
    pub fn manifest_data(&self) -> Result<Vec<u8>> {
        match self {
            ResponseData::ClearText { data } => { Ok(data.as_ref().to_vec()) },
            ResponseData::CypherText { data, tag } => {
                Ok(format!("{:?}{:?}", data.as_ref(), tag).as_bytes().to_vec())
            },
        }
    }
}

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

pub(crate) fn data_length(data: &[u8; constants::FRAGMENT_SIZE]) -> Result<usize> {
    let length_combined = format!("{:02x}{:02x}", data[constants::LENGTH_OF_DATA_STARTING_POSITION], data[constants::LENGTH_OF_DATA_ENDING_POSITION]);
    let length = u16::from_str_radix(&length_combined, 16)?;
    Ok(length as usize)
}

pub(crate) fn manifest(data: Vec<u8>, hbfi: &HBFI, offset: &u64, total: &u64, request_nonce: &Nonce, response_nonce: &Nonce) -> String {
    let manifest = format!("{:?}{}{}{}{:?}{:?}", data, hbfi, offset, total, request_nonce, response_nonce);
    //println!("{:?}", manifest);
    manifest
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LinkPacket {
    pub reply_to: ReplyTo,
    pub nw: NarrowWaistPacket,
}

impl LinkPacket {
    pub fn new(reply_to: ReplyTo, nw: NarrowWaistPacket) -> Self {
        Self { reply_to , nw}
    }
    pub fn narrow_waist(&self) -> NarrowWaistPacket {
        self.nw.clone()
    }
    pub fn reply_to(&self) -> ReplyTo {
        self.reply_to.clone()
    }
    pub fn change_origination(&self, reply_to: ReplyTo) -> Self {
        Self { reply_to, nw: self.nw.clone() }
    }
}

#[derive(Debug, Clone)]
pub struct InterLinkPacket {
    pub link_id: LinkId,
    pub lp: LinkPacket,
}

impl InterLinkPacket {
    pub fn new(link_id: LinkId, lp: LinkPacket) -> Self {
        Self { link_id, lp }
    }
    pub fn link_id(&self) -> LinkId {
        self.link_id.clone()
    }
    pub fn change_destination(&self, link_id: LinkId) -> Self {
        Self { link_id, lp: self.lp.clone() }
    }
    pub fn reply_to(&self) -> ReplyTo {
        self.link_id.reply_to()
    }
    pub fn narrow_waist(&self) -> NarrowWaistPacket {
        self.lp.narrow_waist().clone()
    }
    pub fn wire_packet(&self) -> LinkPacket {
        self.lp.clone()
    }
}

pub fn generate_nonce<R>(rng: &mut R) -> Nonce
where
    R: RngCore + CryptoRng,
{
    let mut nonce: Nonce = [0; constants::NONCE_SIZE];
    rng.fill_bytes(&mut nonce);
    nonce
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        packets::{NarrowWaistPacket},
    };
    use keynesis::{PrivateIdentity, Seed};

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

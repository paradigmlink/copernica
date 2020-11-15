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
    //rand::Rng,
    log::{debug},
};

big_array! { BigArray; }

pub type Nonce = [u8; constants::NONCE_SIZE];
pub type Data = [u8; constants::FRAGMENT_SIZE];

#[derive(Clone, Serialize, Deserialize)]
pub enum ResponseData {
    ClearText {
        #[serde(with = "BigArray")]
        data: Data,
    },
    CypherText {
        #[serde(with = "BigArray")]
        data: Data
    },
}

impl ResponseData {
    pub fn clear_text(data_vec: Vec<u8>) -> Result<Self> {
        if data_vec.len() > constants::CLEARTEXT_SIZE {
            return Err(anyhow!("Ensure data.len() passed into ResponseData::clear_text() is not greater than {}", constants::CLEARTEXT_SIZE))
        }
        let length = data_vec.len();
        let padding: Vec<u8> = (0..(constants::CLEARTEXT_SIZE - length)).map(|_| 0).collect();
        let b1 = length as u8;
        let b2 = (length >> 8) as u8;
        let metadata = vec![b2, b1];
        let data_vec = vec![data_vec, padding, metadata];
        let flattened = data_vec.into_iter().flatten().collect::<Vec<u8>>();
        let mut data: [u8; constants::FRAGMENT_SIZE] = [0; constants::FRAGMENT_SIZE];
        data.copy_from_slice(&flattened[0..constants::FRAGMENT_SIZE]);
        Ok(ResponseData::ClearText { data })
    }
    pub fn cypher_text(_response_sid: PrivateIdentity, _request_pid: PublicIdentity, data_vec: Vec<u8>, _nonce: Nonce) -> Result<Self> {
        if data_vec.len() > constants::CYPHERTEXT_SIZE {
            return Err(anyhow!("Ensure data.len() passed into ResponseData::cypher_text() is not greater than {}", constants::CYPHERTEXT_SIZE))
        }
        //let mut rng = rand::thread_rng();
        //let u8_nonce: u8 = rng.gen();
        //let snonce = format!("{:?}", u8_nonce);
        let data: Data = [0; constants::FRAGMENT_SIZE];
        Ok(ResponseData::CypherText { data })
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum NarrowWaistPacket {
    Request {
        hbfi: HBFI,
        nonce: Nonce,
    },
    Response {
        hbfi: HBFI,
        signature: Signature,
        nonce: Nonce,
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
    pub fn response(response_sid: PrivateIdentity, hbfi: HBFI, data: Vec<u8>, offset: u64, total: u64) -> Result<Self> {
        if hbfi.response_pid != response_sid.public_id() {
            return Err(anyhow!("The Request's Response Public Identity doesn't match the Public Identity used to sign or encypt the Response"));
        }
        let mut rng = rand::thread_rng();
        let nonce: Nonce = generate_nonce(&mut rng);
        match hbfi.request_pid.clone() {
            Some(request_pid) => {
                let response_data = ResponseData::cypher_text(response_sid.clone(), request_pid, data, nonce)?;
                let manifest = match response_data {
                    ResponseData::CypherText { data } => {
                        format!("{:?}{}{}{}{:?}", &data.as_ref(), hbfi, offset, total, nonce)
                    },
                    _ => return Err(anyhow!("ResponseData::ClearText { data } shouldn't exist at this point"))
                };
                let response_signk = response_sid.signing_key();
                let signature = response_signk.sign(manifest);

                Ok(NarrowWaistPacket::Response { hbfi, nonce, offset, total, data: response_data, signature })
            },
            None => {
                let response_data = ResponseData::clear_text(data)?;
                let manifest = match response_data {
                    ResponseData::ClearText { data } => {
                        format!("{:?}{}{}{}{:?}", &data.as_ref(), hbfi, offset, total, nonce)
                    },
                    _ => return Err(anyhow!("ResponseData::CypherText { data } shouldn't exist at this point"))
                };
                let response_signk = response_sid.signing_key();
                let signature = response_signk.sign(manifest);

                Ok(NarrowWaistPacket::Response { hbfi, nonce, offset, total, data: response_data, signature })
            }
        }

    }
    pub fn verify(&self) -> Result<bool> {
        match self {
            NarrowWaistPacket::Request {..} => {
                return Err(anyhow!("No point in verifying a NarrowWaistPacket::Request"))
            },
            NarrowWaistPacket::Response { data, hbfi, offset, total, signature, nonce}=> {
                let manifest = match data {
                    ResponseData::ClearText { data} => {
                        format!("{:?}{}{}{}{:?}", data.as_ref(),  hbfi, offset, total, nonce)
                    },
                    ResponseData::CypherText { data } => {
                        format!("{:?}{}{}{}{:?}", data.as_ref(),  hbfi, offset, total, nonce)
                    },
                };
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
            NarrowWaistPacket::Response { data, hbfi, offset, total, signature, nonce}=> {
                match data {
                    ResponseData::ClearText { data} => {
                        let manifest = format!("{:?}{}{}{}{:?}", data.as_ref(), hbfi, offset, total, nonce);
                        let verify_key = hbfi.response_pid.verify_key();
                        match verify_key.verify(&signature, manifest) {
                            false => {
                                debug!("Verification Fail for hbfi: {}", hbfi);
                                return Err(anyhow!("Signature check didn't succeed when extracting a NarrowWaistPacket::Response"))
                            },
                            true => {
                                let length_combined = format!("{:02x}{:02x}", data[1022], data[1023]);
                                let length = u16::from_str_radix(&length_combined, 16).unwrap();
                                let (chunk, _) = data.split_at(length as usize);
                                return Ok(chunk.to_vec())
                            },
                        };
                    },
                    ResponseData::CypherText { data } => {
                        let manifest = format!("{:?}{}{}{}{:?}", data.as_ref(), hbfi, offset, total, nonce);
                        let verify_key = hbfi.response_pid.verify_key();
                        match verify_key.verify(&signature, manifest) {
                            false => { return Err(anyhow!("Signature check didn't succeed when extracting a NarrowWaistPacket::Response")) },
                            true => {
                                let length_combined = format!("{:02x}{:02x}", data[1022], data[1023]);
                                let length = u16::from_str_radix(&length_combined, 16).unwrap();
                                println!("Printe 2{}", length);
                                let (chunk, _) = data.split_at(length as usize);
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
                nonce,
                ..
            //} => write!(f, "RES {:?} {}/{}\n{}\n{}\n{}\n{:?}", hbfi, offset, total, signature, response_pid ,nonce, &data.data.as_ref()),
            } => write!(f, "RES {:?} {}/{}\n{}\n{:?}", hbfi, offset, total, signature, nonce),
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
        bloom_filter_index,
        packets::{Data, NarrowWaistPacket},
        hbfi::{BFI},
    };
    use keynesis::{PrivateIdentity, Seed};

    #[test]
    fn narrow_waist_packet_integrity() {
        let mut rng = rand::thread_rng();
        let response_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));
        let response_pid = response_sid.public_id();
        let response_signk = response_sid.signing_key();
        let id = bloom_filter_index(format!("{}", response_pid).as_str()).unwrap();
        let h1: BFI = [u16::MAX; constants::BLOOM_FILTER_INDEX_ELEMENT_LENGTH];
        let hbfi = HBFI::new(response_pid.clone(), None, "app", "m0d", "fun", "arg").unwrap();
        let data = [0; constants::FRAGMENT_SIZE as usize];
        let nonce = generate_nonce(&mut rng);
        let manifest = format!("{:?}{}{}{}{}{:?}", &data.as_ref(), response_pid, hbfi, u64::MAX, u64::MAX, nonce);
        let signature = response_signk.sign(manifest);
        let offset = u64::MAX;
        let total = u64::MAX;
        let data = vec![0; 600];
        let nw: NarrowWaistPacket = NarrowWaistPacket::response(response_sid, hbfi, data, offset, total).unwrap();
        assert_eq!(true, nw.verify().unwrap());
    }

    #[test]
    fn narrow_waist_packet_encrypt_decrypt() {
        let mut rng = rand::thread_rng();
        let response_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));
        let response_pid = response_sid.public_id();

        let request_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));
        let request_pid = request_sid.public_id();
        let response_signk = response_sid.signing_key();
        let id = bloom_filter_index(format!("{}", request_pid).as_str()).unwrap();
        let h1: BFI = [u16::MAX; constants::BLOOM_FILTER_INDEX_ELEMENT_LENGTH];
        let hbfi = HBFI::new(response_pid, None, "app", "m0d", "fun", "arg").unwrap();
        let data = [0; constants::FRAGMENT_SIZE as usize];
        let nonce = generate_nonce(&mut rng);
        let manifest = format!("{:?}{}{}{}{}{:?}", &data.as_ref(), request_pid, hbfi, u64::MAX, u64::MAX, nonce);
        let signature = response_signk.sign(manifest);
        let data = vec![0; 600];
        let offset = u64::MAX;
        let total = u64::MAX;
        let nw: NarrowWaistPacket = NarrowWaistPacket::response(response_sid, hbfi, data, offset, total).unwrap();
        println!("{:?}", nw);
        assert_eq!(true, nw.verify().unwrap());
    }
}



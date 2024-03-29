use {
    crate::{
        Data, PublicIdentity, PublicIdentityInterface, PrivateIdentityInterface, Tag, Nonce
    },
    copernica_common::constants::*,
    std::fmt,
    anyhow::{anyhow, Result},
    rand::Rng,
    cryptoxide::{chacha20poly1305::{ChaCha20Poly1305}},
    log::{error},
};
#[derive(Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum ResponseData {
    ClearText {
        data: Box<Data>,
    },
    CypherText {
        data: Box<Data>,
        tag: Tag,
    },
}
impl fmt::Display for ResponseData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            ResponseData::ClearText { data } => write!(f, "RD::ClearText: {}", data),
            ResponseData::CypherText { data, tag } => write!(f, "RD::CypherText: {:?} Tag: {:?}", data, tag),
        }
    }
}
impl fmt::Debug for ResponseData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            ResponseData::ClearText { data } => write!(f, "RD::ClearText: {:?}", data),
            ResponseData::CypherText { data, tag } => write!(f, "RD::CypherText: {:?} Tag: {:?}", data, tag),
        }
    }
}
impl ResponseData {
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = vec![];
        match self {
            ResponseData::ClearText { data } => {
                buf.extend_from_slice(&data.as_bytes());
                buf
            },
            ResponseData::CypherText { data, tag } => {
                buf.extend_from_slice(&tag.as_bytes());
                buf.extend_from_slice(&data.as_bytes());
                buf
            },
        }
    }
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        match data.len() {
            CYPHERTEXT_RESPONSE_DATA_SIZE => {
                let mut tag = Tag([0u8; TAG_SIZE]);
                tag.0.clone_from_slice(&data[..TAG_SIZE]);
                let data = Data::new(&data[TAG_SIZE..])?;
                Ok(ResponseData::CypherText { tag, data: Box::new(data) })
            },
            CLEARTEXT_RESPONSE_DATA_SIZE => {
                let data = Data::new(&data[..])?;
                Ok(ResponseData::ClearText { data: Box::new(data) })
            }
            _ => {
                Err(anyhow!("Length of data used to reconstruct a ResponseData is unrecognised"))
            }
        }
    }
    pub fn insert(response_sid: PrivateIdentityInterface, request_pid: PublicIdentityInterface, data: Vec<u8>, nonce: Nonce) -> Result<Self> {
        match request_pid {
            PublicIdentityInterface::Present { public_identity } => {
                if data.len() > DATA_SIZE {
                    return Err(anyhow!("Ensure data.len() passed into ResponseData::cypher_text() is not greater than {}", DATA_SIZE))
                }
                let mut rng = rand::thread_rng();
                let length = data.len();
                let padding: Vec<u8> = (0..(DATA_SIZE - length)).map(|_| rng.gen()).collect();
                //let padding: Vec<u8> = (0..(DATA_SIZE - length)).map(|_| 0).collect();
                let b1 = length as u8;
                let b2 = (length >> 8) as u8;
                let u8_nonce: u8 = rng.gen();
                let metadata = vec![u8_nonce, b2, b1];
                let data = vec![data, padding, metadata];
                let flattened = data.into_iter().flatten().collect::<Vec<u8>>();
                let mut data: [u8; FRAGMENT_SIZE] = [0; FRAGMENT_SIZE];
                data.copy_from_slice(&flattened[0..FRAGMENT_SIZE]);
                let mut nonce_reverse = nonce.clone();
                nonce_reverse.0.reverse();
                let shared_secret = response_sid.shared_secret(nonce_reverse, public_identity);
                let mut ctx = ChaCha20Poly1305::new(&shared_secret.as_ref(), &nonce.0, &[]);
                let mut encrypted: Vec<u8> = vec![0; data.len()];
                let mut tag = Tag([0; TAG_SIZE]);
                ctx.encrypt(&data, &mut encrypted[..], &mut tag.0);
                let data = Data::new(&encrypted[..])?;
                Ok(ResponseData::CypherText { data: Box::new(data), tag })
            },
            PublicIdentityInterface::Absent => {
                if data.len() > DATA_SIZE {
                    return Err(anyhow!("Ensure data.len()({}) passed into ResponseData::clear_text() is not greater than {}", data.len(), DATA_SIZE))
                }
                let mut rng = rand::thread_rng();
                let length = data.len();
                //let padding: Vec<u8> = (0..(DATA_SIZE - length)).map(|_| rng.gen()).collect();
                let padding: Vec<u8> = (0..(DATA_SIZE - length)).map(|_| 0).collect();
                let b1 = length as u8;
                let b2 = (length >> 8) as u8;
                let u8_nonce: u8 = rng.gen();
                let metadata = vec![u8_nonce, b2, b1];
                let data = vec![data, padding, metadata];
                let data = Data::new(&data.into_iter().flatten().collect::<Vec<u8>>()[..])?;
                Ok(ResponseData::ClearText { data: Box::new(data) })
            },
        }
    }
    pub fn extract(&self, request_sid: PrivateIdentityInterface, request_pid: PublicIdentityInterface, response_pid: PublicIdentity, nonce: Nonce) -> Result<Vec<u8>> {
        match self {
            ResponseData::ClearText { data } => {
                data.data()
            },
            ResponseData::CypherText { data, tag } => {
                match request_pid {
                    PublicIdentityInterface::Present { public_identity } => {
                        if public_identity != request_sid.public_id() {
                            let err_msg = "The Response's Request_PublicIdentity doesn't match the Public Identity used to sign or decypt the Response";
                            error!("{}", err_msg);
                            return Err(anyhow!(err_msg));
                        }
                        let mut nonce_reverse = nonce.clone();
                        nonce_reverse.0.reverse();
                        let shared_secret = request_sid.shared_secret(nonce_reverse, response_pid);
                        let mut ctx = ChaCha20Poly1305::new(&shared_secret.as_ref(), &nonce.0, &[]);
                        let mut decrypted = [0u8; FRAGMENT_SIZE];
                        if ctx.decrypt(&data.as_bytes(), &mut decrypted[..], &tag.0[..]) {
                            let data: Data = Data::new(&decrypted[..])?;
                            Ok(data.data()?)
                        } else {
                            Err(anyhow!("Couldn't decrypt the data"))
                        }
                    },
                    PublicIdentityInterface::Absent => Err(anyhow!("Cannot determine if the Request's PublicIdentity matches the PublicIdentity used to sign or decrypt the encrypted Response"))
                }
            },
        }
    }
}


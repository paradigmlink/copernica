use {
    crate::{
        constants, Data, PublicIdentity, PrivateIdentityInterface, Tag, Nonce
    },
    std::fmt,
    serde::{Deserialize, Serialize},
    anyhow::{anyhow, Result},
    rand::Rng,
    cryptoxide::{chacha20poly1305::{ChaCha20Poly1305}},
    //log::{debug},
};
#[derive(Clone, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ResponseData {
    ClearText {
        data: Data,
    },
    CypherText {
        data: Data,
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
    pub fn reconstitute_cypher_text(tag: [u8; constants::TAG_SIZE], data: Data) -> Self {
        ResponseData::CypherText { tag, data }
    }
    pub fn reconstitute_clear_text(data: Data) -> Self {
        ResponseData::ClearText { data }
    }
    pub fn clear_text(data: Vec<u8>) -> Result<Self> {
        if data.len() > constants::DATA_SIZE {
            return Err(anyhow!("Ensure data.len()({}) passed into ResponseData::clear_text() is not greater than {}", data.len(), constants::DATA_SIZE))
        }
        let mut rng = rand::thread_rng();
        let length = data.len();
        //let padding: Vec<u8> = (0..(constants::DATA_SIZE - length)).map(|_| rng.gen()).collect();
        let padding: Vec<u8> = (0..(constants::DATA_SIZE - length)).map(|_| 0).collect();
        let b1 = length as u8;
        let b2 = (length >> 8) as u8;
        let u8_nonce: u8 = rng.gen();
        let metadata = vec![u8_nonce, b2, b1];
        let data = vec![data, padding, metadata];
        let data = Data::new(data.into_iter().flatten().collect::<Vec<u8>>())?;
        Ok(ResponseData::ClearText { data })
    }
    pub fn cypher_text(response_sid: PrivateIdentityInterface, request_pid: PublicIdentity, data: Vec<u8>, nonce: Nonce) -> Result<Self> {
        if data.len() > constants::DATA_SIZE {
            return Err(anyhow!("Ensure data.len() passed into ResponseData::cypher_text() is not greater than {}", constants::DATA_SIZE))
        }
        let mut rng = rand::thread_rng();
        let length = data.len();
        let padding: Vec<u8> = (0..(constants::DATA_SIZE - length)).map(|_| rng.gen()).collect();
        //let padding: Vec<u8> = (0..(constants::DATA_SIZE - length)).map(|_| 0).collect();
        let b1 = length as u8;
        let b2 = (length >> 8) as u8;
        let u8_nonce: u8 = rng.gen();
        let metadata = vec![u8_nonce, b2, b1];
        let data = vec![data, padding, metadata];
        let flattened = data.into_iter().flatten().collect::<Vec<u8>>();
        let mut data: [u8; constants::FRAGMENT_SIZE] = [0; constants::FRAGMENT_SIZE];
        data.copy_from_slice(&flattened[0..constants::FRAGMENT_SIZE]);
        let mut nonce_reverse = nonce;
        nonce_reverse.reverse();
        let shared_secret = response_sid.shared_secret(nonce_reverse, request_pid);
        let mut ctx = ChaCha20Poly1305::new(&shared_secret.as_ref(), &nonce, &[]);
        let mut encrypted: Vec<u8> = vec![0; data.len()];
        let mut tag: Tag = [0; constants::TAG_SIZE];
        ctx.encrypt(&data, &mut encrypted[..], &mut tag);
        let data = Data::new(encrypted)?;
        Ok(ResponseData::CypherText { data, tag })
    }
    pub fn cleartext_data(&self) -> Result<Vec<u8>> {
        let data = match self {
            ResponseData::ClearText { data } => {
                data.data()
            },
            ResponseData::CypherText { .. } => {
                return Err(anyhow!("Cannot obtain the cleartext for encrypted data, use the decrypt_data method instead"))
            },
        };
        data
    }
    pub fn decrypt_data(&self, request_sid: PrivateIdentityInterface, response_pid: PublicIdentity, nonce: Nonce) -> Result<Vec<u8>> {
        match self {
            ResponseData::CypherText { data, tag } => {
                let mut nonce_reverse = nonce;
                nonce_reverse.reverse();
                let shared_secret = request_sid.shared_secret(nonce_reverse, response_pid);
                let mut ctx = ChaCha20Poly1305::new(&shared_secret.as_ref(), &nonce, &[]);
                let mut decrypted = [0u8; constants::FRAGMENT_SIZE];
                if ctx.decrypt(&data.raw_data(), &mut decrypted[..], &tag[..]) {
                    let data: Data = Data::new(decrypted.to_vec())?;
                    Ok(data.data()?)
                } else {
                    return Err(anyhow!("Couldn't decrypt the data"))
                }
            },
            ResponseData::ClearText { data } => {
                Ok(data.data()?)
            },
        }
    }
    pub fn manifest_data(&self) -> Vec<u8> {
        match self {
            ResponseData::ClearText { data } => { data.raw_data() },
            ResponseData::CypherText { data, tag } => {
                [data.raw_data(), tag[..].to_vec()].concat()
            },
        }
    }
}


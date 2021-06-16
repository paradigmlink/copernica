use {
    crate::{
        hbfi::HBFI,
        constants,
        serialization::*,
    },
    serde::{Deserialize, Serialize},
    std::fmt,
    rand_core::{CryptoRng, RngCore},
    anyhow::{anyhow, Result},
    //log::{debug},
};
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Nonce(pub [u8; constants::NONCE_SIZE]);
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct Tag(pub [u8; constants::TAG_SIZE]);
// the below is a hack, I don't know how to implement Serialize/Deserialize for struct Data(pub [u8; constants::FRAGMENT_SIZE])
#[derive(Clone, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct Data(Vec<u8>);
impl Data {
    pub fn new(data: Vec<u8>) -> Result<Data> {
        if data.len() != constants::FRAGMENT_SIZE {
            return Err(anyhow!("Ensure data.len() passed into Data::new() is equal to {}", constants::FRAGMENT_SIZE))
        }
        Ok(Data(data))
    }
    pub fn raw_data(&self) -> Vec<u8> {
        self.0.clone()
    }
    pub fn data(&self) -> Result<Vec<u8>> {
        let length_combined = format!("{:02x}{:02x}", self.0[constants::LENGTH_OF_DATA_STARTING_POSITION], self.0[constants::LENGTH_OF_DATA_ENDING_POSITION]);
        let length = u16::from_str_radix(&length_combined, 16)?;
        let (data, _) = self.0.split_at(length as usize);
        Ok(data.to_vec())
    }
}

impl PartialEq for Data {
    fn eq(&self, other: &Data) -> bool {
        if self.0.len() == other.0.len() {
            for i in 0..self.0.len() {
                if self.0[i] != other.0[i] {
                    return false
                }
            }
            return true
        } else {
            false
        }
    }
}
impl Eq for Data {}
impl fmt::Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

pub fn manifest(data: Vec<u8>, hbfi: &HBFI, nonce: &Nonce) -> Result<Vec<u8>> {
    let (_hbfi_size, hbfi) = serialize_hbfi(hbfi)?;
    let manifest = [data, hbfi, nonce.0.to_vec()].concat();
    Ok(manifest)
}

pub fn generate_nonce<R>(rng: &mut R) -> Nonce
where
    R: RngCore + CryptoRng,
{
    let mut nonce = Nonce([0; constants::NONCE_SIZE]);
    rng.fill_bytes(&mut nonce.0);
    nonce
}

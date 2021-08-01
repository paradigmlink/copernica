use {
    copernica_common::{constants::*, u8_to_u16 },
    std::fmt,
    anyhow::{anyhow, Result},
    //log::{debug},
};
#[derive(Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Debug)]
pub struct Data([u8; FRAGMENT_SIZE]);
impl Data {
    pub fn new(data_in: &[u8]) -> Result<Self> {
        if data_in.len() != FRAGMENT_SIZE {
            return Err(anyhow!("Ensure data.len() passed into Data::new() is equal to {}", FRAGMENT_SIZE))
        }
        let mut data: [u8; FRAGMENT_SIZE] = [0u8; FRAGMENT_SIZE];
        data.clone_from_slice(&data_in[..]);
        Ok(Data(data))
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.0[..]
    }
    pub fn data(&self) -> Result<Vec<u8>> {
        let length = u8_to_u16([self.0[LENGTH_OF_DATA_STARTING_POSITION], self.0[LENGTH_OF_DATA_ENDING_POSITION]]);
        let (data, _) = self.0.split_at(length as usize);
        Ok(data.to_vec())
    }
}
impl fmt::Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

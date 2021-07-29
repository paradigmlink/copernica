use crate::constants;
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Nonce(pub [u8; constants::NONCE_SIZE]);
impl Nonce {
    pub fn as_bytes(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

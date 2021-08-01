use {
  crate::{
    constants::NONCE_SIZE,
  },
  rand::RngCore,
};
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Nonce(pub [u8; NONCE_SIZE]);
impl Nonce {
    pub fn as_bytes(&self) -> Vec<u8> {
        self.0.to_vec()
    }
    pub fn from_bytes(data: &[u8]) -> Self {
        let mut nonce = [0u8; NONCE_SIZE];
        nonce.clone_from_slice(&data[..NONCE_SIZE]);
        Self(nonce)
    }
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let mut nonce = Nonce([0; NONCE_SIZE]);
        rng.fill_bytes(&mut nonce.0);
        nonce
    }
}

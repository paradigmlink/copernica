use {
    crate::{
        hbfi::HBFI,
        Nonce,
        constants,
    },
    rand_core::{CryptoRng, RngCore},
    anyhow::{Result},
};
pub fn manifest(data: Vec<u8>, hbfi: &HBFI, nonce: &Nonce) -> Result<Vec<u8>> {
    let hbfi_s = hbfi.as_bytes();
    let manifest = [data, hbfi_s, nonce.0.to_vec()].concat();
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

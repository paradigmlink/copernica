use {
    crate::{
        Nonce, HBFI, ResponseData,
        constants,
    },
    rand_core::{CryptoRng, RngCore},
};
pub fn manifest(data: &ResponseData, hbfi: &HBFI, nonce: &Nonce) ->  Vec<u8> {
    [data.as_bytes(), hbfi.as_bytes(), nonce.as_bytes()].concat()
}
pub fn generate_nonce<R>(rng: &mut R) -> Nonce
where
    R: RngCore + CryptoRng,
{
    let mut nonce = Nonce([0; constants::NONCE_SIZE]);
    rng.fill_bytes(&mut nonce.0);
    nonce
}

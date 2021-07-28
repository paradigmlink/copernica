use {
    crate::{
        constants,
    },
};
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Nonce(pub [u8; constants::NONCE_SIZE]);

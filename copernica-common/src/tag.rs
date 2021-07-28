use {
    crate::{
        constants,
    },
};
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Tag(pub [u8; constants::TAG_SIZE]);

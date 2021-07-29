use crate::constants;
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Tag(pub [u8; constants::TAG_SIZE]);
impl Tag {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0[..]
    }
}

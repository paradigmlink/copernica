use crate::constants::TAG_SIZE;
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Tag(pub [u8; TAG_SIZE]);
impl Tag {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0[..]
    }
    pub fn from_bytes(data: &[u8]) -> Self {
        let mut tag = [0u8; TAG_SIZE];
        tag.clone_from_slice(&data[..TAG_SIZE]);
        Self(tag)
    }
    pub fn new_empty_tag() -> Self{
        let tag = [0u8; TAG_SIZE];
        Self(tag)
    }
}

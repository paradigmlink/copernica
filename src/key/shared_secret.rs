use crate::memsec::{self, Scrubbed};
use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
    fmt::{self, Debug, Formatter},
};

/// A Shared Secret that can be used to generate a symmetric key
#[derive(Clone)]
pub struct SharedSecret([u8; 32]);

impl SharedSecret {
    pub const SIZE: usize = 32;

    pub(crate) const fn new(shared_secret: [u8; 32]) -> Self {
        Self(shared_secret)
    }
}

/* Format ****************************************************************** */

#[cfg(test)]
impl Debug for SharedSecret {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("SharedSecret<Ed25519>")
            .field("0", &hex::encode(&self.0))
            .finish()
    }
}

#[cfg(not(test))]
impl Debug for SharedSecret {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        #[cfg(feature = "nightly")]
        {
            f.debug_struct("SharedSecret<Ed25519>")
                .finish_non_exhaustive()
        }

        #[cfg(not(feature = "nightly"))]
        {
            f.debug_struct("SharedSecret<Ed25519>")
                .field("0", &"...")
                .finish()
        }
    }
}

/* Eq ********************************************************************** */

impl PartialEq<Self> for SharedSecret {
    fn eq(&self, other: &Self) -> bool {
        unsafe { memsec::memeq(self.0.as_ptr(), other.0.as_ptr(), Self::SIZE) }
    }
}

impl Eq for SharedSecret {}

/* Ord ********************************************************************* */

impl PartialOrd<Self> for SharedSecret {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SharedSecret {
    fn cmp(&self, other: &Self) -> Ordering {
        unsafe { memsec::memcmp(self.0.as_ptr(), other.0.as_ptr(), Self::SIZE) }
    }
}

/* Hash ******************************************************************** */

impl Hash for SharedSecret {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.as_ref().hash(state)
    }
}

/* AsRef ******************************************************************* */

impl AsRef<[u8]> for SharedSecret {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

/* Drop ******************************************************************** */

/// custom implementation of Drop so we can have more certainty that
/// the shared secret raw data will be scrubbed (zeroed) before releasing
/// the memory
impl Drop for SharedSecret {
    fn drop(&mut self) {
        self.0.scrub()
    }
}
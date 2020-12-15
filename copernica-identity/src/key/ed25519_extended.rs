use crate::{
    key::SharedSecret,
    memsec::{self, Scrubbed as _},
};
use cryptoxide::{
    curve25519::{curve25519, Fe},
    ed25519,
};
use rand_core::{CryptoRng, RngCore};
use std::{
    convert::TryFrom,
    fmt::{self, Debug, Formatter},
    hash::{Hash, Hasher},
    str::FromStr,
};
use thiserror::Error;

#[derive(Clone)]
pub struct SecretKey([u8; Self::SIZE]);

pub use crate::key::ed25519::{PublicKey, PublicKeyError, Signature};

impl SecretKey {
    pub const SIZE: usize = ed25519::PRIVATE_KEY_LENGTH;

    /// create a dummy instance of the object but filled with zeroes
    #[inline(always)]
    const fn zero() -> Self {
        Self([0; Self::SIZE])
    }

    /// generate a new `SecretKey` with the given random number generator
    ///
    pub fn new<Rng>(rng: &mut Rng) -> Self
    where
        Rng: RngCore + CryptoRng,
    {
        let mut s = Self::zero();
        rng.fill_bytes(&mut s.0);

        s.0[0] &= 0b1111_1000;
        s.0[31] &= 0b0011_1111;
        s.0[31] |= 0b0100_0000;

        debug_assert!(
            s.check_structure(),
            "checking we properly set the bit tweaks for the extended Ed25519"
        );

        s
    }

    pub(crate) fn clear_3rd_highest_bit(&mut self) {
        self.0[31] &= 0b1101_1111;
    }

    pub(crate) fn is_3rd_highest_bit_clear(&self) -> bool {
        (self.0[31] & 0b0010_0000) == 0
    }

    #[allow(clippy::verbose_bit_mask)]
    fn check_structure(&self) -> bool {
        (self.0[0] & 0b0000_0111) == 0
            && (self.0[31] & 0b0100_0000) == 0b0100_0000
            && (self.0[31] & 0b1000_0000) == 0
    }

    /// get the `PublicKey` associated to this key
    ///
    /// Unlike the `SecretKey`, the `PublicKey` can be safely
    /// publicly shared. The key can then be used to verify any
    /// `Signature` generated with this `SecretKey` and the original
    /// message.
    pub fn public_key(&self) -> PublicKey {
        let pk = ed25519::to_public(&self.0);

        PublicKey::from(pk)
    }

    /// generate a shared secret between the owner of the given public key and
    /// ourselves.
    ///
    pub fn exchange(&self, public_key: &PublicKey) -> SharedSecret {
        let ed_y = Fe::from_bytes(public_key.as_ref());
        let mont_x = edwards_to_montgomery_x(&ed_y);

        SharedSecret::new(curve25519(&self.0, &mont_x.to_bytes()))
    }

    /// create a `Signature` for the given message with this `SecretKey`.
    ///
    /// The `Signature` can then be verified against the associated `PublicKey`
    /// and the original message.
    pub fn sign<T: AsRef<[u8]>>(&self, msg: T) -> Signature {
        let signature = ed25519::signature_extended(msg.as_ref(), &self.0);

        Signature::from(signature)
    }

    /// get a reference to the inner Seed bytes
    ///
    /// # Security Consideration
    ///
    /// be mindful that leaking the content of the internal signing key
    /// may result in losing the ultimate control of the signing key
    pub fn leak_as_ref(&self) -> &[u8; Self::SIZE] {
        &self.0
    }

    pub fn leak_to_hex(&self) -> String {
        hex::encode(self.0.as_ref())
    }
}

#[inline]
fn edwards_to_montgomery_x(ed_y: &Fe) -> Fe {
    let ed_z = &Fe([1, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    let temp_x = ed_z + ed_y;
    let temp_z = ed_z - ed_y;
    let temp_z_inv = temp_z.invert();

    temp_x * temp_z_inv
}

/* Format ****************************************************************** */

/// conveniently provide a proper implementation to debug for the
/// SecretKey types when only *testing* the library
#[cfg(test)]
impl Debug for SecretKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("SecretKey<Ed25519Extended>")
            .field("0", &hex::encode(&self.0[..]))
            .finish()
    }
}

/// conveniently provide an incomplete implementation of Debug for the
/// SecretKey.
#[cfg(not(test))]
impl Debug for SecretKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        #[cfg(feature = "nightly")]
        {
            f.debug_struct("SecretKey<Ed25519Extended>")
                .finish_non_exhaustive()
        }

        #[cfg(not(feature = "nightly"))]
        {
            f.debug_struct("SecretKey<Ed25519Extended>")
                //.field("0", &hex::encode(&self.0.as_ref()))
                .field("0", &"...")
                .finish()
        }
    }
}

/* Conversion ************************************************************** */

#[derive(Debug, Error)]
pub enum SecretKeyError {
    #[error("Invalid size, expecting {}", SecretKey::SIZE)]
    InvalidSize,
    #[error("Invalid structure")]
    InvalidStructure,
}

impl TryFrom<[u8; Self::SIZE]> for SecretKey {
    type Error = SecretKeyError;

    fn try_from(bytes: [u8; Self::SIZE]) -> Result<Self, Self::Error> {
        let s = Self(bytes);
        if s.check_structure() {
            Ok(s)
        } else {
            Err(Self::Error::InvalidStructure)
        }
    }
}

impl<'a> TryFrom<&'a [u8]> for SecretKey {
    type Error = SecretKeyError;
    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        if value.len() != Self::SIZE {
            Err(Self::Error::InvalidSize)
        } else {
            let mut s = [0; Self::SIZE];
            s.copy_from_slice(value);
            Self::try_from(s)
        }
    }
}

impl FromStr for SecretKey {
    type Err = hex::FromHexError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut r = Self::zero();
        hex::decode_to_slice(s, &mut r.0)?;
        Ok(r)
    }
}

/* Eq ********************************************************************** */

impl PartialEq<Self> for SecretKey {
    fn eq(&self, other: &Self) -> bool {
        unsafe { memsec::memeq(self.0.as_ptr(), other.0.as_ptr(), Self::SIZE) }
    }
}

impl Eq for SecretKey {}

/* Hash ******************************************************************** */

impl Hash for SecretKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.as_ref().hash(state)
    }
}

/* Drop ******************************************************************** */

/// custom implementation of Drop so we can have more certainty that
/// the signing key raw data will be scrubbed (zeroed) before releasing
/// the memory
impl Drop for SecretKey {
    fn drop(&mut self) {
        self.0.scrub()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key::ed25519;
    use quickcheck::{Arbitrary, Gen, TestResult};

    impl Arbitrary for SecretKey {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let mut s = Self::zero();
            g.fill_bytes(&mut s.0);

            s.0[0] &= 0b1111_1000;
            s.0[31] &= 0b0011_1111;
            s.0[31] |= 0b0100_0000;

            s
        }
    }

    #[quickcheck]
    fn verify_exchange_works(alice: SecretKey, bob: SecretKey) -> bool {
        let alice_pk = alice.public_key();
        let bob_pk = bob.public_key();

        alice.exchange(&bob_pk) == bob.exchange(&alice_pk)
    }

    #[quickcheck]
    fn signing_verify_works(signing_key: SecretKey, message: Vec<u8>) -> bool {
        let public_key = signing_key.public_key();
        let signature = signing_key.sign(&message);

        public_key.verify(message, &signature)
    }

    #[quickcheck]
    fn signing_key_try_from_correct_size(signing_key: SecretKey) -> TestResult {
        match SecretKey::try_from(signing_key.leak_as_ref().as_ref()) {
            Ok(_) => TestResult::passed(),
            Err(SecretKeyError::InvalidSize) => {
                TestResult::error("was expecting the test to pass, not an invalid size")
            }
            Err(SecretKeyError::InvalidStructure) => {
                TestResult::error("was expecting the test to pass, not an invalid structure")
            }
        }
    }

    #[quickcheck]
    fn signing_key_try_from_incorrect_size(bytes: Vec<u8>) -> TestResult {
        if bytes.len() == SecretKey::SIZE {
            return TestResult::discard();
        }
        match SecretKey::try_from(bytes.as_slice()) {
            Ok(_) => TestResult::error(
                "Expecting to fail with invalid size instead of having a valid value",
            ),
            Err(SecretKeyError::InvalidSize) => TestResult::passed(),
            Err(SecretKeyError::InvalidStructure) => {
                TestResult::error("was expecting an invalid size error, not an invalid structure")
            }
        }
    }

    #[quickcheck]
    fn signing_key_from_str(signing_key: SecretKey) -> TestResult {
        let s = signing_key.leak_to_hex();

        match s.parse::<SecretKey>() {
            Ok(decoded) => {
                if decoded == signing_key {
                    TestResult::passed()
                } else {
                    TestResult::error("the decoded key is not equal")
                }
            }
            Err(error) => TestResult::error(error.to_string()),
        }
    }

    #[quickcheck]
    fn compatibility_exchange(sk: ed25519::SecretKey, ske: SecretKey) -> bool {
        let pk = sk.public_key();
        let pke = ske.public_key();

        sk.exchange(&pke) == ske.exchange(&pk)
    }
}

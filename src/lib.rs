#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

pub mod key;
pub mod memsec;
mod shielded;

use rand_core::{CryptoRng, RngCore};
use self::memsec::Scrubbed as _;
pub use self::{key::{SharedSecret, ed25519_extended::Signature}, shielded::Shielded};

#[deprecated = "currently unstable API, but now way to mark an API as unstable in the library..."]
const KEYNESIS_PATH_SIGNING_V1:       &[u8] = b"/keynesis/v1/signing";
#[deprecated = "currently unstable API, but now way to mark an API as unstable in the library..."]
const KEYNESIS_PATH_EXCHANGE_V1_ROOT: &[u8] = b"/keynesis/v1/exchange";

/// private identity, to keep close to you, privately and securely
///
/// From this root key multiple key may be generated depending on
/// the needs and protocols.
///
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct PrivateIdentity(key::ed25519_hd::SecretKey);

/// TODO: add function for easy conversion from different bytes types
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone)]
pub struct PublicId(key::ed25519_hd::PublicKey);

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct PrivateSigningKey(key::ed25519_extended::SecretKey);

/// TODO: add function for easy conversion from different bytes types
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone)]
pub struct PublicVerifyKey(key::ed25519_extended::PublicKey);

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct SecretKey(key::ed25519_extended::SecretKey);

/// TODO: add function for easy conversion from different bytes types
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone)]
pub struct PublicKey(key::ed25519_extended::PublicKey);

impl PrivateIdentity {
    /// create a new private identity. This will be the root of all
    /// the other derived keys
    ///
    /// Once generated it is possible to use the `shield` function to
    /// password protect the secret key. Or, if you used a Pseudo
    /// Random Generator, you can use the seed to reconstruct the
    /// initial RNG and re-generate the key with this function
    ///
    pub fn new<RNG>(rng: &mut RNG) -> Self
    where RNG: CryptoRng + RngCore
    {
        Self(key::ed25519_hd::SecretKey::new(rng))
    }

    /// retrieve the `PrivateIdentity` from the shielded data
    /// and the given key
    #[deprecated = "currently unstable API, but now way to mark an API as unstable in the library..."]
    pub fn unshield<K>(shielded: &Shielded, key: K) -> Option<Self>
    where K: AsRef<[u8]>
    {
        use std::convert::TryFrom as _;
        let mut data = shielded::decrypt(key, shielded.as_ref())?;

        let key = key::ed25519_hd::SecretKey::try_from(data.as_slice()).ok()?;

        data.scrub();
        Some(Self(key))
    }

    /// password protect the PrivateIdentity
    ///
    /// Use a strong password to protect your identity
    #[deprecated = "currently unstable API, but now way to mark an API as unstable in the library..."]
    pub fn shield<RNG, K>(&self, rng: &mut RNG, key: K) -> Shielded
    where RNG: CryptoRng + RngCore,
          K: AsRef<[u8]>
    {
        let mut data = [0; key::ed25519_hd::SecretKey::SIZE];
        data[..key::ed25519_extended::SecretKey::SIZE].copy_from_slice(self.0.key().leak_as_ref());
        data[key::ed25519_extended::SecretKey::SIZE..].copy_from_slice(self.0.chain().as_ref());

        let shielded = shielded::encrypt(rng, key, &data).into();

        data.scrub();

        shielded
    }

    pub fn public_id(&self) -> PublicId {
        PublicId(self.0.public_key())
    }

    #[deprecated = "currently unstable API, but now way to mark an API as unstable in the library..."]
    pub fn signing_key(&self) -> PrivateSigningKey {
        PrivateSigningKey(
            self.0.derive(KEYNESIS_PATH_SIGNING_V1).into_key()
        )
    }

    #[deprecated = "currently unstable API, but now way to mark an API as unstable in the library..."]
    pub fn derive<P>(&self, purpose: P) -> SecretKey
    where P: AsRef<[u8]>
    {
        let mut path = KEYNESIS_PATH_EXCHANGE_V1_ROOT.to_vec();
        path.extend_from_slice(purpose.as_ref());
        SecretKey(self.0.derive(path).into_key())
    }
}

impl PublicId {
    #[deprecated = "currently unstable API, but now way to mark an API as unstable in the library..."]
    pub fn verify_key(&self) -> PublicVerifyKey {
        PublicVerifyKey(
            self.0.derive(KEYNESIS_PATH_SIGNING_V1).unwrap().into_key()
        )
    }

    #[deprecated = "currently unstable API, but now way to mark an API as unstable in the library..."]
    pub fn derive<P>(&self, purpose: P) -> PublicKey
    where P: AsRef<[u8]>
    {
        let mut path = KEYNESIS_PATH_EXCHANGE_V1_ROOT.to_vec();
        path.extend_from_slice(purpose.as_ref());
        PublicKey(self.0.derive(path).unwrap().into_key())
    }
}

impl PrivateSigningKey {
    pub fn public(&self) -> PublicVerifyKey {
        PublicVerifyKey(self.0.public_key())
    }

    pub fn sign<M>(&self, message: M) -> Signature
    where M: AsRef<[u8]>
    {
        self.0.sign(message)
    }
}

impl PublicVerifyKey {
    pub fn verify<M>(&self, signature: &Signature, message: M) -> bool
    where M: AsRef<[u8]>{
        self.0.verify(message, signature)
    }
}

impl SecretKey {
    pub fn public(&self) -> PublicKey {
        PublicKey(self.0.public_key())
    }

    pub fn exchange(&self, key: &PublicKey) -> SharedSecret {
        self.0.exchange(&key.0)
    }
}
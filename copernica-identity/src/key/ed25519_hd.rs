use crate::{
    key::{ed25519_extended, SharedSecret},
    memsec::Scrubbed as _,
};
use cryptoxide::{
    curve25519::{ge_scalarmult_base, GeP3},
    hmac::Hmac,
    mac::Mac,
    sha2::Sha512,
};
use rand_core::{CryptoRng, RngCore};
use std::{
    convert::TryFrom,
    fmt::{self, Debug, Display, Formatter},
    ops::Deref,
    str::FromStr,
};
use thiserror::Error;

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct ChainCode([u8; Self::SIZE]);

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct SecretKey {
    key: ed25519_extended::SecretKey,
    chain_code: ChainCode,
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct PublicKey {
    key: ed25519_extended::PublicKey,
    chain_code: ChainCode,
}

pub use crate::key::ed25519::Signature;

impl ChainCode {
    pub const SIZE: usize = 32;

    /// create a dummy instance of the object but filled with zeroes
    #[inline(always)]
    const fn zero() -> Self {
        Self([0; Self::SIZE])
    }

    pub fn from_bytes(bytes: [u8; Self::SIZE]) -> Self {
        Self(bytes)
    }

    /// generate a new `SecretKey` with the given random number generator
    ///
    fn new<Rng>(rng: &mut Rng) -> Self
    where
        Rng: RngCore + CryptoRng,
    {
        let mut s = Self::zero();
        rng.fill_bytes(&mut s.0);
        s
    }
}

impl SecretKey {
    pub const SIZE: usize = ed25519_extended::SecretKey::SIZE + ChainCode::SIZE;

    /// generate a new `SecretKey` with the given random number generator
    ///
    pub fn new<Rng>(rng: &mut Rng) -> Self
    where
        Rng: RngCore + CryptoRng,
    {
        let mut key = ed25519_extended::SecretKey::new(rng);
        let chain_code = ChainCode::new(rng);

        key.clear_3rd_highest_bit();

        let s = Self { key, chain_code };

        debug_assert!(
            s.key.is_3rd_highest_bit_clear(),
            "checking we properly set the bit tweaks for the extended Ed25519 BIP32"
        );

        s
    }

    #[inline]
    pub fn is_3rd_highest_bit_clear(&self) -> bool {
        self.key.is_3rd_highest_bit_clear()
    }

    /// get the `PublicKey` associated to this key
    ///
    /// Unlike the `SecretKey`, the `PublicKey` can be safely
    /// publicly shared. The key can then be used to verify any
    /// `Signature` generated with this `SecretKey` and the original
    /// message.
    pub fn public_key(&self) -> PublicKey {
        let key = self.key.public_key();
        let chain_code = self.chain_code;

        PublicKey { key, chain_code }
    }

    pub fn chain(&self) -> &ChainCode {
        &self.chain_code
    }

    pub fn key(&self) -> &ed25519_extended::SecretKey {
        &self.key
    }

    pub fn into_key(self) -> ed25519_extended::SecretKey {
        self.key
    }

    pub fn leak_to_hex(&self) -> String {
        format!("{}{}", self.key.leak_to_hex(), self.chain())
    }

    /// generate a shared secret between the owner of the given public key and
    /// ourselves.
    ///
    pub fn exchange(&self, public_key: &PublicKey) -> SharedSecret {
        self.key.exchange(public_key)
    }

    /// create a `Signature` for the given message with this `SecretKey`.
    ///
    /// The `Signature` can then be verified against the associated `PublicKey`
    /// and the original message.
    pub fn sign<T: AsRef<[u8]>>(&self, msg: T) -> Signature {
        self.key.sign(msg)
    }

    pub fn derive<P>(&self, path: P) -> Self
    where
        P: AsRef<[u8]>,
    {
        let e_key = &self.key.leak_as_ref()[0..64];
        let kl = &e_key[0..32];
        let kr = &e_key[32..64];
        let chaincode = self.chain_code.as_ref();

        let mut z_mac = Hmac::new(Sha512::new(), &chaincode);
        let mut i_mac = Hmac::new(Sha512::new(), &chaincode);
        let pk = self.public_key();
        let pk = pk.key().as_ref();
        z_mac.input(&[0x2]);
        z_mac.input(&pk);
        z_mac.input(path.as_ref());
        i_mac.input(&[0x3]);
        i_mac.input(&pk);
        i_mac.input(path.as_ref());

        let mut z_out = [0u8; 64];
        z_mac.raw_result(&mut z_out);
        let zl = &z_out[0..32];
        let zr = &z_out[32..64];

        // left = kl + 8 * trunc28(zl)
        let mut left = add_28_mul8(kl, zl);
        // right = zr + kr
        let mut right = add_256bits(kr, zr);

        let mut i_out = [0u8; 64];
        i_mac.raw_result(&mut i_out);
        let cc = &i_out[32..];

        let mut out = [0u8; Self::SIZE];
        out[0..32].clone_from_slice(&left);
        out[32..64].clone_from_slice(&right);
        out[64..96].clone_from_slice(cc);

        i_mac.reset();
        z_mac.reset();

        z_out.scrub();
        left.scrub();
        right.scrub();

        Self::try_from(out).unwrap()
    }
}

impl PublicKey {
    pub const SIZE: usize = ed25519_extended::PublicKey::SIZE + ChainCode::SIZE;

    pub fn key(&self) -> &ed25519_extended::PublicKey {
        &self.key
    }

    pub fn into_key(self) -> ed25519_extended::PublicKey {
        self.key
    }

    pub fn chain_code(&self) -> &ChainCode {
        &self.chain_code
    }

    pub fn reconstitute(key: ed25519_extended::PublicKey, chain_code: ChainCode) -> Self {
        PublicKey { key, chain_code }
    }

    pub fn derive<P>(&self, path: P) -> Option<Self>
    where
        P: AsRef<[u8]>,
    {
        let pk = self.key().as_ref();
        let chaincode = self.chain_code().as_ref();

        let mut z_mac = Hmac::new(Sha512::new(), &chaincode);
        let mut i_mac = Hmac::new(Sha512::new(), &chaincode);
        z_mac.input(&[0x2]);
        z_mac.input(&pk);
        z_mac.input(path.as_ref());
        i_mac.input(&[0x3]);
        i_mac.input(&pk);
        i_mac.input(path.as_ref());

        let mut z_out = [0u8; 64];
        z_mac.raw_result(&mut z_out);
        let zl = &z_out[0..32];
        let _zr = &z_out[32..64];

        // left = kl + 8 * trunc28(zl)
        let left = point_plus(pk, &point_of_trunc28_mul8(zl))?;

        let mut i_out = [0u8; 64];
        i_mac.raw_result(&mut i_out);
        let cc = &i_out[32..];

        let mut out = [0u8; Self::SIZE];
        out[..ed25519_extended::PublicKey::SIZE].copy_from_slice(&left);
        out[ed25519_extended::PublicKey::SIZE..].copy_from_slice(cc);

        i_mac.reset();
        z_mac.reset();

        Some(Self::from(out))
    }
}

/* *************************************************************** */

fn point_of_trunc28_mul8(sk: &[u8]) -> [u8; 32] {
    assert!(sk.len() == 32);
    let copy = add_28_mul8(&[0u8; 32], sk);
    let a = ge_scalarmult_base(&copy);
    a.to_bytes()
}

fn point_plus(p1: &[u8], p2: &[u8]) -> Option<[u8; 32]> {
    let a = GeP3::from_bytes_negate_vartime(p1)?;
    let b = GeP3::from_bytes_negate_vartime(p2)?;
    let r = a + b.to_cached();
    let mut r = r.to_p2().to_bytes();
    r[31] ^= 0x80;
    Some(r)
}

fn add_28_mul8(x: &[u8], y: &[u8]) -> [u8; 32] {
    assert!(x.len() == 32);
    assert!(y.len() == 32);

    let mut carry: u16 = 0;
    let mut out = [0u8; 32];

    for i in 0..28 {
        let r = x[i] as u16 + ((y[i] as u16) << 3) + carry;
        out[i] = (r & 0xff) as u8;
        carry = r >> 8;
    }
    for i in 28..32 {
        let r = x[i] as u16 + carry;
        out[i] = (r & 0xff) as u8;
        carry = r >> 8;
    }
    out
}

fn add_256bits(x: &[u8], y: &[u8]) -> [u8; 32] {
    assert!(x.len() == 32);
    assert!(y.len() == 32);

    let mut carry: u16 = 0;
    let mut out = [0u8; 32];
    for i in 0..32 {
        let r = (x[i] as u16) + (y[i] as u16) + carry;
        out[i] = r as u8;
        carry = r >> 8;
    }
    out
}

/* Deref ******************************************************************* */

impl Deref for PublicKey {
    type Target = ed25519_extended::PublicKey;
    fn deref(&self) -> &Self::Target {
        self.key()
    }
}

/* Format ****************************************************************** */

impl Debug for ChainCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("ChainCode")
            .field("0", &hex::encode(&self.0))
            .finish()
    }
}

impl Debug for SecretKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("SecretKey<Ed25519BIP32>")
            .field("key", &self.key)
            .field("chain_code", &self.chain_code)
            .finish()
    }
}

impl Debug for PublicKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("PublicKey<Ed25519BIP32>")
            .field("key", &self.key)
            .field("chain_code", &self.chain_code)
            .finish()
    }
}

impl Display for ChainCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&hex::encode(&self.0), f)
    }
}

impl Display for PublicKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.key())?;
        write!(f, "{}", self.chain_code())
    }
}

/* Conversion ************************************************************** */

impl From<[u8; Self::SIZE]> for ChainCode {
    fn from(bytes: [u8; Self::SIZE]) -> Self {
        Self(bytes)
    }
}

impl From<[u8; Self::SIZE]> for PublicKey {
    fn from(bytes: [u8; Self::SIZE]) -> Self {
        let mut key = [0; ed25519_extended::PublicKey::SIZE];
        let mut chain_code = [0; ChainCode::SIZE];
        key.copy_from_slice(&bytes[..ed25519_extended::PublicKey::SIZE]);
        chain_code.copy_from_slice(&bytes[ed25519_extended::PublicKey::SIZE..]);
        Self {
            key: ed25519_extended::PublicKey::from(key),
            chain_code: ChainCode::from(chain_code),
        }
    }
}

#[derive(Debug, Error)]
pub enum ChainCodeError {
    #[error("Invalid size, expecting {}", ChainCode::SIZE)]
    InvalidSize,
}

impl<'a> TryFrom<&'a [u8]> for ChainCode {
    type Error = ChainCodeError;

    fn try_from(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        if bytes.len() != Self::SIZE {
            return Err(Self::Error::InvalidSize);
        }
        let mut chain_code = ChainCode::zero();
        chain_code.0.copy_from_slice(bytes);
        Ok(chain_code)
    }
}

#[derive(Debug, Error)]
pub enum PublicKeyError {
    #[error("Invalid size, expecting {}", PublicKey::SIZE)]
    InvalidSize,
    #[error("Invalid verify key")]
    InvalidPublicKey(
        #[from]
        #[source]
        ed25519_extended::PublicKeyError,
    ),
    #[error("Invalid chain code")]
    InvalidChainCode(
        #[from]
        #[source]
        ChainCodeError,
    ),
}

impl<'a> TryFrom<&'a [u8]> for PublicKey {
    type Error = PublicKeyError;

    fn try_from(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        if bytes.len() != Self::SIZE {
            return Err(Self::Error::InvalidSize);
        }

        let key =
            ed25519_extended::PublicKey::try_from(&bytes[..ed25519_extended::PublicKey::SIZE])?;
        let chain_code = ChainCode::try_from(&bytes[ed25519_extended::PublicKey::SIZE..])?;

        Ok(Self { key, chain_code })
    }
}

#[derive(Debug, Error)]
pub enum SecretKeyError {
    #[error("Invalid size, expecting {}", SecretKey::SIZE)]
    InvalidSize,
    #[error("Invalid chain code")]
    InvalidChainCode(
        #[from]
        #[source]
        ChainCodeError,
    ),
    #[error("Invalid structure")]
    InvalidStructure,
    #[error("Invalid hexadecimal string")]
    InvalidHexadecimal(
        #[source]
        #[from]
        hex::FromHexError,
    ),
}

impl TryFrom<[u8; Self::SIZE]> for SecretKey {
    type Error = SecretKeyError;

    fn try_from(bytes: [u8; Self::SIZE]) -> Result<Self, Self::Error> {
        Self::try_from(&bytes[..])
    }
}

impl<'a> TryFrom<&'a [u8]> for SecretKey {
    type Error = SecretKeyError;
    fn try_from(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        if bytes.len() != Self::SIZE {
            return Err(Self::Error::InvalidSize);
        }

        match ed25519_extended::SecretKey::try_from(&bytes[..ed25519_extended::SecretKey::SIZE]) {
            Ok(key) => {
                // we do not check the 3rd highest bit is cleared here as potentially
                // derived keys may overflow as per the bip32 paper. However if the
                // key is expected to be a root key, the check_3rd_highest_bit function
                // needs called to make sure the structure is valid.
                let chain_code = ChainCode::try_from(
                    &bytes[ed25519_extended::SecretKey::SIZE..]
                )?;
                Ok(Self { key, chain_code })
            }
            Err(ed25519_extended::SecretKeyError::InvalidSize) => {
                unreachable!("The Size({}) is already checked, expecting an extended key of {} and a chain code of {}", SecretKey::SIZE, ed25519_extended::SecretKey::SIZE, ChainCode::SIZE)
            }
            Err(ed25519_extended::SecretKeyError::InvalidStructure) => {
                Err(Self::Error::InvalidStructure)
            }
        }
    }
}

impl FromStr for SecretKey {
    type Err = SecretKeyError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut r = [0; Self::SIZE];
        hex::decode_to_slice(s, &mut r)?;

        let sk = Self::try_from(&r[..])?;

        r.scrub();

        Ok(sk)
    }
}

impl FromStr for PublicKey {
    type Err = hex::FromHexError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut r = [0; Self::SIZE];
        hex::decode_to_slice(s, &mut r)?;
        Ok(Self::from(r))
    }
}

impl FromStr for ChainCode {
    type Err = hex::FromHexError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut r = [0; Self::SIZE];
        hex::decode_to_slice(s, &mut r)?;
        Ok(Self::from(r))
    }
}

/* AsRef ******************************************************************* */

impl AsRef<[u8]> for ChainCode {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::{Arbitrary, Gen, TestResult};

    impl Arbitrary for ChainCode {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let mut s = Self::zero();
            g.fill_bytes(&mut s.0);
            s
        }
    }

    impl Arbitrary for PublicKey {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            SecretKey::arbitrary(g).public_key()
        }
    }

    impl Arbitrary for SecretKey {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let key = ed25519_extended::SecretKey::arbitrary(g);
            let chain_code = ChainCode::arbitrary(g);

            // NOTE:
            //   we actually don't call this one function on purpose
            //   as it may be that the highest bit is not set and that
            //   the derived key overflew
            //
            // key.clear_3rd_highest_bit();

            Self { key, chain_code }
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
        let mut bytes = signing_key.key.leak_as_ref().to_vec();
        bytes.extend(&signing_key.chain_code.0);
        match SecretKey::try_from(bytes.as_slice()) {
            Ok(_) => TestResult::passed(),
            Err(SecretKeyError::InvalidSize) => {
                TestResult::error("was expecting the test to pass, not an invalid size")
            }
            Err(SecretKeyError::InvalidChainCode(ChainCodeError::InvalidSize)) => {
                unreachable!("The total size of the key is already being checked")
            }
            Err(SecretKeyError::InvalidStructure) => {
                TestResult::error("was expecting the test to pass, not an invalid structure")
            }
            Err(SecretKeyError::InvalidHexadecimal(_)) => {
                unreachable!("We should not see an hexadecimal error at all in this test")
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
            Err(SecretKeyError::InvalidChainCode(ChainCodeError::InvalidSize)) => {
                unreachable!("The total size of the key is already being checked")
            }
            Err(SecretKeyError::InvalidStructure) => {
                TestResult::error("was expecting an invalid size error, not an invalid structure")
            }
            Err(SecretKeyError::InvalidHexadecimal(_)) => {
                unreachable!("We should not see an hexadecimal error at all in this test")
            }
        }
    }

    #[quickcheck]
    fn public_key_try_from_correct_size(public_key: PublicKey) -> TestResult {
        let mut bytes = public_key.key.as_ref().to_vec();
        bytes.extend(&public_key.chain_code.0);
        match PublicKey::try_from(bytes.as_slice()) {
            Ok(_) => TestResult::passed(),
            Err(PublicKeyError::InvalidSize) => {
                TestResult::error("was expecting the test to pass, not an invalid size")
            }
            Err(PublicKeyError::InvalidPublicKey(
                ed25519_extended::PublicKeyError::InvalidSize,
            )) => unreachable!("The total size of the key is already being checked"),
            Err(PublicKeyError::InvalidChainCode(ChainCodeError::InvalidSize)) => {
                unreachable!("The total size of the key is already being checked")
            }
        }
    }

    #[quickcheck]
    fn public_key_try_from_incorrect_size(bytes: Vec<u8>) -> TestResult {
        if bytes.len() == PublicKey::SIZE {
            return TestResult::discard();
        }
        match PublicKey::try_from(bytes.as_slice()) {
            Ok(_) => TestResult::error(
                "Expecting to fail with invalid size instead of having a valid value",
            ),
            Err(PublicKeyError::InvalidSize) => TestResult::passed(),
            Err(PublicKeyError::InvalidPublicKey(
                ed25519_extended::PublicKeyError::InvalidSize,
            )) => unreachable!("The total size of the key is already being checked"),
            Err(PublicKeyError::InvalidChainCode(ChainCodeError::InvalidSize)) => {
                unreachable!("The total size of the key is already being checked")
            }
        }
    }

    #[quickcheck]
    fn chain_code_try_from_correct_size(chain_code: ChainCode) -> TestResult {
        match ChainCode::try_from(chain_code.0.as_ref()) {
            Ok(_) => TestResult::passed(),
            Err(ChainCodeError::InvalidSize) => {
                TestResult::error("was expecting the test to pass, not an invalid size")
            }
        }
    }

    #[quickcheck]
    fn chain_code_try_from_incorrect_size(bytes: Vec<u8>) -> TestResult {
        if bytes.len() == ChainCode::SIZE {
            return TestResult::discard();
        }
        match ChainCode::try_from(bytes.as_slice()) {
            Ok(_) => TestResult::error(
                "Expecting to fail with invalid size instead of having a valid value",
            ),
            Err(ChainCodeError::InvalidSize) => TestResult::passed(),
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
    fn public_key_from_str(public_key: PublicKey) -> TestResult {
        let mut bytes = public_key.key.as_ref().to_vec();
        bytes.extend(&public_key.chain_code.0);
        let s = hex::encode(bytes);

        match s.parse::<PublicKey>() {
            Ok(decoded) => {
                if decoded == public_key {
                    TestResult::passed()
                } else {
                    TestResult::error("the decoded key is not equal")
                }
            }
            Err(error) => TestResult::error(error.to_string()),
        }
    }

    #[quickcheck]
    fn chain_code_from_str(chain_code: ChainCode) -> TestResult {
        let s = hex::encode(&chain_code);

        match s.parse::<ChainCode>() {
            Ok(decoded) => {
                if decoded == chain_code {
                    TestResult::passed()
                } else {
                    TestResult::error("the decoded chain_code is not equal")
                }
            }
            Err(error) => TestResult::error(error.to_string()),
        }
    }

    #[quickcheck]
    fn derivation_from_signing_and_public_key(root_key: SecretKey, path: Vec<u8>) -> TestResult {
        let root_public_key = root_key.public_key();

        let d1 = root_key.derive(&path);
        let d2 = root_public_key.derive(path).unwrap();

        TestResult::from_bool(Some(d1.public_key()) == Some(d2))
    }

    #[quickcheck]
    fn different_derivation_from_signing_key(
        root_key: SecretKey,
        path1: Vec<u8>,
        path2: Vec<u8>,
    ) -> TestResult {
        if path1 == path2 {
            return TestResult::discard();
        }

        let dp1 = root_key.derive(&path1);
        let dp2 = root_key.derive(&path2);

        TestResult::from_bool(dp1 != dp2)
    }

    #[quickcheck]
    fn different_derivation_from_public_key(
        root_key: PublicKey,
        path1: Vec<u8>,
        path2: Vec<u8>,
    ) -> TestResult {
        if path1 == path2 {
            return TestResult::discard();
        }

        let dp1 = root_key.derive(&path1).unwrap();
        let dp2 = root_key.derive(&path2).unwrap();

        dbg!(hex::encode(&path2));
        dbg!(hex::encode(&path1));
        dbg!(&dp1);
        dbg!(&dp2);
        TestResult::from_bool(dp1 != dp2)
    }
}

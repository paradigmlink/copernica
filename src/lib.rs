/*!
# Keynesis: key management for signing and e2e communication

Keynesis leverage the curve25519 and ed25519 to provide some keys
and APIs to use for different purpose

## Identity

The identity is the **pivot** component of the scheme. It is the root keys
from which everything can be derived. There is a `PrivateIdentity` and a
`PublicId`. The `PrivateIdentity` key needs to be kept private while the
`PublicId` can be safely shared... "Publicly".

Conveniently, the `PrivateIdentity` exposes the `shield` method to safely
password protect the content of the `PrivateIdentity`. The scheme uses
(HMAC PBKDF2 with 10_000 iteration to derive the password into the key of
32 bytes long and a salt of 16 bytes long; for the encryption we use
ChaCha20Poly1305 with a nonce of 12 bytes).

```
# use rand::thread_rng as secure_rng;
# const PASSWORD: &str = "Very Secure Password. This is important: it's my private identity!";
use keynesis::PrivateIdentity;

let private_id = PrivateIdentity::new(&mut secure_rng());
let public_id = private_id.public_id();
println!("Public Identity: {}", public_id);

let shielded_private_id = private_id.shield(&mut secure_rng(), PASSWORD);
println!("Shielded Private Identity: {}", shielded_private_id);
#
# let unshielded = PrivateIdentity::unshield(&shielded_private_id, PASSWORD).unwrap();
# assert_eq!(private_id, unshielded);
```

## Signing Keys

From the `PrivateIdentity` it is possible to "derive" the `PrivateSigningKey`.
This key can then be used to sign messages that can be verified with the
associated `VerifyPublicKey`. This key can be retrieved from the `PrivateSigningKey`
or it can be derived from the `PublicId`.

```
# use rand::thread_rng as secure_rng;
# use keynesis::PrivateIdentity;
#
# let private_id = PrivateIdentity::new(&mut secure_rng());
# let public_id = private_id.public_id();
#
# const MESSAGE: &str = "Important message to sign";
let signing_key = private_id.signing_key();
let verify_key = public_id.verify_key();
# assert_eq!(verify_key, signing_key.public());

let signature = signing_key.sign(MESSAGE);
assert!(verify_key.verify(&signature, MESSAGE));
```

## Establishing secure stream

Now it is possible to generate a `SharedSecret` between 2 `PrivateIdentity` owners
so long they have each other's `PublicId`. To do so we will can generate keys
from the identity keys using an arbitrarily defined scheme to generate a
derivation _path_.

### Alice's and Bob's generating each other's key

First alice will generate her key and send the `PublicId` to Bob.

```
# use rand::thread_rng as secure_rng;
use keynesis::PrivateIdentity;

let alice_private_id = PrivateIdentity::new(&mut secure_rng());
let alice_public_id = alice_private_id.public_id();

// send `alice_public_id` to Bob
```

Then Bob does the same and send the `PublicId` to Alice.

```
# use rand::thread_rng as secure_rng;
use keynesis::PrivateIdentity;

let bob_private_id = PrivateIdentity::new(&mut secure_rng());
let bob_public_id = bob_private_id.public_id();

// send `bob_public_id` to Alice
```

### Generating the keys to establish a shared secret

Now that they both have each other's `PublicId` they can
generate each other's public key. Let say Alice is the initiator
of the channel and tells Bob to generate a `SharedSecret` to
establish an encrypted connection with the nonce.

Alice will have generate the secret key as follow:

```
# use rand::thread_rng as secure_rng;
# use keynesis::PrivateIdentity;
#
# const NONCE: &str = "A nonce for our secure communication channel";
#
# let alice_private_id = PrivateIdentity::new(&mut secure_rng());
# let alice_public_id = alice_private_id.public_id();
#
# let bob_private_id = PrivateIdentity::new(&mut secure_rng());
# let bob_public_id = bob_private_id.public_id();
#
let alice_path = format!("/keynesis/alice/bob/{}", NONCE);
let bob_path = format!("/keynesis/bob/alice/{}", NONCE);

let alice_private_key = alice_private_id.derive(&alice_path);
let bob_public_key = bob_public_id.derive(&bob_path);
#
# assert_eq!(alice_private_key.public(), alice_public_id.derive(alice_path));
# assert_eq!(bob_public_key, bob_private_id.derive(&bob_path).public());

let alice_shared_secret = alice_private_key.exchange(&bob_public_key);
```

Once the request received by Bob (the nonce), bob can derived
the appropriate key on her side:

```
# use rand::thread_rng as secure_rng;
# use keynesis::PrivateIdentity;
#
# const NONCE: &str = "A nonce for our secure communication channel";
#
# let alice_private_id = PrivateIdentity::new(&mut secure_rng());
# let alice_public_id = alice_private_id.public_id();
#
# let bob_private_id = PrivateIdentity::new(&mut secure_rng());
# let bob_public_id = bob_private_id.public_id();
#
let alice_path = format!("/keynesis/alice/bob/{}", NONCE);
let bob_path = format!("/keynesis/bob/alice/{}", NONCE);

let bob_private_key = bob_private_id.derive(&bob_path);
let alice_public_key = alice_public_id.derive(&alice_path);
#
# assert_eq!(bob_private_key.public(), bob_public_id.derive(bob_path));
# assert_eq!(alice_public_key, alice_private_id.derive(alice_path).public());

let bob_shared_secret = bob_private_key.exchange(&alice_public_key);
```

Now these 2 shared secrets (alice's and bob's) are both the **same**.

```
# use rand::thread_rng as secure_rng;
# use keynesis::PrivateIdentity;
#
# const NONCE: &str = "A nonce for our secure communication channel";
#
# let alice_private_id = PrivateIdentity::new(&mut secure_rng());
# let alice_public_id = alice_private_id.public_id();
#
# let bob_private_id = PrivateIdentity::new(&mut secure_rng());
# let bob_public_id = bob_private_id.public_id();
#
# let alice_path = format!("/keynesis/alice/bob/{}", NONCE);
# let bob_path = format!("/keynesis/bob/alice/{}", NONCE);
#
# let bob_private_key = bob_private_id.derive(&bob_path);
# let alice_private_key = alice_private_id.derive(&alice_path);
#
# let alice_shared_secret = alice_private_key.exchange(&bob_private_key.public());
# let bob_shared_secret = bob_private_key.exchange(&alice_private_key.public());
assert_eq!(alice_shared_secret, bob_shared_secret);
```

The `SharedSecret` can now be used to seed a symmetric cipher, [`ChaCha20`]
for example (don't forget an extra random nonce to make the cipher more
secure).

[`ChaCha20`]: https://docs.rs/cryptoxide/0.2.1/cryptoxide/chacha20/index.html
*/

#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

pub mod key;
pub mod memsec;
mod shielded;

use self::memsec::Scrubbed as _;
pub use self::{
    key::{ed25519_extended::Signature, SharedSecret},
    shielded::Shielded,
};
use rand_core::{CryptoRng, RngCore};
use std::{
    convert::{TryFrom, TryInto as _},
    fmt::{self, Display, Formatter},
    str::FromStr,
};

#[deprecated = "currently unstable API, but now way to mark an API as unstable in the library..."]
const KEYNESIS_PATH_SIGNING_V1: &[u8] = b"/keynesis/v1/signing";
#[deprecated = "currently unstable API, but now way to mark an API as unstable in the library..."]
const KEYNESIS_PATH_EXCHANGE_V1_ROOT: &[u8] = b"/keynesis/v1/exchange";

/// private identity, to keep close to you, privately and securely
///
/// From this root key multiple key may be generated depending on
/// the needs and protocols.
///
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct PrivateIdentity(key::ed25519_hd::SecretKey);

/// Public identity
///
/// Making this public (include the public key and the chain code)
/// allows for anyone to derive the public key associated to the
/// different schemes (signing or key exchange).
///
/// This key cannot be used for anything else, we restrict its usage
/// to public derivation of different keys
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone)]
pub struct PublicId(key::ed25519_hd::PublicKey);

/// The Signing Key associated to your `PrivateIdentity`.
///
/// This key is derived from the `PrivateIdentity`. Anyone with
/// the `PublicId` key can derivate the associated `PublicVerifyKey`
/// and verify any signature generated with this key.
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct PrivateSigningKey(key::ed25519_extended::SecretKey);

/// The Verify Key associated to the `PrivateSigningKey`.
///
/// Any signature generated by the `PrivateSigningKey` can be
/// verified with this key. It is not necessary to share this
/// key as it can be derived from the `PublicId`.
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone)]
pub struct PublicVerifyKey(key::ed25519::PublicKey);

/// Secret key to use for key exchange
///
/// This key is derived from the `PrivateIdentity` and are used
/// to established a key exchange (Diffi-Hellman) with a `PublicKey`.
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct SecretKey(key::ed25519_extended::SecretKey);

/// Public key to use for key exchange
///
/// This key is derived from the `PublicId` and is meant for
/// establishing a key exchange (Diffi-Hellman). It is not
/// necessary to share this key as it can be derived from the
/// `PublicId` (assuming the derivation path is known by both
/// party).
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone)]
pub struct PublicKey(key::ed25519::PublicKey);

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
    where
        RNG: CryptoRng + RngCore,
    {
        Self(key::ed25519_hd::SecretKey::new(rng))
    }

    /// retrieve the `PrivateIdentity` from the shielded data
    /// and the given key
    #[deprecated = "currently unstable API, but now way to mark an API as unstable in the library..."]
    pub fn unshield<K>(shielded: &Shielded, key: K) -> Option<Self>
    where
        K: AsRef<[u8]>,
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
    where
        RNG: CryptoRng + RngCore,
        K: AsRef<[u8]>,
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
        PrivateSigningKey(self.0.derive(KEYNESIS_PATH_SIGNING_V1).into_key())
    }

    #[deprecated = "currently unstable API, but now way to mark an API as unstable in the library..."]
    pub fn derive<P>(&self, purpose: P) -> SecretKey
    where
        P: AsRef<[u8]>,
    {
        let mut path = KEYNESIS_PATH_EXCHANGE_V1_ROOT.to_vec();
        path.extend_from_slice(purpose.as_ref());
        SecretKey(self.0.derive(path).into_key())
    }
}

impl PublicId {
    pub const SIZE: usize = key::ed25519_hd::PublicKey::SIZE;

    #[deprecated = "currently unstable API, but now way to mark an API as unstable in the library..."]
    pub fn verify_key(&self) -> PublicVerifyKey {
        PublicVerifyKey(self.0.derive(KEYNESIS_PATH_SIGNING_V1).unwrap().into_key())
    }

    #[deprecated = "currently unstable API, but now way to mark an API as unstable in the library..."]
    pub fn derive<P>(&self, purpose: P) -> PublicKey
    where
        P: AsRef<[u8]>,
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
    where
        M: AsRef<[u8]>,
    {
        self.0.sign(message)
    }
}

impl PublicVerifyKey {
    pub const SIZE: usize = key::ed25519::PublicKey::SIZE;

    pub fn verify<M>(&self, signature: &Signature, message: M) -> bool
    where
        M: AsRef<[u8]>,
    {
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

impl PublicKey {
    pub const SIZE: usize = key::ed25519::PublicKey::SIZE;
}

/* Formatter *************************************************************** */

impl Display for PublicId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Display for PublicKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Display for PublicVerifyKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for PublicId {
    type Err = hex::FromHexError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Self)
    }
}

impl FromStr for PublicVerifyKey {
    type Err = hex::FromHexError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Self)
    }
}

impl FromStr for PublicKey {
    type Err = hex::FromHexError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Self)
    }
}

/* Conversion ************************************************************** */

impl From<[u8; Self::SIZE]> for PublicId {
    fn from(bytes: [u8; Self::SIZE]) -> Self {
        Self(bytes.into())
    }
}

impl From<[u8; Self::SIZE]> for PublicVerifyKey {
    fn from(bytes: [u8; Self::SIZE]) -> Self {
        Self(bytes.into())
    }
}

impl From<[u8; Self::SIZE]> for PublicKey {
    fn from(bytes: [u8; Self::SIZE]) -> Self {
        Self(bytes.into())
    }
}

impl<'a> TryFrom<&'a [u8]> for PublicId {
    type Error = key::ed25519_hd::PublicKeyError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        value.try_into().map(Self)
    }
}

impl<'a> TryFrom<&'a [u8]> for PublicVerifyKey {
    type Error = key::ed25519::PublicKeyError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        value.try_into().map(Self)
    }
}

impl<'a> TryFrom<&'a [u8]> for PublicKey {
    type Error = key::ed25519::PublicKeyError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        value.try_into().map(Self)
    }
}

/* AsRef ******************************************************************* */

impl AsRef<[u8]> for PublicId {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl AsRef<[u8]> for PublicVerifyKey {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl AsRef<[u8]> for PublicKey {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

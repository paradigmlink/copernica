use {
    keynesis::{
        key::{
            ed25519_hd,
            ed25519_extended,
            ed25519,
            ed25519::Signature, SharedSecret
        },
        Seed,
    },
    anyhow::{anyhow},
    serde::{Deserialize, Serialize},
    std::{
        convert::{TryFrom, TryInto as _},
        fmt::{self, Display, Formatter},
        str::FromStr,
    },
    thiserror::Error,
};
const KEYNESIS_PATH_SIGNING_V1: &[u8] = b"/keynesis/v1/signing";
const KEYNESIS_PATH_EXCHANGE_V1_ROOT: &[u8] = b"/keynesis/v1/exchange";

/// private identity, to keep close to you, privately and securely
///
/// From this root key multiple key may be generated depending on
/// the needs and protocols.
///
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct PrivateIdentity(ed25519_hd::SecretKey);

/// Public identity
///
/// Making this public (include the public key and the chain code)
/// allows for anyone to derive the public key associated to the
/// different schemes (signing or key exchange).
///
/// This key cannot be used for anything else, we restrict its usage
/// to public derivation of different keys
///
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct PublicIdentity(ed25519_hd::PublicKey);

/// The Signing Key associated to your `PrivateIdentity`.
///
/// This key is derived from the `PrivateIdentity`. Anyone with
/// the `PublicIdentity` key can derivate the associated `PublicVerifyKey`
/// and verify any signature generated with this key.
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct PrivateSigningKey(ed25519_extended::SecretKey);

/// The Verify Key associated to the `PrivateSigningKey`.
///
/// Any signature generated by the `PrivateSigningKey` can be
/// verified with this key. It is not necessary to share this
/// key as it can be derived from the `PublicIdentity`.
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone)]
pub struct PublicVerifyKey(ed25519::PublicKey);

/// Secret key to use for key exchange
///
/// This key is derived from the `PrivateIdentity` and are used
/// to established a key exchange (Diffie-Hellman) with a `PublicKey`.
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct SecretKey(ed25519_extended::SecretKey);

/// Public key to use for key exchange
///
/// This key is derived from the `PublicIdentity` and is meant for
/// establishing a key exchange (Diffie-Hellman). It is not
/// necessary to share this key as it can be derived from the
/// `PublicIdentity` (assuming the derivation path is known by both
/// party).
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone)]
pub struct PublicKey(ed25519::PublicKey);

impl PrivateIdentity {
    /// build a private identity from the given `Seed`
    ///
    pub fn from_seed(seed: Seed) -> Self {
        let mut rng = seed.into_rand_chacha();
        Self(ed25519_hd::SecretKey::new(&mut rng))
    }
    pub fn public_id(&self) -> PublicIdentity {
        PublicIdentity(self.0.public_key())
    }

    pub fn signing_key(&self) -> PrivateSigningKey {
        PrivateSigningKey(self.0.derive(KEYNESIS_PATH_SIGNING_V1).into_key())
    }

    pub fn derive<P>(&self, purpose: P) -> SecretKey
    where
        P: AsRef<[u8]>,
    {
        let mut path = KEYNESIS_PATH_EXCHANGE_V1_ROOT.to_vec();
        path.extend_from_slice(purpose.as_ref());
        SecretKey(self.0.derive(path).into_key())
    }
}

impl PublicIdentity {
    pub const SIZE: usize = ed25519_hd::PublicKey::SIZE;

    pub fn verify_key(&self) -> anyhow::Result<PublicVerifyKey> {
        //PublicVerifyKey(self.0.derive(KEYNESIS_PATH_SIGNING_V1).unwrap().into_key())
        if let Some(derived) = self.0.derive(KEYNESIS_PATH_SIGNING_V1) {
            return Ok(PublicVerifyKey(derived.into_key()))
        } else {
            return Err(anyhow!("PublicKey Derive returned None"))
        }
    }

    pub fn derive<P>(&self, purpose: P) -> PublicKey
    where
        P: AsRef<[u8]>,
    {
        let mut path = KEYNESIS_PATH_EXCHANGE_V1_ROOT.to_vec();
        path.extend_from_slice(purpose.as_ref());
        PublicKey(self.0.derive(path).unwrap().into_key())
    }

    pub fn key(&self) -> &ed25519_extended::PublicKey {
        self.0.key()
    }

    pub fn chain_code(&self) -> &ed25519_hd::ChainCode {
        self.0.chain_code()
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
    pub const SIZE: usize = ed25519::PublicKey::SIZE;

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
    pub const SIZE: usize = ed25519::PublicKey::SIZE;
}

/* Formatter *************************************************************** */

impl Display for PublicIdentity {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use bech32::ToBase32 as _;
        let mut writer = bech32::Bech32Writer::new("id", f)?;

        let mut bytes = self.0.key().as_ref().to_vec();
        bytes.extend_from_slice(self.0.chain_code().as_ref());

        // self.0.key().write_base32(&mut writer)?;
        // self.0.chain_code().write_base32(&mut writer)?;
        bytes.write_base32(&mut writer)?;

        writer.finalize()
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

#[derive(Debug, Error)]
pub enum PublicIdentityError {
    #[error("Not a valid bech32 encoded PublicIdentity")]
    InvalidBech32(
        #[source]
        #[from]
        bech32::Error,
    ),

    #[error("Invalid key type prefix, expected 'id' but received {hrp}")]
    InvalidHRP { hrp: String },

    #[error("Invalid public key")]
    InvalidKey(
        #[source]
        #[from]
        ed25519_hd::PublicKeyError,
    ),
}

impl FromStr for PublicIdentity {
    type Err = PublicIdentityError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use bech32::FromBase32 as _;

        let (hrp, data) = bech32::decode(s)?;
        if hrp != "id" {
            return Err(Self::Err::InvalidHRP { hrp });
        }
        let data = Vec::<u8>::from_base32(&data)?;
        let pid = ed25519_hd::PublicKey::try_from(data.as_slice()).map(Self)?;
        Ok(pid)
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

impl From<PublicIdentity> for String {
    fn from(pid: PublicIdentity) -> Self {
        pid.to_string()
    }
}

impl From<[u8; Self::SIZE]> for PublicIdentity {
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

impl TryFrom<String> for PublicIdentity {
    type Error = <Self as FromStr>::Err;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}
impl<'a> TryFrom<&'a str> for PublicIdentity {
    type Error = <Self as FromStr>::Err;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl<'a> TryFrom<&'a [u8]> for PublicIdentity {
    type Error = ed25519_hd::PublicKeyError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        value.try_into().map(Self)
    }
}

impl<'a> TryFrom<&'a [u8]> for PublicVerifyKey {
    type Error = ed25519::PublicKeyError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        value.try_into().map(Self)
    }
}

impl<'a> TryFrom<&'a [u8]> for PublicKey {
    type Error = ed25519::PublicKeyError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        value.try_into().map(Self)
    }
}


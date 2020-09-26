use crate::memsec::Scrubbed;
use cryptoxide::{chacha20poly1305::ChaCha20Poly1305, hmac::Hmac, pbkdf2::pbkdf2, sha2::Sha512};
use rand_core::{CryptoRng, RngCore};
use std::{
    fmt::{self, Display, Formatter},
    io::Read,
    str::FromStr,
};

const PASSWORD_DERIVATION_ITERATIONS: u32 = 10_000;
const SALT_SIZE: usize = 16;

const NONCE_SIZE: usize = 12;
const TAG_SIZE: usize = 16;
const KEY_SIZE: usize = 32;

type Key = [u8; KEY_SIZE];
type Salt = [u8; SALT_SIZE];
type Nonce = [u8; NONCE_SIZE];

pub struct Shielded(Vec<u8>);

pub(crate) fn encrypt<R, P>(mut rng: R, password: P, data: &[u8]) -> Vec<u8>
where
    R: RngCore + CryptoRng,
    P: AsRef<[u8]>,
{
    let salt = generate_salt(&mut rng);
    let nonce = generate_nonce(&mut rng);
    let mut key = [0; KEY_SIZE];
    let mut tag = [0; TAG_SIZE];
    let len = data.len();

    let mut bytes = Vec::with_capacity(SALT_SIZE + NONCE_SIZE + len + TAG_SIZE);
    let mut encrypted: Vec<u8> = vec![0; data.len()];

    password_to_key(password, salt, &mut key);
    let mut ctx = ChaCha20Poly1305::new(&key[..], &nonce[..], &[]);
    key.scrub();

    ctx.encrypt(data, &mut encrypted[0..len], &mut tag);

    bytes.extend_from_slice(&salt);
    bytes.extend_from_slice(&nonce);
    bytes.append(&mut encrypted);
    bytes.extend_from_slice(&tag[..]);

    bytes
}

pub(crate) fn decrypt<P: AsRef<[u8]>>(password: P, data: &[u8]) -> Option<Vec<u8>> {
    let mut reader = data;
    let mut salt = [0; SALT_SIZE];
    let mut nonce = [0; NONCE_SIZE];
    let mut key = [0; KEY_SIZE];
    let len = data.len() - TAG_SIZE - SALT_SIZE - NONCE_SIZE;
    let mut bytes: Vec<u8> = vec![0; len];

    reader.read_exact(&mut salt[..]).unwrap();
    reader.read_exact(&mut nonce[..]).unwrap();

    password_to_key(password, salt, &mut key);
    let mut ctx = ChaCha20Poly1305::new(&key[..], &nonce[..], &[]);
    key.scrub();

    if ctx.decrypt(&reader[0..len], &mut bytes[..], &reader[len..]) {
        Some(bytes)
    } else {
        None
    }
}

fn password_to_key<P: AsRef<[u8]>>(password: P, salt: Salt, key: &mut Key) {
    let mut mac = Hmac::new(Sha512::new(), password.as_ref());

    pbkdf2(&mut mac, &salt[..], PASSWORD_DERIVATION_ITERATIONS, key);
}

fn generate_salt<R>(rng: &mut R) -> Salt
where
    R: RngCore + CryptoRng,
{
    let mut salt: Salt = [0; SALT_SIZE];
    rng.fill_bytes(&mut salt);
    salt
}

fn generate_nonce<R>(rng: &mut R) -> Nonce
where
    R: RngCore + CryptoRng,
{
    let mut nonce: Nonce = [0; NONCE_SIZE];
    rng.fill_bytes(&mut nonce);
    nonce
}

impl Display for Shielded {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        hex::encode(&self.0).fmt(f)
    }
}

impl FromStr for Shielded {
    type Err = hex::FromHexError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hex::decode(s).map(Self)
    }
}

impl AsRef<[u8]> for Shielded {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl From<Vec<u8>> for Shielded {
    fn from(v: Vec<u8>) -> Self {
        Self(v)
    }
}

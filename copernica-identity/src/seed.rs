use cryptoxide::{hmac::Hmac, pbkdf2::pbkdf2, sha2::Sha512};
use rand_chacha::ChaChaRng;
use rand_core::{CryptoRng, RngCore, SeedableRng};
use crate::memsec::Scrubbed as _;
use std::{
    fmt::{self, Debug, Display, Formatter},
    str::FromStr
};

#[derive(Clone)]
pub struct Seed([u8; Self::SIZE]);

impl Seed {
    pub const SIZE: usize = 32;

    /// Generate a random see with the given Cryptographically secure
    /// Random Number Generator (RNG).
    ///
    /// This is useful to generate one time only `Seed` that does not
    /// need to be remembered or saved.
    pub fn generate<RNG>(rng: &mut RNG) -> Self
    where
        RNG: RngCore + CryptoRng,
    {
        let mut bytes = [0; Self::SIZE];
        rng.fill_bytes(&mut bytes);
        Self(bytes)
    }

    /// it is possible to derive the Seed from a given key
    ///
    /// the key may be given by a secure hardware or simply be
    /// a mnemonic phrase given by a user and a password.
    ///
    /// It is possible, but not recommended, that the password is left
    /// empty. However, the key needs to be large enough to generate
    /// enough entropy for the derived seed.
    ///
    /// This function uses HMAC PBKDF2 SHA512 with `10_000` iteration
    pub fn derive_from_key<K, P>(key: K, password: P) -> Self
    where
        K: AsRef<[u8]>,
        P: AsRef<[u8]>,
    {
        let iteration = iteration(key.as_ref().len() as u32);
        let mut bytes = [0; Self::SIZE];

        let mut mac = Hmac::new(Sha512::new(), password.as_ref());

        pbkdf2(&mut mac, key.as_ref(), iteration, &mut bytes);

        Self(bytes)
    }

    pub(crate) fn into_rand_chacha(self) -> ChaChaRng {
        ChaChaRng::from_seed(self.0)
    }
}

impl Drop for Seed {
    fn drop(&mut self) {
        self.0.scrub()
    }
}

impl Display for Seed {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&hex::encode(&self.0), f)
    }
}

impl Debug for Seed {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Seed")
        .field("0", &hex::encode(&self.0))
        .finish()
    }
}

impl FromStr for Seed {
    type Err = hex::FromHexError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut bytes = [0; Self::SIZE];
        hex::decode_to_slice(s, &mut bytes)?;
        Ok(Self(bytes))
    }
}

impl From<[u8; Self::SIZE]> for Seed {
    fn from(seed: [u8; Self::SIZE]) -> Self {
        Self(seed)
    }
}

fn iteration(n: u32) -> u32 {
    10_000_000u32.wrapping_div_euclid(10u32.saturating_pow(n.div_euclid(8)))
}

use {
    crate::{
        identity::{
            PrivateIdentity, PublicIdentity, SecretKey, PrivateSigningKey
        },
        Seed,
    },
};
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
enum PrivateIdentityState {
    SentinelOne { key: PrivateIdentity },
    FileSystem  { key: PrivateIdentity },
    Key         { key: PrivateIdentity },
}
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct PrivateIdentityInterface {
    inner: PrivateIdentityState,
}
impl PrivateIdentityInterface {
    pub fn new_key() -> Self {
        let mut rng = rand::thread_rng();
        let key = PrivateIdentity::from_seed(Seed::generate(&mut rng));
        Self { inner: PrivateIdentityState::Key { key } }
    }
    pub fn new_fs() -> Self {
        let mut rng = rand::thread_rng();
        let key = PrivateIdentity::from_seed(Seed::generate(&mut rng));
        Self { inner: PrivateIdentityState::FileSystem { key } }
    }
    pub fn new_sentinel() -> Self  {
        let mut rng = rand::thread_rng();
        let key = PrivateIdentity::from_seed(Seed::generate(&mut rng));
        Self { inner: PrivateIdentityState::SentinelOne { key } }
    }
    pub fn public_id(&self) -> PublicIdentity {
        match &self.inner {
            PrivateIdentityState::Key { key } => {
                key.public_id()
            },
            PrivateIdentityState::FileSystem { key } => {
                key.public_id()
            },
            PrivateIdentityState::SentinelOne { key } => {
                key.public_id()
            },
        }
    }
    pub fn derive<P>(&self, path: P) -> SecretKey
    where
        P: AsRef<[u8]>,
    {
        match &self.inner {
            PrivateIdentityState::Key { key } => {
                key.derive(path)
            },
            PrivateIdentityState::FileSystem { key } => {
                key.derive(path)
            },
            PrivateIdentityState::SentinelOne { key } => {
                key.derive(path)
            },
        }
    }
    pub fn signing_key(&self) -> PrivateSigningKey {
        match &self.inner {
            PrivateIdentityState::Key { key } => {
                key.signing_key()
            },
            PrivateIdentityState::FileSystem { key } => {
                key.signing_key()
            },
            PrivateIdentityState::SentinelOne { key } => {
                key.signing_key()
            },
        }
    }
}




use {
    cryptoxide::{chacha20poly1305::ChaCha20Poly1305, hmac::Hmac, pbkdf2::pbkdf2, sha2::Sha512},
    getrandom::getrandom,
    chain_crypto::{Ed25519, PublicKey, SecretKey},
    chain_crypto::bech32::Bech32,
    rand_chacha::ChaChaRng,
    rand_core::{SeedableRng},
    std::{
        io::{Read},
        fmt,
        iter::repeat,
    },
    chain_addr,
    packets::{Packet, response},
};

const DISCRIMINATION: chain_addr::Discrimination = chain_addr::Discrimination::Production;
const ADDRESS_PREFIX: &str = "ceo";

pub fn generate_identity(password: String) -> (String, Packet) {
    let mut buf = [0u8; 32];
    match getrandom(&mut buf) {
        Err(why) => panic!("{:?}", why),
        Ok(buf) => buf,
    };
    let mut rng = ChaChaRng::from_seed(buf);
    let sk: SecretKey<Ed25519> = SecretKey::generate(&mut rng);
    let pk: PublicKey<Ed25519> = sk.to_public();
    let addr = Address(chain_addr::Address(DISCRIMINATION, chain_addr::Kind::Single(pk.clone())));
    let pk_bech32 = pk.clone().to_bech32_str();
    let crypto_material = format!("{}",sk.to_bech32_str());
    let encrypted = encrypt(password.clone(), &crypto_material);
    (addr.to_string(), response(addr.to_string(), format!("{}{}", pk_bech32, encrypted).as_bytes().to_vec()))
}

const PASSWORD_DERIVATION_ITERATIONS: u32 = 20_000;
const SK_SIZE: usize = 69;
const PK_SIZE: usize = SK_SIZE;
const SALT_SIZE: usize = 16;
const NONCE_SIZE: usize = 12;
const TAG_SIZE: usize = 16;
const KEY_SIZE: usize = 32;
const CLEARTEXT_SIZE: usize = SK_SIZE;
const CIPHERTEXT_SIZE: usize = CLEARTEXT_SIZE;
const DIGEST_SIZE: usize = SALT_SIZE + NONCE_SIZE + CIPHERTEXT_SIZE + TAG_SIZE;

pub type Password = [u8];
type Key = [u8; KEY_SIZE];
type Salt = [u8; SALT_SIZE];
type Nonce = [u8; NONCE_SIZE];
type Tag = [u8; TAG_SIZE];
type SK = [u8; SK_SIZE];
type PK = [u8; PK_SIZE];

fn generate_nonce() -> Nonce {
    let mut buf: Nonce = [0u8; NONCE_SIZE];
    match getrandom(&mut buf) {
        Err(why) => panic!("{:?}", why),
        Ok(buf) => buf,
    };
    buf
}

fn generate_salt() -> Salt {
    let mut buf: Salt = [0u8; SALT_SIZE];
    match getrandom(&mut buf) {
        Err(why) => panic!("{:?}", why),
        Ok(buf) => buf,
    };
    buf
}

fn password_to_key(password: &Password, salt: Salt, key: &mut Key) {
    let mut mac = Hmac::new(Sha512::new(), password);
    pbkdf2(&mut mac, &salt[..], PASSWORD_DERIVATION_ITERATIONS, key);
}

pub fn encrypt(password: String, cleartext: &String) -> String {
    let salt = generate_salt();
    let nonce = generate_nonce();
    let mut key: Key = [0; KEY_SIZE];
    password_to_key(password.as_bytes(), salt, &mut key);
    let mut tag: Tag = [0; TAG_SIZE];
    let cleartext: Vec<u8> = cleartext.as_bytes().to_vec();
    let mut ciphertext: Vec<u8> = vec!(0u8; CIPHERTEXT_SIZE);
    let mut cipher = ChaCha20Poly1305::new(&key, &nonce, &[]);
    cipher.encrypt(&cleartext, &mut ciphertext, &mut tag[..]);
    let mut digest: Vec<u8> = Vec::with_capacity(DIGEST_SIZE);
    digest.extend_from_slice(&salt);
    digest.extend_from_slice(&nonce);
    digest.append(&mut ciphertext);
    digest.extend_from_slice(&tag);
    base64::encode(&digest)
}

pub fn decrypt( password: String, digest : String) -> Option<(String, String)> {
    let digest = base64::decode(&digest).unwrap();
    let mut digest = &digest[..];
    let mut salt: Salt = [0; SALT_SIZE];
    let mut nonce: Nonce = [0; NONCE_SIZE];
    let mut key: Key = [0; KEY_SIZE];
    let mut sk: SK = [0; SK_SIZE];
    let mut pk: PK = [0; PK_SIZE];
    let len = digest.len() - TAG_SIZE - SALT_SIZE - NONCE_SIZE;
    let mut cleartext: Vec<u8> = repeat(0).take(len).collect();
    digest.read_exact(&mut salt[..]).unwrap();
    digest.read_exact(&mut nonce[..]).unwrap();
    password_to_key(password.as_bytes(), salt, &mut key);
    let mut decipher = ChaCha20Poly1305::new(&key[..], &nonce[..], &[]);
    if decipher.decrypt(&digest[0..len], &mut cleartext[..], &digest[len..]) {
        let mut cleartext = &cleartext[..];
        cleartext.read_exact(&mut sk[..]).unwrap();
        cleartext.read_exact(&mut pk[..]).unwrap();
        let sk = String::from_utf8(sk.to_vec()).unwrap();
        let pk = String::from_utf8(pk.to_vec()).unwrap();
        Some((sk, pk))
    } else {
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Address(chain_addr::Address);

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        chain_addr::AddressReadable::from_address(ADDRESS_PREFIX, &self.0).fmt(f)
    }
}

/*!
# Keynesis's Keys

This module defines some kind of keys that can be used for signing,
for deriving a shared secret between 2 keys or building a hierarchical
deterministic keys (see below).

# Key Derivation

The key defined there can be derived. Just like with BIP32 but instead
of using integer for the derivation the derivation's path is a slice
of bytes, allowing the users to build any kind of hierarchy without
limiting themselves to 2^32 options.

It is then possible to define a root master key and to derive specific
purpose keys from this root key.

```
use keynesis::key::ed25519_hd::SecretKey;
# use rand::thread_rng;

# let bob_root_key = SecretKey::new(&mut thread_rng());
# let bob_root_pk = bob_root_key.public_key();
let alice_root_key = SecretKey::new(&mut thread_rng());
# let alice_root_pk = alice_root_key.public_key();

let alice_key_exchange_with_bob = alice_root_key.derive(b"encryption:bob");
# let bob_key_exchange_with_alice = bob_root_key.derive(b"encryption:alice");

// alice retrieves the public key from bob's root public key
// and uses it for the key exchange
let shared_secret_with_bob = alice_key_exchange_with_bob.exchange(
    &bob_root_pk.derive(b"encryption:alice").unwrap()
);

// bob can compute the shared secret with alice's root public key
let shared_secret_with_alice = bob_key_exchange_with_alice.exchange(
    &alice_root_pk.derive(b"encryption:bob").unwrap()
);

// the shared secret is the same for both alice and bob
assert_eq!(shared_secret_with_bob, shared_secret_with_alice);
```

# SharedSecret for encryption

Once a shared secret has been established it is possible to use it to seed
a stream Cipher (authenticated or not, ChaCha20 with (or without) Poly1307).

```
# use keynesis::key::ed25519_hd::SecretKey;
# use rand::thread_rng;
use cryptoxide::{chacha20::ChaCha20, symmetriccipher::SynchronousStreamCipher as _};

# let bob_root_key = SecretKey::new(&mut thread_rng());
# let bob_root_pk = bob_root_key.public_key();
# let alice_root_key = SecretKey::new(&mut thread_rng());
#
# let alice_key_exchange_with_bob = alice_root_key.derive(b"encryption:bob");
# let shared_secret_with_bob = alice_key_exchange_with_bob.exchange(
#     &bob_root_pk.derive(b"encryption:alice").unwrap()
# );
#
const NONCE: &[u8] = b"0123456789ab";
let mut encryption_context = ChaCha20::new(shared_secret_with_bob.as_ref(), &NONCE);
# let mut decryption_context = ChaCha20::new(shared_secret_with_bob.as_ref(), &NONCE);
let message: &[u8] = b"Secret Message between alice and bob";
let mut encrypted = vec![0; message.len()];
# let mut decrypted = vec![0; message.len()];

encryption_context.process(message, &mut encrypted);
# decryption_context.process(&encrypted, &mut decrypted);
# assert_eq!(message, decrypted.as_slice());
```

*/

mod shared_secret;
pub mod ed25519;
pub mod ed25519_extended;
pub mod ed25519_hd;

pub use shared_secret::SharedSecret;
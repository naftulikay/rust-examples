use openssl::pkey::{Id, PKey, Private};
use openssl::sign::Signer;
use rand::{thread_rng, RngCore};

pub enum EllipticCurve {
    Ed25519,
    Ed448,
    SECP256R1,
    SECP384R1,
}

pub struct EdDSASigner {
    pub key: PKey<Private>,
}

pub struct Ed25519Signer {
    key: PKey<Private>,
}

impl Ed25519Signer {
    /// Generate a new signer with a randomly generated key.
    ///
    /// All values are legal in EdDSA keys, so it's simply generating 32 bytes from the CSPNG.
    pub fn random() -> Self {
        // generate 32 bytes of random data on the stack
        let key_bytes = {
            let mut k = [0; 32];
            thread_rng().fill_bytes(&mut k);
            k
        };

        // construct a pkey
        let key = PKey::private_key_from_raw_bytes(&key_bytes, Id::ED25519)
            .expect("unable to create ed25519 private key");

        Self { key }
    }

    pub fn sign(&self, data: &[u8]) -> [u8; 64] {
        let mut sig = [0; 64];

        let _signature_length = Signer::new_without_digest(&self.key)
            .expect("unable to create signer")
            .sign_oneshot(&mut sig, data)
            .expect("unable to sign data");

        sig
    }
}

pub struct Ed448Signer {
    key: PKey<Private>,
}

impl Ed448Signer {
    /// Generate a new signer with a randomly generated key.
    ///
    /// All values are legal in EdDSA keys, so it's simply generating 32 bytes from the CSPNG.
    pub fn random() -> Self {
        Self {
            key: PKey::generate_ed448().unwrap(),
        }
    }

    pub fn sign(&self, data: &[u8]) -> [u8; 64] {
        let mut sig = [0; 64];

        let _signature_length = Signer::new_without_digest(&self.key)
            .expect("unable to create signer")
            .sign_oneshot(&mut sig, data)
            .expect("unable to sign data");

        sig
    }
}

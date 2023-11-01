use openssl::ec::{EcGroup, EcKey};
use openssl::pkey::PKey;
use openssl::rsa::Rsa;

pub struct KeyPair {
    pub private: Vec<u8>,
    pub public: Vec<u8>,
}

pub fn keygen(group: &EcGroup) -> KeyPair {
    let private = EcKey::generate(group).unwrap();

    KeyPair {
        private: private.private_key().to_vec(),
        public: private.public_key_to_der().unwrap(),
    }
}

pub fn keygen_ed25519() -> KeyPair {
    let pk = PKey::generate_ed25519().unwrap();

    KeyPair {
        private: pk.raw_private_key().unwrap(),
        public: pk.raw_public_key().unwrap(),
    }
}

pub fn keygen_ed448() -> KeyPair {
    let pk = PKey::generate_ed448().unwrap();

    KeyPair {
        private: pk.raw_private_key().unwrap(),
        public: pk.raw_public_key().unwrap(),
    }
}

pub fn keygen_x25519() -> KeyPair {
    let pk = PKey::generate_x25519().unwrap();

    KeyPair {
        private: pk.raw_private_key().unwrap(),
        public: pk.raw_public_key().unwrap(),
    }
}

pub fn keygen_x448() -> KeyPair {
    let pk = PKey::generate_x448().unwrap();

    KeyPair {
        private: pk.raw_private_key().unwrap(),
        public: pk.raw_public_key().unwrap(),
    }
}

pub fn keygen_rsa(bits: u32) -> KeyPair {
    let rsa = Rsa::generate(bits).unwrap();

    KeyPair {
        private: rsa.private_key_to_der().unwrap(),
        public: rsa.public_key_to_der().unwrap(),
    }
}

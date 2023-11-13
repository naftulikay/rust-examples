use openssl::ec::{EcGroup, EcKey};
use openssl::hash::MessageDigest;
use openssl::nid::Nid;
use openssl::pkey::{PKey, Private};

pub struct KeyGenerator {}

impl KeyGenerator {
    fn ec(nid: Nid) -> PKey<Private> {
        PKey::from_ec_key(EcKey::generate(&EcGroup::from_curve_name(nid).unwrap()).unwrap())
            .unwrap()
    }

    pub fn prime256v1() -> PKey<Private> {
        Self::ec(Nid::X9_62_PRIME256V1)
    }

    pub fn secp256k1() -> PKey<Private> {
        Self::ec(Nid::SECP256K1)
    }

    pub fn secp384r1() -> PKey<Private> {
        Self::ec(Nid::SECP384R1)
    }

    pub fn ed25519() -> PKey<Private> {
        PKey::generate_ed25519().unwrap()
    }

    pub fn ed448() -> PKey<Private> {
        PKey::generate_ed448().unwrap()
    }
}

pub struct SignVerifyKey {
    pub algo: SignatureAlgo,
    pub pkey: PKey<Private>,
    pub digest: MessageDigest,
}

pub enum SignatureAlgo {
    ECDSA,
    EdDSA,
}

impl SignVerifyKey {
    fn ec(nid: Nid, digest: MessageDigest) -> Self {
        Self {
            algo: SignatureAlgo::ECDSA,
            pkey: PKey::from_ec_key(
                EcKey::generate(&EcGroup::from_curve_name(nid).unwrap()).unwrap(),
            )
            .unwrap(),
            digest,
        }
    }

    pub fn prime256v1(digest: MessageDigest) -> Self {
        Self::ec(Nid::X9_62_PRIME256V1, digest)
    }

    pub fn secp256k1(digest: MessageDigest) -> Self {
        Self::ec(Nid::SECP256K1, digest)
    }

    pub fn secp384r1(digest: MessageDigest) -> Self {
        Self::ec(Nid::SECP384R1, digest)
    }

    pub fn ed25519() -> Self {
        Self {
            algo: SignatureAlgo::EdDSA,
            pkey: KeyGenerator::ed25519(),
            digest: MessageDigest::null(),
        }
    }

    pub fn ed448() -> Self {
        Self {
            algo: SignatureAlgo::EdDSA,
            pkey: KeyGenerator::ed448(),
            digest: MessageDigest::null(),
        }
    }
}

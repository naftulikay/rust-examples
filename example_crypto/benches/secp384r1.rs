use criterion::{criterion_group, criterion_main, Criterion};
use openssl::ec::{EcGroup, EcKey};
use openssl::hash::MessageDigest;
use openssl::nid::Nid;
use openssl::pkey::{PKey, Private};
use openssl::sign::Signer;

struct Key {
    pkey: PKey<Private>,
    digest: MessageDigest,
}

impl Key {
    fn secp384r1(digest: MessageDigest) -> Self {
        Self {
            pkey: PKey::from_ec_key(
                EcKey::generate(EcGroup::from_curve_name(Nid::SECP384R1).unwrap().as_ref())
                    .unwrap(),
            )
            .unwrap(),
            digest,
        }
    }

    pub fn secp384r1_sha256() -> Self {
        Self::secp384r1(MessageDigest::sha256())
    }

    pub fn secp384r1_sha384() -> Self {
        Self::secp384r1(MessageDigest::sha384())
    }
}

fn bench_secp384r1(c: &mut Criterion) {
    c.bench_function("secp384r1::generate", |b| b.ite)
}

criterion_group! {
    name = secp384r1;
    config = Criterion::default();
    targets = bench_secp384r1
}

criterion_main!(secp384r1);

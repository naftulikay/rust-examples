use criterion::{criterion_group, criterion_main, Criterion};
use example_crypto::openssl::keygen::{keygen_ec, keygen_ed25519, keygen_ed448, keygen_rsa, keygen_x25519, keygen_x448};
use openssl::ec::EcGroup;
use openssl::nid::Nid;

pub fn bench_keygen(c: &mut Criterion) {
    let secp256k1 = EcGroup::from_curve_name(Nid::SECP256K1).unwrap();

    // // eddsa
    c.bench_function("openssl::keygen::ed25519", |b| b.iter(|| keygen_ed25519()));
    c.bench_function("openssl::keygen::ed448", |b| b.iter(|| keygen_ed448()));
    c.bench_function("openssl::keygen::x25519", |b| b.iter(|| keygen_x25519()));
    c.bench_function("openssl::keygen::x448", |b| b.iter(|| keygen_x448()));

    // ecdsa
    c.bench_function("openssl::keygen::secp256", |b| {
        b.iter(|| keygen_ec(&secp256k1))
    });

    // rsa
    c.bench_function("openssl::keygen::rsa2048", |b| b.iter(|| keygen_rsa(2048)));
    c.bench_function("openssl::keygen::rsa3072", |b| b.iter(|| keygen_rsa(3072)));
    c.bench_function("openssl::keygen::rsa4096", |b| b.iter(|| keygen_rsa(4096)));
}

criterion_group!{
    name = keygen;
    config = Criterion::default();
    targets = bench_keygen
}

criterion_main!(keygen);
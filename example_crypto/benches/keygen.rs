use criterion::{black_box, criterion_group, criterion_main, Criterion};
use example_crypto::openssl::benches::KeyGenerator;

fn bench_keygen(c: &mut Criterion) {
    let mut group = c.benchmark_group("openssl::keygen");

    group.bench_function("prime256v1", |b| {
        b.iter(black_box(KeyGenerator::prime256v1))
    });

    group.bench_function("secp256k1", |b| b.iter(black_box(KeyGenerator::secp256k1)));
    group.bench_function("secp384r", |b| b.iter(black_box(KeyGenerator::secp384r1)));
    group.bench_function("ed25519", |b| b.iter(black_box(KeyGenerator::ed25519)));
    group.bench_function("ed448", |b| b.iter(black_box(KeyGenerator::ed448)));
}

criterion_group! {
    name = keygen;
    config = Criterion::default();
    targets = bench_keygen
}

criterion_main!(keygen);

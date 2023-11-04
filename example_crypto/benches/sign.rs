use criterion::{criterion_group, criterion_main, Criterion};
use example_crypto::openssl::sign::Ed25519Signer;
use rand::{thread_rng, RngCore};
use std::sync::atomic::{AtomicUsize, Ordering};

fn bench_sign(c: &mut Criterion) {
    // eddsa
    c.bench_function("openssl::sign::ed25519", |b| {
        const KEY_COUNT: usize = 256;
        const DATA_COUNT: usize = 512;

        // build 256 signers
        let keys: Vec<Ed25519Signer> = (0..KEY_COUNT)
            .into_iter()
            .map(|_| Ed25519Signer::random())
            .collect();

        let data: Vec<[u8; 32]> = (0..DATA_COUNT)
            .into_iter()
            .map(|_| {
                let mut d = [0; 32];
                thread_rng().fill_bytes(&mut d);
                d
            })
            .collect();

        let (signer_index, data_index) = (AtomicUsize::new(0), AtomicUsize::new(0));

        b.iter(|| {
            let (current_signer, current_data) = (
                signer_index.fetch_add(1, Ordering::AcqRel) % KEY_COUNT,
                data_index.fetch_add(1, Ordering::AcqRel) % DATA_COUNT,
            );

            // get em fast
            let _sig = unsafe {
                keys.get_unchecked(current_signer)
                    .sign(data.get_unchecked(current_data))
            };
        })
    });
}

criterion_group! {
    name = sign;
    config = Criterion::default();
    targets = bench_sign
}

criterion_main!(sign);

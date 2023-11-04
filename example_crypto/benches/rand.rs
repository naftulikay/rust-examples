use criterion::{criterion_group, criterion_main, Criterion};
use rand::{thread_rng, RngCore};

fn bench_rand_thread_rng(c: &mut Criterion) {
    // stack tests
    c.bench_function("rand::thread_rng::stack_32", |b| {
        b.iter(|| {
            let mut s = [0; 32];
            thread_rng().fill_bytes(&mut s);
        })
    });
    c.bench_function("rand::thread_rng::stack_64", |b| {
        b.iter(|| {
            let mut s = [0; 64];
            thread_rng().fill_bytes(&mut s);
        })
    });
    // vec tests
    c.bench_function("rand::thread_rng::alloc_32", |b| {
        b.iter(|| {
            let mut v = vec![0; 32];
            thread_rng().fill_bytes(&mut v.as_mut_slice());
        })
    });
    c.bench_function("rand::thread_rng::alloc_64", |b| {
        b.iter(|| {
            let mut v = vec![0; 64];
            thread_rng().fill_bytes(&mut v.as_mut_slice());
        })
    });
}

criterion_group! {
    name = rand;
    config = Criterion::default();
    targets = bench_rand_thread_rng
}

criterion_main!(rand);

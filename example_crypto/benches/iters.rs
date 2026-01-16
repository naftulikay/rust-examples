use criterion::{criterion_group, criterion_main, Criterion};

pub struct EndlessIter<T> {
    pub values: Vec<T>,
}

impl<T> Iterator for EndlessIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

fn bench(c: &mut Criterion) {}

criterion_group! {
    name = iters;
    config = Criterion::default();
    targets = bench
}

criterion_main!(iters);

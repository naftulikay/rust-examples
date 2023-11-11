use criterion::measurement::{Measurement, WallTime};
use criterion::{criterion_group, criterion_main, BenchmarkGroup, Criterion};
use example_crypto::rng::{
    DevRandomBufRng, DevRandomDirectRng, DevUrandomBufRng, DevUrandomDirectRng, OpenSslRng,
    RandGenerator, RandGeneratorBenchmark, RandOsRng, RandThreadRng,
};
use std::sync::RwLock;

pub struct RngBencher<'a, 'b, R: RandGenerator> {
    rng: R,
    prefix: &'static str,
    groups: &'b mut BenchmarkGroups<'a, WallTime>,
}

impl<R> RngBencher<'_, '_, R>
where
    R: RandGenerator + RandGeneratorBenchmark,
{
    pub fn new(rng: R, groups: &mut BenchmarkGroups<WallTime>) -> Self {
        Self {
            rng,
            prefix: R::PREFIX,
            groups,
        }
    }

    /// Conduct the benchmark using fixed-size arrays on the stack.
    pub fn bench_arr(&mut self) -> &mut Self {
        self.bench_arr_specific::<32, _>(&self.groups.a32);
        self.bench_arr_specific::<64, _>(&self.groups.a64);
        self.bench_arr_specific::<128, _>(&self.groups.a128);
        self.bench_arr_specific::<256, _>(&self.groups.a256);
        self.bench_arr_specific::<512, _>(&self.groups.a512);
        self.bench_arr_specific::<1024, _>(&self.groups.a1024);
        self.bench_arr_specific::<2048, _>(&self.groups.a2048);
        self.bench_arr_specific::<4096, _>(&self.groups.a4096);
        self.bench_arr_specific::<8192, _>(&self.groups.a8192);
        self
    }

    fn bench_arr_specific<const S: usize, M: Measurement>(
        &mut self,
        c: &RwLock<BenchmarkGroup<M>>,
    ) {
        let mut c = c.write().unwrap();
        c.bench_function(format!("{}::array::{}", self.prefix, S).as_str(), |b| {
            b.iter(|| self.rng.generate_array::<S>());
        });
    }

    /// Conduct the benchmarks using fixed-size vectors on the heap.
    pub fn bench_vec(&mut self) -> &mut Self {
        self.bench_vec_specific::<32, _>(&self.groups.v32);
        self.bench_vec_specific::<64, _>(&self.groups.v64);

        self.bench_vec_specific::<128, _>(&self.groups.v128);

        self.bench_vec_specific::<256, _>(&self.groups.v256);

        self.bench_vec_specific::<512, _>(&self.groups.v512);

        self.bench_vec_specific::<1024, _>(&self.groups.v1024);

        self.bench_vec_specific::<2048, _>(&self.groups.v2048);

        self.bench_vec_specific::<4096, _>(&self.groups.v4096);

        self.bench_vec_specific::<8192, _>(&self.groups.v8192);

        self
    }

    fn bench_vec_specific<const S: usize, M: Measurement>(
        &mut self,
        c: &RwLock<BenchmarkGroup<M>>,
    ) {
        let mut c = c.write().unwrap();
        c.bench_function(format!("{}::vec::{}", self.prefix, S).as_str(), |b| {
            b.iter(|| self.rng.generate_vec::<S>());
        });
    }
}

pub struct BenchmarkGroups<'a, M: Measurement> {
    a32: RwLock<BenchmarkGroup<'a, M>>,
    a64: RwLock<BenchmarkGroup<'a, M>>,
    a128: RwLock<BenchmarkGroup<'a, M>>,
    a256: RwLock<BenchmarkGroup<'a, M>>,
    a512: RwLock<BenchmarkGroup<'a, M>>,
    a1024: RwLock<BenchmarkGroup<'a, M>>,
    a2048: RwLock<BenchmarkGroup<'a, M>>,
    a4096: RwLock<BenchmarkGroup<'a, M>>,
    a8192: RwLock<BenchmarkGroup<'a, M>>,
    v32: RwLock<BenchmarkGroup<'a, M>>,
    v64: RwLock<BenchmarkGroup<'a, M>>,
    v128: RwLock<BenchmarkGroup<'a, M>>,
    v256: RwLock<BenchmarkGroup<'a, M>>,
    v512: RwLock<BenchmarkGroup<'a, M>>,
    v1024: RwLock<BenchmarkGroup<'a, M>>,
    v2048: RwLock<BenchmarkGroup<'a, M>>,
    v4096: RwLock<BenchmarkGroup<'a, M>>,
    v8192: RwLock<BenchmarkGroup<'a, M>>,
}

impl<'a, 'r> BenchmarkGroups<'a, WallTime> {
    pub fn new(c: &'a mut Criterion) -> Self {
        let a32 = { c.benchmark_group("rng::array::32").into() };
        let a64 = { c.benchmark_group("rng::array::64") };
        Self {
            a32,
            a64: { c.benchmark_group("rng::array::64").into() },
            a128: c.benchmark_group("rng::array::128").into(),
            a256: c.benchmark_group("rng::array::256").into(),
            a512: c.benchmark_group("rng::array::512").into(),
            a1024: c.benchmark_group("rng::array::1024").into(),
            a2048: c.benchmark_group("rng::array::2048").into(),
            a4096: c.benchmark_group("rng::array::4096").into(),
            a8192: c.benchmark_group("rng::array::8192").into(),
            v32: c.benchmark_group("rng::vec::32").into(),
            v64: c.benchmark_group("rng::vec::64").into(),
            v128: c.benchmark_group("rng::vec::128").into(),
            v256: c.benchmark_group("rng::vec::256").into(),
            v512: c.benchmark_group("rng::vec::512").into(),
            v1024: c.benchmark_group("rng::vec::1024").into(),
            v2048: c.benchmark_group("rng::vec::2048").into(),
            v4096: c.benchmark_group("rng::vec::4096").into(),
            v8192: c.benchmark_group("rng::vec::8192").into(),
        }
    }
}

fn bench_fast_rands(c: &mut Criterion) {
    let mut groups = BenchmarkGroups::new(c);

    // buffered /dev/urandom
    RngBencher::new(DevUrandomBufRng::new(), &mut groups)
        .bench_arr()
        .bench_vec();

    // direct /dev/urandom
    RngBencher::new(DevUrandomDirectRng::new(), &mut groups)
        .bench_arr()
        .bench_vec();

    // openssl
    RngBencher::new(OpenSslRng::new(), &mut groups)
        .bench_arr()
        .bench_vec();

    // rand: os rng
    RngBencher::new(RandOsRng::new(), &mut groups)
        .bench_arr()
        .bench_vec();

    // rand: thread rng
    RngBencher::new(RandThreadRng::new(), &mut groups)
        .bench_arr()
        .bench_vec();
}

fn bench_slow_rands(c: &mut Criterion) {
    let mut groups = BenchmarkGroups::new(c);

    // buffered /dev/random
    RngBencher::new(DevRandomBufRng::new(), &mut groups)
        .bench_arr()
        .bench_vec();

    // direct /dev/random
    RngBencher::new(DevRandomDirectRng::new(), &mut groups)
        .bench_arr()
        .bench_vec();
}

criterion_group! {
    name = rand;
    config = Criterion::default();
    targets = bench_fast_rands
}

criterion_main!(rand);

use criterion::{criterion_group, criterion_main, Criterion};
use example_crypto::rng::{
    DevRandomBufRng, DevRandomDirectRng, DevUrandomBufRng, DevUrandomDirectRng, OpenSslRng,
    RandCrateOs, RandCrateThread, RandGenerator, RandOsRng, RandThreadRng, SysRandomBuffered,
    SysRandomDirect, SysUrandomBuffered, SysUrandomDirect,
};

const BUFFER_SIZES: [usize; 9] = [32, 64, 128, 256, 512, 1024, 2048, 4096, 8192];

pub struct RngBencher<R: RandGenerator> {
    rng: R,
    prefix: &'static str,
}

impl<R> RngBencher<R>
where
    R: RandGenerator,
{
    pub fn new(rng: R, prefix: &'static str) -> Self {
        Self { rng, prefix }
    }

    /// Conduct the benchmark using fixed-size arrays on the stack.
    pub fn bench_arr(&mut self, c: &mut Criterion) -> &mut Self {
        self.bench_arr_specific::<32>(c);
        self.bench_arr_specific::<64>(c);
        self.bench_arr_specific::<128>(c);
        self.bench_arr_specific::<256>(c);
        self.bench_arr_specific::<512>(c);
        self.bench_arr_specific::<1024>(c);
        self.bench_arr_specific::<2048>(c);
        self.bench_arr_specific::<4096>(c);
        self.bench_arr_specific::<8192>(c);
        self
    }

    fn bench_arr_specific<const S: usize>(&mut self, c: &mut Criterion) {
        c.bench_function(format!("{}::array::{}", self.prefix, S).as_str(), |b| {
            b.iter(|| self.rng.generate_array::<S>());
        });
    }

    /// Conduct the benchmarks using fixed-size vectors on the heap.
    pub fn bench_vec(&mut self, c: &mut Criterion) -> &mut Self {
        self.bench_vec_specific::<32>(c);
        self.bench_vec_specific::<64>(c);
        self.bench_vec_specific::<128>(c);
        self.bench_vec_specific::<256>(c);
        self.bench_vec_specific::<512>(c);
        self.bench_vec_specific::<1024>(c);
        self.bench_vec_specific::<2048>(c);
        self.bench_vec_specific::<4096>(c);
        self.bench_vec_specific::<8192>(c);
        self
    }

    fn bench_vec_specific<const S: usize>(&mut self, c: &mut Criterion) {
        c.bench_function(format!("{}::vec::{}", self.prefix, S).as_str(), |b| {
            b.iter(|| self.rng.generate_vec::<S>());
        });
    }
}

fn bench_fast_rands(c: &mut Criterion) {
    // buffered /dev/urandom
    RngBencher::new(DevUrandomBufRng::new(), DevUrandomBufRng::PREFIX)
        .bench_arr(c)
        .bench_vec(c);

    // direct /dev/urandom
    RngBencher::new(DevUrandomDirectRng::new(), DevUrandomDirectRng::PREFIX)
        .bench_arr(c)
        .bench_vec(c);

    // openssl
    RngBencher::new(OpenSslRng::new(), OpenSslRng::PREFIX)
        .bench_arr(c)
        .bench_vec(c);

    // rand: os rng
    RngBencher::new(RandOsRng::new(), RandOsRng::PREFIX)
        .bench_arr(c)
        .bench_vec(c);

    // rand: thread rng
    RngBencher::new(RandThreadRng::new(), RandThreadRng::PREFIX)
        .bench_arr(c)
        .bench_vec(c);
}

fn bench_slow_rands(c: &mut Criterion) {
    // buffered /dev/random
    RngBencher::new(DevRandomBufRng::new(), DevRandomBufRng::PREFIX)
        .bench_arr(c)
        .bench_vec(c);

    // direct /dev/random
    RngBencher::new(DevRandomDirectRng::new(), DevRandomDirectRng::PREFIX)
        .bench_arr(c)
        .bench_vec(c);
}

criterion_group! {
    name = rand;
    config = Criterion::default();
    targets = bench_fast_rands
}

criterion_main!(rand);

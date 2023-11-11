use super::{
    DevRandomBufRng, DevRandomDirectRng, DevUrandomBufRng, DevUrandomDirectRng, OpenSslRng,
    RandGenerator, RandGeneratorBenchmark, RandOsRng, RandThreadRng,
};

#[test]
fn test_urandom_buf_rng() {
    let mut rng = DevUrandomBufRng::new();

    assert_ne!([0; 32], rng.generate_array::<32>());
    assert_ne!(vec![0; 32], rng.generate_vec::<32>());
}

#[test]
fn test_urandom_direct_rng() {
    let mut rng = DevUrandomDirectRng::new();

    assert_ne!([0; 32], rng.generate_array::<32>());
    assert_ne!(vec![0; 32], rng.generate_vec::<32>());
}

#[test]
fn test_openssl_rng() {
    let mut rng = OpenSslRng::new();

    assert_ne!([0; 32], rng.generate_array::<32>());
    assert_ne!(vec![0; 32], rng.generate_vec::<32>());
}

#[test]
fn test_rand_os_rng() {
    let mut rng = RandOsRng::new();

    assert_ne!([0; 32], rng.generate_array::<32>());
    assert_ne!(vec![0; 32], rng.generate_vec::<32>());
}

#[test]
fn test_rand_thread_rng() {
    let mut rng = RandThreadRng::new();

    assert_ne!([0; 32], rng.generate_array::<32>());
    assert_ne!(vec![0; 32], rng.generate_vec::<32>());
}

#[test]
fn test_assert_rand_generator_impl() {
    fn _dev_random_direct() -> impl RandGenerator + RandGeneratorBenchmark {
        DevRandomDirectRng::new()
    }
    fn _dev_random_buf() -> impl RandGenerator + RandGeneratorBenchmark {
        DevRandomBufRng::new()
    }
    fn _dev_urandom_direct() -> impl RandGenerator + RandGeneratorBenchmark {
        DevUrandomDirectRng::new()
    }
    fn _dev_urandom_buf() -> impl RandGenerator + RandGeneratorBenchmark {
        DevUrandomBufRng::new()
    }
    fn _openssl() -> impl RandGenerator + RandGeneratorBenchmark {
        OpenSslRng::new()
    }
    fn _rand_os() -> impl RandGenerator + RandGeneratorBenchmark {
        RandOsRng::new()
    }
    fn _rand_thread() -> impl RandGenerator + RandGeneratorBenchmark {
        RandThreadRng::new()
    }
}

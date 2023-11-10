use super::{
    DevUrandomBufRng, DevUrandomDirectRng, OpenSslRng, RandGenerator, RandOsRng, RandThreadRng,
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

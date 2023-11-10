use crate::rng::RandGenerator;
use rand::rngs::{OsRng, ThreadRng};
use rand::{thread_rng, CryptoRng, Rng, RngCore};

/// CSPRNG built on the [rand] crate's [OsRng] implementation.
pub struct SecureOsGenerator(OsRng);

impl SecureOsGenerator {
    pub fn new() -> Self {
        Self(OsRng::default())
    }
}

impl RandGenerator for SecureOsGenerator {
    fn fill(&mut self, bytes: &mut [u8]) {
        self.0.fill(bytes);
    }
}

/// CSPRNG built on the [rand] crate's [ThreadRng] implementation.
pub struct SecureThreadGenerator(ThreadRng);

impl SecureThreadGenerator {
    pub fn new() -> Self {
        Self(thread_rng())
    }
}

impl RandGenerator for SecureThreadGenerator {
    fn fill(&mut self, bytes: &mut [u8]) {
        self.0.fill(bytes);
    }
}

/// Constant assertion that [OsRng] is a [CryptoRng] implementor.
#[allow(unused)]
fn assert_secure_os_rng() -> impl Rng + RngCore + CryptoRng {
    OsRng::default()
}

/// Constant assertion that [ThreadRng] is a [CryptoRng] implementor.
#[allow(unused)]
fn assert_secure_thread_rng() -> impl Rng + RngCore + CryptoRng {
    thread_rng()
}

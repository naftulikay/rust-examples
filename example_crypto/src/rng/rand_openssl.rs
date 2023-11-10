use crate::rng::RandGenerator;

/// CSPRNG using [openssl::rand::rand_bytes].
pub struct OpenSslRng {}

impl OpenSslRng {
    /// Constructs a new instance.
    pub fn new() -> Self {
        Self {}
    }
}

impl RandGenerator for OpenSslRng {
    fn fill(&mut self, bytes: &mut [u8]) {
        openssl::rand::rand_bytes(bytes).expect("unable to read random from openssl");
    }
}

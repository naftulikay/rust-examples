#[cfg(test)]
mod tests;

use rand::rngs::{OsRng, ThreadRng};
use rand::Rng;
use std::fs::File;
use std::io::{BufReader, Read};

/// Generates random data into either a mutable slice, a stack-allocated array, or a heap-allocated
/// [Vec] of bytes.
pub trait RandGenerator {
    /// Fill a mutable buffer of bytes with random data.
    fn fill(&mut self, bytes: &mut [u8]);
    /// Create, fill, and return a fixed-size array on the stack.
    fn generate_array<const S: usize>(&mut self) -> [u8; S] {
        let mut arr = [0; S];
        self.fill(&mut arr);
        arr
    }
    /// Create, fill, and return a fixed-size vector on the heap.
    fn generate_vec<const S: usize>(&mut self) -> Vec<u8> {
        let mut buf = vec![0; S];
        self.fill(buf.as_mut_slice());
        buf
    }
}

/// Provides a prefix for benchmark suites.
pub trait RandGeneratorBenchmark {
    /// The prefix name for benchmarking.
    const PREFIX: &'static str;
}

struct ReadRng<R: Read> {
    reader: R,
}

impl<R> ReadRng<R>
where
    R: Read,
{
    pub fn new(reader: R) -> Self {
        Self { reader }
    }
}

impl<R> RandGenerator for ReadRng<R>
where
    R: Read,
{
    fn fill(&mut self, bytes: &mut [u8]) {
        let mut written = 0;

        while written < bytes.len() {
            match self.reader.read(&mut bytes[written..]) {
                Ok(w) => written += w,
                Err(e) => panic!("{:?}", e),
            }
        }

        if written != bytes.len() {
            panic!("did not fill the entire buffer");
        }
    }
}

/// RNG implementation over `/dev/random` directly.
pub struct DevRandomDirectRng(ReadRng<File>);

impl DevRandomDirectRng {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Default for DevRandomDirectRng {
    fn default() -> Self {
        Self(ReadRng {
            reader: File::options()
                .read(true)
                .open("/dev/random")
                .expect("unable to open /dev/random direct reader"),
        })
    }
}

impl RandGenerator for DevRandomDirectRng {
    fn fill(&mut self, bytes: &mut [u8]) {
        self.0.fill(bytes);
    }
}

impl RandGeneratorBenchmark for DevRandomDirectRng {
    const PREFIX: &'static str = "rng::sys::random::direct";
}

/// RNG implementation over `/dev/urandom` directly.
pub struct DevUrandomDirectRng(ReadRng<File>);

impl DevUrandomDirectRng {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Default for DevUrandomDirectRng {
    fn default() -> Self {
        Self(ReadRng {
            reader: File::options()
                .read(true)
                .open("/dev/urandom")
                .expect("unable to open /dev/urandom direct reader"),
        })
    }
}

impl RandGenerator for DevUrandomDirectRng {
    fn fill(&mut self, bytes: &mut [u8]) {
        self.0.fill(bytes);
    }
}

impl RandGeneratorBenchmark for DevUrandomDirectRng {
    const PREFIX: &'static str = "rng::sys::urandom::direct";
}

/// RNG implementation over `/dev/random` with a buffer.
pub struct DevRandomBufRng(ReadRng<BufReader<File>>);

impl DevRandomBufRng {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Default for DevRandomBufRng {
    fn default() -> Self {
        Self(ReadRng {
            reader: BufReader::new(
                File::options()
                    .read(true)
                    .open("/dev/random")
                    .expect("unable to open /dev/random buffered reader"),
            ),
        })
    }
}

impl RandGenerator for DevRandomBufRng {
    fn fill(&mut self, bytes: &mut [u8]) {
        self.0.fill(bytes);
    }
}

impl RandGeneratorBenchmark for DevRandomBufRng {
    const PREFIX: &'static str = "rng::sys::random::buffered";
}

/// RNG implementation over `/dev/urandom` with a buffer.
pub struct DevUrandomBufRng(ReadRng<BufReader<File>>);

impl DevUrandomBufRng {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Default for DevUrandomBufRng {
    fn default() -> Self {
        Self(ReadRng {
            reader: BufReader::new(
                File::options()
                    .read(true)
                    .open("/dev/urandom")
                    .expect("unable to open /dev/urandom buffered reader"),
            ),
        })
    }
}

impl RandGenerator for DevUrandomBufRng {
    fn fill(&mut self, bytes: &mut [u8]) {
        self.0.fill(bytes);
    }
}

impl RandGeneratorBenchmark for DevUrandomBufRng {
    const PREFIX: &'static str = "rng::sys::urandom::buffered";
}

/// RNG implementation using [openssl::rand::rand_bytes].
#[derive(Default)]
pub struct OpenSslRng {}

impl OpenSslRng {
    pub fn new() -> Self {
        Default::default()
    }
}

impl RandGenerator for OpenSslRng {
    fn fill(&mut self, bytes: &mut [u8]) {
        openssl::rand::rand_bytes(bytes).expect("unable to read from OpenSSL rng");
    }
}

impl RandGeneratorBenchmark for OpenSslRng {
    const PREFIX: &'static str = "rng::openssl";
}

/// RNG implementation over [OsRng].
#[derive(Default)]
pub struct RandOsRng {
    source: OsRng,
}

impl RandOsRng {
    pub fn new() -> Self {
        Default::default()
    }
}

impl RandGenerator for RandOsRng {
    fn fill(&mut self, bytes: &mut [u8]) {
        self.source.fill(bytes);
    }
}

impl RandGeneratorBenchmark for RandOsRng {
    const PREFIX: &'static str = "rng::rand_crate::os";
}

/// RNG implementation over [rand::rngs::ThreadRng].
#[derive(Default)]
pub struct RandThreadRng {
    source: ThreadRng,
}

impl RandThreadRng {
    pub fn new() -> Self {
        Default::default()
    }
}

impl RandGenerator for RandThreadRng {
    fn fill(&mut self, bytes: &mut [u8]) {
        self.source.fill(bytes);
    }
}

impl RandGeneratorBenchmark for RandThreadRng {
    const PREFIX: &'static str = "rng::rand_crate::thread";
}

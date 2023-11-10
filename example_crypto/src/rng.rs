mod rand_crate;
mod rand_openssl;
mod rand_sys;

#[cfg(test)]
mod tests;

use rand::rngs::{OsRng, ThreadRng};
use rand::Rng;
use std::fs::File;
use std::io::{BufReader, Read};

pub use rand_crate::SecureOsGenerator as RandCrateOs;
pub use rand_crate::SecureThreadGenerator as RandCrateThread;
pub use rand_sys::SysRandomBufferedGenerator as SysRandomBuffered;
pub use rand_sys::SysRandomDirectGenerator as SysRandomDirect;
pub use rand_sys::SysUrandomBufferedGenerator as SysUrandomBuffered;
pub use rand_sys::SysUrandomDirectGenerator as SysUrandomDirect;

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

pub struct FileRng {
    file: File,
}

impl RandGenerator for FileRng {
    fn fill(&mut self, bytes: &mut [u8]) {
        assert_eq!(
            bytes.len(),
            self.file.read(bytes).expect("unable to read from rng file")
        );
    }
}

pub struct DevRandomDirectRng(FileRng);

impl DevRandomDirectRng {
    pub const PREFIX: &'static str = "rng::sys::random::direct";

    pub fn new() -> Self {
        Self(FileRng {
            file: File::options()
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

pub struct DevUrandomDirectRng(FileRng);

impl DevUrandomDirectRng {
    pub const PREFIX: &'static str = "rng::sys::urandom::direct";

    pub fn new() -> Self {
        Self(FileRng {
            file: File::options()
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

pub struct BufFileRng {
    file: BufReader<File>,
}

impl RandGenerator for BufFileRng {
    fn fill(&mut self, bytes: &mut [u8]) {
        assert_eq!(
            bytes.len(),
            self.file
                .read(bytes)
                .expect("unable to read from buffered rng file")
        );
    }
}

pub struct DevRandomBufRng(BufFileRng);

impl DevRandomBufRng {
    pub const PREFIX: &'static str = "rng::sys::random::buffered";

    pub fn new() -> Self {
        Self(BufFileRng {
            file: BufReader::new(
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

pub struct DevUrandomBufRng(BufFileRng);

impl DevUrandomBufRng {
    pub const PREFIX: &'static str = "rng::sys::urandom::buffered";

    pub fn new() -> Self {
        Self(BufFileRng {
            file: BufReader::new(
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

pub struct OpenSslRng {}

impl OpenSslRng {
    pub const PREFIX: &'static str = "rng::openssl";

    pub fn new() -> Self {
        Self {}
    }
}

impl RandGenerator for OpenSslRng {
    fn fill(&mut self, bytes: &mut [u8]) {
        openssl::rand::rand_bytes(bytes).expect("unable to read from OpenSSL rng");
    }
}

pub struct RandOsRng {
    source: OsRng,
}

impl RandOsRng {
    pub const PREFIX: &'static str = "rng::rand_crate::os";

    pub fn new() -> Self {
        Self {
            source: OsRng::default(),
        }
    }
}

impl RandGenerator for RandOsRng {
    fn fill(&mut self, bytes: &mut [u8]) {
        self.source.fill(bytes);
    }
}

pub struct RandThreadRng {
    source: ThreadRng,
}

impl RandThreadRng {
    pub const PREFIX: &'static str = "rng::rand_crate::thread";

    pub fn new() -> Self {
        Self {
            source: ThreadRng::default(),
        }
    }
}

impl RandGenerator for RandThreadRng {
    fn fill(&mut self, bytes: &mut [u8]) {
        self.source.fill(bytes);
    }
}

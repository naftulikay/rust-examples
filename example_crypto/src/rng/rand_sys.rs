use crate::rng::RandGenerator;
use anyhow::Error;
use std::fs::File;
use std::io::{BufReader, Read};

/// CSPRNG using `/dev/random` without a buffer.
pub struct SysRandomDirectGenerator(File);

impl SysRandomDirectGenerator {
    /// Constructs a new instance.
    ///
    /// Returns an error if it cannot open `/dev/random`.
    pub fn new() -> Result<Self, Error> {
        Ok(Self(File::options().read(true).open("/dev/random")?))
    }
}

impl RandGenerator for SysRandomDirectGenerator {
    fn fill(&mut self, bytes: &mut [u8]) {
        // try to read the exact amount or die trying
        assert_eq!(
            self.0.read(bytes).expect("unable to read random data"),
            bytes.len()
        );
    }
}

/// CSPRNG using `/dev/random` with a buffer.
pub struct SysRandomBufferedGenerator(BufReader<File>);

impl SysRandomBufferedGenerator {
    /// Constructs a new instance.
    ///
    /// Returns an error if it cannot open `/dev/random`.
    pub fn new() -> Result<Self, Error> {
        Ok(Self(BufReader::new(
            File::options().read(true).open("/dev/random")?,
        )))
    }
}

impl RandGenerator for SysRandomBufferedGenerator {
    fn fill(&mut self, bytes: &mut [u8]) {
        // try to read the exact amount or die trying
        assert_eq!(
            self.0.read(bytes).expect("unable to read random data"),
            bytes.len()
        );
    }
}

/// CSPRNG using `/dev/urandom` without a buffer.
pub struct SysUrandomDirectGenerator(File);

impl SysUrandomDirectGenerator {
    /// Constructs a new instance.
    ///
    /// Returns an error if it cannot open '/dev/urandom`.
    pub fn new() -> Result<Self, Error> {
        Ok(Self(File::options().read(true).open("/dev/urandom")?))
    }
}

impl RandGenerator for SysUrandomDirectGenerator {
    fn fill(&mut self, bytes: &mut [u8]) {
        // try to read the exact amount or die trying
        assert_eq!(
            self.0.read(bytes).expect("unable to read random data"),
            bytes.len()
        );
    }
}

/// CSPRNG using `/dev/urandom` with a buffer.
pub struct SysUrandomBufferedGenerator(BufReader<File>);

impl SysUrandomBufferedGenerator {
    /// Constructs a new instance.
    ///
    /// Returns an error if it cannot open `/dev/urandom`.
    pub fn new() -> Result<Self, Error> {
        Ok(Self(BufReader::new(
            File::options().read(true).open("/dev/urandom")?,
        )))
    }
}

impl RandGenerator for SysUrandomBufferedGenerator {
    fn fill(&mut self, bytes: &mut [u8]) {
        // try to read the exact amount or die trying
        assert_eq!(
            self.0.read(bytes).expect("unable to read random data"),
            bytes.len()
        );
    }
}

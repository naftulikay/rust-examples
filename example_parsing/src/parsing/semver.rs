#[cfg(test)]
mod tests;

use std::fmt::{Display, Formatter};

pub struct SemanticVersion {
    pub major: u64,
    pub minor: u64,
    pub bugfix: Option<u64>,
}

impl SemanticVersion {
    pub fn new(major: u64, minor: u64, bugfix: u64) -> Self {
        Self {
            major,
            minor,
            bugfix: Some(bugfix),
        }
    }

    pub fn abridged(major: u64, minor: u64) -> Self {
        Self {
            major,
            minor,
            bugfix: None,
        }
    }
}

impl Display for SemanticVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(bugfix) = self.bugfix.as_ref() {
            write!(f, "{}.{}.{}", self.major, self.minor, *bugfix)
        } else {
            write!(f, "{}.{}", self.major, self.minor)
        }
    }
}

pub fn parse_major_minor(_input: &str) {}

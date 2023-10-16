//! Simple example utilizing [nom] to parse semantic versions as strings into fully-structured
//! [SemanticVersion] objects.
//!
//! ## Displaying and Parsing
//!
//! Valid semantic version strings can look like these:
//!
//!  - `0.2`
//!  - `v1.2`
//!  - `1.23.4`
//!  - `v0.5.6`
//!
//! The `v` prefix is optional and is discarded during parsing. [Display] is implemented, and will
//! output versions without a `v` prefix and works exactly how one would expect: if there is a
//! bugfix revision, it will be included, if there is not, it won't be.
//!
//! If a `v` prefix is preferred, [SemanticVersion::v] will produce a string accordingly.
//!
//! [SemanticVersion] implements [FromStr], and provides a `parse` function which internally calls
//! the [FromStr] implementation.

#[cfg(test)]
mod tests;

use nom::character::{complete::char, complete::digit1};
use nom::combinator::{map, opt};
use nom::IResult;

use anyhow::{Context, Error, Result};
use nom::sequence::preceded;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// Representation of a semantic version with an optional bugfix revision.
#[derive(Debug, Eq, PartialEq)]
pub struct SemanticVersion {
    pub major: u64,
    pub minor: u64,
    pub bugfix: Option<u64>,
}

impl SemanticVersion {
    /// Create a new, full semantic version.
    pub fn new(major: u64, minor: u64, bugfix: u64) -> Self {
        Self {
            major,
            minor,
            bugfix: Some(bugfix),
        }
    }

    /// Create an abridged (major/minor only) semantic version.
    pub fn abridged(major: u64, minor: u64) -> Self {
        Self {
            major,
            minor,
            bugfix: None,
        }
    }

    /// Format this [SemanticVersion] to a string with a `v` prefix.
    pub fn v(&self) -> String {
        format!("v{}", self.to_string())
    }

    /// Parse a [SemanticVersion] from a string.
    pub fn parse<S: AsRef<str>>(s: S) -> Result<Self> {
        Self::from_str(s.as_ref())
    }

    /// Parse the input using [nom], returning a [IResult].
    fn nom_parse(input: &str) -> IResult<&str, Self> {
        // parse an optional 'v' prefix
        let (input, _) = opt(char('v'))(input)?;
        // as long as characters are base-10 digits, collect them and parse into a u64
        let (input, major) = map(digit1, |s: &str| s.parse::<u64>().unwrap())(input)?;
        // parse a '.'
        let (input, _) = char('.')(input)?;
        // as long as characters are base-10 digits, collect them and parse into a u64
        let (input, minor) = map(digit1, |s: &str| s.parse::<u64>().unwrap())(input)?;
        // optionally parse a '.' followed by base-10 digits, collect those into an Option<u64>
        let (input, bugfix) = opt(preceded(
            char('.'),
            map(digit1, |s: &str| s.parse::<u64>().unwrap()),
        ))(input)?;

        Ok((
            input,
            Self {
                major,
                minor,
                bugfix,
            },
        ))
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

impl FromStr for SemanticVersion {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(SemanticVersion::nom_parse(s)
            .map(|(_, v)| v)
            .map_err(|e| e.to_owned())
            .with_context(|| format!("Unable to parse input as semantic version"))?)
    }
}

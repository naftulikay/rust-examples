//! Simple example utilizing [nom] to parse semantic versions as strings into fully-structured
//! [SemanticVersion] objects.
//!
//! ## Comparing, Sorting, and Equality
//!
//! We are doing something non-trivial here with [Ord] and [PartialOrd]. Generally **you should
//! never implement these traits yourself**, but in this case, we must because of a certain feature
//! we require. In our case, a _less-specific_ semantic version containing only a major and a minor
//! version should always be sorted _before_ a more specific version.
//!
//! For instance, using the derive macro, Rust would see `1.0.0` as sorting _before_ `1.0` due to
//! the bugfix version being an [Option<u64>]. We had to implement [Ord] and [PartialOrd] in order
//! to override this, to ensure that `1.0` always sorts as higher priority than `1.0.0`.
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
//! If a `v` prefix is preferred, [SemanticVersion::prefixed] will produce a string accordingly.
//!
//! [SemanticVersion] implements [FromStr], and provides a `parse` function which internally calls
//! the [FromStr] implementation.
//!
//! ## Serde
//!
//! [serde] support is also included with [Serialize] and [Deserialize] support. The default
//! implementation will not include a `v` prefix.
//!
//! To serialize with a `v` prefix, use `#[serde(with = "example_parsing::semver::prefixed")]`:
//!
//! ```rust
//! use example_parsing::semver::{self, SemanticVersion};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, Deserialize, Serialize)]
//! pub struct Container {
//!     #[serde(with = "semver::prefixed")]
//!     pub version: SemanticVersion,
//! }
//! ```
//!
//! ## Clap
//!
//! [SemanticVersion] also supports [clap] right out-of-the-box:
//!
//! ```rust
//! use example_parsing::semver::SemanticVersion;
//! use clap::Parser;
//!
//! #[derive(Debug, Parser)]
//! #[command(name = "prog")]
//! pub struct Args {
//!     #[arg(short = 'V', long = "semver")]
//!     pub version: SemanticVersion,
//! }
//! ```

#[cfg(test)]
mod tests;

use anyhow::{Context, Error, Result};
use nom::character::{complete::char, complete::digit1};
use nom::combinator::{map, opt};
use nom::sequence::preceded;
use nom::IResult;
use serde::{de, ser, Deserializer, Serializer};
use std::cmp::Ordering;

use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// Representation of a semantic version with an optional bugfix revision.
#[derive(Debug, Clone, Eq, PartialEq)]
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
    pub fn prefixed(&self) -> String {
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

impl Ord for SemanticVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        // NOTE you should never implement PartialOrd yourself, but we have a unique case: when
        //      bugfix is None, it should always be equal/ge/le than an unabridged release with the
        //      same major/minor.
        if self.major.gt(&other.major) {
            return Ordering::Greater;
        } else if self.major.lt(&other.major) {
            return Ordering::Less;
        }

        if self.minor.gt(&other.minor) {
            return Ordering::Greater;
        } else if self.minor.lt(&other.minor) {
            return Ordering::Less;
        }

        // at this point, major is equal, and minor is equal, so it's bugfix comparison time

        // if both bugfixes are empty, we are equal
        if self.bugfix.is_none() && other.bugfix.is_none() {
            return Ordering::Equal;
        }

        // if my bugfix is empty and the other is not, I am greater
        if self.bugfix.is_none() && other.bugfix.is_some() {
            return Ordering::Greater;
        }

        // if the other bugfix is empty and mine is not, I am lesser
        if self.bugfix.is_some() && other.bugfix.is_none() {
            return Ordering::Less;
        }

        // at this point, we know that we both have bugfix versions, so compare them and end
        self.bugfix.cmp(&other.bugfix)
    }
}

impl PartialOrd for SemanticVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl ser::Serialize for SemanticVersion {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

struct SemanticVersionVisitor;

impl<'de> de::Visitor<'de> for SemanticVersionVisitor {
    type Value = SemanticVersion;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a semantic version including at least major and minor versions, optionally a bugfix version, delimited by '.', and optionally prefixed with a literal 'v'")
    }

    fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
    where
        E: de::Error,
    {
        v.parse().map_err(E::custom)
    }
}

impl<'de> de::Deserialize<'de> for SemanticVersion {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(SemanticVersionVisitor)
    }
}

/// Convenience module for use with [serde]'s `with` derive parameter.
///
/// Example:
///
/// ```rust
/// use example_parsing::semver::{self, SemanticVersion};///
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Debug, Deserialize, Serialize)]
/// pub struct PrefixedContainer {
///     #[serde(with = "semver::prefixed")]
///     pub version: SemanticVersion,
/// }
/// ```
pub mod prefixed {
    use super::{SemanticVersion, SemanticVersionVisitor};

    use serde::{de, ser};

    /// Serialize a [SemanticVersion] including a `v` prefix.
    pub fn serialize<S>(v: &SemanticVersion, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_str(v.prefixed().as_str())
    }

    /// Deserialize a [SemanticVersion].
    ///
    /// Does not differ from the default implementation.
    pub fn deserialize<'de, D>(d: D) -> Result<SemanticVersion, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_str(SemanticVersionVisitor)
    }
}

use super::SemanticVersion;

use serde::{Deserialize, Serialize};
use serde_json;

use std::str::FromStr;

#[test]
fn test_parse_strip_leading_zeroes() {
    assert_eq!(123u64, "000123".parse::<u64>().unwrap());
    assert_eq!(
        SemanticVersion::new(9, 8, 7),
        SemanticVersion::parse("0009.008.07").expect("unable to parse with leading zeroes")
    );
}
#[test]
fn test_display() {
    // abridged, no prefix
    assert_eq!("5.6", SemanticVersion::abridged(5, 6).to_string());
    // full, no prefix
    assert_eq!("1.2.3", SemanticVersion::new(1, 2, 3).to_string());
    // full, v prefix
    assert_eq!("v0.0.1", SemanticVersion::new(0, 0, 1).prefixed());
    // abridged, v prefix
    assert_eq!("v0.1", SemanticVersion::abridged(0, 1).prefixed());
}

#[test]
fn test_nom_parse() {
    // test full, no prefix
    let (_, v) = SemanticVersion::nom_parse("1.2.3").expect("unable to parse 1.2.3");
    assert_eq!(SemanticVersion::new(1, 2, 3), v);
    // test full, prefix
    let (_, v) = SemanticVersion::nom_parse("v5.6.7").expect("unable to parse v5.6.7");
    assert_eq!(SemanticVersion::new(5, 6, 7), v);
    // test abridged, no prefix
    let (_, v) = SemanticVersion::nom_parse("0.1").expect("unable to parse 0.1");
    assert_eq!(SemanticVersion::abridged(0, 1), v);
    // test abridged, prefix
    let (_, v) = SemanticVersion::nom_parse("v2.3").expect("unable to parse v2.3");
    assert_eq!(SemanticVersion::abridged(2, 3), v);
    // test failures
    assert!(SemanticVersion::nom_parse("unrelated").is_err());
    // FIXME do we care enough to make the parser complete, not allowing trailing data?
}

#[test]
fn test_from_str() {
    assert_eq!(
        SemanticVersion::new(1, 2, 3),
        SemanticVersion::from_str("1.2.3").expect("unable to parse 1.2.3")
    );
    assert_eq!(
        SemanticVersion::new(5, 6, 7),
        SemanticVersion::from_str("v5.6.7").expect("unable to parse v5.6.7")
    );
    assert_eq!(
        SemanticVersion::abridged(7, 8),
        SemanticVersion::from_str("7.8").expect("unable to parse 7.8")
    );
    assert_eq!(
        SemanticVersion::abridged(0, 1),
        SemanticVersion::from_str("v0.1").expect("unable to parse v0.1")
    );
}

/// Tests equal/le/ge between [SemanticVersion]s.
#[test]
fn test_ord_equal() {
    // major
    assert!(SemanticVersion::new(1, 0, 0).eq(&SemanticVersion::new(1, 0, 0)));
    assert!(SemanticVersion::new(1, 0, 0).le(&SemanticVersion::new(1, 0, 0)));
    assert!(SemanticVersion::new(1, 0, 0).ge(&SemanticVersion::new(1, 0, 0)));
    // minor
    assert!(SemanticVersion::new(0, 1, 0).eq(&SemanticVersion::new(0, 1, 0)));
    assert!(SemanticVersion::new(0, 1, 0).le(&SemanticVersion::new(0, 1, 0)));
    assert!(SemanticVersion::new(0, 1, 0).ge(&SemanticVersion::new(0, 1, 0)));
    // bugfix
    assert!(SemanticVersion::new(0, 0, 1).eq(&SemanticVersion::new(0, 0, 1)));
    assert!(SemanticVersion::new(0, 0, 1).le(&SemanticVersion::new(0, 0, 1)));
    assert!(SemanticVersion::new(0, 0, 1).ge(&SemanticVersion::new(0, 0, 1)));
    // abridged
    assert!(SemanticVersion::abridged(1, 0).eq(&SemanticVersion::abridged(1, 0)));
    assert!(SemanticVersion::abridged(1, 0).le(&SemanticVersion::abridged(1, 0)));
    assert!(SemanticVersion::abridged(1, 0).ge(&SemanticVersion::abridged(1, 0)));
}

/// Tests normal greater/equal between [SemanticVersion]s.
///
/// Does not test abridged superiority over bugfix releases.
#[test]
fn test_ord_greater() {
    // major
    assert!(SemanticVersion::new(1, 0, 0).ge(&SemanticVersion::new(0, 1, 1)));
    assert!(SemanticVersion::new(1, 0, 0).gt(&SemanticVersion::new(0, 1, 1)));
    assert!(SemanticVersion::abridged(1, 0).ge(&SemanticVersion::new(0, 1, 0)));
    assert!(SemanticVersion::abridged(1, 0).gt(&SemanticVersion::new(0, 1, 0)));
    // minor
    assert!(SemanticVersion::new(0, 1, 0).ge(&SemanticVersion::new(0, 0, 1)));
    assert!(SemanticVersion::new(0, 1, 0).gt(&SemanticVersion::new(0, 0, 1)));
    assert!(SemanticVersion::abridged(0, 1).ge(&SemanticVersion::new(0, 0, 1)));
    assert!(SemanticVersion::abridged(0, 1).gt(&SemanticVersion::new(0, 0, 1)));
    // bugfix
    assert!(SemanticVersion::new(0, 0, 1).ge(&SemanticVersion::new(0, 0, 0)));
    assert!(SemanticVersion::new(0, 0, 1).gt(&SemanticVersion::new(0, 0, 0)));
}

/// Tests normal less/equal between [SemanticVersion]s.
///
/// Does not test abridged superiority over bugfix releases.
#[test]
fn test_ord_less() {
    // major
    assert!(SemanticVersion::new(0, 1, 1).le(&SemanticVersion::new(1, 0, 0)));
    assert!(SemanticVersion::new(0, 1, 1).lt(&SemanticVersion::new(1, 0, 0)));
    assert!(SemanticVersion::abridged(0, 1).le(&SemanticVersion::new(1, 0, 0)));
    assert!(SemanticVersion::abridged(0, 1).lt(&SemanticVersion::new(1, 0, 0)));
    // minor
    assert!(SemanticVersion::new(0, 0, 1).le(&SemanticVersion::new(0, 1, 0)));
    assert!(SemanticVersion::new(0, 0, 1).lt(&SemanticVersion::new(0, 1, 0)));
    assert!(SemanticVersion::abridged(0, 0).le(&SemanticVersion::new(0, 1, 0)));
    assert!(SemanticVersion::abridged(0, 0).lt(&SemanticVersion::new(0, 1, 0)));
    // bugfix
    assert!(SemanticVersion::new(0, 0, 0).le(&SemanticVersion::new(0, 0, 1)));
    assert!(SemanticVersion::new(0, 0, 0).lt(&SemanticVersion::new(0, 0, 1)));
}

#[test]
fn test_ord_greater_abridged() {
    assert!(SemanticVersion::abridged(1, 0).ge(&SemanticVersion::new(1, 0, 0)));
    assert!(SemanticVersion::abridged(1, 0).gt(&SemanticVersion::new(1, 0, 0)));
}

#[test]
fn test_ord_less_abridged() {
    assert!(SemanticVersion::new(1, 0, 0).le(&SemanticVersion::abridged(1, 0)));
    assert!(SemanticVersion::new(1, 0, 0).lt(&SemanticVersion::abridged(1, 0)));
}

#[derive(Debug, Deserialize, Serialize)]
struct Container {
    version: SemanticVersion,
}

impl Container {
    fn new(version: SemanticVersion) -> Self {
        Self { version }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct MaybeContainer {
    version: Option<SemanticVersion>,
}

impl MaybeContainer {
    fn new() -> Self {
        Self { version: None }
    }

    fn with(version: SemanticVersion) -> Self {
        Self {
            version: Some(version),
        }
    }
}

#[test]
fn test_deserialize() {
    // full
    assert_eq!(
        SemanticVersion::new(0, 26, 5),
        serde_json::from_str::<Container>(r#"{"version":"0.26.5"}"#)
            .expect("unable to deserialize")
            .version
    );
    // partial
    assert_eq!(
        SemanticVersion::abridged(1, 5),
        serde_json::from_str::<Container>(r#"{"version":"1.5"}"#)
            .expect("unable to deserialize")
            .version
    );
    // null
    assert_eq!(
        None,
        serde_json::from_str::<MaybeContainer>(r#"{"version":null}"#)
            .expect("unable to deserialize null")
            .version
    );
    // not null
    assert_eq!(
        Some(SemanticVersion::new(0, 0, 1)),
        serde_json::from_str::<MaybeContainer>(r#"{"version":"0.0.1"}"#)
            .expect("unable to deserialize not null")
            .version
    );
}

#[test]
fn test_serialize() {
    // full
    assert_eq!(
        r#"{"version":"1.2.3"}"#,
        serde_json::to_string(&Container::new(SemanticVersion::new(1, 2, 3)))
            .expect("unable to serialize full")
    );
    // partial
    assert_eq!(
        r#"{"version":"0.1"}"#,
        serde_json::to_string(&Container::new(SemanticVersion::abridged(0, 1)))
            .expect("unable to serialize abridged")
    );
    // null
    assert_eq!(
        r#"{"version":null}"#,
        serde_json::to_string(&MaybeContainer::new()).expect("unable to serialize null")
    );
    // not null
    assert_eq!(
        r#"{"version":"1.3.1"}"#,
        serde_json::to_string(&MaybeContainer::with(SemanticVersion::new(1, 3, 1)))
            .expect("unable to serialize not null")
    );
}

#[test]
fn test_prefixed_serialize() {
    #[derive(Debug, Deserialize, Serialize)]
    struct Container {
        #[serde(with = "super::prefixed")]
        version: SemanticVersion,
    }

    impl Container {
        fn new(version: SemanticVersion) -> Self {
            Self { version }
        }
    }

    assert_eq!(
        r#"{"version":"v9.4.2"}"#,
        serde_json::to_string(&Container::new(SemanticVersion::new(9, 4, 2)))
            .expect("unable to serialize with prefix")
    );
}

#[test]
fn test_clap() {
    use clap::Parser;

    #[derive(Debug, Parser)]
    #[command(name = "semver")]
    struct Args {
        #[arg(short = 'V', long = "semver")]
        version: SemanticVersion,
    }

    let args =
        Args::try_parse_from(&["semver", "--semver", "0.1.2"]).expect("unable to parse args");

    assert_eq!(SemanticVersion::new(0, 1, 2), args.version);
}

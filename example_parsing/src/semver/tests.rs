use super::SemanticVersion;

use std::str::FromStr;

#[test]
fn test_u64_parse_strip_leading_zeroes() {
    assert_eq!(123u64, "000123".parse().unwrap());
}
#[test]
fn test_display() {
    // abridged, no prefix
    assert_eq!("5.6", SemanticVersion::abridged(5, 6).to_string());
    // full, no prefix
    assert_eq!("1.2.3", SemanticVersion::new(1, 2, 3).to_string());
    // full, v prefix
    assert_eq!("v0.0.1", SemanticVersion::new(0, 0, 1).v());
    // abridged, v prefix
    assert_eq!("v0.1", SemanticVersion::abridged(0, 1).v());
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

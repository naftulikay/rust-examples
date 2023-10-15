use super::SemanticVersion;

#[test]
fn test_semver_display() {
    assert_eq!("5.6", SemanticVersion::abridged(5, 6).to_string());
    assert_eq!("1.2.3", SemanticVersion::new(1, 2, 3).to_string());
}

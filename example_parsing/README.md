# example_parsing Crate

Examples for parsing values, mostly using [`nom`][nom].

## `example_parsing::semver`

Exposes a `SemanticVersion` struct which is defined like this:

```rust
pub struct SemanticVersion {
    pub major: u64,
    pub minor: u64,
    pub bugfix: Option<u64>,
}
```

This type implements `Display`, `FromStr`, `serde::Deserialize` and `serde::Serialize`, and `clap` support is included
and tested. The `prefixed` method will construct a semantic version string with a `v` prefix.

Values can be constructed using `FromStr` and from `SemanticVersion::parse`, as well as with all three components using
`SemanticVersion::new`, or with only two components with `SemanticVersion::abridged`.

Parsing is done via `nom`, and should be extraordinarily fast.

Consult the [module doc-strings](./src/semver.rs) for more information as well as the source code.

 [nom]: https://docs.rs/nom/latest/nom/
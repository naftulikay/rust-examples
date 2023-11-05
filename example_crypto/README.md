# `example_crypto` Crate

Examples and benchmarks for crypto stuff.

## Examples

### OpenSSL: Create Client CA and Certificate

Using ed25519 keys, create a root CA, intermediate CA, and client certificate for client TLS (mTLS):

```shell
cargo run --example client_ca
```

Private keys, certificates, chains, and a `*.p12` encrypted archive containing the full client cert chain and private
key will be generated in [`output/client-ca`](../output/client-ca).

## Benchmarks
a
### Key Generation

Benchmark key generation for common asymmetric algorithms such as RSA, Ed25519, Ed448, secp256k1, and secp384r1:

```shell
cargo bench --bench keygen
```

### Digital Signatures

Benchmark signature generation for common asymmetric algorithms such as RSA, Ed25519, Ed448, secp256k1, and secp384r1:

```shell
cargo bench --bench sign
```

### Random Number Generation

Benchmark RNG performance:

```shell
cargo bench --bench rand
```
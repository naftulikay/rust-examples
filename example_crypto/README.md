# `example_crypto` Crate

Examples and benchmarks for crypto stuff.

## Examples

### OpenSSL: Create Client CA and Certificate

Using ed25519 keys, create a root CA, intermediate CA, and client certificate for client TLS (mTLS):

```shell
cargo run --example client_ca
```

Private keys, certificates, chains, and a `*.p12` encrypted archive containing the full client cert chain and private
key will be generated in `output/client-ca` in the repository root.

X509 is configured fairly securely, limiting key usage/basic/extended constraints, such that there are at max two
levels of CAs, and the client certificate is limited to only being used for client authentication.

### OpenSSL + Sequoia OpenPGP: Import Keypair

To demonstrate generation of public/private ed25519 keys in OpenSSL and then to import these keys into a new PGP
identity:

```shell
cargo run --example openssl_sequoia
```

Most PGP implementations do not allow users to import a keypair from an external source. Generally speaking, you should
not do things the way that this example presents. This is being done as an experiment to determine whether it would be
possible to sign X509 certificates using a private key in in a PGP keyring.

## Benchmarks

Various benchmark suites exist.

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
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::process::{exit, Command, Stdio};

use example_crypto::openssl::client_ca::{ClientCAConfig, ClientCAExample};

use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use base64::Engine as _;
use openssl::pkcs12::Pkcs12;
use openssl::stack::Stack;
use rand::{thread_rng, RngCore};

#[cfg(not(windows))]
const LINE_ENDING: &str = "\n";

#[cfg(windows)]
const LINE_ENDING: &str = "\r\n";

fn main() {
    let repo_root = {
        let child = Command::new("git")
            .arg("rev-parse")
            .arg("--show-toplevel")
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let mut s = String::with_capacity(256);

        child.stdout.unwrap().read_to_string(&mut s).unwrap();

        PathBuf::from(s.trim())
    };

    // if this isn't a dir, then we're screwed
    assert!(repo_root.is_dir());

    let output_dir = repo_root.join("output").join("client-ca");

    if !output_dir.is_dir() {
        // if the dir doesn't exist, create it
        fs::create_dir_all(&output_dir).unwrap();
    }

    // generate everything
    eprintln!("Generating full CA chain...");
    let ca = ClientCAExample::generate(ClientCAConfig::default()).unwrap();

    eprintln!("Validating full CA chain...");
    if !ca.verify().unwrap() {
        eprintln!("Verification failed");
        exit(1);
    }

    // write root key
    fs::write(
        output_dir.join("root-ca.key.pem"),
        ca.root_key.private_key_to_pem_pkcs8().unwrap(),
    )
    .unwrap();

    // write root cert
    fs::write(
        output_dir.join("root-ca.crt.pem"),
        ca.root_cert.to_pem().unwrap(),
    )
    .unwrap();

    // write intermediate key
    fs::write(
        output_dir.join("intermediate-ca.key.pem"),
        ca.intermediate_key.private_key_to_pem_pkcs8().unwrap(),
    )
    .unwrap();

    // write intermediate cert
    fs::write(
        output_dir.join("intermediate-ca.crt.pem"),
        ca.intermediate_cert.to_pem().unwrap(),
    )
    .unwrap();

    // write intermediate cert chain
    let intermediate_chain = {
        // root cert, then newline, then intermediate cert
        let mut v = ca.root_cert.to_pem().unwrap();
        v.extend_from_slice(LINE_ENDING.as_bytes());
        v.extend(ca.intermediate_cert.to_pem().unwrap());
        v
    };

    fs::write(
        output_dir.join("intermediate-ca-chain.crt.pem"),
        intermediate_chain,
    )
    .unwrap();

    // write client key
    fs::write(
        output_dir.join("client.key.pem"),
        ca.client_key.private_key_to_pem_pkcs8().unwrap(),
    )
    .unwrap();

    // write client cert
    fs::write(
        output_dir.join("client.crt.pem"),
        ca.client_cert.to_pem().unwrap(),
    )
    .unwrap();

    // write client cert chain
    let client_chain = {
        let mut v = ca.root_cert.to_pem().unwrap();
        v.extend_from_slice(LINE_ENDING.as_bytes());
        v.extend(ca.intermediate_cert.to_pem().unwrap());
        v.extend(ca.client_cert.to_pem().unwrap());
        v
    };

    fs::write(output_dir.join("client-chain.crt.pem"), client_chain).unwrap();

    // generate a password for the pkcs12 archive
    let key = {
        let mut b = [0; 32];
        thread_rng().fill_bytes(&mut b);
        BASE64_URL_SAFE_NO_PAD.encode(b)
    };

    println!("Generated password for client key PKCS12 bundle: {key}");

    let pkcs12 = {
        let mut p = Pkcs12::builder();
        p.pkey(&ca.client_key);
        p.cert(&ca.client_cert);

        let mut cert_stack = Stack::new().unwrap();
        cert_stack.push(ca.root_cert).unwrap();
        cert_stack.push(ca.intermediate_cert).unwrap();

        p.ca(cert_stack);

        p.build2(key.as_str()).unwrap()
    };

    fs::write(
        output_dir.join("client-bundle.p12"),
        pkcs12.to_der().unwrap(),
    )
    .unwrap();
}

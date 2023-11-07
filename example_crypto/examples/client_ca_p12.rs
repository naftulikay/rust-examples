use std::env::current_dir;
use std::fs::File;
use std::io::{IoSlice, Write};
use std::ops::{Add, Sub};
use std::sync::Once;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Error;
use base64::alphabet::URL_SAFE;
use base64::engine::general_purpose::NO_PAD;
use base64::engine::GeneralPurpose;
use base64::Engine;
use openssl::asn1::{Asn1Integer, Asn1Time};
use openssl::bn::BigNum;
use openssl::error::ErrorStack;
use openssl::hash::MessageDigest;
use openssl::nid::Nid;
use openssl::pkcs12::Pkcs12;
use openssl::pkey::{PKey, Private};
use openssl::stack::Stack;
use openssl::x509::extension::{BasicConstraints, KeyUsage};
use openssl::x509::{X509Extension, X509Name, X509};
use rand::{thread_rng, RngCore};
use tracing::{self, Level};

static LOG_INIT: Once = Once::new();

pub struct SelfSignedCert {
    pub key: PKey<Private>,
    pub cert: X509,
}

fn main() -> Result<(), Error> {
    LOG_INIT.call_once(|| {
        tracing_subscriber::fmt()
            .with_max_level(Level::DEBUG)
            .pretty()
            .try_init()
            .expect("unable to setup logging");
    });

    let data = gen_self_signed_cert()?;

    let fingerprint = data.cert.digest(MessageDigest::sha256())?;

    tracing::info!(
        "Certificate Fingerprint (SHA-256): {}",
        hex::encode_upper(fingerprint.as_ref())
    );

    let key_material: [u8; 16] = rand_bytes();

    let pkcs12_pass = GeneralPurpose::new(&URL_SAFE, NO_PAD).encode(key_material);

    let pkcs12 = {
        let mut p = Pkcs12::builder();
        p.pkey(&data.key);
        p.cert(&data.cert);

        let mut cert_stack = Stack::new().unwrap();
        cert_stack.push(data.cert).unwrap();

        p.build2(pkcs12_pass.as_str()).unwrap()
    };

    let output = current_dir().unwrap().join("root-ca.p12");

    let mut f = File::options()
        .write(true)
        .truncate(true)
        .create(true)
        .open(&output)?;

    let _ = f.write_vectored(&[IoSlice::new(pkcs12.to_der()?.as_ref())])?;

    println!("PKCS12 Bundle Passphrase: {}", pkcs12_pass);
    println!(
        "PKCS12 Bundle Saved To {}",
        output.strip_prefix(current_dir()?)?.display()
    );

    Ok(())
}

fn gen_key() -> Result<PKey<Private>, ErrorStack> {
    tracing::debug!("generating prime256v1 key");
    PKey::ec_gen("prime256v1")
}

fn gen_subject() -> Result<X509Name, ErrorStack> {
    tracing::debug!("generating certificate name fields");

    let mut n = X509Name::builder()?;
    n.append_entry_by_nid(Nid::COMMONNAME, "Temporary OpenSSL Cert")?;
    n.append_entry_by_nid(Nid::COUNTRYNAME, "US")?;
    n.append_entry_by_nid(Nid::ORGANIZATIONNAME, "Naftuli, Inc.")?;
    n.append_entry_by_nid(Nid::ORGANIZATIONALUNITNAME, "naftuli.wtf")?;

    Ok(n.build())
}

fn gen_key_usage() -> Result<X509Extension, ErrorStack> {
    tracing::debug!("generating key usage data");

    KeyUsage::new()
        .critical()
        .digital_signature()
        .key_cert_sign()
        .build()
}

fn gen_basic_constraints() -> Result<X509Extension, ErrorStack> {
    tracing::debug!("generating basic constraints");

    BasicConstraints::new().critical().ca().pathlen(1).build()
}

struct ValidityPeriod {
    not_before: Asn1Time,
    not_after: Asn1Time,
}

fn gen_validity() -> Result<ValidityPeriod, ErrorStack> {
    tracing::debug!("generating certificate validity period");

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

    Ok(ValidityPeriod {
        not_before: Asn1Time::from_unix(now.sub(Duration::from_secs(60 * 30)).as_secs() as i64)?,
        not_after: Asn1Time::from_unix(now.add(Duration::from_secs(60 * 60 * 12)).as_secs() as i64)?,
    })
}

fn gen_self_signed_cert() -> Result<SelfSignedCert, ErrorStack> {
    let key = gen_key()?;
    let subject_name = gen_subject()?;
    let key_usage = gen_key_usage()?;
    let basic_constraints = gen_basic_constraints()?;
    let validity = gen_validity()?;

    let cert = {
        tracing::info!("constructing self-signed certificate");
        let mut builder = X509::builder()?;
        builder.set_version(2)?; // v3
        builder.set_subject_name(&subject_name)?;
        builder.set_issuer_name(&subject_name)?;
        builder
            .set_serial_number(Asn1Integer::from_bn(BigNum::from_u32(1000)?.as_ref())?.as_ref())?;
        builder.set_not_before(&validity.not_before)?;
        builder.set_not_after(&validity.not_after)?;
        builder.set_pubkey(&key)?;
        builder.append_extension(key_usage)?;
        builder.append_extension(basic_constraints)?;
        builder.sign(&key, MessageDigest::sha256())?;
        builder.build()
    };

    Ok(SelfSignedCert { key, cert })
}

fn rand_bytes<const S: usize>() -> [u8; S] {
    let mut v: [u8; S] = [0; S];
    thread_rng().fill_bytes(&mut v);
    v
}

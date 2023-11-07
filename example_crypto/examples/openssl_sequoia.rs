use sequoia_openpgp as openpgp;

use anyhow::Error;
use openpgp::packet::key::Key4;
use openpgp::packet::key::{PrimaryRole, PublicParts, SecretParts};
use openssl::asn1::{Asn1Integer, Asn1Time};
use openssl::bn::BigNum;
use openssl::error::ErrorStack;
use openssl::nid::Nid;
use openssl::pkey::{PKey, Private};
use openssl::x509::extension::{BasicConstraints, KeyUsage};
use openssl::x509::{X509Extension, X509Name, X509};
use tracing::{self, Level};
use tracing_subscriber;

use sequoia_openpgp::packet::UserID;
use std::ops::{Add, Sub};
use std::sync::Once;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

static LOG_INIT: Once = Once::new();

pub struct UnsignedCert {
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

    let data = gen_unsigned_cert()?;

    tracing::info!(
        "Public key as hex: {}",
        hex::encode_upper(data.key.raw_public_key()?.as_slice())
    );

    let pgp_uid = UserID::from_address("Nafutli Kay", "Ol' Yeller", "unreachable@naftuli.wtf")?;

    tracing::info!(
        name = pgp_uid.name2()?,
        comment = pgp_uid.comment2()?,
        email = pgp_uid.email2()?,
        "Created PGP user id"
    );

    let (pgp_public, _pgp_private) = gen_openpgp_key_packets(&data.key)?;

    tracing::info!("PGP public key fingerprint: {}", pgp_public.fingerprint());

    Ok(())
}

fn gen_key() -> Result<PKey<Private>, ErrorStack> {
    tracing::debug!("generating ed25519 key");
    PKey::generate_ed25519()
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

    Ok(KeyUsage::new()
        .critical()
        .digital_signature()
        .key_cert_sign()
        .build()?)
}

fn gen_basic_constraints() -> Result<X509Extension, ErrorStack> {
    tracing::debug!("generating basic constraints");

    Ok(BasicConstraints::new().critical().ca().pathlen(1).build()?)
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

fn gen_unsigned_cert() -> Result<UnsignedCert, ErrorStack> {
    let key = gen_key()?;
    let subject_name = gen_subject()?;
    let key_usage = gen_key_usage()?;
    let basic_constraints = gen_basic_constraints()?;
    let validity = gen_validity()?;

    let cert = {
        tracing::info!("constructing self-referential unsigned certificate");
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
        builder.build()
    };

    Ok(UnsignedCert { key, cert })
}

fn gen_openpgp_key_packets(
    key: &PKey<Private>,
) -> Result<
    (
        Key4<PublicParts, PrimaryRole>,
        Key4<SecretParts, PrimaryRole>,
    ),
    Error,
> {
    tracing::debug!("creating PGP public/private keys from generated keypair");

    let now = SystemTime::now();

    Ok((
        Key4::import_public_ed25519(key.raw_public_key()?.as_slice(), now)?,
        Key4::import_secret_ed25519(key.raw_private_key()?.as_slice(), now)?,
    ))
}

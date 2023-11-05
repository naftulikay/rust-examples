//! Example demonstrating the generation of a two-layer certificate authority (CA) and a client
//! certificate, verifying the whole process.

use openssl::asn1::{Asn1Integer, Asn1Time};
use openssl::error::ErrorStack;
use openssl::hash::MessageDigest;
use openssl::nid::Nid;
use openssl::pkey::{PKey, Private};
use openssl::x509::extension::{BasicConstraints, ExtendedKeyUsage, KeyUsage};
use openssl::x509::{X509Name, X509NameRef, X509VerifyResult, X509};

use openssl::bn::BigNum;
use std::ops::{Add, Sub};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// The amount of time before now to allow this certificate to be used for.
///
/// This is set to 30 minutes. This is important because of user clock drift. With this setting, we
/// ensure that users with a clock up to 30 minutes off will still pass validation.
pub const NOT_BEFORE_DRIFT_DURATION: Duration = Duration::from_secs(60 * 30);

/// The certificate expiry duration for the client certificate.
///
/// Set to 1 hour.
pub const CLIENT_EXPIRY_DURATION: Duration = Duration::from_secs(60 * 60);

/// The certificate expiry duration for the intermediate CA.
///
/// Set to 3 hours.
pub const INTERMEDIATE_EXPIRY_DURATION: Duration = Duration::from_secs(60 * 60 * 3);

/// The certificate expiry duration for the root CA.
///
/// Set to 6 hours.
pub const ROOT_EXPIRY_DURATION: Duration = Duration::from_secs(60 * 60 * 6);

/// The X509 version to use when generating certificates.
///
/// This is zero-indexed, so version 3 is represented by `2`.
pub const X509_CERT_VERSION: i32 = 3 - 1;

/// The subject name to use for the client certificate.
pub const X509_CLIENT_SUBJECT_NAME: &str = "Rust Example Client Certificate";

/// The subject name to use for the intermediate certificate.
pub const X509_INTERMEDIATE_SUBJECT_NAME: &str = "Rust Example Intermediate CA";

/// The subject name to use for the root certificate.
pub const X509_ROOT_SUBJECT_NAME: &str = "Rust Example Root CA";

/// An example which generates a root CA, intermediate CA, and a client certificate.
pub struct ClientCAExample {
    /// The root CA's public/private keypair.
    pub root_key: PKey<Private>,
    /// The root CA's self-signed certificate.
    pub root_cert: X509,
    /// The intermediate CA's public/private keypair.
    pub intermediate_key: PKey<Private>,
    /// The intermediate CA's certificate, signed by the root CA cert.
    pub intermediate_cert: X509,
    /// The client certificate's public/private keypair.
    pub client_key: PKey<Private>,
    /// The client certificate, signed by the intermediate CA cert.
    pub client_cert: X509,
}

impl ClientCAExample {
    /// Generate a two-layer TLS client CA with a root, intermediate, and single client certificate
    /// and private keys.
    ///
    /// Use [Default::default] to generate the CA config if you'd like to accept default settings.
    pub fn generate(config: ClientCAConfig) -> Result<Self, ErrorStack> {
        let root_key = Self::generate_key()?;
        let root_cert = Self::generate_root_cert(&config.root_config, &root_key)?;

        let intermediate_key = Self::generate_key()?;
        let intermediate_cert = Self::generate_intermediate_cert(
            &config.intermediate_config,
            &intermediate_key,
            &root_key,
            root_cert.subject_name(),
        )?;

        let client_key = Self::generate_key()?;
        let client_cert = Self::generate_client_cert(
            &config.client_config,
            &client_key,
            &intermediate_key,
            intermediate_cert.subject_name(),
        )?;

        Ok(Self {
            root_key,
            root_cert,
            intermediate_key,
            intermediate_cert,
            client_key,
            client_cert,
        })
    }

    /// Generate a private Ed25519 key.
    fn generate_key() -> Result<PKey<Private>, ErrorStack> {
        PKey::generate_ed25519()
    }

    /// Generate the root CA certificate.
    fn generate_root_cert(config: &CAConfig, key: &PKey<Private>) -> Result<X509, ErrorStack> {
        let subject_name = {
            let mut n = X509Name::builder()?;
            n.append_entry_by_nid(Nid::COMMONNAME, config.subject_name.as_str())?;
            n.append_entry_by_nid(Nid::COUNTRYNAME, "US")?;
            n.append_entry_by_nid(Nid::ORGANIZATIONNAME, "Naftuli, Inc.")?;
            n.append_entry_by_nid(Nid::ORGANIZATIONALUNITNAME, "naftuli.wtf")?;
            n.build()
        };

        // set key usage
        let key_usage = KeyUsage::new()
            .critical()
            .digital_signature()
            .key_cert_sign()
            .build()?;

        // set basic constraints to being critical, being a CA, and only supporting one level of
        // intermediate CA certificates
        let basic = BasicConstraints::new()
            .critical()
            .ca()
            .pathlen(config.ca_type.path_length())
            .build()?;

        // set window of validity for certificate
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let not_before = Asn1Time::from_unix(now.sub(config.max_drift).as_secs() as i64)?;
        let not_after = Asn1Time::from_unix(now.add(config.validity).as_secs() as i64)?;

        let cert = {
            let mut builder = X509::builder()?;
            builder.set_version(X509_CERT_VERSION)?;
            builder.set_subject_name(&subject_name)?;
            builder.set_serial_number(
                Asn1Integer::from_bn(BigNum::from_u32(config.serial_number)?.as_ref())?.as_ref(),
            )?;
            // set issuer name to subject name (self-referential) on root ca
            builder.set_issuer_name(&subject_name)?;
            builder.set_not_before(&not_before)?;
            builder.set_not_after(&not_after)?;
            builder.set_pubkey(key)?;
            builder.append_extension(key_usage)?;
            builder.append_extension(basic)?;
            // with ed25519, it seems openssl does not support message digest for the signature
            builder.sign(key, MessageDigest::null())?;
            builder.build()
        };

        Ok(cert)
    }

    /// Generate the intermediate CA certificate.
    pub fn generate_intermediate_cert(
        config: &CAConfig,
        key: &PKey<Private>,
        root_key: &PKey<Private>,
        root_subject_name: &X509NameRef,
    ) -> Result<X509, ErrorStack> {
        let subject_name = {
            let mut n = X509Name::builder()?;
            n.append_entry_by_nid(Nid::COMMONNAME, config.subject_name.as_str())?;
            n.append_entry_by_nid(Nid::COUNTRYNAME, "US")?;
            n.append_entry_by_nid(Nid::ORGANIZATIONNAME, "Naftuli, Inc.")?;
            n.append_entry_by_nid(Nid::ORGANIZATIONALUNITNAME, "naftuli.wtf")?;
            n.build()
        };

        // set key usage
        let key_usage = KeyUsage::new()
            .critical()
            .digital_signature()
            .key_cert_sign()
            .build()?;

        // set basic constraints
        let basic = BasicConstraints::new()
            .critical()
            .ca()
            .pathlen(config.ca_type.path_length())
            .build()?;

        // set window of validity
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let not_before = Asn1Time::from_unix(now.sub(config.max_drift).as_secs() as i64)?;
        let not_after = Asn1Time::from_unix(now.add(config.validity).as_secs() as i64)?;

        let cert = {
            let mut builder = X509::builder()?;
            builder.set_version(X509_CERT_VERSION)?;
            builder.set_subject_name(&subject_name)?;
            builder.set_serial_number(
                Asn1Integer::from_bn(BigNum::from_u32(config.serial_number)?.as_ref())?.as_ref(),
            )?;
            // set issuer since we have a parent
            builder.set_issuer_name(root_subject_name)?;
            builder.set_not_before(&not_before)?;
            builder.set_not_after(&not_after)?;
            builder.set_pubkey(key)?;
            builder.append_extension(key_usage)?;
            builder.append_extension(basic)?;

            // sign using the root ca key
            builder.sign(root_key, MessageDigest::null())?;

            builder.build()
        };

        Ok(cert)
    }

    /// Generate the client certificate.
    pub fn generate_client_cert(
        config: &ClientCertConfig,
        key: &PKey<Private>,
        intermediate_key: &PKey<Private>,
        intermediate_subject_name: &X509NameRef,
    ) -> Result<X509, ErrorStack> {
        let subject_name = {
            let mut n = X509Name::builder()?;
            n.append_entry_by_nid(Nid::COMMONNAME, config.subject_name.as_str())?;
            n.build()
        };

        // set basic constraints
        let basic = BasicConstraints::new().critical().build()?;

        // set extended constraints: critical but only for client auth
        let extended = ExtendedKeyUsage::new().critical().client_auth().build()?;

        // set window of validity
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let not_before = Asn1Time::from_unix(now.sub(config.max_drift).as_secs() as i64)?;
        let not_after = Asn1Time::from_unix(now.add(config.validity).as_secs() as i64)?;

        let cert = {
            let mut builder = X509::builder()?;
            builder.set_version(X509_CERT_VERSION)?;
            builder.set_subject_name(&subject_name)?;
            builder.set_serial_number(
                Asn1Integer::from_bn(BigNum::from_u32(config.serial_number)?.as_ref())?.as_ref(),
            )?;
            // set issuer since we have a parent
            builder.set_issuer_name(intermediate_subject_name)?;
            builder.set_not_before(&not_before)?;
            builder.set_not_after(&not_after)?;
            builder.set_pubkey(key)?;
            // NOTE on client certificates, key usage should not be set and extended should include client auth
            // basic constraints
            builder.append_extension(basic)?;
            // extended constraints
            builder.append_extension(extended)?;

            // sign the key
            builder.sign(intermediate_key, MessageDigest::null())?;

            builder.build()
        };

        Ok(cert)
    }

    pub fn verify(&self) -> Result<bool, ErrorStack> {
        // verify that root is self-signed
        if !self
            .root_cert
            .verify(self.root_cert.public_key()?.as_ref())
            .unwrap()
        {
            eprintln!("Verification of self-signed root CA failed");
            return Ok(false);
        }

        if self.root_cert.issued(&self.root_cert) != X509VerifyResult::OK {
            eprintln!(
                "Verification that root CA issued itself failed: {}",
                self.root_cert.issued(&self.root_cert).error_string()
            );
            return Ok(false);
        }

        // verify that intermediate is signed by root
        if !self
            .intermediate_cert
            .verify(self.root_cert.public_key()?.as_ref())
            .unwrap()
        {
            eprintln!("Verification of intermediate CA failed");
            return Ok(false);
        }

        if self.root_cert.issued(&self.intermediate_cert) != X509VerifyResult::OK {
            eprintln!(
                "Verification that root CA issued intermediate CA failed: {}",
                self.root_cert
                    .issued(&self.intermediate_cert)
                    .error_string()
            );
            return Ok(false);
        }

        // verify that client is signed by intermediate
        if !self
            .client_cert
            .verify(self.intermediate_cert.public_key()?.as_ref())
            .unwrap()
        {
            eprintln!("Verification of client certificate failed");
        }

        if self.intermediate_cert.issued(&self.client_cert) != X509VerifyResult::OK {
            eprintln!(
                "Verification that intermediate CA issued client certificate failed: {}",
                self.intermediate_cert
                    .issued(&self.client_cert)
                    .error_string()
            );
            return Ok(false);
        }

        Ok(true)
    }
}

#[derive(Debug)]
pub struct ClientCAConfig {
    pub root_config: CAConfig,
    pub intermediate_config: CAConfig,
    pub client_config: ClientCertConfig,
}

impl Default for ClientCAConfig {
    fn default() -> Self {
        Self {
            root_config: CAConfig {
                ca_type: CAType::Root,
                subject_name: X509_ROOT_SUBJECT_NAME.into(),
                max_drift: NOT_BEFORE_DRIFT_DURATION,
                validity: ROOT_EXPIRY_DURATION,
                serial_number: 1000,
            },
            intermediate_config: CAConfig {
                ca_type: CAType::Intermediate,
                subject_name: X509_INTERMEDIATE_SUBJECT_NAME.into(),
                max_drift: NOT_BEFORE_DRIFT_DURATION,
                validity: INTERMEDIATE_EXPIRY_DURATION,
                serial_number: 2000,
            },
            client_config: Default::default(),
        }
    }
}

#[derive(Debug)]
pub struct CAConfig {
    pub ca_type: CAType,
    pub subject_name: String,
    pub max_drift: Duration,
    pub validity: Duration,
    pub serial_number: u32,
}

#[derive(Debug)]
pub enum CAType {
    Root,
    Intermediate,
}

impl CAType {
    pub fn path_length(&self) -> u32 {
        match &self {
            Self::Root => 1,
            Self::Intermediate => 0,
        }
    }
}

#[derive(Debug)]
pub struct ClientCertConfig {
    pub subject_name: String,
    pub max_drift: Duration,
    pub validity: Duration,
    pub serial_number: u32,
}

impl Default for ClientCertConfig {
    fn default() -> Self {
        Self {
            subject_name: X509_CLIENT_SUBJECT_NAME.into(),
            max_drift: NOT_BEFORE_DRIFT_DURATION,
            validity: CLIENT_EXPIRY_DURATION,
            serial_number: 3000,
        }
    }
}

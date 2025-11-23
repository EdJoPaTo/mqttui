use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;

use anyhow::Context as _;
use rumqttc::TlsConfiguration;
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified};
use rustls::{ClientConfig, DigitallySignedStruct, KeyLogFile, SignatureScheme};
use rustls_pki_types::{CertificateDer, PrivateKeyDer, ServerName, UnixTime};

#[derive(Debug)]
struct NoVerifier;
impl rustls::client::danger::ServerCertVerifier for NoVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![
            SignatureScheme::RSA_PKCS1_SHA1,
            SignatureScheme::ECDSA_SHA1_Legacy,
            SignatureScheme::RSA_PKCS1_SHA256,
            SignatureScheme::ECDSA_NISTP256_SHA256,
            SignatureScheme::RSA_PKCS1_SHA384,
            SignatureScheme::ECDSA_NISTP384_SHA384,
            SignatureScheme::RSA_PKCS1_SHA512,
            SignatureScheme::ECDSA_NISTP521_SHA512,
            SignatureScheme::RSA_PSS_SHA256,
            SignatureScheme::RSA_PSS_SHA384,
            SignatureScheme::RSA_PSS_SHA512,
            SignatureScheme::ED25519,
            SignatureScheme::ED448,
        ]
    }
}

pub fn create_tls_configuration(
    insecure: bool,
    client_cert: Option<&Path>,
    client_private_key: Option<&Path>,
    ca_cert: Option<&Path>,
) -> anyhow::Result<TlsConfiguration> {
    let mut roots = rustls::RootCertStore::empty();
    let native_certs = rustls_native_certs::load_native_certs();
    for error in native_certs.errors {
        eprintln!(
            "Warning: might skip some native certificates because of an error while loading: {error}"
        );
    }
    roots.add_parsable_certificates(native_certs.certs);

    if let Some(path) = ca_cert {
        let certificates = read_certificate_file(path).context("while reading ca-cert")?;
        anyhow::ensure!(!certificates.is_empty(), "no certificates in ca-cert");
        for certificate in certificates {
            roots
                .add(certificate)
                .context("while adding ca-cert to cert store")?;
        }
    }

    let conf = ClientConfig::builder().with_root_certificates(roots);

    let mut conf = match (client_cert, client_private_key) {
        (Some(client_cert), Some(client_private_key)) => conf
            .with_client_auth_cert(
                read_certificate_file(client_cert).context("while reading client-cert")?,
                read_private_key_file(client_private_key)
                    .context("while reading client-private-key")?,
            )
            .context("while setting client auth cert")?,
        (None, None) => conf.with_no_client_auth(),
        _ => unreachable!("requires both cert and key which should be ensured by clap"),
    };
    conf.key_log = Arc::new(KeyLogFile::new());

    if insecure {
        let mut danger = conf.dangerous();
        danger.set_certificate_verifier(Arc::new(NoVerifier {}));
    }

    Ok(TlsConfiguration::Rustls(Arc::new(conf)))
}

fn read_certificate_file(file: &Path) -> anyhow::Result<Vec<CertificateDer<'static>>> {
    let file = File::open(file)?;
    let mut file = BufReader::new(file);
    let certs = rustls_pemfile::certs(&mut file);
    let mut result = Vec::new();
    for cert in certs {
        result.push(cert?);
    }
    Ok(result)
}

fn read_private_key_file(path: &Path) -> anyhow::Result<PrivateKeyDer<'static>> {
    let file = File::open(path)?;
    let mut file = BufReader::new(file);
    loop {
        match rustls_pemfile::read_one(&mut file)? {
            Some(rustls_pemfile::Item::Pkcs1Key(key)) => return Ok(key.into()),
            Some(rustls_pemfile::Item::Pkcs8Key(key)) => return Ok(key.into()),
            Some(rustls_pemfile::Item::Sec1Key(key)) => return Ok(key.into()),
            None => break,
            _ => {}
        }
    }
    Err(anyhow::anyhow!(
        "no keys found in {} (encrypted keys not supported)",
        path.display()
    ))
}

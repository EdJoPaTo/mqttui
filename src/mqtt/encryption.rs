use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;

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
    client_certificate_path: Option<&Path>,
    client_private_key_path: Option<&Path>,
) -> anyhow::Result<TlsConfiguration> {
    let mut roots = rustls::RootCertStore::empty();
    let native_certs = rustls_native_certs::load_native_certs();
    for error in native_certs.errors {
        eprintln!(
            "Warning: might skip some native certificates because of an error while loading: {error}"
        );
    }
    for cert in native_certs.certs {
        _ = roots.add(cert);
    }

    let conf = ClientConfig::builder().with_root_certificates(roots);

    let mut conf = match (client_certificate_path, client_private_key_path) {
        (Some(certificate_path), Some(private_key_path)) => conf.with_client_auth_cert(
            read_certificate_file(certificate_path)?,
            read_private_key_file(private_key_path)?,
        )?,
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

fn read_private_key_file(file: &Path) -> anyhow::Result<PrivateKeyDer<'static>> {
    let file = File::open(file)?;
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
        "no keys found in {file:?} (encrypted keys not supported)"
    ))
}

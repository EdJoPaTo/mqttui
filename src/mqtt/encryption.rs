use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use rumqttc::TlsConfiguration;
use rustls::client::{ServerCertVerified, ServerCertVerifier, WantsTransparencyPolicyOrClientCert};
use rustls::{Certificate, ClientConfig, ConfigBuilder, PrivateKey};

struct NoVerifier;
impl ServerCertVerifier for NoVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: SystemTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }
}

pub fn create_tls_configuration(
    insecure: bool,
    client_certificate_path: &Option<PathBuf>,
    client_private_key_path: &Option<PathBuf>,
) -> anyhow::Result<TlsConfiguration> {
    let mut roots = rustls::RootCertStore::empty();
    let certs = rustls_native_certs::load_native_certs()?;
    for cert in certs {
        let _ = roots.add(&rustls::Certificate(cert.0));
    }

    let conf = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(roots);
    let mut conf = configure_client_auth(conf, client_certificate_path, client_private_key_path)?;

    if insecure {
        let mut danger = conf.dangerous();
        danger.set_certificate_verifier(Arc::new(NoVerifier {}));
    }

    Ok(TlsConfiguration::Rustls(Arc::new(conf)))
}

fn configure_client_auth(
    conf: ConfigBuilder<ClientConfig, WantsTransparencyPolicyOrClientCert>,
    certificate_path: &Option<PathBuf>,
    private_key_path: &Option<PathBuf>,
) -> anyhow::Result<ClientConfig> {
    if let (Some(certificate_path), Some(private_key_path)) = (certificate_path, private_key_path) {
        Ok(conf.with_single_cert(
            read_certificate_file(certificate_path)?,
            read_private_key_file(private_key_path)?,
        )?)
    } else {
        Ok(conf.with_no_client_auth())
    }
}

fn read_certificate_file(file: &Path) -> anyhow::Result<Vec<Certificate>> {
    let file = File::open(file)?;
    let mut file = BufReader::new(file);
    let certs = rustls_pemfile::certs(&mut file)?;
    Ok(certs.into_iter().map(Certificate).collect())
}

fn read_private_key_file(file: &Path) -> anyhow::Result<PrivateKey> {
    let file = File::open(file)?;
    let mut file = BufReader::new(file);
    let keys = rustls_pemfile::pkcs8_private_keys(&mut file)?;
    if let [key] = keys.as_slice() {
        Ok(PrivateKey(key.clone()))
    } else {
        anyhow::bail!("Private key file must contain exactly one key");
    }
}

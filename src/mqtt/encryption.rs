use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use rumqttc::TlsConfiguration;
use rustls::client::{ServerCertVerified, ServerCertVerifier, WantsTransparencyPolicyOrClientCert};
use rustls::{Certificate, ClientConfig, ConfigBuilder, PrivateKey};
use rustls_pemfile::Item;

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
    client_cert_path: &Option<PathBuf>,
    client_key_path: &Option<PathBuf>,
) -> anyhow::Result<TlsConfiguration> {
    let certs = rustls_native_certs::load_native_certs().unwrap();
    let mut roots = rustls::RootCertStore::empty();
    for cert in certs {
        let _ = roots.add(&rustls::Certificate(cert.0));
    }
    let conf = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(roots);

    let mut conf = configure_client_cert_auth(conf, client_cert_path, client_key_path)?;

    if insecure {
        let mut danger = conf.dangerous();
        danger.set_certificate_verifier(Arc::new(NoVerifier {}));
    }

    Ok(TlsConfiguration::Rustls(Arc::new(conf)))
}

pub fn configure_client_cert_auth(
    conf: ConfigBuilder<ClientConfig, WantsTransparencyPolicyOrClientCert>,
    client_cert_path: &Option<PathBuf>,
    client_key_path: &Option<PathBuf>,
) -> anyhow::Result<ClientConfig> {
    if let (Some(client_cert_path), Some(priv_key_path)) = (client_cert_path, client_key_path) {
        let client_cert = read_cert(client_cert_path)?;
        let private_key = read_priv(priv_key_path)?;

        Ok(conf.with_single_cert(client_cert, private_key)?)
    } else {
        Ok(conf.with_no_client_auth())
    }
}

pub fn read_cert(filename: &Path) -> anyhow::Result<Vec<Certificate>> {
    println!("cert file is:");
    debug_pemfile(filename).ok();
    let f = File::open(filename)?;
    let mut f = BufReader::new(f);

    let certs = rustls_pemfile::certs(&mut f)?;

    Ok(certs.into_iter().map(Certificate).collect())
}

pub fn read_priv(filename: &Path) -> anyhow::Result<PrivateKey> {
    println!("priv key file is:");
    debug_pemfile(filename).ok();
    let f = File::open(filename)?;
    let mut f = BufReader::new(f);

    let keys = rustls_pemfile::pkcs8_private_keys(&mut f)?;
    let vec = keys[0].clone();

    let privkey = PrivateKey(vec);

    Ok(privkey)
}

pub fn debug_pemfile(filename: &Path) -> anyhow::Result<()> {
    let f = File::open(filename)?;
    let mut f = BufReader::new(f);

    let keys = rustls_pemfile::read_all(&mut f)?;

    for item in keys.iter() {
        println!("{:?}", item);
        match item {
            Item::X509Certificate(cert) => println!("certificate {:?}", cert),
            Item::RSAKey(key) => println!("rsa pkcs1 key {:?}", key),
            Item::PKCS8Key(key) => println!("pkcs8 key {:?}", key),
            Item::ECKey(key) => println!("sec1 ec key {:?}", key),
            _ => println!("invalid item!"),
        }
    }

    Ok(())
}

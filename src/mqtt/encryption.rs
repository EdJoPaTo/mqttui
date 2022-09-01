use std::sync::Arc;
use std::time::SystemTime;

use rumqttc::TlsConfiguration;
use rustls::client::{ServerCertVerified, ServerCertVerifier};
use rustls::ClientConfig;

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

pub fn create_tls_configuration(insecure: bool) -> TlsConfiguration {
    let certs = rustls_native_certs::load_native_certs().unwrap();
    let mut roots = rustls::RootCertStore::empty();
    for cert in certs {
        let _ = roots.add(&rustls::Certificate(cert.0));
    }
    let mut conf = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(roots)
        .with_no_client_auth();

    if insecure {
        let mut danger = conf.dangerous();
        danger.set_certificate_verifier(Arc::new(NoVerifier {}));
    }

    TlsConfiguration::Rustls(Arc::new(conf))
}

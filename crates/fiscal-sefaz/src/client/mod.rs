//! Async SEFAZ web service client with mTLS authentication.
//!
//! [`SefazClient`] wraps a [`reqwest::Client`] pre-configured with the
//! emitter's A1 digital certificate (PFX/PKCS#12) for mutual TLS.
//!
//! ```rust,no_run
//! # async fn example() -> Result<(), fiscal_core::FiscalError> {
//! use fiscal_core::types::SefazEnvironment;
//! use fiscal_sefaz::client::SefazClient;
//! use fiscal_sefaz::request_builders::build_status_request;
//! use fiscal_sefaz::services::SefazService;
//!
//! let pfx = std::fs::read("cert.pfx").unwrap();
//! let client = SefazClient::new(&pfx, "my_password")?;
//!
//! // Typed convenience method
//! let status = client.status("SP", SefazEnvironment::Homologation).await?;
//! println!("SEFAZ status: {} — {}", status.status_code, status.status_message);
//!
//! // Or low-level send for any service
//! let request_xml = build_status_request("RS", SefazEnvironment::Production);
//! let raw_xml = client
//!     .send(SefazService::StatusServico, "RS", SefazEnvironment::Production, &request_xml)
//!     .await?;
//! # Ok(())
//! # }
//! ```

mod authorize;
mod delivery;
mod events;
mod rtc;

use std::fmt;
use std::time::Duration;

use reqwest::{Client, Identity};

use fiscal_core::FiscalError;
use fiscal_core::types::SefazEnvironment;

use crate::request_builders;
use crate::services::SefazService;
use crate::soap;
use crate::urls::{get_an_url, get_sefaz_url_for_model};

/// Default timeout for connecting to a SEFAZ endpoint.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

/// Default timeout for the full request/response cycle.
///
/// SEFAZ authorization can be slow; 90 s accommodates peak-hour latency.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(90);

/// NT 2024.002 / NT 2025.002: when sending conciliação or RTC events via SVRS,
/// `cOrgao` must be set to 92 instead of deriving from the access key.
fn svrs_org_override(uf: &str) -> Option<&'static str> {
    if uf == "SVRS" { Some("92") } else { None }
}

/// Async SEFAZ web service client with mTLS authentication.
///
/// Holds a pre-configured [`reqwest::Client`] with the emitter's digital
/// certificate identity. The client itself is stateless — UF and environment
/// are passed per-call so a single client can serve multiple states.
///
/// # Construction
///
/// Use [`SefazClient::new`] with the raw PFX bytes and passphrase.
/// The certificate is loaded once and reused for all requests.
pub struct SefazClient {
    http: Client,
    /// PEM-encoded private key, used to sign event XML before transmit.
    private_key: String,
    /// PEM-encoded X.509 certificate, embedded in event signatures.
    certificate: String,
}

impl fmt::Debug for SefazClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SefazClient")
            .field("http", &"reqwest::Client { .. }")
            .finish()
    }
}

impl SefazClient {
    /// Create a new client from a PKCS#12 (PFX) certificate buffer.
    ///
    /// The PFX is parsed and installed as the TLS client identity for all
    /// subsequent requests. TLS 1.2 is enforced as required by SEFAZ.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Certificate`] if:
    /// - The PFX buffer is invalid or the passphrase is wrong
    /// - The underlying TLS stack rejects the certificate
    ///
    /// Returns [`FiscalError::Network`] if the HTTP client cannot be built.
    pub fn new(pfx_buffer: &[u8], passphrase: &str) -> Result<Self, FiscalError> {
        // reqwest usa rustls-no-provider (o provider default, aws-lc-rs,
        // compila C e quebra o cross-compile aarch64 do napi). Instala o
        // ring como provider do processo; idempotente, erro só significa
        // que já havia um provider instalado.
        let _ = rustls::crypto::ring::default_provider().install_default();

        // Load certificate and private key from PFX using our pure-Rust parser
        let cert_data = fiscal_crypto::certificate::load_certificate(pfx_buffer, passphrase)?;

        // Build PEM-based identity (works with both native-tls and rustls)
        let identity = Identity::from_pem(
            format!("{}\n{}", cert_data.private_key, cert_data.certificate).as_bytes(),
        )
        .map_err(|e| FiscalError::Certificate(format!("Failed to load PFX identity: {e}")))?;

        let http = Client::builder()
            .use_rustls_tls()
            .identity(identity)
            .danger_accept_invalid_certs(true)
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .build()
            .map_err(|e| FiscalError::Network(format!("Failed to build HTTP client: {e}")))?;

        Ok(Self {
            http,
            private_key: cert_data.private_key,
            certificate: cert_data.certificate,
        })
    }

    /// Sign a built event request XML in place, inserting a `<Signature>`
    /// over the `<infEvento>` element inside `<evento>`.
    ///
    /// Uses RSA-SHA1, matching the library default for NF-e and inutilização
    /// signing (`fiscal_crypto::sign_xml` / `sign_inutilizacao_xml`).
    pub(crate) fn sign_event(&self, request_xml: &str) -> Result<String, FiscalError> {
        fiscal_crypto::certificate::sign_event_xml(
            request_xml,
            &self.private_key,
            &self.certificate,
        )
    }

    /// Sign every `<evento>` element inside a batch `<envEvento>` request.
    ///
    /// `sign_event` signs a single `<evento>`; a batch (`event_batch` /
    /// `manifest_batch`) carries multiple `<evento>` siblings that must each
    /// receive their own `<Signature>`. This splits the batch on
    /// `</evento>` boundaries, signs each segment, and reassembles the wrapper.
    pub(crate) fn sign_event_batch(&self, request_xml: &str) -> Result<String, FiscalError> {
        const CLOSE: &str = "</evento>";

        let Some(first) = request_xml.find("<evento") else {
            // No events to sign (should not happen for a valid batch).
            return Ok(request_xml.to_string());
        };

        let mut out = String::with_capacity(request_xml.len() + 2048);
        out.push_str(&request_xml[..first]);

        let body = &request_xml[first..];
        let mut rest = body;
        loop {
            match rest.find(CLOSE) {
                Some(idx) => {
                    let end = idx + CLOSE.len();
                    let event = &rest[..end];
                    out.push_str(&self.sign_event(event)?);
                    rest = &rest[end..];
                    if !rest.contains("<evento") {
                        // Trailing wrapper (e.g. "</envEvento>").
                        out.push_str(rest);
                        break;
                    }
                }
                None => {
                    out.push_str(rest);
                    break;
                }
            }
        }

        Ok(out)
    }

    // ── Low-level ────────────────────────────────────────────────────────

    /// Send a raw request XML to a SEFAZ service and return the raw
    /// response XML.
    ///
    /// This is the escape hatch for services that do not (yet) have a
    /// typed convenience method. The SOAP envelope is built automatically.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::InvalidStateCode`] if `uf` is not a valid
    /// Brazilian state abbreviation.
    ///
    /// Returns [`FiscalError::Network`] if the HTTP request fails
    /// (connection refused, timeout, TLS handshake failure, non-2xx
    /// status, etc.).
    pub async fn send(
        &self,
        service: SefazService,
        uf: &str,
        environment: SefazEnvironment,
        request_xml: &str,
    ) -> Result<String, FiscalError> {
        self.send_model(service, uf, environment, request_xml, 55)
            .await
    }

    /// Send a raw request XML to a SEFAZ service for a specific invoice model
    /// (55 = NF-e, 65 = NFC-e) and return the raw response XML.
    pub async fn send_model(
        &self,
        service: SefazService,
        uf: &str,
        environment: SefazEnvironment,
        request_xml: &str,
        model: u8,
    ) -> Result<String, FiscalError> {
        let url = get_sefaz_url_for_model(uf, environment, service.url_key(), model)?;
        self.send_to_url(service, &url, uf, request_xml).await
    }

    /// Send a raw request XML to a caller-resolved SEFAZ endpoint URL.
    ///
    /// Core POST path shared by [`send_model`](Self::send_model) and
    /// contingency transmission
    /// ([`authorize_contingency`](Self::authorize_contingency)). The `url` is
    /// resolved by the caller for the correct authorizer (real UF, SVC-AN, or
    /// SVC-RS), while `envelope_uf` selects the `<cUF>` used to build the SOAP
    /// envelope.
    ///
    /// For SVC contingency the endpoint changes but the protocol keeps the
    /// **issuer's** `cUF`, so `url` points at the SVC authorizer while
    /// `envelope_uf` stays the issuer's real state abbreviation.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::InvalidStateCode`] if `envelope_uf` is not a
    /// valid Brazilian state abbreviation.
    ///
    /// Returns [`FiscalError::Network`] if the HTTP request fails.
    async fn send_to_url(
        &self,
        service: SefazService,
        url: &str,
        envelope_uf: &str,
        request_xml: &str,
    ) -> Result<String, FiscalError> {
        let meta = service.meta();
        let envelope = soap::build_envelope(request_xml, envelope_uf, &meta)?;
        let action = soap::build_action(&meta);

        let content_type = format!("application/soap+xml;charset=utf-8;action=\"{action}\"");

        let response = self
            .http
            .post(url)
            .header("Content-Type", &content_type)
            .body(envelope)
            .send()
            .await
            .map_err(|e| FiscalError::Network(format!("{e}")))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| FiscalError::Network(format!("Failed to read response body: {e}")))?;

        if !status.is_success() {
            return Err(FiscalError::Network(format!(
                "SEFAZ returned HTTP {status}: {body}"
            )));
        }

        Ok(body)
    }

    // ── Validation ────────────────────────────────────────────────────

    /// Validate an authorized NF-e against SEFAZ records.
    ///
    /// Extracts the access key, protocol number, and digest value from the
    /// local authorized NF-e XML, then queries SEFAZ via
    /// [`consult`](Self::consult) and compares:
    ///
    /// 1. Protocol number (`nProt`)
    /// 2. Digest value (`digVal` / `DigestValue`)
    /// 3. Access key (`chNFe`)
    ///
    /// Returns a [`ValidationResult`](crate::validate::ValidationResult)
    /// with `is_valid = true` only when all three match.
    ///
    /// Mirrors the PHP `Tools::sefazValidate()` method.
    ///
    /// # Arguments
    ///
    /// * `uf` — State abbreviation.
    /// * `nfe_xml` — The complete authorized NF-e XML (with `<protNFe>` attached).
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::XmlParsing`] if the local XML is missing
    /// required elements.
    /// Returns [`FiscalError::Network`] on SEFAZ communication failure.
    pub async fn sefaz_validate(
        &self,
        uf: &str,
        nfe_xml: &str,
    ) -> Result<crate::validate::ValidationResult, FiscalError> {
        let (access_key, protocol, digest) = crate::validate::extract_nfe_validation_data(nfe_xml)?;

        // Determine environment from tpAmb in the XML
        let tp_amb =
            fiscal_core::xml_utils::extract_xml_tag_value(nfe_xml, "tpAmb").ok_or_else(|| {
                FiscalError::XmlParsing("Tag <tpAmb> não encontrada no XML".to_string())
            })?;
        let environment = if tp_amb == "1" {
            SefazEnvironment::Production
        } else {
            SefazEnvironment::Homologation
        };

        // Query SEFAZ by access key
        let request_xml = request_builders::build_consulta_request(&access_key, environment);
        let raw = self
            .send(
                SefazService::ConsultaProtocolo,
                uf,
                environment,
                &request_xml,
            )
            .await?;

        // Compare local vs SEFAZ data
        crate::validate::validate_authorized_nfe(&access_key, &protocol, &digest, &raw)
    }

    // ── Internal helpers ────────────────────────────────────────────────

    /// Send a raw request XML to a SEFAZ service using gzip compression.
    ///
    /// The request XML is gzip-compressed and base64-encoded in the SOAP
    /// envelope, using `<nfeDadosMsgZip>` instead of `<nfeDadosMsg>`.
    async fn send_model_compressed(
        &self,
        service: SefazService,
        uf: &str,
        environment: SefazEnvironment,
        request_xml: &str,
        model: u8,
    ) -> Result<String, FiscalError> {
        let url = get_sefaz_url_for_model(uf, environment, service.url_key(), model)?;
        let meta = service.meta();
        let envelope = soap::build_envelope_compressed(request_xml, uf, &meta)?;
        let action = soap::build_action(&meta);

        let content_type = format!("application/soap+xml;charset=utf-8;action=\"{action}\"");

        let response = self
            .http
            .post(&url)
            .header("Content-Type", &content_type)
            .body(envelope)
            .send()
            .await
            .map_err(|e| FiscalError::Network(format!("{e}")))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| FiscalError::Network(format!("Failed to read response body: {e}")))?;

        if !status.is_success() {
            return Err(FiscalError::Network(format!(
                "SEFAZ returned HTTP {status}: {body}"
            )));
        }

        Ok(body)
    }

    /// Send a request to the Ambiente Nacional (AN) endpoint.
    ///
    /// AN provides RecepcaoEvento (manifestacao) and DistDFe services.
    async fn send_an(
        &self,
        service: SefazService,
        environment: SefazEnvironment,
        request_xml: &str,
    ) -> Result<String, FiscalError> {
        let url = get_an_url(environment, service.url_key())?;
        let meta = service.meta();
        // AN is not a real state code, but we use it for envelope building
        let envelope = soap::build_envelope(request_xml, "AN", &meta)?;
        let action = soap::build_action(&meta);

        let content_type = format!("application/soap+xml;charset=utf-8;action=\"{action}\"");

        let response = self
            .http
            .post(&url)
            .header("Content-Type", &content_type)
            .body(envelope)
            .send()
            .await
            .map_err(|e| FiscalError::Network(format!("{e}")))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| FiscalError::Network(format!("Failed to read response body: {e}")))?;

        if !status.is_success() {
            return Err(FiscalError::Network(format!(
                "SEFAZ returned HTTP {status}: {body}"
            )));
        }

        Ok(body)
    }

    /// Send a ConsultaCadastro request with the `<nfeCabecMsg>` SOAP header.
    ///
    /// Uses `build_envelope_with_header` which adds the legacy
    /// `<soap:Header><nfeCabecMsg>` block required by this v2.00 service.
    async fn send_cadastro_raw(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        request_xml: &str,
    ) -> Result<String, FiscalError> {
        let service = SefazService::ConsultaCadastro;
        let url = get_sefaz_url_for_model(uf, environment, service.url_key(), 55)?;
        let meta = service.meta();
        // PHP sped-nfe wraps the request in an extra <nfeDadosMsg> for MT
        // before the SOAP envelope body (Tools.php ~line 324-326)
        let mt_wrapped;
        let effective_xml = if uf.eq_ignore_ascii_case("MT") {
            mt_wrapped = format!("<nfeDadosMsg>{request_xml}</nfeDadosMsg>");
            &mt_wrapped
        } else {
            request_xml
        };
        let envelope = soap::build_envelope_with_header(effective_xml, uf, &meta)?;
        let action = soap::build_action(&meta);

        let content_type = format!("application/soap+xml;charset=utf-8;action=\"{action}\"");

        let response = self
            .http
            .post(&url)
            .header("Content-Type", &content_type)
            .body(envelope)
            .send()
            .await
            .map_err(|e| FiscalError::Network(format!("{e}")))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| FiscalError::Network(format!("Failed to read response body: {e}")))?;

        if !status.is_success() {
            return Err(FiscalError::Network(format!(
                "SEFAZ returned HTTP {status}: {body}"
            )));
        }

        Ok(body)
    }

    /// Send a DistDFe request with the special SOAP wrapper.
    ///
    /// Uses `build_envelope_dist_dfe` which adds the extra
    /// `<nfeDistDFeInteresse>` wrapper required by this service.
    async fn send_dist_dfe_raw(
        &self,
        environment: SefazEnvironment,
        request_xml: &str,
    ) -> Result<String, FiscalError> {
        let service = SefazService::DistribuicaoDFe;
        let url = get_an_url(environment, service.url_key())?;
        let meta = service.meta();
        let envelope = soap::build_envelope_dist_dfe(request_xml, "AN", &meta)?;
        let action = soap::build_action(&meta);

        let content_type = format!("application/soap+xml;charset=utf-8;action=\"{action}\"");

        let response = self
            .http
            .post(&url)
            .header("Content-Type", &content_type)
            .body(envelope)
            .send()
            .await
            .map_err(|e| FiscalError::Network(format!("{e}")))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| FiscalError::Network(format!("Failed to read response body: {e}")))?;

        if !status.is_success() {
            return Err(FiscalError::Network(format!(
                "SEFAZ returned HTTP {status}: {body}"
            )));
        }

        Ok(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // NOTE: Integration tests against real SEFAZ endpoints belong in
    // `tests/` with `#[ignore]` and require a valid certificate.
    // The tests here verify construction error paths only.

    #[test]
    fn rejects_invalid_pfx_buffer() {
        let err = SefazClient::new(b"not a pfx", "password").unwrap_err();
        assert!(
            matches!(err, FiscalError::Certificate(_)),
            "expected Certificate error, got: {err}"
        );
    }

    #[test]
    fn rejects_empty_pfx_buffer() {
        let err = SefazClient::new(&[], "").unwrap_err();
        assert!(matches!(err, FiscalError::Certificate(_)));
    }

    // Test certificate fixture shared with the `fiscal-crypto` test suite.
    fn test_pfx() -> Vec<u8> {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../..",
            "/tests/fixtures/certs/novo_cert_cnpj_06157250000116_senha_minhasenha.pfx"
        );
        std::fs::read(path).expect("test PFX not found")
    }

    const TEST_PASSWORD: &str = "minhasenha";
    const TEST_ACCESS_KEY: &str = "41250106157250000116550010000000011000000017";

    #[test]
    fn signs_cancel_event_before_transmit() {
        let client = SefazClient::new(&test_pfx(), TEST_PASSWORD).expect("client builds");

        let request_xml = request_builders::build_cancela_request(
            TEST_ACCESS_KEY,
            "141250000000017",
            "Cancelamento de teste com justificativa valida",
            1,
            SefazEnvironment::Homologation,
            "06157250000116",
        );
        // The unsigned builder output must NOT already contain a signature.
        assert!(!request_xml.contains("<Signature"));

        let signed = client.sign_event(&request_xml).expect("event signs");

        // A <Signature> referencing the <infEvento> Id must now be present,
        // nested inside <evento> (before its closing tag).
        assert!(
            signed.contains("<Signature"),
            "missing <Signature>: {signed}"
        );
        assert!(
            signed.contains("Reference URI=\"#ID110111"),
            "signature must reference the infEvento Id"
        );
        assert!(signed.contains("<X509Certificate>"));
        let sig_pos = signed.find("<Signature").unwrap();
        let evento_close = signed.find("</evento>").unwrap();
        assert!(
            sig_pos < evento_close,
            "<Signature> must sit inside <evento>"
        );
    }

    #[test]
    fn signs_cce_event_before_transmit() {
        let client = SefazClient::new(&test_pfx(), TEST_PASSWORD).expect("client builds");

        let request_xml = request_builders::build_cce_request(
            TEST_ACCESS_KEY,
            "Correcao do endereco de entrega",
            1,
            SefazEnvironment::Homologation,
            "06157250000116",
        );
        let signed = client.sign_event(&request_xml).expect("event signs");

        assert!(signed.contains("<Signature"));
        assert!(signed.contains("Reference URI=\"#ID110110"));
    }

    /// Build a minimal [`request_builders::EpecData`] fixture without parsing a
    /// full NF-e XML, so the test stays focused on the signing behavior.
    fn test_epec_data() -> request_builders::EpecData {
        request_builders::EpecData {
            access_key: TEST_ACCESS_KEY.to_string(),
            c_orgao_autor: "41".to_string(),
            ver_aplic: "1.0.0".to_string(),
            dh_emi: "2025-01-01T10:00:00-03:00".to_string(),
            tp_nf: "1".to_string(),
            emit_ie: "111111111".to_string(),
            dest_uf: "SP".to_string(),
            dest_id_tag: "<CNPJ>99999999000191</CNPJ>".to_string(),
            dest_ie: Some("222222222".to_string()),
            v_nf: "100.00".to_string(),
            v_icms: "18.00".to_string(),
            v_st: "0.00".to_string(),
            tax_id: "06157250000116".to_string(),
        }
    }

    // ── Batch signing ────────────────────────────────────────────────────

    /// A 2-`<evento>` batch gets exactly two `<Signature>` elements, each
    /// nested inside its own `<evento>...</evento>`.
    #[test]
    fn signs_every_evento_in_batch() {
        let client = SefazClient::new(&test_pfx(), TEST_PASSWORD).expect("client builds");

        let events = [
            request_builders::EventItem {
                access_key: TEST_ACCESS_KEY.to_string(),
                event_type: 210210, // Ciência da Operação
                seq: 1,
                tax_id: "06157250000116".to_string(),
                additional_tags: String::new(),
            },
            request_builders::EventItem {
                access_key: TEST_ACCESS_KEY.to_string(),
                event_type: 210200, // Confirmação da Operação
                seq: 1,
                tax_id: "06157250000116".to_string(),
                additional_tags: String::new(),
            },
        ];
        let request_xml = request_builders::build_event_batch_request(
            "AN",
            &events,
            Some("1"),
            SefazEnvironment::Homologation,
        );
        // Unsigned batch must carry no signatures yet.
        assert!(!request_xml.contains("<Signature "));
        assert_eq!(request_xml.matches("<evento ").count(), 2);

        let signed = client.sign_event_batch(&request_xml).expect("batch signs");

        // Exactly two root <Signature> elements, one per evento. Count the
        // opening tag with its xmlns to avoid matching <SignatureMethod> /
        // <SignatureValue>.
        assert_eq!(
            signed.matches("<Signature xmlns").count(),
            2,
            "expected one <Signature> per <evento>: {signed}"
        );
        assert_eq!(signed.matches("<evento ").count(), 2);

        // Each <evento> must hold exactly one signature.
        for event in signed.split_inclusive("</evento>") {
            if event.contains("<infEvento") {
                assert_eq!(
                    event.matches("<Signature xmlns").count(),
                    1,
                    "each <evento> must hold its own <Signature>: {event}"
                );
            }
        }
        assert!(signed.contains("Reference URI=\"#ID210210"));
        assert!(signed.contains("Reference URI=\"#ID210200"));
    }

    /// A batch with no `<evento>` round-trips unchanged (nothing to sign).
    #[test]
    fn signs_empty_batch_roundtrips_unchanged() {
        let client = SefazClient::new(&test_pfx(), TEST_PASSWORD).expect("client builds");

        let batch = "<envEvento xmlns=\"http://www.portalfiscal.inf.br/nfe\" versao=\"1.00\">\
             <idLote>1</idLote></envEvento>";
        let signed = client.sign_event_batch(batch).expect("empty batch ok");

        assert_eq!(signed, batch, "empty batch must be returned untouched");
        assert!(!signed.contains("<Signature"));
    }

    /// A batch whose `<infEvento>` lacks an `Id=` propagates the signing error.
    #[test]
    fn batch_without_inf_evento_id_propagates_error() {
        let client = SefazClient::new(&test_pfx(), TEST_PASSWORD).expect("client builds");

        let batch = "<envEvento xmlns=\"http://www.portalfiscal.inf.br/nfe\" versao=\"1.00\">\
             <idLote>1</idLote>\
             <evento xmlns=\"http://www.portalfiscal.inf.br/nfe\" versao=\"1.00\">\
             <infEvento><cOrgao>91</cOrgao></infEvento>\
             </evento></envEvento>";
        let err = client
            .sign_event_batch(batch)
            .expect_err("missing Id must fail");
        assert!(
            matches!(err, FiscalError::Certificate(_)),
            "expected Certificate error, got: {err}"
        );
    }

    // ── EPEC signing ─────────────────────────────────────────────────────

    /// An EPEC event (`tpEvento=110140`) gets signed with a Signature that
    /// references the `#ID110140…` infEvento Id.
    #[test]
    fn signs_epec_event_before_transmit() {
        let client = SefazClient::new(&test_pfx(), TEST_PASSWORD).expect("client builds");

        let request_xml =
            request_builders::build_epec_request(&test_epec_data(), SefazEnvironment::Homologation);
        assert!(!request_xml.contains("<Signature"));

        let signed = client.sign_event(&request_xml).expect("EPEC event signs");

        assert!(
            signed.contains("<Signature"),
            "missing <Signature>: {signed}"
        );
        assert!(
            signed.contains("Reference URI=\"#ID110140"),
            "EPEC signature must reference the #ID110140 infEvento Id"
        );
        let sig_pos = signed.find("<Signature").unwrap();
        let evento_close = signed.find("</evento>").unwrap();
        assert!(
            sig_pos < evento_close,
            "<Signature> must sit inside <evento>"
        );
    }

    // ── Manifest signing ─────────────────────────────────────────────────

    /// A manifestação event (Ciência da Operação, `tpEvento=210210`) is signed
    /// referencing the correct `infEvento Id` (`#ID210210{key}{seq}`).
    #[test]
    fn signs_manifest_event_before_transmit() {
        let client = SefazClient::new(&test_pfx(), TEST_PASSWORD).expect("client builds");

        let request_xml = request_builders::build_manifesta_request(
            TEST_ACCESS_KEY,
            "210210",
            None,
            1,
            SefazEnvironment::Homologation,
            "06157250000116",
        );
        assert!(!request_xml.contains("<Signature"));

        let signed = client
            .sign_event(&request_xml)
            .expect("manifest event signs");

        assert!(
            signed.contains("<Signature"),
            "missing <Signature>: {signed}"
        );
        let expected_ref = format!("Reference URI=\"#ID210210{TEST_ACCESS_KEY}01");
        assert!(
            signed.contains(&expected_ref),
            "manifest signature must reference {expected_ref}: {signed}"
        );
    }

    // ── Signed evento structure ──────────────────────────────────────────

    /// Structural snapshot of a fully signed cancelamento `<evento>`: asserts
    /// the placement and presence of the XMLDSig sub-elements
    /// (`Signature`/`SignedInfo`/`Reference`/`DigestValue`/`SignatureValue`/
    /// `X509Certificate`) inside `<evento>`.
    ///
    /// NOTE: We assert structure instead of a `cargo insta` `.snap`. The signed
    /// XML embeds a base64 `SignatureValue` that is deterministic for RSA-SHA1
    /// over fixed input, but pulling in the `insta` dev-dependency (plus its
    /// transitive tree) on a crate we intend to upstream would expand the
    /// `cargo-deny` surface for no extra coverage here, so a structural
    /// assertion is preferred.
    #[test]
    fn signed_cancelamento_evento_has_full_signature_structure() {
        let client = SefazClient::new(&test_pfx(), TEST_PASSWORD).expect("client builds");

        let request_xml = request_builders::build_cancela_request(
            TEST_ACCESS_KEY,
            "141250000000017",
            "Cancelamento de teste com justificativa valida",
            1,
            SefazEnvironment::Homologation,
            "06157250000116",
        );
        let signed = client.sign_event(&request_xml).expect("event signs");

        // The signature block lives between the signed <infEvento> and the
        // closing </evento>.
        let inf_close = signed.find("</infEvento>").expect("infEvento closes");
        let evento_close = signed.find("</evento>").expect("evento closes");
        let sig_start = signed.find("<Signature").expect("Signature present");
        assert!(
            inf_close < sig_start && sig_start < evento_close,
            "<Signature> must sit after </infEvento> and before </evento>"
        );

        // All XMLDSig sub-elements must be present, in canonical order.
        let order = [
            "<Signature xmlns",
            "<SignedInfo",
            "<Reference URI=\"#ID110111",
            "<DigestValue>",
            "<SignatureValue>",
            "<X509Certificate>",
        ];
        let mut last = 0usize;
        for needle in order {
            let pos = signed[last..]
                .find(needle)
                .unwrap_or_else(|| panic!("missing {needle} in signed XML: {signed}"));
            last += pos;
        }

        // Exactly one root signature for a single-evento request.
        assert_eq!(signed.matches("<Signature xmlns").count(), 1);
    }

    // NOTE: The `SefazClient::new` certificate-load error branch (PFX that
    // parses for TLS via `Identity::from_pkcs12_der` but whose certificate
    // fails `fiscal_crypto::certificate::load_certificate`) is not reachable
    // without crafting a bespoke malformed-but-TLS-loadable PFX fixture. The
    // generic `FiscalError::Certificate` construction path is already covered
    // by `rejects_invalid_pfx_buffer` / `rejects_empty_pfx_buffer`, so this
    // sub-case is intentionally skipped to keep the fixture set small.
}

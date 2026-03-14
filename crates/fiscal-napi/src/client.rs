use napi::bindgen_prelude::Buffer;
use napi_derive::napi;

use fiscal_core::types::SefazEnvironment;

// ── SefazClient class ───────────────────────────────────────────────────────

#[napi]
pub struct SefazClient {
    inner: fiscal_sefaz::client::SefazClient,
}

#[napi]
impl SefazClient {
    /// Create a new SEFAZ client from a PKCS#12 (PFX) certificate buffer.
    #[napi(constructor)]
    pub fn new(pfx_buffer: Buffer, passphrase: String) -> napi::Result<Self> {
        let inner =
            fiscal_sefaz::client::SefazClient::new(&pfx_buffer, &passphrase).map_err(to_napi)?;
        Ok(Self { inner })
    }

    /// Check SEFAZ operational status.
    #[napi(
        ts_return_type = "Promise<{ statusCode: string; statusMessage: string; averageTime: string | null }>"
    )]
    pub async fn status(&self, uf: String, environment: String) -> napi::Result<serde_json::Value> {
        let env = parse_env(&environment)?;
        let resp = self.inner.status(&uf, env).await.map_err(to_napi)?;
        to_json(&resp)
    }

    /// Submit a signed NF-e for authorization (synchronous mode).
    #[napi(
        ts_return_type = "Promise<{ statusCode: string; statusMessage: string; protocolNumber: string | null; protocolXml: string | null; authorizedAt: string | null; receiptNumber: string | null }>"
    )]
    pub async fn authorize(
        &self,
        uf: String,
        environment: String,
        signed_xml: String,
        lot_id: String,
    ) -> napi::Result<serde_json::Value> {
        let env = parse_env(&environment)?;
        let resp = self
            .inner
            .authorize(&uf, env, &signed_xml, &lot_id)
            .await
            .map_err(to_napi)?;
        to_json(&resp)
    }

    /// Submit a signed NF-e for authorization with gzip compression.
    #[napi(
        ts_return_type = "Promise<{ statusCode: string; statusMessage: string; protocolNumber: string | null; protocolXml: string | null; authorizedAt: string | null; receiptNumber: string | null }>"
    )]
    pub async fn authorize_compressed(
        &self,
        uf: String,
        environment: String,
        signed_xml: String,
        lot_id: String,
    ) -> napi::Result<serde_json::Value> {
        let env = parse_env(&environment)?;
        let resp = self
            .inner
            .authorize_compressed(&uf, env, &signed_xml, &lot_id)
            .await
            .map_err(to_napi)?;
        to_json(&resp)
    }

    /// Submit a signed NFC-e (model 65) for authorization.
    #[napi(
        ts_return_type = "Promise<{ statusCode: string; statusMessage: string; protocolNumber: string | null; protocolXml: string | null; authorizedAt: string | null; receiptNumber: string | null }>"
    )]
    pub async fn authorize_nfce(
        &self,
        uf: String,
        environment: String,
        signed_xml: String,
        lot_id: String,
    ) -> napi::Result<serde_json::Value> {
        let env = parse_env(&environment)?;
        let resp = self
            .inner
            .authorize_nfce(&uf, env, &signed_xml, &lot_id)
            .await
            .map_err(to_napi)?;
        to_json(&resp)
    }

    /// Submit a signed NFC-e (model 65) for authorization with compression.
    #[napi(
        ts_return_type = "Promise<{ statusCode: string; statusMessage: string; protocolNumber: string | null; protocolXml: string | null; authorizedAt: string | null; receiptNumber: string | null }>"
    )]
    pub async fn authorize_nfce_compressed(
        &self,
        uf: String,
        environment: String,
        signed_xml: String,
        lot_id: String,
    ) -> napi::Result<serde_json::Value> {
        let env = parse_env(&environment)?;
        let resp = self
            .inner
            .authorize_nfce_compressed(&uf, env, &signed_xml, &lot_id)
            .await
            .map_err(to_napi)?;
        to_json(&resp)
    }

    /// Consult an NF-e by access key.
    #[napi(
        ts_return_type = "Promise<{ statusCode: string; statusMessage: string; protocolNumber: string | null; protocolXml: string | null; authorizedAt: string | null; receiptNumber: string | null }>"
    )]
    pub async fn consult(
        &self,
        uf: String,
        environment: String,
        access_key: String,
    ) -> napi::Result<serde_json::Value> {
        let env = parse_env(&environment)?;
        let resp = self
            .inner
            .consult(&uf, env, &access_key)
            .await
            .map_err(to_napi)?;
        to_json(&resp)
    }

    /// Consult a batch receipt.
    #[napi(
        ts_return_type = "Promise<{ statusCode: string; statusMessage: string; protocolNumber: string | null; protocolXml: string | null; authorizedAt: string | null; receiptNumber: string | null }>"
    )]
    pub async fn consult_receipt(
        &self,
        uf: String,
        environment: String,
        receipt_number: String,
    ) -> napi::Result<serde_json::Value> {
        let env = parse_env(&environment)?;
        let resp = self
            .inner
            .consult_receipt(&uf, env, &receipt_number)
            .await
            .map_err(to_napi)?;
        to_json(&resp)
    }

    /// Cancel an authorized NF-e.
    #[napi(
        ts_return_type = "Promise<{ statusCode: string; statusMessage: string; protocolNumber: string | null }>"
    )]
    pub async fn cancel(
        &self,
        uf: String,
        environment: String,
        access_key: String,
        protocol: String,
        justification: String,
        tax_id: String,
    ) -> napi::Result<serde_json::Value> {
        let env = parse_env(&environment)?;
        let resp = self
            .inner
            .cancel(&uf, env, &access_key, &protocol, &justification, &tax_id)
            .await
            .map_err(to_napi)?;
        to_json(&resp)
    }

    /// Send a CCe (Carta de Correção Eletrônica) event.
    #[napi(
        ts_return_type = "Promise<{ statusCode: string; statusMessage: string; protocolNumber: string | null }>"
    )]
    pub async fn cce(
        &self,
        uf: String,
        environment: String,
        access_key: String,
        correction_text: String,
        sequence_number: u32,
        tax_id: String,
    ) -> napi::Result<serde_json::Value> {
        let env = parse_env(&environment)?;
        let resp = self
            .inner
            .cce(
                &uf,
                env,
                &access_key,
                &correction_text,
                sequence_number,
                &tax_id,
            )
            .await
            .map_err(to_napi)?;
        to_json(&resp)
    }

    /// Inutilize a range of NF-e numbers.
    ///
    /// The `signedInutXml` must be a signed inutilização XML
    /// (built with request_builders and signed with signInutilizacaoXml).
    #[napi]
    pub async fn inutilize(
        &self,
        uf: String,
        environment: String,
        signed_inut_xml: String,
    ) -> napi::Result<String> {
        let env = parse_env(&environment)?;
        self.inner
            .inutilize(&uf, env, &signed_inut_xml)
            .await
            .map_err(to_napi)
    }

    /// Consult cadastro (taxpayer registration).
    #[napi(
        ts_return_type = "Promise<{ statusCode: string; statusMessage: string; rawXml: string }>"
    )]
    pub async fn cadastro(
        &self,
        uf: String,
        environment: String,
        search_type: String,
        search_value: String,
    ) -> napi::Result<serde_json::Value> {
        let env = parse_env(&environment)?;
        let resp = self
            .inner
            .cadastro(&uf, env, &search_type, &search_value)
            .await
            .map_err(to_napi)?;
        to_json(&resp)
    }

    /// Send a raw request XML to any SEFAZ service (escape hatch).
    ///
    /// Service names: "StatusServico", "Autorizacao", "ConsultaProtocolo",
    /// "RecepcaoEvento", "Inutilizacao", "RetAutorizacao", "ConsultaCadastro",
    /// "DistribuicaoDFe", "AdminCscNFCe".
    #[napi]
    pub async fn send(
        &self,
        service: String,
        uf: String,
        environment: String,
        request_xml: String,
    ) -> napi::Result<String> {
        let env = parse_env(&environment)?;
        let svc = parse_service(&service)?;
        self.inner
            .send(svc, &uf, env, &request_xml)
            .await
            .map_err(to_napi)
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn to_napi(e: fiscal_core::FiscalError) -> napi::Error {
    napi::Error::from_reason(e.to_string())
}

fn to_json(v: &impl serde::Serialize) -> napi::Result<serde_json::Value> {
    serde_json::to_value(v).map_err(|e| napi::Error::from_reason(e.to_string()))
}

fn parse_env(s: &str) -> napi::Result<SefazEnvironment> {
    match s.to_lowercase().as_str() {
        "production" | "1" => Ok(SefazEnvironment::Production),
        "homologation" | "2" => Ok(SefazEnvironment::Homologation),
        _ => Err(napi::Error::from_reason(format!(
            "Invalid environment: \"{s}\". Expected \"production\" or \"homologation\"."
        ))),
    }
}

fn parse_service(s: &str) -> napi::Result<fiscal_sefaz::services::SefazService> {
    use fiscal_sefaz::services::SefazService;
    match s {
        "StatusServico" => Ok(SefazService::StatusServico),
        "Autorizacao" => Ok(SefazService::Autorizacao),
        "ConsultaProtocolo" => Ok(SefazService::ConsultaProtocolo),
        "RecepcaoEvento" => Ok(SefazService::RecepcaoEvento),
        "Inutilizacao" => Ok(SefazService::Inutilizacao),
        "RetAutorizacao" => Ok(SefazService::RetAutorizacao),
        "ConsultaCadastro" => Ok(SefazService::ConsultaCadastro),
        "DistribuicaoDFe" => Ok(SefazService::DistribuicaoDFe),
        "CscNFCe" => Ok(SefazService::CscNFCe),
        _ => Err(napi::Error::from_reason(format!(
            "Unknown service: \"{s}\""
        ))),
    }
}

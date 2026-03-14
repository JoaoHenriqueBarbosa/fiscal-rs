use napi::bindgen_prelude::Buffer;
use napi_derive::napi;

use fiscal_core::types::SefazEnvironment;

#[napi]
pub struct SefazClient {
    inner: fiscal_sefaz::client::SefazClient,
}

#[napi]
impl SefazClient {
    /// Check SEFAZ operational status (`NFeStatusServico4`).
    ///
    /// Builds the `<consStatServ>` request, sends it, and parses the
    /// `<retConsStatServ>` response.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::InvalidStateCode`] if `uf` is invalid.
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns [`FiscalError::XmlParsing`] if the response is malformed.
    #[napi(ts_return_type = "Promise<Record<string, unknown>>")]
    pub async fn status(&self, uf: String, environment: String) -> napi::Result<serde_json::Value> {
        let sefaz_environment = parse_sefaz_environment(&environment)?;
        let resp = self
            .inner
            .status(&uf, sefaz_environment)
            .await
            .map_err(to_napi)?;
        to_json(&resp)
    }

    /// Submit a signed NF-e for authorization (`NFeAutorizacao4`).
    ///
    /// The `signed_xml` must already be signed with `fiscal_crypto::sign_xml`.
    /// Uses synchronous processing (`indSinc=1`) for a single document.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns [`FiscalError::XmlParsing`] if the response is malformed.
    #[napi(ts_return_type = "Promise<Record<string, unknown>>")]
    pub async fn authorize(
        &self,
        uf: String,
        environment: String,
        signed_xml: String,
        lot_id: String,
    ) -> napi::Result<serde_json::Value> {
        let sefaz_environment = parse_sefaz_environment(&environment)?;
        let resp = self
            .inner
            .authorize(&uf, sefaz_environment, &signed_xml, &lot_id)
            .await
            .map_err(to_napi)?;
        to_json(&resp)
    }

    /// Submit a signed NF-e for authorization with gzip compression.
    ///
    /// Same as [`SefazClient::authorize`] but compresses the XML with gzip and uses
    /// `<nfeDadosMsgZip>` in the SOAP envelope, matching PHP sped-nfe's
    /// `$compactar = true` mode.
    #[napi(ts_return_type = "Promise<Record<string, unknown>>")]
    pub async fn authorize_compressed(
        &self,
        uf: String,
        environment: String,
        signed_xml: String,
        lot_id: String,
    ) -> napi::Result<serde_json::Value> {
        let sefaz_environment = parse_sefaz_environment(&environment)?;
        let resp = self
            .inner
            .authorize_compressed(&uf, sefaz_environment, &signed_xml, &lot_id)
            .await
            .map_err(to_napi)?;
        to_json(&resp)
    }

    /// Submit a signed NFC-e (model 65) for authorization.
    ///
    /// Same as [`SefazClient::authorize`] but routes to the NFC-e endpoint.
    #[napi(ts_return_type = "Promise<Record<string, unknown>>")]
    pub async fn authorize_nfce(
        &self,
        uf: String,
        environment: String,
        signed_xml: String,
        lot_id: String,
    ) -> napi::Result<serde_json::Value> {
        let sefaz_environment = parse_sefaz_environment(&environment)?;
        let resp = self
            .inner
            .authorize_nfce(&uf, sefaz_environment, &signed_xml, &lot_id)
            .await
            .map_err(to_napi)?;
        to_json(&resp)
    }

    /// Submit a signed NFC-e (model 65) for authorization with gzip compression.
    ///
    /// Same as [`SefazClient::authorize_nfce`] but compresses the XML with gzip.
    #[napi(ts_return_type = "Promise<Record<string, unknown>>")]
    pub async fn authorize_nfce_compressed(
        &self,
        uf: String,
        environment: String,
        signed_xml: String,
        lot_id: String,
    ) -> napi::Result<serde_json::Value> {
        let sefaz_environment = parse_sefaz_environment(&environment)?;
        let resp = self
            .inner
            .authorize_nfce_compressed(&uf, sefaz_environment, &signed_xml, &lot_id)
            .await
            .map_err(to_napi)?;
        to_json(&resp)
    }

    /// Query batch processing result by receipt number (`NFeRetAutorizacao4`).
    ///
    /// Used after asynchronous authorization (`indSinc=0`) to poll for
    /// the result.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns [`FiscalError::XmlParsing`] if the response is malformed.
    #[napi(ts_return_type = "Promise<Record<string, unknown>>")]
    pub async fn consult_receipt(
        &self,
        uf: String,
        environment: String,
        receipt: String,
    ) -> napi::Result<serde_json::Value> {
        let sefaz_environment = parse_sefaz_environment(&environment)?;
        let resp = self
            .inner
            .consult_receipt(&uf, sefaz_environment, &receipt)
            .await
            .map_err(to_napi)?;
        to_json(&resp)
    }

    /// Consult an NF-e by its 44-digit access key (`NFeConsultaProtocolo4`).
    ///
    /// Returns the current status, protocol number, and authorization
    /// timestamp for an existing NF-e.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns [`FiscalError::XmlParsing`] if the response is malformed.
    #[napi(ts_return_type = "Promise<Record<string, unknown>>")]
    pub async fn consult(
        &self,
        uf: String,
        environment: String,
        access_key: String,
    ) -> napi::Result<serde_json::Value> {
        let sefaz_environment = parse_sefaz_environment(&environment)?;
        let resp = self
            .inner
            .consult(&uf, sefaz_environment, &access_key)
            .await
            .map_err(to_napi)?;
        to_json(&resp)
    }

    #[napi(ts_return_type = "Promise<Record<string, unknown>>")]
    pub async fn ator_interessado(
        &self,
        environment: String,
        access_key: String,
        tp_autor: u32,
        ver_aplic: String,
        authorized_cnpj: Option<String>,
        authorized_cpf: Option<String>,
        tp_autorizacao: u32,
        issuer_uf: String,
        seq: u32,
        tax_id: String,
    ) -> napi::Result<serde_json::Value> {
        let sefaz_environment = parse_sefaz_environment(&environment)?;
        let resp = self
            .inner
            .ator_interessado(
                sefaz_environment,
                &access_key,
                tp_autor as u8,
                &ver_aplic,
                authorized_cnpj.as_deref(),
                authorized_cpf.as_deref(),
                tp_autorizacao as u8,
                &issuer_uf,
                seq,
                &tax_id,
            )
            .await
            .map_err(to_napi)?;
        to_json(&resp)
    }

    #[napi(ts_return_type = "Promise<Record<string, unknown>>")]
    pub async fn comprovante_entrega(
        &self,
        environment: String,
        access_key: String,
        ver_aplic: String,
        delivery_date: String,
        doc_number: String,
        name: String,
        lat: Option<String>,
        long: Option<String>,
        hash: String,
        hash_date: String,
        issuer_uf: String,
        seq: u32,
        tax_id: String,
    ) -> napi::Result<serde_json::Value> {
        let sefaz_environment = parse_sefaz_environment(&environment)?;
        let resp = self
            .inner
            .comprovante_entrega(
                sefaz_environment,
                &access_key,
                &ver_aplic,
                &delivery_date,
                &doc_number,
                &name,
                lat.as_deref(),
                long.as_deref(),
                &hash,
                &hash_date,
                &issuer_uf,
                seq,
                &tax_id,
            )
            .await
            .map_err(to_napi)?;
        to_json(&resp)
    }

    #[napi(ts_return_type = "Promise<Record<string, unknown>>")]
    pub async fn cancel_comprovante_entrega(
        &self,
        environment: String,
        access_key: String,
        ver_aplic: String,
        event_protocol: String,
        issuer_uf: String,
        seq: u32,
        tax_id: String,
    ) -> napi::Result<serde_json::Value> {
        let sefaz_environment = parse_sefaz_environment(&environment)?;
        let resp = self
            .inner
            .cancel_comprovante_entrega(
                sefaz_environment,
                &access_key,
                &ver_aplic,
                &event_protocol,
                &issuer_uf,
                seq,
                &tax_id,
            )
            .await
            .map_err(to_napi)?;
        to_json(&resp)
    }

    #[napi(ts_return_type = "Promise<Record<string, unknown>>")]
    pub async fn insucesso_entrega(
        &self,
        environment: String,
        access_key: String,
        ver_aplic: String,
        attempt_date: String,
        attempt_number: Option<u32>,
        reason_type: u32,
        reason_justification: Option<String>,
        lat: Option<String>,
        long: Option<String>,
        hash: String,
        hash_date: String,
        issuer_uf: String,
        seq: u32,
        tax_id: String,
    ) -> napi::Result<serde_json::Value> {
        let sefaz_environment = parse_sefaz_environment(&environment)?;
        let resp = self
            .inner
            .insucesso_entrega(
                sefaz_environment,
                &access_key,
                &ver_aplic,
                &attempt_date,
                attempt_number,
                reason_type as u8,
                reason_justification.as_deref(),
                lat.as_deref(),
                long.as_deref(),
                &hash,
                &hash_date,
                &issuer_uf,
                seq,
                &tax_id,
            )
            .await
            .map_err(to_napi)?;
        to_json(&resp)
    }

    #[napi(ts_return_type = "Promise<Record<string, unknown>>")]
    pub async fn cancel_insucesso_entrega(
        &self,
        environment: String,
        access_key: String,
        ver_aplic: String,
        event_protocol: String,
        issuer_uf: String,
        seq: u32,
        tax_id: String,
    ) -> napi::Result<serde_json::Value> {
        let sefaz_environment = parse_sefaz_environment(&environment)?;
        let resp = self
            .inner
            .cancel_insucesso_entrega(
                sefaz_environment,
                &access_key,
                &ver_aplic,
                &event_protocol,
                &issuer_uf,
                seq,
                &tax_id,
            )
            .await
            .map_err(to_napi)?;
        to_json(&resp)
    }

    #[napi(ts_return_type = "Promise<Record<string, unknown>>")]
    pub async fn cancel_prorrogacao(
        &self,
        uf: String,
        environment: String,
        access_key: String,
        protocol: String,
        second_term: bool,
        seq: u32,
        tax_id: String,
    ) -> napi::Result<serde_json::Value> {
        let sefaz_environment = parse_sefaz_environment(&environment)?;
        let resp = self
            .inner
            .cancel_prorrogacao(
                &uf,
                sefaz_environment,
                &access_key,
                &protocol,
                second_term,
                seq,
                &tax_id,
            )
            .await
            .map_err(to_napi)?;
        to_json(&resp)
    }

    /// Cancel a previously authorized NF-e (`RecepcaoEvento4`, tpEvento=110111).
    ///
    /// # Arguments
    ///
    /// * `access_key` — 44-digit access key of the NF-e to cancel.
    /// * `protocol` — protocol number from the authorization response.
    /// * `justification` — reason for cancellation (min 15 characters).
    /// * `tax_id` — CNPJ or CPF of the issuer.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns [`FiscalError::XmlParsing`] if the response is malformed.
    #[napi(ts_return_type = "Promise<Record<string, unknown>>")]
    pub async fn cancel(
        &self,
        uf: String,
        environment: String,
        access_key: String,
        protocol: String,
        justification: String,
        tax_id: String,
    ) -> napi::Result<serde_json::Value> {
        let sefaz_environment = parse_sefaz_environment(&environment)?;
        let resp = self
            .inner
            .cancel(
                &uf,
                sefaz_environment,
                &access_key,
                &protocol,
                &justification,
                &tax_id,
            )
            .await
            .map_err(to_napi)?;
        to_json(&resp)
    }

    /// Send a Carta de Correcao / CCe (`RecepcaoEvento4`, tpEvento=110110).
    ///
    /// # Arguments
    ///
    /// * `access_key` — 44-digit access key of the NF-e to correct.
    /// * `correction` — correction text describing the change.
    /// * `seq` — event sequence number (increments per correction on same NF-e).
    /// * `tax_id` — CNPJ or CPF of the issuer.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns [`FiscalError::XmlParsing`] if the response is malformed.
    #[napi(ts_return_type = "Promise<Record<string, unknown>>")]
    pub async fn cce(
        &self,
        uf: String,
        environment: String,
        access_key: String,
        correction: String,
        seq: u32,
        tax_id: String,
    ) -> napi::Result<serde_json::Value> {
        let sefaz_environment = parse_sefaz_environment(&environment)?;
        let resp = self
            .inner
            .cce(
                &uf,
                sefaz_environment,
                &access_key,
                &correction,
                seq,
                &tax_id,
            )
            .await
            .map_err(to_napi)?;
        to_json(&resp)
    }

    /// Submit an inutilizacao request to void unused number ranges
    /// (`NFeInutilizacao4`).
    ///
    /// The signed `<inutNFe>` XML must be pre-built via
    /// [`request_builders::build_inutilizacao_request`] and signed with
    /// `fiscal_crypto::sign_xml` before calling this method.
    ///
    /// Returns the raw SEFAZ response XML. Parse it with
    /// [`response_parsers`] or extract fields manually.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    #[napi]
    pub async fn inutilize(
        &self,
        uf: String,
        environment: String,
        signed_inut_xml: String,
    ) -> napi::Result<String> {
        let sefaz_environment = parse_sefaz_environment(&environment)?;
        self.inner
            .inutilize(&uf, sefaz_environment, &signed_inut_xml)
            .await
            .map_err(to_napi)
    }

    /// Submit a manifestacao do destinatario event (`RecepcaoEvento4`).
    ///
    /// Routes to the Ambiente Nacional (AN) endpoint. Valid event types:
    /// - `"210200"` — Confirmacao da Operacao
    /// - `"210210"` — Ciencia da Operacao
    /// - `"210220"` — Desconhecimento da Operacao
    /// - `"210240"` — Operacao nao Realizada (requires justification)
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns [`FiscalError::XmlParsing`] if the response is malformed.
    #[napi(ts_return_type = "Promise<Record<string, unknown>>")]
    pub async fn manifest(
        &self,
        environment: String,
        access_key: String,
        event_type: String,
        justification: Option<String>,
        seq: u32,
        tax_id: String,
    ) -> napi::Result<serde_json::Value> {
        let sefaz_environment = parse_sefaz_environment(&environment)?;
        let resp = self
            .inner
            .manifest(
                sefaz_environment,
                &access_key,
                &event_type,
                justification.as_deref(),
                seq,
                &tax_id,
            )
            .await
            .map_err(to_napi)?;
        to_json(&resp)
    }

    /// Query the distribution of fiscal documents (DF-e) from the national
    /// environment (`NFeDistribuicaoDFe`).
    ///
    /// # Arguments
    ///
    /// * `uf` — State abbreviation of the interested party.
    /// * `tax_id` — CNPJ or CPF of the interested party.
    /// * `nsu` — Optional specific NSU or last NSU to query.
    /// * `access_key` — Optional 44-digit access key for direct lookup.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns [`FiscalError::XmlParsing`] if the response is malformed.
    #[napi(ts_return_type = "Promise<Record<string, unknown>>")]
    pub async fn dist_dfe(
        &self,
        uf: String,
        environment: String,
        tax_id: String,
        nsu: Option<String>,
        access_key: Option<String>,
    ) -> napi::Result<serde_json::Value> {
        let sefaz_environment = parse_sefaz_environment(&environment)?;
        let resp = self
            .inner
            .dist_dfe(
                &uf,
                sefaz_environment,
                &tax_id,
                nsu.as_deref(),
                access_key.as_deref(),
            )
            .await
            .map_err(to_napi)?;
        to_json(&resp)
    }

    /// Query the SEFAZ taxpayer registry (`CadConsultaCadastro4`).
    ///
    /// # Arguments
    ///
    /// * `uf` — State to query.
    /// * `search_type` — One of `"CNPJ"`, `"IE"`, or `"CPF"`.
    /// * `search_value` — The document number to search for.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns [`FiscalError::XmlParsing`] if the response is malformed.
    #[napi(ts_return_type = "Promise<Record<string, unknown>>")]
    pub async fn cadastro(
        &self,
        uf: String,
        environment: String,
        search_type: String,
        search_value: String,
    ) -> napi::Result<serde_json::Value> {
        let sefaz_environment = parse_sefaz_environment(&environment)?;
        let resp = self
            .inner
            .cadastro(&uf, sefaz_environment, &search_type, &search_value)
            .await
            .map_err(to_napi)?;
        to_json(&resp)
    }

    /// Check EPEC NFC-e service status (`EPECStatusServico`, SP only).
    ///
    /// Queries the operational status of the EPEC NFC-e service. This
    /// service exists only in SP (São Paulo) for model 65 (NFC-e),
    /// matching the PHP `sefazStatusEpecNfce()` method from
    /// `TraitEPECNfce`.
    ///
    /// # Arguments
    ///
    /// * `uf` — State abbreviation (must be `"SP"`).
    /// * `environment` — SEFAZ environment.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns [`FiscalError::XmlParsing`] if the response is malformed.
    #[napi(ts_return_type = "Promise<Record<string, unknown>>")]
    pub async fn epec_nfce_status(
        &self,
        uf: String,
        environment: String,
    ) -> napi::Result<serde_json::Value> {
        let sefaz_environment = parse_sefaz_environment(&environment)?;
        let resp = self
            .inner
            .epec_nfce_status(&uf, sefaz_environment)
            .await
            .map_err(to_napi)?;
        to_json(&resp)
    }

    #[napi(ts_return_type = "Promise<Record<string, unknown>>")]
    pub async fn cancel_substituicao(
        &self,
        uf: String,
        environment: String,
        access_key: String,
        ref_access_key: String,
        protocol: String,
        justification: String,
        ver_aplic: String,
        tax_id: String,
    ) -> napi::Result<serde_json::Value> {
        let sefaz_environment = parse_sefaz_environment(&environment)?;
        let resp = self
            .inner
            .cancel_substituicao(
                &uf,
                sefaz_environment,
                &access_key,
                &ref_access_key,
                &protocol,
                &justification,
                &ver_aplic,
                &tax_id,
            )
            .await
            .map_err(to_napi)?;
        to_json(&resp)
    }

    /// Download an NF-e by its 44-digit access key (`NFeDistribuicaoDFe`).
    ///
    /// A convenience wrapper around [`dist_dfe`](Self::dist_dfe) that passes
    /// the access key directly. Matches the PHP `sefazDownload()` method.
    ///
    /// # Arguments
    ///
    /// * `uf` — State abbreviation of the interested party.
    /// * `environment` — SEFAZ environment.
    /// * `tax_id` — CNPJ or CPF of the interested party.
    /// * `access_key` — 44-digit access key of the NF-e to download.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns [`FiscalError::XmlParsing`] if the response is malformed.
    #[napi(ts_return_type = "Promise<Record<string, unknown>>")]
    pub async fn download(
        &self,
        uf: String,
        environment: String,
        tax_id: String,
        access_key: String,
    ) -> napi::Result<serde_json::Value> {
        let sefaz_environment = parse_sefaz_environment(&environment)?;
        let resp = self
            .inner
            .download(&uf, sefaz_environment, &tax_id, &access_key)
            .await
            .map_err(to_napi)?;
        to_json(&resp)
    }

    /// Manage CSC (Código de Segurança do Contribuinte) for NFC-e
    /// (`CscNFCe`).
    ///
    /// Matches the PHP `sefazCsc()` method. Used exclusively with NFC-e
    /// (model 65).
    ///
    /// # Arguments
    ///
    /// * `uf` — State abbreviation of the issuer.
    /// * `environment` — SEFAZ environment.
    /// * `ind_op` — Operation type: 1=query, 2=request new, 3=revoke.
    /// * `cnpj` — Full CNPJ of the taxpayer (14 digits).
    /// * `csc_id` — CSC identifier (required only for `ind_op=3`).
    /// * `csc_code` — CSC code/value (required only for `ind_op=3`).
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    #[napi]
    pub async fn csc(
        &self,
        uf: String,
        environment: String,
        ind_op: u32,
        cnpj: String,
        csc_id: Option<String>,
        csc_code: Option<String>,
    ) -> napi::Result<String> {
        let sefaz_environment = parse_sefaz_environment(&environment)?;
        self.inner
            .csc(
                &uf,
                sefaz_environment,
                ind_op as u8,
                &cnpj,
                csc_id.as_deref(),
                csc_code.as_deref(),
            )
            .await
            .map_err(to_napi)
    }

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
    #[napi(constructor)]
    pub fn new(pfx_buffer: Buffer, passphrase: String) -> napi::Result<Self> {
        let inner =
            fiscal_sefaz::client::SefazClient::new(&pfx_buffer, &passphrase).map_err(to_napi)?;
        Ok(Self { inner })
    }

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
    #[napi]
    pub async fn send(
        &self,
        service: String,
        uf: String,
        environment: String,
        request_xml: String,
    ) -> napi::Result<String> {
        let sefaz_service = parse_sefaz_service(&service)?;
        let sefaz_environment = parse_sefaz_environment(&environment)?;
        self.inner
            .send(sefaz_service, &uf, sefaz_environment, &request_xml)
            .await
            .map_err(to_napi)
    }

    /// Send a raw request XML to a SEFAZ service for a specific invoice model
    /// (55 = NF-e, 65 = NFC-e) and return the raw response XML.
    #[napi]
    pub async fn send_model(
        &self,
        service: String,
        uf: String,
        environment: String,
        request_xml: String,
        model: u32,
    ) -> napi::Result<String> {
        let sefaz_service = parse_sefaz_service(&service)?;
        let sefaz_environment = parse_sefaz_environment(&environment)?;
        self.inner
            .send_model(
                sefaz_service,
                &uf,
                sefaz_environment,
                &request_xml,
                model as u8,
            )
            .await
            .map_err(to_napi)
    }

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
    #[napi(ts_return_type = "Promise<Record<string, unknown>>")]
    pub async fn sefaz_validate(
        &self,
        uf: String,
        nfe_xml: String,
    ) -> napi::Result<serde_json::Value> {
        let resp = self
            .inner
            .sefaz_validate(&uf, &nfe_xml)
            .await
            .map_err(to_napi)?;
        to_json(&resp)
    }

    #[napi(ts_return_type = "Promise<Record<string, unknown>>")]
    pub async fn rtc_info_pagto_integral(
        &self,
        uf: String,
        environment: String,
        access_key: String,
        seq: u32,
        tax_id: String,
        ver_aplic: String,
    ) -> napi::Result<serde_json::Value> {
        let sefaz_environment = parse_sefaz_environment(&environment)?;
        let resp = self
            .inner
            .rtc_info_pagto_integral(
                &uf,
                sefaz_environment,
                &access_key,
                seq,
                &tax_id,
                &ver_aplic,
            )
            .await
            .map_err(to_napi)?;
        to_json(&resp)
    }

    #[napi(ts_return_type = "Promise<Record<string, unknown>>")]
    pub async fn rtc_aceite_debito(
        &self,
        uf: String,
        environment: String,
        access_key: String,
        seq: u32,
        tax_id: String,
        ver_aplic: String,
        ind_aceitacao: u32,
    ) -> napi::Result<serde_json::Value> {
        let sefaz_environment = parse_sefaz_environment(&environment)?;
        let resp = self
            .inner
            .rtc_aceite_debito(
                &uf,
                sefaz_environment,
                &access_key,
                seq,
                &tax_id,
                &ver_aplic,
                ind_aceitacao as u8,
            )
            .await
            .map_err(to_napi)?;
        to_json(&resp)
    }

    #[napi(ts_return_type = "Promise<Record<string, unknown>>")]
    pub async fn rtc_manif_transf_cred_ibs(
        &self,
        uf: String,
        environment: String,
        access_key: String,
        seq: u32,
        tax_id: String,
        ver_aplic: String,
        ind_aceitacao: u32,
    ) -> napi::Result<serde_json::Value> {
        let sefaz_environment = parse_sefaz_environment(&environment)?;
        let resp = self
            .inner
            .rtc_manif_transf_cred_ibs(
                &uf,
                sefaz_environment,
                &access_key,
                seq,
                &tax_id,
                &ver_aplic,
                ind_aceitacao as u8,
            )
            .await
            .map_err(to_napi)?;
        to_json(&resp)
    }

    #[napi(ts_return_type = "Promise<Record<string, unknown>>")]
    pub async fn rtc_manif_transf_cred_cbs(
        &self,
        uf: String,
        environment: String,
        access_key: String,
        seq: u32,
        tax_id: String,
        ver_aplic: String,
        ind_aceitacao: u32,
    ) -> napi::Result<serde_json::Value> {
        let sefaz_environment = parse_sefaz_environment(&environment)?;
        let resp = self
            .inner
            .rtc_manif_transf_cred_cbs(
                &uf,
                sefaz_environment,
                &access_key,
                seq,
                &tax_id,
                &ver_aplic,
                ind_aceitacao as u8,
            )
            .await
            .map_err(to_napi)?;
        to_json(&resp)
    }

    #[napi(ts_return_type = "Promise<Record<string, unknown>>")]
    pub async fn rtc_cancela_evento(
        &self,
        uf: String,
        environment: String,
        access_key: String,
        seq: u32,
        tax_id: String,
        ver_aplic: String,
        tp_evento_aut: String,
        n_prot_evento: String,
    ) -> napi::Result<serde_json::Value> {
        let sefaz_environment = parse_sefaz_environment(&environment)?;
        let resp = self
            .inner
            .rtc_cancela_evento(
                &uf,
                sefaz_environment,
                &access_key,
                seq,
                &tax_id,
                &ver_aplic,
                &tp_evento_aut,
                &n_prot_evento,
            )
            .await
            .map_err(to_napi)?;
        to_json(&resp)
    }

    #[napi(ts_return_type = "Promise<Record<string, unknown>>")]
    pub async fn rtc_atualizacao_data_entrega(
        &self,
        uf: String,
        environment: String,
        access_key: String,
        seq: u32,
        tax_id: String,
        ver_aplic: String,
        data_prevista: String,
    ) -> napi::Result<serde_json::Value> {
        let sefaz_environment = parse_sefaz_environment(&environment)?;
        let resp = self
            .inner
            .rtc_atualizacao_data_entrega(
                &uf,
                sefaz_environment,
                &access_key,
                seq,
                &tax_id,
                &ver_aplic,
                &data_prevista,
            )
            .await
            .map_err(to_napi)?;
        to_json(&resp)
    }
}

// Skipped: authorize_batch (unsupported param type), authorize_batch_compressed (unsupported param type), authorize_batch_nfce (unsupported param type), prorrogacao (unsupported param type), epec (unsupported param type), epec_nfce (unsupported param type), event_batch (unsupported param type), manifest_batch (unsupported param type), conciliacao (unsupported param type), rtc_importacao_zfm (unsupported param type), rtc_roubo_perda_fornecedor (unsupported param type), rtc_fornecimento_nao_realizado (unsupported param type), rtc_sol_aprop_cred_presumido (unsupported param type), rtc_destino_consumo_pessoal (unsupported param type), rtc_roubo_perda_adquirente (unsupported param type), rtc_imobilizacao_item (unsupported param type), rtc_apropriacao_credito_comb (unsupported param type), rtc_apropriacao_credito_bens (unsupported param type)

// ── Helpers ─────────────────────────────────────────────────────────────────

fn to_napi(e: fiscal_core::FiscalError) -> napi::Error {
    napi::Error::from_reason(e.to_string())
}

fn to_json(v: &impl serde::Serialize) -> napi::Result<serde_json::Value> {
    serde_json::to_value(v).map_err(|e| napi::Error::from_reason(e.to_string()))
}

fn parse_sefaz_environment(s: &str) -> napi::Result<SefazEnvironment> {
    match s.to_lowercase().as_str() {
        "production" | "1" => Ok(SefazEnvironment::Production),
        "homologation" | "2" => Ok(SefazEnvironment::Homologation),
        _ => Err(napi::Error::from_reason(format!(
            "Invalid SefazEnvironment: \"{s}\""
        ))),
    }
}

fn parse_sefaz_service(s: &str) -> napi::Result<fiscal_sefaz::services::SefazService> {
    match s.to_lowercase().as_str() {
        "statusServico" => Ok(fiscal_sefaz::services::SefazService::StatusServico),
        "autorizacao" => Ok(fiscal_sefaz::services::SefazService::Autorizacao),
        "retAutorizacao" => Ok(fiscal_sefaz::services::SefazService::RetAutorizacao),
        "consultaProtocolo" => Ok(fiscal_sefaz::services::SefazService::ConsultaProtocolo),
        "inutilizacao" => Ok(fiscal_sefaz::services::SefazService::Inutilizacao),
        "recepcaoEvento" => Ok(fiscal_sefaz::services::SefazService::RecepcaoEvento),
        "distribuicaoDFe" => Ok(fiscal_sefaz::services::SefazService::DistribuicaoDFe),
        "consultaCadastro" => Ok(fiscal_sefaz::services::SefazService::ConsultaCadastro),
        "cscNFCe" => Ok(fiscal_sefaz::services::SefazService::CscNFCe),
        "recepcaoEPEC" => Ok(fiscal_sefaz::services::SefazService::RecepcaoEPEC),
        "ePECStatusServico" => Ok(fiscal_sefaz::services::SefazService::EPECStatusServico),
        "recepcaoEpecNfce" => Ok(fiscal_sefaz::services::SefazService::RecepcaoEpecNfce),
        "epecNfceStatusServico" => Ok(fiscal_sefaz::services::SefazService::EpecNfceStatusServico),
        "nfeConsultaDest" => Ok(fiscal_sefaz::services::SefazService::NfeConsultaDest),
        "nfeDownloadNF" => Ok(fiscal_sefaz::services::SefazService::NfeDownloadNF),
        _ => Err(napi::Error::from_reason(format!(
            "Invalid SefazService: \"{s}\""
        ))),
    }
}

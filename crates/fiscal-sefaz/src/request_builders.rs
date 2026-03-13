use fiscal_core::constants::{NFE_NAMESPACE, NFE_VERSION};
use fiscal_core::state_codes::get_state_code;
use fiscal_core::types::SefazEnvironment;

/// Event type constants (`tpEvento`) matching the SEFAZ specification.
pub mod event_types {
    /// Carta de Correção Eletrônica (CC-e), code `110110`.
    pub const CCE: u32 = 110110;
    /// Cancellation event, code `110111`.
    pub const CANCELLATION: u32 = 110111;
    /// Recipient confirms operation, code `210200`.
    pub const CONFIRMATION: u32 = 210200;
    /// Recipient is aware of operation, code `210210`.
    pub const AWARENESS: u32 = 210210;
    /// Recipient does not recognize the operation, code `210220`.
    pub const UNKNOWN_OPERATION: u32 = 210220;
    /// Recipient confirms operation was not performed, code `210240`.
    pub const OPERATION_NOT_PERFORMED: u32 = 210240;
}

/// Event descriptions matching the SEFAZ specification.
fn event_description(event_type: u32) -> &'static str {
    match event_type {
        110110 => "Carta de Correcao",
        110111 => "Cancelamento",
        110112 => "Cancelamento por substituicao",
        110140 => "EPEC",
        110150 => "Ator interessado na NF-e",
        110130 => "Comprovante de Entrega da NF-e",
        110131 => "Cancelamento Comprovante de Entrega da NF-e",
        111500 | 111501 => "Pedido de Prorrogacao",
        111502 | 111503 => "Cancelamento de Pedido de Prorrogacao",
        210200 => "Confirmacao da Operacao",
        210210 => "Ciencia da Operacao",
        210220 => "Desconhecimento da Operacao",
        210240 => "Operacao nao Realizada",
        110192 => "Insucesso na Entrega da NF-e",
        110193 => "Cancelamento Insucesso na Entrega da NF-e",
        _ => "",
    }
}

/// Build the event ID string: `ID{tpEvento}{chNFe}{nSeqEvento:02}`.
fn build_event_id(event_type: u32, access_key: &str, seq: u32) -> String {
    format!("ID{event_type}{access_key}{seq:02}")
}

/// Generate a CNPJ XML tag from a tax ID string.
///
/// If the tax ID is 11 digits it is treated as CPF; otherwise CNPJ.
fn tax_id_xml_tag(tax_id: &str) -> String {
    let digits: String = tax_id.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() == 11 {
        format!("<CPF>{digits}</CPF>")
    } else {
        format!("<CNPJ>{digits}</CNPJ>")
    }
}

/// Build a SEFAZ authorization request XML (`<enviNFe>`).
///
/// Wraps one or more signed NF-e XML documents in an `<enviNFe>` envelope
/// for submission to the SEFAZ authorization web service.
///
/// # Arguments
///
/// * `xml` - The signed NF-e XML (XML declaration is stripped automatically).
/// * `lot_id` - Lot identifier for the submission batch.
/// * `sync` - Whether to use synchronous processing (`indSinc=1`).
/// * `compressed` - Whether the XML content is gzip-compressed (flag only,
///   actual compression is handled at the transport layer).
///
/// # Panics
///
/// Panics if `xml` is empty.
///
/// # Errors
///
/// This function does not return `Result` errors but panics on invalid input.
pub fn build_autorizacao_request(xml: &str, lot_id: &str, sync: bool, _compressed: bool) -> String {
    assert!(
        !xml.is_empty(),
        "XML content is required for authorization request"
    );

    // Strip XML declaration if present
    let content = xml.trim().trim_start_matches(|c: char| c != '<');
    let stripped = strip_xml_declaration(content);

    let ind_sinc = if sync { "1" } else { "0" };

    format!(
        "<enviNFe xmlns=\"{NFE_NAMESPACE}\" versao=\"{NFE_VERSION}\"><idLote>{lot_id}</idLote><indSinc>{ind_sinc}</indSinc>{stripped}</enviNFe>"
    )
}

/// Build a SEFAZ service status request XML (`<consStatServ>`).
///
/// Queries the operational status of a SEFAZ web service for the given state.
///
/// # Panics
///
/// Panics if `uf` is not a valid Brazilian state code.
///
/// # Errors
///
/// This function panics on invalid state codes rather than returning `Result`.
pub fn build_status_request(uf: &str, environment: SefazEnvironment) -> String {
    let cuf = get_state_code(uf).expect("Invalid state code");
    let tp_amb = environment.as_str();

    format!(
        "<consStatServ xmlns=\"{NFE_NAMESPACE}\" versao=\"{NFE_VERSION}\"><tpAmb>{tp_amb}</tpAmb><cUF>{cuf}</cUF><xServ>STATUS</xServ></consStatServ>"
    )
}

/// Build a SEFAZ consultation request XML (`<consSitNFe>`) for an access key.
///
/// Queries the current status of an NF-e by its 44-digit access key.
///
/// # Panics
///
/// Panics if `access_key` is empty, not exactly 44 characters, or non-numeric.
///
/// # Errors
///
/// This function panics on invalid input rather than returning `Result`.
pub fn build_consulta_request(access_key: &str, environment: SefazEnvironment) -> String {
    validate_access_key(access_key);
    let tp_amb = environment.as_str();

    format!(
        "<consSitNFe xmlns=\"{NFE_NAMESPACE}\" versao=\"{NFE_VERSION}\"><tpAmb>{tp_amb}</tpAmb><xServ>CONSULTAR</xServ><chNFe>{access_key}</chNFe></consSitNFe>"
    )
}

/// Build a SEFAZ receipt consultation request XML (`<consReciNFe>`).
///
/// Queries the processing result of a previously submitted batch by receipt number.
///
/// # Panics
///
/// Panics if `receipt` is empty.
///
/// # Errors
///
/// This function panics on invalid input rather than returning `Result`.
pub fn build_consulta_recibo_request(receipt: &str, environment: SefazEnvironment) -> String {
    assert!(!receipt.is_empty(), "Receipt number (recibo) is required");
    let tp_amb = environment.as_str();

    format!(
        "<consReciNFe xmlns=\"{NFE_NAMESPACE}\" versao=\"{NFE_VERSION}\"><tpAmb>{tp_amb}</tpAmb><nRec>{receipt}</nRec></consReciNFe>"
    )
}

/// Build a SEFAZ number voiding request XML (`<inutNFe>`).
///
/// Requests voiding (inutilizacao) of a range of NF-e/NFC-e numbers that
/// were skipped and will not be used.
///
/// # Arguments
///
/// * `year` - Two-digit year (e.g. `22` for 2022).
/// * `tax_id` - CNPJ of the issuer (14 digits, no formatting).
/// * `model` - Invoice model (`"55"` for NF-e, `"65"` for NFC-e).
/// * `series` - Series number.
/// * `start_number` - First invoice number in the range.
/// * `end_number` - Last invoice number in the range.
/// * `justification` - Justification text for the voiding.
/// * `environment` - SEFAZ environment (production or homologation).
/// * `uf` - State abbreviation (e.g. `"SP"`).
///
/// # Panics
///
/// Panics if `uf` is not a valid Brazilian state code.
///
/// # Errors
///
/// This function panics on invalid state codes rather than returning `Result`.
#[allow(clippy::too_many_arguments)]
pub fn build_inutilizacao_request(
    year: u16,
    tax_id: &str,
    model: &str,
    series: u32,
    start_number: u32,
    end_number: u32,
    justification: &str,
    environment: SefazEnvironment,
    uf: &str,
) -> String {
    let cuf = get_state_code(uf).expect("Invalid state code");
    let tp_amb = environment.as_str();
    let digits: String = tax_id.chars().filter(|c| c.is_ascii_digit()).collect();

    // PHP: str_pad($cnpj, 14, '0', STR_PAD_LEFT) — always pad to 14 for the ID
    let padded_id_tax = format!("{digits:0>14}");
    let id = format!(
        "ID{cuf}{year:02}{padded_id_tax}{model:0>2}{series:03}{start_number:09}{end_number:09}"
    );

    // PHP: if siglaUF == 'MT' && strlen($cnpj) == 11 => use <CPF>, else <CNPJ>
    let tax_tag = if digits.len() == 11 {
        format!("<CPF>{digits}</CPF>")
    } else {
        format!("<CNPJ>{digits}</CNPJ>")
    };

    format!(
        "<inutNFe xmlns=\"{NFE_NAMESPACE}\" versao=\"{NFE_VERSION}\"><infInut Id=\"{id}\"><tpAmb>{tp_amb}</tpAmb><xServ>INUTILIZAR</xServ><cUF>{cuf}</cUF><ano>{year:02}</ano>{tax_tag}<mod>{model}</mod><serie>{series}</serie><nNFIni>{start_number}</nNFIni><nNFFin>{end_number}</nNFFin><xJust>{justification}</xJust></infInut></inutNFe>"
    )
}

/// Build a SEFAZ cancellation event request XML.
///
/// Builds the complete `<envEvento>` wrapper containing a cancellation event
/// (`tpEvento=110111`) for a previously authorized NF-e.
///
/// # Arguments
///
/// * `access_key` - The 44-digit access key of the NF-e to cancel.
/// * `protocol` - The protocol number from the authorization response.
/// * `justification` - Justification text (minimum 15 characters).
/// * `seq` - Event sequence number (usually 1).
/// * `environment` - SEFAZ environment.
/// * `tax_id` - CNPJ or CPF of the issuer.
///
/// # Panics
///
/// Panics if `justification` is empty.
///
/// # Errors
///
/// This function panics on invalid input rather than returning `Result`.
pub fn build_cancela_request(
    access_key: &str,
    protocol: &str,
    justification: &str,
    seq: u32,
    environment: SefazEnvironment,
    tax_id: &str,
) -> String {
    assert!(
        !justification.is_empty(),
        "Cancellation justification (xJust) is required"
    );

    let additional = format!("<nProt>{protocol}</nProt><xJust>{justification}</xJust>");

    build_event_xml(
        access_key,
        event_types::CANCELLATION,
        seq,
        tax_id,
        environment,
        &additional,
    )
}

/// Build a SEFAZ CCe (correction letter) event request XML.
///
/// Builds the complete `<envEvento>` wrapper containing a Carta de Correcao
/// (`tpEvento=110110`) for a previously authorized NF-e.
///
/// # Arguments
///
/// * `access_key` - The 44-digit access key of the NF-e to correct.
/// * `correction` - The correction text describing what is being changed.
/// * `seq` - Event sequence number (increments for each correction on the same NF-e).
/// * `environment` - SEFAZ environment.
/// * `tax_id` - CNPJ or CPF of the issuer.
///
/// # Panics
///
/// Panics if `correction` is empty.
///
/// # Errors
///
/// This function panics on invalid input rather than returning `Result`.
pub fn build_cce_request(
    access_key: &str,
    correction: &str,
    seq: u32,
    environment: SefazEnvironment,
    tax_id: &str,
) -> String {
    assert!(
        !correction.is_empty(),
        "Correction text (xCorrecao) is required for CCe"
    );

    let x_cond_uso = concat!(
        "A Carta de Correcao e disciplinada pelo paragrafo ",
        "1o-A do art. 7o do Convenio S/N, de 15 de dezembro de 1970 ",
        "e pode ser utilizada para regularizacao de erro ocorrido ",
        "na emissao de documento fiscal, desde que o erro nao esteja ",
        "relacionado com: I - as variaveis que determinam o valor ",
        "do imposto tais como: base de calculo, aliquota, ",
        "diferenca de preco, quantidade, valor da operacao ou da ",
        "prestacao; II - a correcao de dados cadastrais que implique ",
        "mudanca do remetente ou do destinatario; III - a data de ",
        "emissao ou de saida."
    );

    let additional =
        format!("<xCorrecao>{correction}</xCorrecao><xCondUso>{x_cond_uso}</xCondUso>");

    build_event_xml(
        access_key,
        event_types::CCE,
        seq,
        tax_id,
        environment,
        &additional,
    )
}

/// Build a SEFAZ manifest (manifestacao do destinatario) event request XML.
///
/// Builds the complete `<envEvento>` wrapper containing a manifestation event.
/// Valid event types are:
/// - `"210200"` (Confirmacao da Operacao)
/// - `"210210"` (Ciencia da Operacao)
/// - `"210220"` (Desconhecimento da Operacao)
/// - `"210240"` (Operacao nao Realizada) - requires justification
///
/// # Arguments
///
/// * `access_key` - The 44-digit access key of the NF-e.
/// * `event_type` - Event type code as string (e.g. `"210210"`).
/// * `justification` - Required only for `"210240"` (operation not performed).
/// * `seq` - Event sequence number.
/// * `environment` - SEFAZ environment.
/// * `tax_id` - CNPJ or CPF of the recipient.
///
/// # Errors
///
/// This function does not return errors but may produce invalid XML if the
/// event type is not one of the valid manifest types.
pub fn build_manifesta_request(
    access_key: &str,
    event_type: &str,
    justification: Option<&str>,
    seq: u32,
    environment: SefazEnvironment,
    tax_id: &str,
) -> String {
    let tp_evento: u32 = event_type.parse().unwrap_or(0);

    let additional = if tp_evento == event_types::OPERATION_NOT_PERFORMED {
        match justification {
            Some(just) if !just.is_empty() => format!("<xJust>{just}</xJust>"),
            _ => String::new(),
        }
    } else {
        String::new()
    };

    // Manifestacao do destinatario always uses cOrgao=91 (Ambiente Nacional)
    // PHP: $this->sefazEvento('AN', ...) where UFList::getCodeByUF('AN') = 91
    build_event_xml_with_org(
        access_key,
        tp_evento,
        seq,
        tax_id,
        environment,
        &additional,
        Some("91"),
    )
}

/// Build a SEFAZ DistDFe (distribution) request XML (`<distDFeInt>`).
///
/// Queries the distribution of fiscal documents (DF-e) from the national
/// environment. Can search by last NSU, specific NSU, or access key.
///
/// # Arguments
///
/// * `uf` - State abbreviation of the interested party.
/// * `tax_id` - CNPJ or CPF of the interested party.
/// * `nsu` - Specific NSU or last NSU to query. If this is a 44-digit
///   all-numeric string, it is treated as an access key (`consChNFe`).
///   If `Some` with a 15-digit NSU, it uses `consNSU`.
///   If `None`, defaults to `distNSU` with `ultNSU=000000000000000`.
/// * `access_key` - Optional 44-digit access key for direct lookup.
/// * `environment` - SEFAZ environment.
///
/// # Panics
///
/// Panics if `uf` is not a valid Brazilian state code.
///
/// # Errors
///
/// This function panics on invalid state codes rather than returning `Result`.
pub fn build_dist_dfe_request(
    uf: &str,
    tax_id: &str,
    nsu: Option<&str>,
    access_key: Option<&str>,
    environment: SefazEnvironment,
) -> String {
    let cuf = get_state_code(uf).expect("Invalid state code");
    let tp_amb = environment.as_str();
    let tax_id_tag = tax_id_xml_tag(tax_id);

    let query_tag = if let Some(ch_nfe) = access_key {
        if ch_nfe.len() == 44 && ch_nfe.chars().all(|c| c.is_ascii_digit()) {
            format!("<consChNFe><chNFe>{ch_nfe}</chNFe></consChNFe>")
        } else {
            // Treat as specific NSU
            format!("<consNSU><NSU>{ch_nfe}</NSU></consNSU>")
        }
    } else if let Some(nsu_val) = nsu {
        if nsu_val == "000000000000000" || nsu_val.starts_with('0') {
            // ultNSU (last NSU for incremental distribution)
            format!("<distNSU><ultNSU>{nsu_val}</ultNSU></distNSU>")
        } else {
            // Specific NSU
            format!("<consNSU><NSU>{nsu_val}</NSU></consNSU>")
        }
    } else {
        "<distNSU><ultNSU>000000000000000</ultNSU></distNSU>".to_string()
    };

    format!(
        "<distDFeInt xmlns=\"{NFE_NAMESPACE}\" versao=\"1.01\"><tpAmb>{tp_amb}</tpAmb><cUFAutor>{cuf}</cUFAutor>{tax_id_tag}{query_tag}</distDFeInt>"
    )
}

/// Build a SEFAZ cadastro (taxpayer registration) query XML (`<ConsCad>`).
///
/// Queries the SEFAZ taxpayer registry for a given state, searching by
/// CNPJ, IE (state tax ID), or CPF.
///
/// # Arguments
///
/// * `uf` - State abbreviation to query.
/// * `search_type` - One of `"CNPJ"`, `"IE"`, or `"CPF"`.
/// * `search_value` - The document number to search for.
///
/// # Errors
///
/// This function does not return `Result` errors.
pub fn build_cadastro_request(uf: &str, search_type: &str, search_value: &str) -> String {
    let filter = match search_type {
        "CNPJ" => format!("<CNPJ>{search_value}</CNPJ>"),
        "IE" => format!("<IE>{search_value}</IE>"),
        "CPF" => format!("<CPF>{search_value}</CPF>"),
        _ => String::new(),
    };

    format!(
        "<ConsCad xmlns=\"{NFE_NAMESPACE}\" versao=\"2.00\"><infCons><xServ>CONS-CAD</xServ><UF>{uf}</UF>{filter}</infCons></ConsCad>"
    )
}

// ── Internal helpers ────────────────────────────────────────────────────────

/// Build a generic SEFAZ event XML (`<envEvento>` with a single `<evento>`).
///
/// This is the core event builder used by cancellation, CCe, manifestation,
/// and other event-type request builders.
///
/// When `org_code_override` is `Some`, the provided value is used as `cOrgao`
/// instead of deriving it from the access key. This is needed for manifestation
/// events which must use code 91 (Ambiente Nacional).
fn build_event_xml(
    access_key: &str,
    event_type: u32,
    seq: u32,
    tax_id: &str,
    environment: SefazEnvironment,
    additional_tags: &str,
) -> String {
    build_event_xml_with_org(
        access_key,
        event_type,
        seq,
        tax_id,
        environment,
        additional_tags,
        None,
    )
}

/// Build a generic SEFAZ event XML with an optional `cOrgao` override.
fn build_event_xml_with_org(
    access_key: &str,
    event_type: u32,
    seq: u32,
    tax_id: &str,
    environment: SefazEnvironment,
    additional_tags: &str,
    org_code_override: Option<&str>,
) -> String {
    let event_id = build_event_id(event_type, access_key, seq);
    let desc_evento = event_description(event_type);
    let tax_id_tag = tax_id_xml_tag(tax_id);
    let tp_amb = environment.as_str();
    // cOrgao: use override when provided (e.g. "91" for manifestacao),
    // otherwise derive from the first 2 digits of the access key.
    let org_code_owned;
    let org_code = match org_code_override {
        Some(code) => code,
        None => {
            org_code_owned = access_key[..2].to_string();
            &org_code_owned
        }
    };
    // Use current timestamp as lot ID (milliseconds)
    let lot_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis().to_string())
        .unwrap_or_else(|_| "1".to_string());
    // Event datetime with BRT offset (-03:00)
    let dh_evento = chrono::Utc::now()
        .with_timezone(&chrono::FixedOffset::west_opt(3 * 3600).unwrap())
        .format("%Y-%m-%dT%H:%M:%S%:z")
        .to_string();

    format!(
        "<envEvento xmlns=\"{NFE_NAMESPACE}\" versao=\"1.00\"><idLote>{lot_id}</idLote><evento xmlns=\"{NFE_NAMESPACE}\" versao=\"1.00\"><infEvento Id=\"{event_id}\"><cOrgao>{org_code}</cOrgao><tpAmb>{tp_amb}</tpAmb>{tax_id_tag}<chNFe>{access_key}</chNFe><dhEvento>{dh_evento}</dhEvento><tpEvento>{event_type}</tpEvento><nSeqEvento>{seq}</nSeqEvento><verEvento>1.00</verEvento><detEvento versao=\"1.00\"><descEvento>{desc_evento}</descEvento>{additional_tags}</detEvento></infEvento></evento></envEvento>"
    )
}

/// Validate that an access key is exactly 44 numeric digits.
fn validate_access_key(access_key: &str) {
    assert!(!access_key.is_empty(), "Access key is required");
    assert!(
        access_key.len() == 44,
        "Invalid access key: must be exactly 44 digits, got {}",
        access_key.len()
    );
    assert!(
        access_key.chars().all(|c| c.is_ascii_digit()),
        "Invalid access key: must contain only digits"
    );
}

/// Strip XML declaration (`<?xml ... ?>`) from a string.
fn strip_xml_declaration(xml: &str) -> &str {
    if let Some(start) = xml.find("<?xml") {
        if let Some(end) = xml[start..].find("?>") {
            let after = &xml[start + end + 2..];
            return after.trim_start();
        }
    }
    xml
}

#[cfg(test)]
mod tests {
    use super::*;

    // Synthetic 44-digit access key for tests (all zeros is fine for XML structure tests).
    const TEST_KEY: &str = "35240112345678000195550010000000011000000019";
    const TEST_CNPJ: &str = "12345678000195";
    const TEST_CPF: &str = "12345678901";

    // ── Fix #1: cOrgao = 91 for manifestacao ────────────────────────

    #[test]
    fn manifesta_request_uses_c_orgao_91() {
        let xml = build_manifesta_request(
            TEST_KEY,
            "210210",
            None,
            1,
            SefazEnvironment::Homologation,
            TEST_CNPJ,
        );
        assert!(
            xml.contains("<cOrgao>91</cOrgao>"),
            "Manifestacao must use cOrgao=91 (Ambiente Nacional), got: {xml}"
        );
    }

    #[test]
    fn manifesta_request_confirmation_has_correct_desc() {
        let xml = build_manifesta_request(
            TEST_KEY,
            "210200",
            None,
            1,
            SefazEnvironment::Homologation,
            TEST_CNPJ,
        );
        assert!(xml.contains("<descEvento>Confirmacao da Operacao</descEvento>"));
        assert!(xml.contains("<tpEvento>210200</tpEvento>"));
    }

    #[test]
    fn manifesta_request_operation_not_performed_includes_justification() {
        let xml = build_manifesta_request(
            TEST_KEY,
            "210240",
            Some("Motivo teste da operacao nao realizada"),
            1,
            SefazEnvironment::Homologation,
            TEST_CNPJ,
        );
        assert!(xml.contains("<xJust>Motivo teste da operacao nao realizada</xJust>"));
        assert!(xml.contains("<tpEvento>210240</tpEvento>"));
    }

    #[test]
    fn cancela_request_uses_c_orgao_from_access_key() {
        let xml = build_cancela_request(
            TEST_KEY,
            "135220000009921",
            "Erro na emissao da NF-e",
            1,
            SefazEnvironment::Homologation,
            TEST_CNPJ,
        );
        // First 2 digits of TEST_KEY = "35" (SP)
        assert!(
            xml.contains("<cOrgao>35</cOrgao>"),
            "Cancellation must use cOrgao from access key (35), got: {xml}"
        );
    }

    // ── Fix #3: CPF in inutilizacao ─────────────────────────────────

    #[test]
    fn inutilizacao_with_cnpj_uses_cnpj_tag() {
        let xml = build_inutilizacao_request(
            24,
            TEST_CNPJ,
            "55",
            1,
            1,
            10,
            "Pulo de numeracao",
            SefazEnvironment::Homologation,
            "SP",
        );
        assert!(
            xml.contains(&format!("<CNPJ>{TEST_CNPJ}</CNPJ>")),
            "Should use <CNPJ> tag for 14-digit tax ID"
        );
        assert!(!xml.contains("<CPF>"), "Should not contain <CPF> tag");
    }

    #[test]
    fn inutilizacao_with_cpf_uses_cpf_tag() {
        let xml = build_inutilizacao_request(
            24,
            TEST_CPF,
            "55",
            1,
            1,
            10,
            "Pulo de numeracao",
            SefazEnvironment::Homologation,
            "MT",
        );
        assert!(
            xml.contains(&format!("<CPF>{TEST_CPF}</CPF>")),
            "Should use <CPF> tag for 11-digit tax ID"
        );
        assert!(!xml.contains("<CNPJ>"), "Should not contain <CNPJ> tag");
    }

    // ── Fix #4: CNPJ/CPF padding in inutilizacao ID ────────────────

    #[test]
    fn inutilizacao_id_pads_cnpj_to_14_digits() {
        let xml = build_inutilizacao_request(
            24,
            TEST_CNPJ,
            "55",
            1,
            1,
            10,
            "Pulo de numeracao",
            SefazEnvironment::Homologation,
            "SP",
        );
        // Expected ID: ID + cUF(35) + year(24) + padded_cnpj(14 digits) + model(55) + serie(001) + ini(000000001) + fin(000000010)
        // CNPJ "12345678000195" is already 14 digits, no padding needed
        let expected_id = format!("ID3524{TEST_CNPJ}55001000000001000000010");
        assert!(
            xml.contains(&format!("Id=\"{expected_id}\"")),
            "ID should contain padded CNPJ (14 digits), expected {expected_id}, got:\n{xml}"
        );
    }

    #[test]
    fn inutilizacao_id_pads_cpf_to_14_digits() {
        let xml = build_inutilizacao_request(
            24,
            TEST_CPF,
            "55",
            1,
            1,
            10,
            "Pulo de numeracao",
            SefazEnvironment::Homologation,
            "MT",
        );
        // CPF "12345678901" padded to 14 = "00012345678901"
        let padded = format!("{:0>14}", TEST_CPF);
        let expected_id = format!("ID5124{padded}55001000000001000000010");
        assert!(
            xml.contains(&format!("Id=\"{expected_id}\"")),
            "ID should pad CPF to 14 digits, expected {expected_id}, got:\n{xml}"
        );
    }

    // ── DistDFe request ─────────────────────────────────────────────

    #[test]
    fn dist_dfe_request_with_ult_nsu() {
        let xml =
            build_dist_dfe_request("SP", TEST_CNPJ, None, None, SefazEnvironment::Homologation);
        assert!(xml.contains("<distNSU><ultNSU>000000000000000</ultNSU></distNSU>"));
        assert!(xml.contains(&format!("<CNPJ>{TEST_CNPJ}</CNPJ>")));
        assert!(xml.contains("<cUFAutor>35</cUFAutor>"));
    }

    #[test]
    fn dist_dfe_request_with_access_key() {
        let xml = build_dist_dfe_request(
            "SP",
            TEST_CNPJ,
            None,
            Some(TEST_KEY),
            SefazEnvironment::Homologation,
        );
        assert!(xml.contains(&format!("<consChNFe><chNFe>{TEST_KEY}</chNFe></consChNFe>")));
    }

    #[test]
    fn dist_dfe_request_with_cpf() {
        let xml =
            build_dist_dfe_request("SP", TEST_CPF, None, None, SefazEnvironment::Homologation);
        assert!(xml.contains(&format!("<CPF>{TEST_CPF}</CPF>")));
    }

    // ── Cadastro request ────────────────────────────────────────────

    #[test]
    fn cadastro_request_with_cnpj() {
        let xml = build_cadastro_request("SP", "CNPJ", TEST_CNPJ);
        assert!(xml.contains(&format!("<CNPJ>{TEST_CNPJ}</CNPJ>")));
        assert!(xml.contains("<UF>SP</UF>"));
        assert!(xml.contains("<xServ>CONS-CAD</xServ>"));
    }

    #[test]
    fn cadastro_request_with_cpf() {
        let xml = build_cadastro_request("MT", "CPF", TEST_CPF);
        assert!(xml.contains(&format!("<CPF>{TEST_CPF}</CPF>")));
        assert!(xml.contains("<UF>MT</UF>"));
    }

    #[test]
    fn cadastro_request_with_ie() {
        let xml = build_cadastro_request("SP", "IE", "123456789");
        assert!(xml.contains("<IE>123456789</IE>"));
    }

    // ── tax_id_xml_tag helper ───────────────────────────────────────

    #[test]
    fn tax_id_xml_tag_detects_cpf_and_cnpj() {
        assert_eq!(tax_id_xml_tag("12345678901"), "<CPF>12345678901</CPF>");
        assert_eq!(
            tax_id_xml_tag("12345678000195"),
            "<CNPJ>12345678000195</CNPJ>"
        );
    }

    // ── Event ID format ─────────────────────────────────────────────

    #[test]
    fn event_id_format() {
        let id = build_event_id(210210, TEST_KEY, 1);
        assert_eq!(id, format!("ID210210{TEST_KEY}01"));
    }

    #[test]
    fn event_id_seq_padding() {
        let id = build_event_id(110111, TEST_KEY, 3);
        assert_eq!(id, format!("ID110111{TEST_KEY}03"));
    }
}

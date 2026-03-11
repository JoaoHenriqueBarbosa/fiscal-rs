use fiscal_core::constants::{NFE_NAMESPACE, NFE_VERSION};
use fiscal_core::state_codes::get_state_code;
use fiscal_core::types::SefazEnvironment;

/// Event type constants matching the SEFAZ specification.
pub mod event_types {
    pub const CCE: u32 = 110110;
    pub const CANCELLATION: u32 = 110111;
    pub const CONFIRMATION: u32 = 210200;
    pub const AWARENESS: u32 = 210210;
    pub const UNKNOWN_OPERATION: u32 = 210220;
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
pub fn build_autorizacao_request(
    xml: &str,
    lot_id: &str,
    sync: bool,
    _compressed: bool,
) -> String {
    assert!(!xml.is_empty(), "XML content is required for authorization request");

    // Strip XML declaration if present
    let content = xml
        .trim()
        .trim_start_matches(|c: char| c != '<' || false);
    let stripped = strip_xml_declaration(content);

    let ind_sinc = if sync { "1" } else { "0" };

    format!(
        "<enviNFe xmlns=\"{NFE_NAMESPACE}\" versao=\"{NFE_VERSION}\">\
         <idLote>{lot_id}</idLote>\
         <indSinc>{ind_sinc}</indSinc>\
         {stripped}\
         </enviNFe>"
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
        "<consStatServ xmlns=\"{NFE_NAMESPACE}\" versao=\"{NFE_VERSION}\">\
         <tpAmb>{tp_amb}</tpAmb>\
         <cUF>{cuf}</cUF>\
         <xServ>STATUS</xServ>\
         </consStatServ>"
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
        "<consSitNFe xmlns=\"{NFE_NAMESPACE}\" versao=\"{NFE_VERSION}\">\
         <tpAmb>{tp_amb}</tpAmb>\
         <xServ>CONSULTAR</xServ>\
         <chNFe>{access_key}</chNFe>\
         </consSitNFe>"
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
pub fn build_consulta_recibo_request(
    receipt: &str,
    environment: SefazEnvironment,
) -> String {
    assert!(!receipt.is_empty(), "Receipt number (recibo) is required");
    let tp_amb = environment.as_str();

    format!(
        "<consReciNFe xmlns=\"{NFE_NAMESPACE}\" versao=\"{NFE_VERSION}\">\
         <tpAmb>{tp_amb}</tpAmb>\
         <nRec>{receipt}</nRec>\
         </consReciNFe>"
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

    let id = format!(
        "ID{cuf}{year:02}{tax_id}{model:0>2}{series:03}{start_number:09}{end_number:09}"
    );

    format!(
        "<inutNFe xmlns=\"{NFE_NAMESPACE}\" versao=\"{NFE_VERSION}\">\
         <infInut Id=\"{id}\">\
         <tpAmb>{tp_amb}</tpAmb>\
         <xServ>INUTILIZAR</xServ>\
         <cUF>{cuf}</cUF>\
         <ano>{year:02}</ano>\
         <CNPJ>{tax_id}</CNPJ>\
         <mod>{model}</mod>\
         <serie>{series}</serie>\
         <nNFIni>{start_number}</nNFIni>\
         <nNFFin>{end_number}</nNFFin>\
         <xJust>{justification}</xJust>\
         </infInut>\
         </inutNFe>"
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

    let additional = format!(
        "<nProt>{protocol}</nProt><xJust>{justification}</xJust>"
    );

    build_event_xml(
        access_key,
        event_types::CANCELLATION,
        seq,
        tax_id,
        "91",
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

    let additional = format!(
        "<xCorrecao>{correction}</xCorrecao><xCondUso>{x_cond_uso}</xCondUso>"
    );

    build_event_xml(
        access_key,
        event_types::CCE,
        seq,
        tax_id,
        "91",
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

    build_event_xml(
        access_key,
        tp_evento,
        seq,
        tax_id,
        "91",
        environment,
        &additional,
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
        "<distDFeInt xmlns=\"{NFE_NAMESPACE}\" versao=\"1.01\">\
         <tpAmb>{tp_amb}</tpAmb>\
         <cUFAutor>{cuf}</cUFAutor>\
         {tax_id_tag}\
         {query_tag}\
         </distDFeInt>"
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
pub fn build_cadastro_request(
    uf: &str,
    search_type: &str,
    search_value: &str,
) -> String {
    let filter = match search_type {
        "CNPJ" => format!("<CNPJ>{search_value}</CNPJ>"),
        "IE" => format!("<IE>{search_value}</IE>"),
        "CPF" => format!("<CPF>{search_value}</CPF>"),
        _ => String::new(),
    };

    format!(
        "<ConsCad xmlns=\"{NFE_NAMESPACE}\" versao=\"2.00\">\
         <infCons>\
         <xServ>CONS-CAD</xServ>\
         <UF>{uf}</UF>\
         {filter}\
         </infCons>\
         </ConsCad>"
    )
}

// ── Internal helpers ────────────────────────────────────────────────────────

/// Build a generic SEFAZ event XML (`<envEvento>` with a single `<evento>`).
///
/// This is the core event builder used by cancellation, CCe, manifestation,
/// and other event-type request builders.
fn build_event_xml(
    access_key: &str,
    event_type: u32,
    seq: u32,
    tax_id: &str,
    org_code: &str,
    environment: SefazEnvironment,
    additional_tags: &str,
) -> String {
    let event_id = build_event_id(event_type, access_key, seq);
    let desc_evento = event_description(event_type);
    let tax_id_tag = tax_id_xml_tag(tax_id);
    let tp_amb = environment.as_str();
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
        "<envEvento xmlns=\"{NFE_NAMESPACE}\" versao=\"1.00\">\
         <idLote>{lot_id}</idLote>\
         <evento xmlns=\"{NFE_NAMESPACE}\" versao=\"1.00\">\
         <infEvento Id=\"{event_id}\">\
         <cOrgao>{org_code}</cOrgao>\
         <tpAmb>{tp_amb}</tpAmb>\
         {tax_id_tag}\
         <chNFe>{access_key}</chNFe>\
         <dhEvento>{dh_evento}</dhEvento>\
         <tpEvento>{event_type}</tpEvento>\
         <nSeqEvento>{seq}</nSeqEvento>\
         <verEvento>1.00</verEvento>\
         <detEvento versao=\"1.00\">\
         <descEvento>{desc_evento}</descEvento>\
         {additional_tags}\
         </detEvento>\
         </infEvento>\
         </evento>\
         </envEvento>"
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

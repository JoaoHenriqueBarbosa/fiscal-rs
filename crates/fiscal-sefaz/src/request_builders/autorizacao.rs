use fiscal_core::constants::{NFE_NAMESPACE, NFE_VERSION};
use fiscal_core::state_codes::get_state_code;
use fiscal_core::types::SefazEnvironment;

use super::helpers::{strip_xml_declaration, tax_id_xml_tag, validate_access_key};

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

/// Build a SEFAZ batch authorization request XML (`<enviNFe>`) for multiple NF-e documents.
///
/// Wraps one or more signed NF-e XML documents in a single `<enviNFe>` envelope
/// for batch submission to the SEFAZ authorization web service.
///
/// When `ind_sinc` is `0` (asynchronous), SEFAZ returns a receipt number (`nRec`)
/// that must be polled via [`build_consulta_recibo_request`] to obtain the result.
/// When `ind_sinc` is `1` (synchronous), only a single NF-e is allowed per the
/// MOC specification.
///
/// This matches the PHP `sefazEnviaLote()` method from `Tools.php`.
///
/// # Arguments
///
/// * `xmls` - Slice of signed NF-e XML strings. Each one is stripped of its
///   XML declaration automatically.
/// * `id_lote` - Lot identifier for the submission batch.
/// * `ind_sinc` - Synchronous indicator: `0` for asynchronous (batch),
///   `1` for synchronous (single document).
///
/// # Errors
///
/// Returns [`FiscalError::InvalidTaxData`] if:
/// - `xmls` is empty
/// - `xmls` has more than 50 documents
/// - `ind_sinc` is `1` but `xmls` has more than 1 document
pub fn build_autorizacao_batch_request(
    xmls: &[&str],
    id_lote: &str,
    ind_sinc: u8,
) -> Result<String, fiscal_core::FiscalError> {
    if xmls.is_empty() {
        return Err(fiscal_core::FiscalError::InvalidTaxData(
            "At least one NF-e XML is required for batch authorization".into(),
        ));
    }
    if xmls.len() > 50 {
        return Err(fiscal_core::FiscalError::InvalidTaxData(format!(
            "Batch authorization accepts at most 50 NF-e documents, got {}",
            xmls.len()
        )));
    }
    if ind_sinc == 1 && xmls.len() > 1 {
        return Err(fiscal_core::FiscalError::InvalidTaxData(format!(
            "Synchronous mode (indSinc=1) accepts only 1 NF-e, got {}",
            xmls.len()
        )));
    }

    let mut nfe_concat = String::new();
    for xml in xmls {
        let content = xml.trim().trim_start_matches(|c: char| c != '<');
        let stripped = strip_xml_declaration(content);
        nfe_concat.push_str(stripped);
    }

    Ok(format!(
        "<enviNFe xmlns=\"{NFE_NAMESPACE}\" versao=\"{NFE_VERSION}\"><idLote>{id_lote}</idLote><indSinc>{ind_sinc}</indSinc>{nfe_concat}</enviNFe>"
    ))
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

/// Build a SEFAZ CSC (Código de Segurança do Contribuinte) request XML
/// (`<admCscNFCe>`).
///
/// Manages the CSC for NFC-e (model 65). Used exclusively with the
/// `CscNFCe` web service.
///
/// # Arguments
///
/// * `environment` — SEFAZ environment (production or homologation).
/// * `ind_op` — Operation type: 1=query active CSCs, 2=request new CSC,
///   3=revoke active CSC.
/// * `cnpj` — Full CNPJ of the taxpayer (14 digits).
/// * `csc_id` — CSC identifier (required only for `ind_op=3`).
/// * `csc_code` — CSC code/value (required only for `ind_op=3`).
pub fn build_csc_request(
    environment: SefazEnvironment,
    ind_op: u8,
    cnpj: &str,
    csc_id: Option<&str>,
    csc_code: Option<&str>,
) -> String {
    let tp_amb = environment.as_str();
    let digits: String = cnpj.chars().filter(|c| c.is_ascii_digit()).collect();
    // raizCNPJ = first 8 digits of the CNPJ
    let raiz_cnpj = if digits.len() >= 8 {
        &digits[..8]
    } else {
        &digits
    };

    if ind_op == 3 {
        let id_csc = csc_id.unwrap_or("");
        let codigo_csc = csc_code.unwrap_or("");
        format!(
            "<admCscNFCe versao=\"1.00\" xmlns=\"{NFE_NAMESPACE}\">\
             <tpAmb>{tp_amb}</tpAmb>\
             <indOp>{ind_op}</indOp>\
             <raizCNPJ>{raiz_cnpj}</raizCNPJ>\
             <dadosCsc>\
             <idCsc>{id_csc}</idCsc>\
             <codigoCsc>{codigo_csc}</codigoCsc>\
             </dadosCsc>\
             </admCscNFCe>"
        )
    } else {
        format!(
            "<admCscNFCe versao=\"1.00\" xmlns=\"{NFE_NAMESPACE}\">\
             <tpAmb>{tp_amb}</tpAmb>\
             <indOp>{ind_op}</indOp>\
             <raizCNPJ>{raiz_cnpj}</raizCNPJ>\
             </admCscNFCe>"
        )
    }
}

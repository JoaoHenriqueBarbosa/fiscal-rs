use fiscal_core::constants::{NFE_NAMESPACE, NFE_VERSION};
use fiscal_core::state_codes::get_state_code;
use fiscal_core::types::SefazEnvironment;
use fiscal_core::xml_utils::extract_xml_tag_value;

use super::event_core::event_types;
use super::helpers::{build_event_xml_with_org, extract_section};

/// Data extracted from an NF-e XML for building an EPEC event request.
///
/// All fields are extracted from the signed NF-e XML. The struct is used
/// as input to [`build_epec_request`] to avoid a long parameter list.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpecData {
    /// 44-digit NF-e access key (from `infNFe@Id`, without "NFe" prefix).
    pub access_key: String,
    /// IBGE code of the issuer's state (first 2 digits of the access key).
    pub c_orgao_autor: String,
    /// Application version string (from `<verProc>` or caller override).
    pub ver_aplic: String,
    /// Emission date-time (from `<dhEmi>`).
    pub dh_emi: String,
    /// Fiscal operation type (from `<tpNF>`): 0=entrada, 1=saída.
    pub tp_nf: String,
    /// Issuer's state tax registration (from `<emit><IE>`).
    pub emit_ie: String,
    /// Recipient's state abbreviation (from `<dest><UF>`).
    pub dest_uf: String,
    /// Recipient's tax ID XML fragment: `<CNPJ>...</CNPJ>`, `<CPF>...</CPF>`,
    /// or `<idEstrangeiro>...</idEstrangeiro>`.
    pub dest_id_tag: String,
    /// Recipient's state tax registration (from `<dest><IE>`), if any.
    pub dest_ie: Option<String>,
    /// Total NF-e value (from `<total><ICMSTot><vNF>`).
    pub v_nf: String,
    /// Total ICMS value (from `<total><ICMSTot><vICMS>`).
    pub v_icms: String,
    /// Total ICMS-ST value (from `<total><ICMSTot><vST>`).
    pub v_st: String,
    /// CNPJ or CPF of the issuer (for the event envelope).
    pub tax_id: String,
}

/// Extract [`EpecData`] from a signed NF-e XML string.
///
/// Parses the XML to extract all fields needed by the EPEC event. The
/// `ver_aplic_override` parameter, when `Some`, overrides the `<verProc>`
/// value from the XML (matching the PHP behavior where `$this->verAplic`
/// can override).
///
/// # Errors
///
/// Returns [`fiscal_core::FiscalError::XmlParsing`] if required tags are missing.
pub fn extract_epec_data(
    nfe_xml: &str,
    ver_aplic_override: Option<&str>,
) -> Result<EpecData, fiscal_core::FiscalError> {
    use fiscal_core::FiscalError;

    // Extract access key from infNFe@Id
    let inf_nfe_start = nfe_xml
        .find("<infNFe")
        .ok_or_else(|| FiscalError::XmlParsing("Missing <infNFe> in NF-e XML".into()))?;
    let inf_nfe_header_end = nfe_xml[inf_nfe_start..]
        .find('>')
        .ok_or_else(|| FiscalError::XmlParsing("Malformed <infNFe> tag".into()))?
        + inf_nfe_start;
    let inf_nfe_header = &nfe_xml[inf_nfe_start..inf_nfe_header_end];

    let id_pattern = "Id=\"";
    let id_start = inf_nfe_header
        .find(id_pattern)
        .ok_or_else(|| FiscalError::XmlParsing("Missing Id attribute in <infNFe>".into()))?
        + id_pattern.len();
    let id_end = inf_nfe_header[id_start..]
        .find('"')
        .ok_or_else(|| FiscalError::XmlParsing("Malformed Id attribute in <infNFe>".into()))?
        + id_start;
    let id_value = &inf_nfe_header[id_start..id_end];

    let access_key = id_value.strip_prefix("NFe").unwrap_or(id_value).to_string();
    if access_key.len() != 44 {
        return Err(FiscalError::XmlParsing(format!(
            "Invalid access key length: expected 44, got {}",
            access_key.len()
        )));
    }

    let c_orgao_autor = access_key[..2].to_string();

    // Extract emit section for IE
    let emit_section = extract_section(nfe_xml, "emit")
        .ok_or_else(|| FiscalError::XmlParsing("Missing <emit> section in NF-e XML".into()))?;
    let emit_ie = extract_xml_tag_value(&emit_section, "IE")
        .ok_or_else(|| FiscalError::XmlParsing("Missing <IE> in <emit>".into()))?;

    // Extract tax_id from emit (CNPJ or CPF)
    let tax_id = extract_xml_tag_value(&emit_section, "CNPJ")
        .or_else(|| extract_xml_tag_value(&emit_section, "CPF"))
        .ok_or_else(|| FiscalError::XmlParsing("Missing CNPJ/CPF in <emit>".into()))?;

    // Extract dest section
    let dest_section = extract_section(nfe_xml, "dest")
        .ok_or_else(|| FiscalError::XmlParsing("Missing <dest> section in NF-e XML".into()))?;
    let dest_uf = extract_xml_tag_value(&dest_section, "UF")
        .ok_or_else(|| FiscalError::XmlParsing("Missing <UF> in <dest>".into()))?;

    // Dest ID: try CNPJ, then CPF, then idEstrangeiro (same order as PHP)
    let dest_id_tag = if let Some(cnpj) = extract_xml_tag_value(&dest_section, "CNPJ") {
        format!("<CNPJ>{cnpj}</CNPJ>")
    } else if let Some(cpf) = extract_xml_tag_value(&dest_section, "CPF") {
        format!("<CPF>{cpf}</CPF>")
    } else if let Some(id_est) = extract_xml_tag_value(&dest_section, "idEstrangeiro") {
        format!("<idEstrangeiro>{id_est}</idEstrangeiro>")
    } else {
        return Err(FiscalError::XmlParsing(
            "Missing CNPJ/CPF/idEstrangeiro in <dest>".into(),
        ));
    };

    // Dest IE (optional)
    let dest_ie = extract_xml_tag_value(&dest_section, "IE").filter(|v| !v.is_empty());

    // Extract total section
    let total_section = extract_section(nfe_xml, "total")
        .ok_or_else(|| FiscalError::XmlParsing("Missing <total> section in NF-e XML".into()))?;
    let v_nf = extract_xml_tag_value(&total_section, "vNF")
        .ok_or_else(|| FiscalError::XmlParsing("Missing <vNF> in <total>".into()))?;
    let v_icms = extract_xml_tag_value(&total_section, "vICMS")
        .ok_or_else(|| FiscalError::XmlParsing("Missing <vICMS> in <total>".into()))?;
    let v_st = extract_xml_tag_value(&total_section, "vST")
        .ok_or_else(|| FiscalError::XmlParsing("Missing <vST> in <total>".into()))?;

    // Other fields
    let ver_proc = extract_xml_tag_value(nfe_xml, "verProc").unwrap_or_default();
    let ver_aplic = match ver_aplic_override {
        Some(v) if !v.is_empty() => v.to_string(),
        _ => ver_proc,
    };

    let dh_emi = extract_xml_tag_value(nfe_xml, "dhEmi")
        .ok_or_else(|| FiscalError::XmlParsing("Missing <dhEmi> in NF-e XML".into()))?;
    let tp_nf = extract_xml_tag_value(nfe_xml, "tpNF")
        .ok_or_else(|| FiscalError::XmlParsing("Missing <tpNF> in NF-e XML".into()))?;

    Ok(EpecData {
        access_key,
        c_orgao_autor,
        ver_aplic,
        dh_emi,
        tp_nf,
        emit_ie,
        dest_uf,
        dest_id_tag,
        dest_ie,
        v_nf,
        v_icms,
        v_st,
        tax_id,
    })
}

/// Build a SEFAZ EPEC (Evento Prévio de Emissão em Contingência) event
/// request XML (`tpEvento=110140`).
///
/// The EPEC event is sent to the Ambiente Nacional (AN) with `cOrgao`
/// set to the IBGE code of the issuer's state. This matches the PHP
/// `Tools::sefazEPEC()` behavior.
///
/// # Arguments
///
/// * `epec_data` - Pre-extracted NF-e data (see [`extract_epec_data`]).
/// * `environment` - SEFAZ environment (production or homologation).
///
/// # Example
///
/// ```no_run
/// use fiscal_sefaz::request_builders::{build_epec_request, extract_epec_data};
/// use fiscal_core::types::SefazEnvironment;
///
/// let nfe_xml = "...signed NF-e XML...";
/// let data = extract_epec_data(nfe_xml, None).unwrap();
/// let request = build_epec_request(&data, SefazEnvironment::Homologation);
/// ```
pub fn build_epec_request(epec_data: &EpecData, environment: SefazEnvironment) -> String {
    // Build the EPEC-specific additional tags (detEvento content after descEvento)
    let dest_ie_tag = match &epec_data.dest_ie {
        Some(ie) => format!("<IE>{ie}</IE>"),
        None => String::new(),
    };

    let additional = format!(
        "<cOrgaoAutor>{c_orgao}</cOrgaoAutor>\
         <tpAutor>1</tpAutor>\
         <verAplic>{ver_aplic}</verAplic>\
         <dhEmi>{dh_emi}</dhEmi>\
         <tpNF>{tp_nf}</tpNF>\
         <IE>{emit_ie}</IE>\
         <dest>\
         <UF>{dest_uf}</UF>\
         {dest_id}\
         {dest_ie}\
         <vNF>{v_nf}</vNF>\
         <vICMS>{v_icms}</vICMS>\
         <vST>{v_st}</vST>\
         </dest>",
        c_orgao = epec_data.c_orgao_autor,
        ver_aplic = epec_data.ver_aplic,
        dh_emi = epec_data.dh_emi,
        tp_nf = epec_data.tp_nf,
        emit_ie = epec_data.emit_ie,
        dest_uf = epec_data.dest_uf,
        dest_id = epec_data.dest_id_tag,
        dest_ie = dest_ie_tag,
        v_nf = epec_data.v_nf,
        v_icms = epec_data.v_icms,
        v_st = epec_data.v_st,
    );

    // EPEC always goes to AN (cOrgao in the evento envelope = IBGE code of issuer's UF)
    // PHP: $this->sefazEvento('AN', $chNFe, self::EVT_EPEC, 1, $tagAdic, null, null)
    // In sefazEvento, when uf='AN', cOrgao = UFList::getCodeByUF('AN') = 91
    // But $ignore = $tpEvento == self::EVT_EPEC skips the servico() call's UF validation
    // The actual cOrgao in the evento XML is derived from the access key's first 2 digits
    // when not in the special list, BUT for EPEC the PHP calls:
    //   $cOrgao = UFList::getCodeByUF($uf)  where $uf = 'AN'
    // So cOrgao = 91 for EPEC events.
    build_event_xml_with_org(
        &epec_data.access_key,
        event_types::EPEC,
        1, // nSeqEvento = 1 always for EPEC
        &epec_data.tax_id,
        environment,
        &additional,
        Some("91"), // AN = 91
    )
}

/// Build a SEFAZ EPEC NFC-e status request XML (`<consStatServ>`).
///
/// Queries the operational status of the EPEC NFC-e service. This service
/// exists only for SP (São Paulo) and model 65 (NFC-e), matching the PHP
/// `sefazStatusEpecNfce()` method from `TraitEPECNfce`.
///
/// # Arguments
///
/// * `uf` - State abbreviation (must be `"SP"`).
/// * `environment` - SEFAZ environment (production or homologation).
///
/// # Panics
///
/// Panics if `uf` is not a valid Brazilian state code.
pub fn build_epec_nfce_status_request(uf: &str, environment: SefazEnvironment) -> String {
    let cuf = get_state_code(uf).expect("Invalid state code");
    let tp_amb = environment.as_str();

    format!(
        "<consStatServ xmlns=\"{NFE_NAMESPACE}\" versao=\"{NFE_VERSION}\"><tpAmb>{tp_amb}</tpAmb><cUF>{cuf}</cUF><xServ>STATUS</xServ></consStatServ>"
    )
}

/// Data extracted from an NFC-e XML for building an EPEC NFC-e event request.
///
/// Similar to [`EpecData`] but adapted for NFC-e EPEC (SP only). Key
/// differences from the regular EPEC:
/// - No `vST` field (NFC-e EPEC does not include ICMS-ST value)
/// - Destination section is optional (NFC-e can have no recipient)
/// - Event is sent to the state's EPEC endpoint, not Ambiente Nacional
///
/// All fields are extracted from the signed NFC-e XML. Used as input to
/// [`build_epec_nfce_request`].
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpecNfceData {
    /// 44-digit NFC-e access key (from `infNFe@Id`, without "NFe" prefix).
    pub access_key: String,
    /// IBGE code of the issuer's state (first 2 digits of the access key).
    pub c_orgao_autor: String,
    /// Application version string (from `<verProc>` or caller override).
    pub ver_aplic: String,
    /// Emission date-time (from `<dhEmi>`).
    pub dh_emi: String,
    /// Fiscal operation type (from `<tpNF>`): 0=entrada, 1=saída.
    pub tp_nf: String,
    /// Issuer's state tax registration (from `<emit><IE>`).
    pub emit_ie: String,
    /// Recipient's state abbreviation (from `<dest><UF>`), if present.
    pub dest_uf: Option<String>,
    /// Recipient's tax ID XML fragment: `<CNPJ>...</CNPJ>`, `<CPF>...</CPF>`,
    /// or `<idEstrangeiro>...</idEstrangeiro>`. `None` if no recipient.
    pub dest_id_tag: Option<String>,
    /// Recipient's state tax registration (from `<dest><IE>`), if any.
    pub dest_ie: Option<String>,
    /// Total NF-e value (from `<total><ICMSTot><vNF>`).
    pub v_nf: String,
    /// Total ICMS value (from `<total><ICMSTot><vICMS>`).
    pub v_icms: String,
    /// CNPJ or CPF of the issuer (for the event envelope).
    pub tax_id: String,
}

/// Extract [`EpecNfceData`] from a signed NFC-e XML string.
///
/// Parses the XML to extract all fields needed by the EPEC NFC-e event.
/// Unlike [`extract_epec_data`], the destination section is optional (NFC-e
/// can be issued without a recipient) and `vST` is not extracted.
///
/// # Errors
///
/// Returns [`fiscal_core::FiscalError::XmlParsing`] if required tags are missing.
pub fn extract_epec_nfce_data(
    nfce_xml: &str,
    ver_aplic_override: Option<&str>,
) -> Result<EpecNfceData, fiscal_core::FiscalError> {
    use fiscal_core::FiscalError;

    // Extract access key from infNFe@Id
    let inf_nfe_start = nfce_xml
        .find("<infNFe")
        .ok_or_else(|| FiscalError::XmlParsing("Missing <infNFe> in NFC-e XML".into()))?;
    let inf_nfe_header_end = nfce_xml[inf_nfe_start..]
        .find('>')
        .ok_or_else(|| FiscalError::XmlParsing("Malformed <infNFe> tag".into()))?
        + inf_nfe_start;
    let inf_nfe_header = &nfce_xml[inf_nfe_start..inf_nfe_header_end];

    let id_pattern = "Id=\"";
    let id_start = inf_nfe_header
        .find(id_pattern)
        .ok_or_else(|| FiscalError::XmlParsing("Missing Id attribute in <infNFe>".into()))?
        + id_pattern.len();
    let id_end = inf_nfe_header[id_start..]
        .find('"')
        .ok_or_else(|| FiscalError::XmlParsing("Malformed Id attribute in <infNFe>".into()))?
        + id_start;
    let id_value = &inf_nfe_header[id_start..id_end];

    let access_key = id_value.strip_prefix("NFe").unwrap_or(id_value).to_string();
    if access_key.len() != 44 {
        return Err(FiscalError::XmlParsing(format!(
            "Invalid access key length: expected 44, got {}",
            access_key.len()
        )));
    }

    let c_orgao_autor = access_key[..2].to_string();

    // Extract emit section for IE and tax_id
    let emit_section = extract_section(nfce_xml, "emit")
        .ok_or_else(|| FiscalError::XmlParsing("Missing <emit> section in NFC-e XML".into()))?;
    let emit_ie = extract_xml_tag_value(&emit_section, "IE")
        .ok_or_else(|| FiscalError::XmlParsing("Missing <IE> in <emit>".into()))?;

    let tax_id = extract_xml_tag_value(&emit_section, "CNPJ")
        .or_else(|| extract_xml_tag_value(&emit_section, "CPF"))
        .ok_or_else(|| FiscalError::XmlParsing("Missing CNPJ/CPF in <emit>".into()))?;

    // Extract dest section (optional for NFC-e)
    let dest_section = extract_section(nfce_xml, "dest");
    let (dest_uf, dest_id_tag, dest_ie) = if let Some(ref dest) = dest_section {
        let uf = extract_xml_tag_value(dest, "UF");

        let id_tag = if let Some(cnpj) = extract_xml_tag_value(dest, "CNPJ") {
            Some(format!("<CNPJ>{cnpj}</CNPJ>"))
        } else if let Some(cpf) = extract_xml_tag_value(dest, "CPF") {
            Some(format!("<CPF>{cpf}</CPF>"))
        } else {
            extract_xml_tag_value(dest, "idEstrangeiro")
                .map(|id_est| format!("<idEstrangeiro>{id_est}</idEstrangeiro>"))
        };

        let ie = extract_xml_tag_value(dest, "IE").filter(|v| !v.is_empty());

        (uf, id_tag, ie)
    } else {
        (None, None, None)
    };

    // Extract total section
    let total_section = extract_section(nfce_xml, "total")
        .ok_or_else(|| FiscalError::XmlParsing("Missing <total> section in NFC-e XML".into()))?;
    let v_nf = extract_xml_tag_value(&total_section, "vNF")
        .ok_or_else(|| FiscalError::XmlParsing("Missing <vNF> in <total>".into()))?;
    let v_icms = extract_xml_tag_value(&total_section, "vICMS")
        .ok_or_else(|| FiscalError::XmlParsing("Missing <vICMS> in <total>".into()))?;

    // Other fields
    let ver_proc = extract_xml_tag_value(nfce_xml, "verProc").unwrap_or_default();
    let ver_aplic = match ver_aplic_override {
        Some(v) if !v.is_empty() => v.to_string(),
        _ => ver_proc,
    };

    let dh_emi = extract_xml_tag_value(nfce_xml, "dhEmi")
        .ok_or_else(|| FiscalError::XmlParsing("Missing <dhEmi> in NFC-e XML".into()))?;
    let tp_nf = extract_xml_tag_value(nfce_xml, "tpNF")
        .ok_or_else(|| FiscalError::XmlParsing("Missing <tpNF> in NFC-e XML".into()))?;

    Ok(EpecNfceData {
        access_key,
        c_orgao_autor,
        ver_aplic,
        dh_emi,
        tp_nf,
        emit_ie,
        dest_uf,
        dest_id_tag,
        dest_ie,
        v_nf,
        v_icms,
        tax_id,
    })
}

/// Build a SEFAZ EPEC NFC-e event request XML (`tpEvento=110140`).
///
/// Builds the complete `<envEvento>` wrapper for an EPEC event specific to
/// NFC-e (model 65). This is only available in SP and differs from the
/// standard EPEC in several ways:
///
/// - Sent to the state's `RecepcaoEPEC` endpoint (not Ambiente Nacional)
/// - `cOrgao` is the state's IBGE code (not 91)
/// - No `<vST>` field in the event detail
/// - The `<dest>` section is optional (NFC-e can have no recipient)
///
/// Matches the PHP `sefazEpecNfce()` method from `TraitEPECNfce`.
///
/// # Arguments
///
/// * `epec_data` - Pre-extracted NFC-e data (see [`extract_epec_nfce_data`]).
/// * `environment` - SEFAZ environment (production or homologation).
pub fn build_epec_nfce_request(epec_data: &EpecNfceData, environment: SefazEnvironment) -> String {
    // Build the optional dest section
    let dest_tag = if let Some(ref dest_id) = epec_data.dest_id_tag {
        let dest_uf = epec_data
            .dest_uf
            .as_deref()
            .unwrap_or(&epec_data.c_orgao_autor);
        let dest_ie_tag = match &epec_data.dest_ie {
            Some(ie) => format!("<IE>{ie}</IE>"),
            None => String::new(),
        };
        format!("<dest><UF>{dest_uf}</UF>{dest_id}{dest_ie_tag}</dest>")
    } else {
        String::new()
    };

    let additional = format!(
        "<cOrgaoAutor>{c_orgao}</cOrgaoAutor>\
         <tpAutor>1</tpAutor>\
         <verAplic>{ver_aplic}</verAplic>\
         <dhEmi>{dh_emi}</dhEmi>\
         <tpNF>{tp_nf}</tpNF>\
         <IE>{emit_ie}</IE>\
         {dest_tag}\
         <vNF>{v_nf}</vNF>\
         <vICMS>{v_icms}</vICMS>",
        c_orgao = epec_data.c_orgao_autor,
        ver_aplic = epec_data.ver_aplic,
        dh_emi = epec_data.dh_emi,
        tp_nf = epec_data.tp_nf,
        emit_ie = epec_data.emit_ie,
        v_nf = epec_data.v_nf,
        v_icms = epec_data.v_icms,
    );

    // EPEC NFC-e goes to the state endpoint, NOT AN.
    // cOrgao in the event envelope = IBGE code of the issuer's state.
    build_event_xml_with_org(
        &epec_data.access_key,
        event_types::EPEC,
        1, // nSeqEvento = 1 always for EPEC
        &epec_data.tax_id,
        environment,
        &additional,
        Some(&epec_data.c_orgao_autor),
    )
}

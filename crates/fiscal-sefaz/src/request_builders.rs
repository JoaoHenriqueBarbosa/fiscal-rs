use fiscal_core::constants::{NFE_NAMESPACE, NFE_VERSION};
use fiscal_core::state_codes::get_state_code;
use fiscal_core::types::SefazEnvironment;
use fiscal_core::xml_utils::extract_xml_tag_value;

/// Event type constants (`tpEvento`) matching the SEFAZ specification.
pub mod event_types {
    /// Carta de Correção Eletrônica (CC-e), code `110110`.
    pub const CCE: u32 = 110110;
    /// Cancellation event, code `110111`.
    pub const CANCELLATION: u32 = 110111;
    /// Cancelamento por substituição (NFC-e only), code `110112`.
    pub const CANCEL_SUBSTITUICAO: u32 = 110112;
    /// Comprovante de entrega da NF-e, code `110130`.
    pub const COMPROVANTE_ENTREGA: u32 = 110130;
    /// Cancelamento do comprovante de entrega da NF-e, code `110131`.
    pub const CANCEL_COMPROVANTE_ENTREGA: u32 = 110131;
    /// Ator interessado na NF-e, code `110150`.
    pub const ATOR_INTERESSADO: u32 = 110150;
    /// Insucesso na entrega da NF-e, code `110192`.
    pub const INSUCESSO_ENTREGA: u32 = 110192;
    /// Cancelamento do insucesso na entrega da NF-e, code `110193`.
    pub const CANCEL_INSUCESSO_ENTREGA: u32 = 110193;
    /// Recipient confirms operation, code `210200`.
    pub const CONFIRMATION: u32 = 210200;
    /// Recipient is aware of operation, code `210210`.
    pub const AWARENESS: u32 = 210210;
    /// Recipient does not recognize the operation, code `210220`.
    pub const UNKNOWN_OPERATION: u32 = 210220;
    /// Recipient confirms operation was not performed, code `210240`.
    pub const OPERATION_NOT_PERFORMED: u32 = 210240;
    /// EPEC — Evento Prévio de Emissão em Contingência, code `110140`.
    pub const EPEC: u32 = 110140;
    /// Pedido de prorrogação ICMS — 1.o prazo, code `111500`.
    pub const PRORROGACAO_1: u32 = 111500;
    /// Pedido de prorrogação ICMS — 2.o prazo, code `111501`.
    pub const PRORROGACAO_2: u32 = 111501;
    /// Cancelamento de pedido de prorrogação ICMS — 1.o prazo, code `111502`.
    pub const CANCEL_PRORROGACAO_1: u32 = 111502;
    /// Cancelamento de pedido de prorrogação ICMS — 2.o prazo, code `111503`.
    pub const CANCEL_PRORROGACAO_2: u32 = 111503;
    /// Conciliação financeira, code `110750`.
    pub const CONCILIACAO: u32 = 110750;
    /// Cancelamento de conciliação financeira, code `110751`.
    pub const CANCEL_CONCILIACAO: u32 = 110751;

    // ── RTC (Reforma Tributária Complementar) events ────────────────────

    /// Cancelamento de evento RTC, code `110001`.
    pub const RTC_CANCELA_EVENTO: u32 = 110001;
    /// Info pagamento integral (crédito presumido), code `112110`.
    pub const RTC_INFO_PAGTO_INTEGRAL: u32 = 112110;
    /// Importação ZFM não convertida em isenção, code `112120`.
    pub const RTC_IMPORTACAO_ZFM: u32 = 112120;
    /// Perecimento/roubo transporte fornecedor, code `112130`.
    pub const RTC_ROUBO_PERDA_FORNECEDOR: u32 = 112130;
    /// Fornecimento não realizado (pagamento antecipado), code `112140`.
    pub const RTC_FORNECIMENTO_NAO_REALIZADO: u32 = 112140;
    /// Atualização data previsão entrega, code `112150`.
    pub const RTC_ATUALIZACAO_DATA_ENTREGA: u32 = 112150;
    /// Solicitação apropriação crédito presumido, code `211110`.
    pub const RTC_SOL_APROP_CRED_PRESUMIDO: u32 = 211110;
    /// Destinação consumo pessoal, code `211120`.
    pub const RTC_DESTINO_CONSUMO_PESSOAL: u32 = 211120;
    /// Perecimento/roubo transporte adquirente, code `211124`.
    pub const RTC_ROUBO_PERDA_ADQUIRENTE: u32 = 211124;
    /// Aceite débito apuração, code `211128`.
    pub const RTC_ACEITE_DEBITO: u32 = 211128;
    /// Imobilização de item, code `211130`.
    pub const RTC_IMOBILIZACAO_ITEM: u32 = 211130;
    /// Apropriação crédito combustível, code `211140`.
    pub const RTC_APROPRIACAO_CREDITO_COMB: u32 = 211140;
    /// Apropriação crédito bens/serviços, code `211150`.
    pub const RTC_APROPRIACAO_CREDITO_BENS: u32 = 211150;
    /// Manifestação transferência crédito IBS, code `212110`.
    pub const RTC_MANIF_TRANSF_CRED_IBS: u32 = 212110;
    /// Manifestação transferência crédito CBS, code `212120`.
    pub const RTC_MANIF_TRANSF_CRED_CBS: u32 = 212120;
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
        110750 => "ECONF",
        110751 => "Cancelamento Conciliacao Financeira",
        // RTC events
        110001 => "Cancelamento de Evento",
        112110 => "Informacao de Pagamento Integral",
        112120 => "Importacao ZFM nao convertida em isencao",
        112130 => "Perecimento Perda Roubo Furto Transporte Fornecedor",
        112140 => "Fornecimento nao realizado",
        112150 => "Atualizacao da Data de Previsao de Entrega",
        211110 => "Solicitacao de Apropriacao de Credito Presumido",
        211120 => "Destinacao de Item para Consumo Pessoal",
        211124 => "Perecimento Perda Roubo Furto Transporte Adquirente",
        211128 => "Aceite de Debito na Apuracao",
        211130 => "Imobilizacao de Item",
        211140 => "Apropriacao de Credito de Combustivel",
        211150 => "Apropriacao de Credito para Bens e Servicos",
        212110 => "Manifestacao Transferencia de Credito IBS",
        212120 => "Manifestacao Transferencia de Credito CBS",
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

/// Build a SEFAZ cancellation-by-substitution event request XML
/// (`tpEvento=110112`).
///
/// Used exclusively for NFC-e (model 65). Sends the event to
/// `RecepcaoEvento` of the issuing state (cOrgao from access key).
///
/// # Arguments
///
/// * `access_key` - The 44-digit access key of the NFC-e being cancelled.
/// * `ref_access_key` - The 44-digit access key of the replacement NFC-e.
/// * `protocol` - The authorization protocol of the original NFC-e.
/// * `justification` - Reason for cancellation (max 255 chars).
/// * `ver_aplic` - Version of the issuing application.
/// * `seq` - Event sequence number (usually 1).
/// * `environment` - SEFAZ environment.
/// * `tax_id` - CNPJ or CPF of the issuer.
#[allow(clippy::too_many_arguments)]
pub fn build_cancel_substituicao_request(
    access_key: &str,
    ref_access_key: &str,
    protocol: &str,
    justification: &str,
    ver_aplic: &str,
    seq: u32,
    environment: SefazEnvironment,
    tax_id: &str,
) -> String {
    assert!(
        !justification.is_empty(),
        "Cancellation justification (xJust) is required"
    );
    assert!(
        !ref_access_key.is_empty(),
        "Reference access key (chNFeRef) is required"
    );
    assert!(
        !ver_aplic.is_empty(),
        "Application version (verAplic) is required"
    );

    let c_orgao = &access_key[..2];
    let additional = format!(
        "<cOrgaoAutor>{c_orgao}</cOrgaoAutor>\
         <tpAutor>1</tpAutor>\
         <verAplic>{ver_aplic}</verAplic>\
         <nProt>{protocol}</nProt>\
         <xJust>{justification}</xJust>\
         <chNFeRef>{ref_access_key}</chNFeRef>"
    );

    // Cancelamento por substituição goes to state endpoint (cOrgao from key)
    build_event_xml(
        access_key,
        event_types::CANCEL_SUBSTITUICAO,
        seq,
        tax_id,
        environment,
        &additional,
    )
}

/// Build a SEFAZ ator interessado event request XML (`tpEvento=110150`).
///
/// Authorizes a transporter (or subcontracted transporters) to access the
/// NF-e. Sent to the Ambiente Nacional (AN, cOrgao=91).
///
/// # Arguments
///
/// * `access_key` - The 44-digit access key of the NF-e.
/// * `tp_autor` - Author type (1=emitente, 2=destinatário, 3=transportador).
/// * `ver_aplic` - Version of the issuing application.
/// * `authorized_cnpj` - Optional CNPJ to authorize (mutually exclusive with `authorized_cpf`).
/// * `authorized_cpf` - Optional CPF to authorize (mutually exclusive with `authorized_cnpj`).
/// * `tp_autorizacao` - Authorization type (0=not allowed to subcontract, 1=allowed).
/// * `issuer_uf` - UF of the issuer (for cOrgaoAutor).
/// * `seq` - Event sequence number.
/// * `environment` - SEFAZ environment.
/// * `tax_id` - CNPJ or CPF of the event sender.
#[allow(clippy::too_many_arguments)]
pub fn build_ator_interessado_request(
    access_key: &str,
    tp_autor: u8,
    ver_aplic: &str,
    authorized_cnpj: Option<&str>,
    authorized_cpf: Option<&str>,
    tp_autorizacao: u8,
    issuer_uf: &str,
    seq: u32,
    environment: SefazEnvironment,
    tax_id: &str,
) -> String {
    let c_uf = get_state_code(issuer_uf).expect("Invalid state code");

    let auth_tag = if let Some(cnpj) = authorized_cnpj {
        format!("<CNPJ>{cnpj}</CNPJ>")
    } else if let Some(cpf) = authorized_cpf {
        format!("<CPF>{cpf}</CPF>")
    } else {
        panic!("Either authorized_cnpj or authorized_cpf must be provided");
    };

    let mut additional = format!(
        "<cOrgaoAutor>{c_uf}</cOrgaoAutor>\
         <tpAutor>{tp_autor}</tpAutor>\
         <verAplic>{ver_aplic}</verAplic>\
         <autXML>{auth_tag}</autXML>\
         <tpAutorizacao>{tp_autorizacao}</tpAutorizacao>"
    );

    if tp_autorizacao == 1 {
        let x_cond_uso = concat!(
            "O emitente ou destinatario da NF-e, declara que permite o ",
            "transportador declarado no campo CNPJ/CPF deste evento a ",
            "autorizar os transportadores subcontratados ou redespachados a ",
            "terem acesso ao download da NF-e"
        );
        additional.push_str(&format!("<xCondUso>{x_cond_uso}</xCondUso>"));
    }

    // Ator interessado always goes to AN (cOrgao=91)
    build_event_xml_with_org(
        access_key,
        event_types::ATOR_INTERESSADO,
        seq,
        tax_id,
        environment,
        &additional,
        Some("91"),
    )
}

/// Build a SEFAZ comprovante de entrega event request XML
/// (`tpEvento=110130`).
///
/// Records proof of delivery for an NF-e. Sent to Ambiente Nacional (AN).
///
/// # Arguments
///
/// * `access_key` - The 44-digit access key of the NF-e.
/// * `ver_aplic` - Version of the issuing application.
/// * `delivery_date` - Date/time of delivery (ISO 8601 format).
/// * `doc_number` - Document number of the receiver.
/// * `name` - Name of the receiver.
/// * `lat` - Optional GPS latitude.
/// * `long` - Optional GPS longitude.
/// * `hash` - Base64-encoded SHA-1 hash of the delivery proof.
/// * `hash_date` - Date/time of the hash generation (ISO 8601 format).
/// * `issuer_uf` - UF of the issuer (for cOrgaoAutor).
/// * `seq` - Event sequence number.
/// * `environment` - SEFAZ environment.
/// * `tax_id` - CNPJ or CPF of the event sender.
#[allow(clippy::too_many_arguments)]
pub fn build_comprovante_entrega_request(
    access_key: &str,
    ver_aplic: &str,
    delivery_date: &str,
    doc_number: &str,
    name: &str,
    lat: Option<&str>,
    long: Option<&str>,
    hash: &str,
    hash_date: &str,
    issuer_uf: &str,
    seq: u32,
    environment: SefazEnvironment,
    tax_id: &str,
) -> String {
    let c_uf = get_state_code(issuer_uf).expect("Invalid state code");

    let mut additional = format!(
        "<cOrgaoAutor>{c_uf}</cOrgaoAutor>\
         <tpAutor>1</tpAutor>\
         <verAplic>{ver_aplic}</verAplic>\
         <dhEntrega>{delivery_date}</dhEntrega>\
         <nDoc>{doc_number}</nDoc>\
         <xNome>{name}</xNome>"
    );

    if let (Some(lat_val), Some(long_val)) = (lat, long) {
        if !lat_val.is_empty() && !long_val.is_empty() {
            additional.push_str(&format!(
                "<latGPS>{lat_val}</latGPS><longGPS>{long_val}</longGPS>"
            ));
        }
    }

    additional.push_str(&format!(
        "<hashComprovante>{hash}</hashComprovante>\
         <dhHashComprovante>{hash_date}</dhHashComprovante>"
    ));

    // Comprovante de entrega goes to AN (cOrgao=91)
    build_event_xml_with_org(
        access_key,
        event_types::COMPROVANTE_ENTREGA,
        seq,
        tax_id,
        environment,
        &additional,
        Some("91"),
    )
}

/// Build a SEFAZ cancelamento do comprovante de entrega event request XML
/// (`tpEvento=110131`).
///
/// Cancels a previously registered delivery receipt. Sent to AN.
///
/// # Arguments
///
/// * `access_key` - The 44-digit access key of the NF-e.
/// * `ver_aplic` - Version of the issuing application.
/// * `event_protocol` - Protocol number of the delivery receipt event being cancelled.
/// * `issuer_uf` - UF of the issuer (for cOrgaoAutor).
/// * `seq` - Event sequence number.
/// * `environment` - SEFAZ environment.
/// * `tax_id` - CNPJ or CPF of the event sender.
#[allow(clippy::too_many_arguments)]
pub fn build_cancel_comprovante_entrega_request(
    access_key: &str,
    ver_aplic: &str,
    event_protocol: &str,
    issuer_uf: &str,
    seq: u32,
    environment: SefazEnvironment,
    tax_id: &str,
) -> String {
    let c_uf = get_state_code(issuer_uf).expect("Invalid state code");

    let additional = format!(
        "<cOrgaoAutor>{c_uf}</cOrgaoAutor>\
         <tpAutor>1</tpAutor>\
         <verAplic>{ver_aplic}</verAplic>\
         <nProtEvento>{event_protocol}</nProtEvento>"
    );

    build_event_xml_with_org(
        access_key,
        event_types::CANCEL_COMPROVANTE_ENTREGA,
        seq,
        tax_id,
        environment,
        &additional,
        Some("91"),
    )
}

/// Build a SEFAZ insucesso na entrega event request XML
/// (`tpEvento=110192`).
///
/// Records a failed delivery attempt. Sent to Ambiente Nacional (AN).
///
/// # Arguments
///
/// * `access_key` - The 44-digit access key of the NF-e.
/// * `ver_aplic` - Version of the issuing application.
/// * `attempt_date` - Date/time of the delivery attempt (ISO 8601 format).
/// * `attempt_number` - Optional attempt number.
/// * `reason_type` - Reason code (1=recusado, 2=ausente, 3=não localizado, 4=outros).
/// * `reason_justification` - Required when reason_type=4.
/// * `lat` - Optional GPS latitude.
/// * `long` - Optional GPS longitude.
/// * `hash` - Base64-encoded SHA-1 hash of the attempt proof.
/// * `hash_date` - Date/time of the hash generation (ISO 8601 format).
/// * `issuer_uf` - UF of the issuer (for cOrgaoAutor).
/// * `seq` - Event sequence number.
/// * `environment` - SEFAZ environment.
/// * `tax_id` - CNPJ or CPF of the event sender.
#[allow(clippy::too_many_arguments)]
pub fn build_insucesso_entrega_request(
    access_key: &str,
    ver_aplic: &str,
    attempt_date: &str,
    attempt_number: Option<u32>,
    reason_type: u8,
    reason_justification: Option<&str>,
    lat: Option<&str>,
    long: Option<&str>,
    hash: &str,
    hash_date: &str,
    issuer_uf: &str,
    seq: u32,
    environment: SefazEnvironment,
    tax_id: &str,
) -> String {
    let c_uf = get_state_code(issuer_uf).expect("Invalid state code");

    let mut additional = format!(
        "<cOrgaoAutor>{c_uf}</cOrgaoAutor>\
         <verAplic>{ver_aplic}</verAplic>\
         <dhTentativaEntrega>{attempt_date}</dhTentativaEntrega>"
    );

    if let Some(n) = attempt_number {
        if n > 0 {
            additional.push_str(&format!("<nTentativa>{n}</nTentativa>"));
        }
    }

    additional.push_str(&format!("<tpMotivo>{reason_type}</tpMotivo>"));

    if reason_type == 4 {
        if let Some(just) = reason_justification {
            if !just.is_empty() {
                additional.push_str(&format!("<xJustMotivo>{just}</xJustMotivo>"));
            }
        }
    }

    if let (Some(lat_val), Some(long_val)) = (lat, long) {
        if !lat_val.is_empty() && !long_val.is_empty() {
            additional.push_str(&format!(
                "<latGPS>{lat_val}</latGPS><longGPS>{long_val}</longGPS>"
            ));
        }
    }

    additional.push_str(&format!(
        "<hashTentativaEntrega>{hash}</hashTentativaEntrega>\
         <dhHashTentativaEntrega>{hash_date}</dhHashTentativaEntrega>"
    ));

    // Insucesso na entrega goes to AN (cOrgao=91)
    build_event_xml_with_org(
        access_key,
        event_types::INSUCESSO_ENTREGA,
        seq,
        tax_id,
        environment,
        &additional,
        Some("91"),
    )
}

/// Build a SEFAZ cancelamento do insucesso de entrega event request XML
/// (`tpEvento=110193`).
///
/// Cancels a previously registered delivery failure event. Sent to AN.
///
/// # Arguments
///
/// * `access_key` - The 44-digit access key of the NF-e.
/// * `ver_aplic` - Version of the issuing application.
/// * `event_protocol` - Protocol number of the delivery failure event being cancelled.
/// * `issuer_uf` - UF of the issuer (for cOrgaoAutor).
/// * `seq` - Event sequence number.
/// * `environment` - SEFAZ environment.
/// * `tax_id` - CNPJ or CPF of the event sender.
#[allow(clippy::too_many_arguments)]
pub fn build_cancel_insucesso_entrega_request(
    access_key: &str,
    ver_aplic: &str,
    event_protocol: &str,
    issuer_uf: &str,
    seq: u32,
    environment: SefazEnvironment,
    tax_id: &str,
) -> String {
    let c_uf = get_state_code(issuer_uf).expect("Invalid state code");

    let additional = format!(
        "<cOrgaoAutor>{c_uf}</cOrgaoAutor>\
         <verAplic>{ver_aplic}</verAplic>\
         <nProtEvento>{event_protocol}</nProtEvento>"
    );

    build_event_xml_with_org(
        access_key,
        event_types::CANCEL_INSUCESSO_ENTREGA,
        seq,
        tax_id,
        environment,
        &additional,
        Some("91"),
    )
}

/// Data extracted from an NF-e XML for building an EPEC event request.
///
/// All fields are extracted from the signed NF-e XML. The struct is used
/// as input to [`build_epec_request`] to avoid a long parameter list.
#[derive(Debug, Clone)]
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

/// Item for a prorrogacao (ICMS extension) request.
///
/// Each item contains the item number and the requested quantity.
#[derive(Debug, Clone)]
pub struct ProrrogacaoItem {
    /// Item number in the original NF-e (starting from 1).
    pub num_item: u32,
    /// Quantity requested for prorrogacao.
    pub qtde: f64,
}

/// Build a SEFAZ prorrogacao (ICMS extension) event request XML.
///
/// Creates an `<envEvento>` wrapper containing a pedido de prorrogacao
/// (`tpEvento=111500` for first term, `111501` for second term).
///
/// # Arguments
///
/// * `access_key` — 44-digit access key of the NF-e.
/// * `protocol` — authorization protocol number of the original NF-e.
/// * `items` — list of items with quantities being extended.
/// * `second_term` — if `true`, uses the second-term event type (111501).
/// * `seq` — event sequence number.
/// * `environment` — SEFAZ environment.
/// * `tax_id` — CNPJ or CPF of the issuer.
pub fn build_prorrogacao_request(
    access_key: &str,
    protocol: &str,
    items: &[ProrrogacaoItem],
    second_term: bool,
    seq: u32,
    environment: SefazEnvironment,
    tax_id: &str,
) -> String {
    let tp_evento = if second_term {
        event_types::PRORROGACAO_2
    } else {
        event_types::PRORROGACAO_1
    };

    let mut additional = format!("<nProt>{protocol}</nProt>");
    for item in items {
        additional.push_str(&format!(
            "<itemPedido numItem=\"{}\"><qtdeItem>{}</qtdeItem></itemPedido>",
            item.num_item, item.qtde
        ));
    }

    build_event_xml(access_key, tp_evento, seq, tax_id, environment, &additional)
}

/// Build a SEFAZ cancel-prorrogacao (cancel ICMS extension) event request XML.
///
/// Creates an `<envEvento>` wrapper containing a cancelamento de pedido de
/// prorrogacao (`tpEvento=111502` for first term, `111503` for second term).
///
/// # Arguments
///
/// * `access_key` — 44-digit access key of the NF-e.
/// * `protocol` — authorization protocol number of the prorrogacao event.
/// * `second_term` — if `true`, uses the second-term event type (111503).
/// * `seq` — event sequence number.
/// * `environment` — SEFAZ environment.
/// * `tax_id` — CNPJ or CPF of the issuer.
pub fn build_cancel_prorrogacao_request(
    access_key: &str,
    protocol: &str,
    second_term: bool,
    seq: u32,
    environment: SefazEnvironment,
    tax_id: &str,
) -> String {
    let (tp_evento, orig_event) = if second_term {
        (
            event_types::CANCEL_PRORROGACAO_2,
            event_types::PRORROGACAO_2,
        )
    } else {
        (
            event_types::CANCEL_PRORROGACAO_1,
            event_types::PRORROGACAO_1,
        )
    };

    let id_pedido_cancelado = format!("ID{orig_event}{access_key}{seq:02}");
    let additional = format!(
        "<idPedidoCancelado>{id_pedido_cancelado}</idPedidoCancelado><nProt>{protocol}</nProt>"
    );

    build_event_xml(access_key, tp_evento, seq, tax_id, environment, &additional)
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

/// An individual event for batch event submission.
///
/// Used with [`build_event_batch_request`] to send multiple events in a
/// single SOAP request, matching the PHP `sefazEventoLote()` pattern.
#[derive(Debug, Clone)]
pub struct EventItem {
    /// 44-digit access key of the NF-e.
    pub access_key: String,
    /// Event type code (e.g., `210200`, `210210`, `210220`, `210240`).
    pub event_type: u32,
    /// Event sequence number.
    pub seq: u32,
    /// CNPJ or CPF of the event sender.
    pub tax_id: String,
    /// Additional XML tags for `<detEvento>` (after `<descEvento>`).
    pub additional_tags: String,
}

/// Build a SEFAZ batch event request XML (`<envEvento>`) containing
/// multiple `<evento>` elements.
///
/// This matches the PHP `sefazEventoLote()` method. Each event is built
/// individually and concatenated inside a single `<envEvento>` wrapper.
///
/// Events with type EPEC (`110140`) are skipped, matching PHP behavior.
///
/// # Arguments
///
/// * `uf` — State abbreviation or `"AN"` for Ambiente Nacional.
/// * `events` — Slice of [`EventItem`] structs (max 20).
/// * `lot_id` — Lot identifier. If `None`, a timestamp-based ID is generated.
/// * `environment` — SEFAZ environment.
///
/// # Panics
///
/// Panics if `events` has more than 20 items or is empty.
pub fn build_event_batch_request(
    uf: &str,
    events: &[EventItem],
    lot_id: Option<&str>,
    environment: SefazEnvironment,
) -> String {
    assert!(
        !events.is_empty(),
        "Event batch must contain at least one event"
    );
    assert!(
        events.len() <= 20,
        "Event batch is limited to 20 events, got {}",
        events.len()
    );

    let c_orgao = get_state_code(uf).expect("Invalid state code");
    let tp_amb = environment.as_str();

    // Event datetime with BRT offset (-03:00)
    let dh_evento = chrono::Utc::now()
        .with_timezone(&chrono::FixedOffset::west_opt(3 * 3600).expect("valid offset"))
        .format("%Y-%m-%dT%H:%M:%S%:z")
        .to_string();

    let mut batch = String::new();
    for evt in events {
        // Skip EPEC events in batch — matches PHP behavior
        if evt.event_type == event_types::EPEC {
            continue;
        }

        let event_id = build_event_id(evt.event_type, &evt.access_key, evt.seq);
        let desc_evento = event_description(evt.event_type);
        let tax_tag = tax_id_xml_tag(&evt.tax_id);

        batch.push_str(&format!(
            "<evento xmlns=\"{NFE_NAMESPACE}\" versao=\"1.00\">\
             <infEvento Id=\"{event_id}\">\
             <cOrgao>{c_orgao}</cOrgao>\
             <tpAmb>{tp_amb}</tpAmb>\
             {tax_tag}\
             <chNFe>{access_key}</chNFe>\
             <dhEvento>{dh_evento}</dhEvento>\
             <tpEvento>{event_type}</tpEvento>\
             <nSeqEvento>{seq}</nSeqEvento>\
             <verEvento>1.00</verEvento>\
             <detEvento versao=\"1.00\">\
             <descEvento>{desc_evento}</descEvento>\
             {additional}\
             </detEvento>\
             </infEvento>\
             </evento>",
            access_key = evt.access_key,
            event_type = evt.event_type,
            seq = evt.seq,
            additional = evt.additional_tags,
        ));
    }

    let id_lote = match lot_id {
        Some(id) => id.to_string(),
        None => std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis().to_string())
            .unwrap_or_else(|_| "1".to_string()),
    };

    format!(
        "<envEvento xmlns=\"{NFE_NAMESPACE}\" versao=\"1.00\">\
         <idLote>{id_lote}</idLote>\
         {batch}\
         </envEvento>"
    )
}

/// Payment detail for a conciliação financeira event.
///
/// Used with [`build_conciliacao_request`] to describe each payment
/// method in the financial reconciliation.
#[derive(Debug, Clone)]
pub struct ConciliacaoDetPag {
    /// Payment indicator (optional, e.g., `"0"` = à vista, `"1"` = a prazo).
    pub ind_pag: Option<String>,
    /// Payment type code (e.g., `"01"` = dinheiro, `"03"` = cartão crédito).
    pub t_pag: String,
    /// Payment description (optional).
    pub x_pag: Option<String>,
    /// Payment value.
    pub v_pag: String,
    /// Payment date (YYYY-MM-DD).
    pub d_pag: String,
    /// CNPJ of the payment institution (optional).
    pub cnpj_pag: Option<String>,
    /// UF of the payment institution (optional, required with `cnpj_pag`).
    pub uf_pag: Option<String>,
    /// CNPJ of the payment intermediary (optional).
    pub cnpj_if: Option<String>,
    /// Card brand type (optional).
    pub t_band: Option<String>,
    /// Authorization code (optional).
    pub c_aut: Option<String>,
    /// CNPJ of the receiver (optional).
    pub cnpj_receb: Option<String>,
    /// UF of the receiver (optional, required with `cnpj_receb`).
    pub uf_receb: Option<String>,
}

/// Build a SEFAZ conciliação financeira event request XML
/// (`tpEvento=110750` or `110751` for cancellation).
///
/// Implements the PHP `sefazConciliacao()` method.
///
/// # Arguments
///
/// * `uf` — State abbreviation. For model 55 (NF-e), use `"SVRS"`;
///   for model 65 (NFC-e), use the actual state abbreviation.
/// * `access_key` — 44-digit access key.
/// * `ver_aplic` — Application version string.
/// * `det_pag` — Payment details (required for new conciliation, empty for cancel).
/// * `cancel` — If `true`, sends cancellation event (110751) instead.
/// * `cancel_protocol` — Protocol of the conciliation event being cancelled
///   (required when `cancel=true`).
/// * `seq` — Event sequence number.
/// * `environment` — SEFAZ environment.
/// * `tax_id` — CNPJ or CPF of the issuer.
#[allow(clippy::too_many_arguments)]
pub fn build_conciliacao_request(
    access_key: &str,
    ver_aplic: &str,
    det_pag: &[ConciliacaoDetPag],
    cancel: bool,
    cancel_protocol: Option<&str>,
    seq: u32,
    environment: SefazEnvironment,
    tax_id: &str,
) -> String {
    if cancel {
        let protocol = cancel_protocol.unwrap_or("");
        let additional = format!(
            "<verAplic>{ver_aplic}</verAplic>\
             <nProtEvento>{protocol}</nProtEvento>"
        );
        build_event_xml(
            access_key,
            event_types::CANCEL_CONCILIACAO,
            seq,
            tax_id,
            environment,
            &additional,
        )
    } else {
        let mut additional = format!("<verAplic>{ver_aplic}</verAplic>");
        for pag in det_pag {
            additional.push_str("<detPag>");
            if let Some(ref ind) = pag.ind_pag {
                additional.push_str(&format!("<indPag>{ind}</indPag>"));
            }
            additional.push_str(&format!("<tPag>{}</tPag>", pag.t_pag));
            if let Some(ref x) = pag.x_pag {
                additional.push_str(&format!("<xPag>{x}</xPag>"));
            }
            additional.push_str(&format!("<vPag>{}</vPag>", pag.v_pag));
            additional.push_str(&format!("<dPag>{}</dPag>", pag.d_pag));
            if let (Some(cnpj), Some(uf)) = (&pag.cnpj_pag, &pag.uf_pag) {
                additional.push_str(&format!("<CNPJPag>{cnpj}</CNPJPag>"));
                additional.push_str(&format!("<UFPag>{uf}</UFPag>"));
                if let Some(ref cnpj_if) = pag.cnpj_if {
                    additional.push_str(&format!("<CNPJIF>{cnpj_if}</CNPJIF>"));
                }
            }
            if let Some(ref t_band) = pag.t_band {
                additional.push_str(&format!("<tBand>{t_band}</tBand>"));
            }
            if let Some(ref c_aut) = pag.c_aut {
                additional.push_str(&format!("<cAut>{c_aut}</cAut>"));
            }
            if let (Some(cnpj_receb), Some(uf_receb)) = (&pag.cnpj_receb, &pag.uf_receb) {
                additional.push_str(&format!("<CNPJReceb>{cnpj_receb}</CNPJReceb>"));
                additional.push_str(&format!("<UFReceb>{uf_receb}</UFReceb>"));
            }
            additional.push_str("</detPag>");
        }
        build_event_xml(
            access_key,
            event_types::CONCILIACAO,
            seq,
            tax_id,
            environment,
            &additional,
        )
    }
}

/// Extract a named section from XML (e.g., `<emit>...</emit>`).
///
/// Returns the full content between the opening and closing tags, inclusive.
fn extract_section(xml: &str, tag_name: &str) -> Option<String> {
    let open = format!("<{tag_name}");
    let close = format!("</{tag_name}>");

    let start = xml.find(&open)?;
    // Verify delimiter
    let after_open = start + open.len();
    if after_open < xml.len() {
        let next = xml.as_bytes()[after_open];
        if next != b' ' && next != b'>' && next != b'/' && next != b'\n' && next != b'\t' {
            return None;
        }
    }

    let end = xml[start..].find(&close)? + start + close.len();
    Some(xml[start..end].to_string())
}

// ── Internal helpers ────────────────────────────────────────────────────────

// ── RTC (Reforma Tributária Complementar) event builders ────────────────────

/// Item data for RTC events that list per-item IBS/CBS values.
#[derive(Debug, Clone)]
pub struct RtcItem {
    /// Item number (nItem attribute).
    pub item: u32,
    /// IBS value.
    pub v_ibs: f64,
    /// CBS value.
    pub v_cbs: f64,
    /// Quantity (used by stock-control events). Optional.
    pub quantidade: Option<f64>,
    /// Unit of measure (used by stock-control events). Optional.
    pub unidade: Option<String>,
    /// Referenced access key (DFeReferenciado). Optional.
    pub chave: Option<String>,
    /// Referenced item number. Optional.
    pub n_item: Option<u32>,
    /// IBS value inside gControleEstoque (fornecedor events). Optional.
    pub g_controle_estoque_v_ibs: Option<f64>,
    /// CBS value inside gControleEstoque (fornecedor events). Optional.
    pub g_controle_estoque_v_cbs: Option<f64>,
    /// IBS credit value (for bens/servicos events). Optional.
    pub v_cred_ibs: Option<f64>,
    /// CBS credit value (for bens/servicos events). Optional.
    pub v_cred_cbs: Option<f64>,
}

impl RtcItem {
    /// Create a minimal RTC item with IBS and CBS values.
    pub fn new(item: u32, v_ibs: f64, v_cbs: f64) -> Self {
        Self {
            item,
            v_ibs,
            v_cbs,
            quantidade: None,
            unidade: None,
            chave: None,
            n_item: None,
            g_controle_estoque_v_ibs: None,
            g_controle_estoque_v_cbs: None,
            v_cred_ibs: None,
            v_cred_cbs: None,
        }
    }
    /// Set quantity.
    pub fn quantidade(mut self, v: f64) -> Self {
        self.quantidade = Some(v);
        self
    }
    /// Set unit.
    pub fn unidade(mut self, v: impl Into<String>) -> Self {
        self.unidade = Some(v.into());
        self
    }
    /// Set referenced access key.
    pub fn chave(mut self, v: impl Into<String>) -> Self {
        self.chave = Some(v.into());
        self
    }
    /// Set referenced item number.
    pub fn n_item(mut self, v: u32) -> Self {
        self.n_item = Some(v);
        self
    }
}

/// Item for crédito presumido events.
#[derive(Debug, Clone)]
pub struct RtcCredPresItem {
    /// Item number.
    pub item: u32,
    /// Base de cálculo.
    pub v_bc: f64,
    /// IBS sub-group. Optional.
    pub g_ibs: Option<RtcCredPresSub>,
    /// CBS sub-group. Optional.
    pub g_cbs: Option<RtcCredPresSub>,
}

/// Sub-group for crédito presumido IBS or CBS.
#[derive(Debug, Clone)]
pub struct RtcCredPresSub {
    /// Código do crédito presumido.
    pub c_cred_pres: String,
    /// Percentual.
    pub p_cred_pres: f64,
    /// Valor.
    pub v_cred_pres: f64,
}

/// Build RTC event: Informação de pagamento integral (tpEvento=112110).
pub fn build_rtc_info_pagto_integral(
    access_key: &str,
    seq: u32,
    environment: SefazEnvironment,
    tax_id: &str,
    c_uf: &str,
    ver_aplic: &str,
) -> String {
    let c_orgao = get_state_code(c_uf)
        .map(|c| c.to_string())
        .unwrap_or_default();
    let additional = format!(
        "<cOrgaoAutor>{c_orgao}</cOrgaoAutor>\
         <tpAutor>1</tpAutor>\
         <verAplic>{ver_aplic}</verAplic>\
         <indQuitacao>1</indQuitacao>"
    );
    build_event_xml(
        access_key,
        event_types::RTC_INFO_PAGTO_INTEGRAL,
        seq,
        tax_id,
        environment,
        &additional,
    )
}

/// Build RTC event: Solicitação de apropriação de crédito presumido (tpEvento=211110).
pub fn build_rtc_sol_aprop_cred_presumido(
    access_key: &str,
    seq: u32,
    environment: SefazEnvironment,
    tax_id: &str,
    c_uf: &str,
    ver_aplic: &str,
    itens: &[RtcCredPresItem],
) -> String {
    let c_orgao = get_state_code(c_uf)
        .map(|c| c.to_string())
        .unwrap_or_default();
    let mut additional = format!(
        "<cOrgaoAutor>{c_orgao}</cOrgaoAutor>\
         <tpAutor>2</tpAutor>\
         <verAplic>{ver_aplic}</verAplic>"
    );
    for item in itens {
        let bc = format!("{:.2}", item.v_bc);
        additional.push_str(&format!(
            "<gCredPres nItem=\"{}\"><vBC>{bc}</vBC>",
            item.item
        ));
        if let Some(ref g) = item.g_ibs {
            let pc = format!("{:.4}", g.p_cred_pres);
            let vc = format!("{:.2}", g.v_cred_pres);
            additional.push_str(&format!(
                "<gIBS><cCredPres>{}</cCredPres><pCredPres>{pc}</pCredPres><vCredPres>{vc}</vCredPres></gIBS>",
                g.c_cred_pres
            ));
        }
        if let Some(ref g) = item.g_cbs {
            let pc = format!("{:.4}", g.p_cred_pres);
            let vc = format!("{:.2}", g.v_cred_pres);
            additional.push_str(&format!(
                "<gCBS><cCredPres>{}</cCredPres><pCredPres>{pc}</pCredPres><vCredPres>{vc}</vCredPres></gCBS>",
                g.c_cred_pres
            ));
        }
        additional.push_str("</gCredPres>");
    }
    build_event_xml(
        access_key,
        event_types::RTC_SOL_APROP_CRED_PRESUMIDO,
        seq,
        tax_id,
        environment,
        &additional,
    )
}

/// Build RTC event: Destinação de item para consumo pessoal (tpEvento=211120).
#[allow(clippy::too_many_arguments)]
pub fn build_rtc_destino_consumo_pessoal(
    access_key: &str,
    seq: u32,
    environment: SefazEnvironment,
    tax_id: &str,
    c_uf: &str,
    tp_autor: u8,
    ver_aplic: &str,
    itens: &[RtcItem],
) -> String {
    let c_orgao = get_state_code(c_uf)
        .map(|c| c.to_string())
        .unwrap_or_default();
    let mut additional = format!(
        "<cOrgaoAutor>{c_orgao}</cOrgaoAutor>\
         <tpAutor>{tp_autor}</tpAutor>\
         <verAplic>{ver_aplic}</verAplic>"
    );
    for item in itens {
        let vi = format!("{:.2}", item.v_ibs);
        let vc = format!("{:.2}", item.v_cbs);
        let qt = format!("{:.4}", item.quantidade.unwrap_or(0.0));
        let u = item.unidade.as_deref().unwrap_or("");
        let ch = item.chave.as_deref().unwrap_or("");
        let ni = item.n_item.unwrap_or(0);
        additional.push_str(&format!(
            "<gConsumo nItem=\"{}\"><vIBS>{vi}</vIBS><vCBS>{vc}</vCBS>\
             <gControleEstoque><qConsumo>{qt}</qConsumo><uConsumo>{u}</uConsumo></gControleEstoque>\
             <DFeReferenciado><chaveAcesso>{ch}</chaveAcesso><nItem>{ni}</nItem></DFeReferenciado>\
             </gConsumo>",
            item.item
        ));
    }
    build_event_xml(
        access_key,
        event_types::RTC_DESTINO_CONSUMO_PESSOAL,
        seq,
        tax_id,
        environment,
        &additional,
    )
}

/// Build RTC event: Aceite de débito na apuração (tpEvento=211128).
pub fn build_rtc_aceite_debito(
    access_key: &str,
    seq: u32,
    environment: SefazEnvironment,
    tax_id: &str,
    c_uf: &str,
    ver_aplic: &str,
    ind_aceitacao: u8,
) -> String {
    let c_orgao = get_state_code(c_uf)
        .map(|c| c.to_string())
        .unwrap_or_default();
    let additional = format!(
        "<cOrgaoAutor>{c_orgao}</cOrgaoAutor>\
         <tpAutor>2</tpAutor>\
         <verAplic>{ver_aplic}</verAplic>\
         <indAceitacao>{ind_aceitacao}</indAceitacao>"
    );
    build_event_xml(
        access_key,
        event_types::RTC_ACEITE_DEBITO,
        seq,
        tax_id,
        environment,
        &additional,
    )
}

/// Build RTC event: Imobilização de item (tpEvento=211130).
pub fn build_rtc_imobilizacao_item(
    access_key: &str,
    seq: u32,
    environment: SefazEnvironment,
    tax_id: &str,
    c_uf: &str,
    ver_aplic: &str,
    itens: &[RtcItem],
) -> String {
    let c_orgao = get_state_code(c_uf)
        .map(|c| c.to_string())
        .unwrap_or_default();
    let mut additional = format!(
        "<cOrgaoAutor>{c_orgao}</cOrgaoAutor>\
         <tpAutor>2</tpAutor>\
         <verAplic>{ver_aplic}</verAplic>"
    );
    for item in itens {
        let vi = format!("{:.2}", item.v_ibs);
        let vc = format!("{:.2}", item.v_cbs);
        let qtd = format!("{:.4}", item.quantidade.unwrap_or(0.0));
        let u = item.unidade.as_deref().unwrap_or("");
        additional.push_str(&format!(
            "<gImobilizacao nItem=\"{}\"><vIBS>{vi}</vIBS><vCBS>{vc}</vCBS>\
             <gControleEstoque><qImobilizado>{qtd}</qImobilizado><uImobilizado>{u}</uImobilizado></gControleEstoque>\
             </gImobilizacao>",
            item.item
        ));
    }
    build_event_xml(
        access_key,
        event_types::RTC_IMOBILIZACAO_ITEM,
        seq,
        tax_id,
        environment,
        &additional,
    )
}

/// Build RTC event: Apropriação de crédito combustível (tpEvento=211140).
pub fn build_rtc_apropriacao_credito_comb(
    access_key: &str,
    seq: u32,
    environment: SefazEnvironment,
    tax_id: &str,
    c_uf: &str,
    ver_aplic: &str,
    itens: &[RtcItem],
) -> String {
    let c_orgao = get_state_code(c_uf)
        .map(|c| c.to_string())
        .unwrap_or_default();
    let mut additional = format!(
        "<cOrgaoAutor>{c_orgao}</cOrgaoAutor>\
         <tpAutor>2</tpAutor>\
         <verAplic>{ver_aplic}</verAplic>"
    );
    for item in itens {
        let vi = format!("{:.2}", item.v_ibs);
        let vc = format!("{:.2}", item.v_cbs);
        let qtd = format!("{:.4}", item.quantidade.unwrap_or(0.0));
        let u = item.unidade.as_deref().unwrap_or("");
        additional.push_str(&format!(
            "<gConsumoComb nItem=\"{}\"><vIBS>{vi}</vIBS><vCBS>{vc}</vCBS>\
             <gControleEstoque><qComb>{qtd}</qComb><uComb>{u}</uComb></gControleEstoque>\
             </gConsumoComb>",
            item.item
        ));
    }
    build_event_xml(
        access_key,
        event_types::RTC_APROPRIACAO_CREDITO_COMB,
        seq,
        tax_id,
        environment,
        &additional,
    )
}

/// Build RTC event: Apropriação de crédito bens/serviços (tpEvento=211150).
pub fn build_rtc_apropriacao_credito_bens(
    access_key: &str,
    seq: u32,
    environment: SefazEnvironment,
    tax_id: &str,
    c_uf: &str,
    ver_aplic: &str,
    itens: &[RtcItem],
) -> String {
    let c_orgao = get_state_code(c_uf)
        .map(|c| c.to_string())
        .unwrap_or_default();
    let mut additional = format!(
        "<cOrgaoAutor>{c_orgao}</cOrgaoAutor>\
         <tpAutor>2</tpAutor>\
         <verAplic>{ver_aplic}</verAplic>"
    );
    for item in itens {
        let vi = format!("{:.2}", item.v_cred_ibs.unwrap_or(item.v_ibs));
        let vc = format!("{:.2}", item.v_cred_cbs.unwrap_or(item.v_cbs));
        additional.push_str(&format!(
            "<gCredito nItem=\"{}\"><vCredIBS>{vi}</vCredIBS><vCredCBS>{vc}</vCredCBS></gCredito>",
            item.item
        ));
    }
    build_event_xml(
        access_key,
        event_types::RTC_APROPRIACAO_CREDITO_BENS,
        seq,
        tax_id,
        environment,
        &additional,
    )
}

/// Build RTC event: Manifestação transferência crédito IBS (tpEvento=212110).
pub fn build_rtc_manif_transf_cred_ibs(
    access_key: &str,
    seq: u32,
    environment: SefazEnvironment,
    tax_id: &str,
    c_uf: &str,
    ver_aplic: &str,
    ind_aceitacao: u8,
) -> String {
    let c_orgao = get_state_code(c_uf)
        .map(|c| c.to_string())
        .unwrap_or_default();
    let additional = format!(
        "<cOrgaoAutor>{c_orgao}</cOrgaoAutor>\
         <tpAutor>8</tpAutor>\
         <verAplic>{ver_aplic}</verAplic>\
         <indAceitacao>{ind_aceitacao}</indAceitacao>"
    );
    build_event_xml(
        access_key,
        event_types::RTC_MANIF_TRANSF_CRED_IBS,
        seq,
        tax_id,
        environment,
        &additional,
    )
}

/// Build RTC event: Manifestação transferência crédito CBS (tpEvento=212120).
pub fn build_rtc_manif_transf_cred_cbs(
    access_key: &str,
    seq: u32,
    environment: SefazEnvironment,
    tax_id: &str,
    c_uf: &str,
    ver_aplic: &str,
    ind_aceitacao: u8,
) -> String {
    let c_orgao = get_state_code(c_uf)
        .map(|c| c.to_string())
        .unwrap_or_default();
    let additional = format!(
        "<cOrgaoAutor>{c_orgao}</cOrgaoAutor>\
         <tpAutor>8</tpAutor>\
         <verAplic>{ver_aplic}</verAplic>\
         <indAceitacao>{ind_aceitacao}</indAceitacao>"
    );
    build_event_xml(
        access_key,
        event_types::RTC_MANIF_TRANSF_CRED_CBS,
        seq,
        tax_id,
        environment,
        &additional,
    )
}

/// Build RTC event: Cancelamento de evento (tpEvento=110001).
#[allow(clippy::too_many_arguments)]
pub fn build_rtc_cancela_evento(
    access_key: &str,
    seq: u32,
    environment: SefazEnvironment,
    tax_id: &str,
    c_uf: &str,
    ver_aplic: &str,
    tp_evento_aut: &str,
    n_prot_evento: &str,
) -> String {
    let c_orgao = get_state_code(c_uf)
        .map(|c| c.to_string())
        .unwrap_or_default();
    let additional = format!(
        "<cOrgaoAutor>{c_orgao}</cOrgaoAutor>\
         <verAplic>{ver_aplic}</verAplic>\
         <tpEventoAut>{tp_evento_aut}</tpEventoAut>\
         <nProtEvento>{n_prot_evento}</nProtEvento>"
    );
    build_event_xml(
        access_key,
        event_types::RTC_CANCELA_EVENTO,
        seq,
        tax_id,
        environment,
        &additional,
    )
}

/// Build RTC event: Importação ZFM não convertida em isenção (tpEvento=112120).
pub fn build_rtc_importacao_zfm(
    access_key: &str,
    seq: u32,
    environment: SefazEnvironment,
    tax_id: &str,
    c_uf: &str,
    ver_aplic: &str,
    itens: &[RtcItem],
) -> String {
    let c_orgao = get_state_code(c_uf)
        .map(|c| c.to_string())
        .unwrap_or_default();
    let mut additional = format!(
        "<cOrgaoAutor>{c_orgao}</cOrgaoAutor>\
         <tpAutor>1</tpAutor>\
         <verAplic>{ver_aplic}</verAplic>"
    );
    for item in itens {
        let vi = format!("{:.2}", item.v_ibs);
        let vc = format!("{:.2}", item.v_cbs);
        let qtd = format!("{:.4}", item.quantidade.unwrap_or(0.0));
        let u = item.unidade.as_deref().unwrap_or("");
        additional.push_str(&format!(
            "<gConsumo nItem=\"{}\"><vIBS>{vi}</vIBS><vCBS>{vc}</vCBS>\
             <gControleEstoque><qtde>{qtd}</qtde><unidade>{u}</unidade></gControleEstoque>\
             </gConsumo>",
            item.item
        ));
    }
    build_event_xml(
        access_key,
        event_types::RTC_IMPORTACAO_ZFM,
        seq,
        tax_id,
        environment,
        &additional,
    )
}

/// Build RTC event: Perecimento/roubo transporte adquirente (tpEvento=211124).
pub fn build_rtc_roubo_perda_adquirente(
    access_key: &str,
    seq: u32,
    environment: SefazEnvironment,
    tax_id: &str,
    c_uf: &str,
    ver_aplic: &str,
    itens: &[RtcItem],
) -> String {
    let c_orgao = get_state_code(c_uf)
        .map(|c| c.to_string())
        .unwrap_or_default();
    let mut additional = format!(
        "<cOrgaoAutor>{c_orgao}</cOrgaoAutor>\
         <tpAutor>2</tpAutor>\
         <verAplic>{ver_aplic}</verAplic>"
    );
    for item in itens {
        let vi = format!("{:.2}", item.v_ibs);
        let vc = format!("{:.2}", item.v_cbs);
        let qtd = format!("{:.4}", item.quantidade.unwrap_or(0.0));
        let u = item.unidade.as_deref().unwrap_or("");
        additional.push_str(&format!(
            "<gPerecimento nItem=\"{}\"><vIBS>{vi}</vIBS><vCBS>{vc}</vCBS>\
             <gControleEstoque><qPerecimento>{qtd}</qPerecimento><uPerecimento>{u}</uPerecimento></gControleEstoque>\
             </gPerecimento>",
            item.item
        ));
    }
    build_event_xml(
        access_key,
        event_types::RTC_ROUBO_PERDA_ADQUIRENTE,
        seq,
        tax_id,
        environment,
        &additional,
    )
}

/// Build RTC event: Perecimento/roubo transporte fornecedor (tpEvento=112130).
pub fn build_rtc_roubo_perda_fornecedor(
    access_key: &str,
    seq: u32,
    environment: SefazEnvironment,
    tax_id: &str,
    c_uf: &str,
    ver_aplic: &str,
    itens: &[RtcItem],
) -> String {
    let c_orgao = get_state_code(c_uf)
        .map(|c| c.to_string())
        .unwrap_or_default();
    let mut additional = format!(
        "<cOrgaoAutor>{c_orgao}</cOrgaoAutor>\
         <tpAutor>1</tpAutor>\
         <verAplic>{ver_aplic}</verAplic>"
    );
    for item in itens {
        let vi = format!("{:.2}", item.v_ibs);
        let vc = format!("{:.2}", item.v_cbs);
        let gvi = format!("{:.2}", item.g_controle_estoque_v_ibs.unwrap_or(0.0));
        let gvc = format!("{:.2}", item.g_controle_estoque_v_cbs.unwrap_or(0.0));
        let qtd = format!("{:.4}", item.quantidade.unwrap_or(0.0));
        let u = item.unidade.as_deref().unwrap_or("");
        additional.push_str(&format!(
            "<gPerecimento nItem=\"{}\"><vIBS>{vi}</vIBS><vCBS>{vc}</vCBS>\
             <gControleEstoque><qPerecimento>{qtd}</qPerecimento><uPerecimento>{u}</uPerecimento>\
             <vIBS>{gvi}</vIBS><vCBS>{gvc}</vCBS></gControleEstoque>\
             </gPerecimento>",
            item.item
        ));
    }
    build_event_xml(
        access_key,
        event_types::RTC_ROUBO_PERDA_FORNECEDOR,
        seq,
        tax_id,
        environment,
        &additional,
    )
}

/// Build RTC event: Fornecimento não realizado (tpEvento=112140).
pub fn build_rtc_fornecimento_nao_realizado(
    access_key: &str,
    seq: u32,
    environment: SefazEnvironment,
    tax_id: &str,
    c_uf: &str,
    ver_aplic: &str,
    itens: &[RtcItem],
) -> String {
    let c_orgao = get_state_code(c_uf)
        .map(|c| c.to_string())
        .unwrap_or_default();
    let mut additional = format!(
        "<cOrgaoAutor>{c_orgao}</cOrgaoAutor>\
         <tpAutor>1</tpAutor>\
         <verAplic>{ver_aplic}</verAplic>"
    );
    for item in itens {
        let vi = format!("{:.2}", item.v_ibs);
        let vc = format!("{:.2}", item.v_cbs);
        let qtd = format!("{:.4}", item.quantidade.unwrap_or(0.0));
        let u = item.unidade.as_deref().unwrap_or("");
        additional.push_str(&format!(
            "<gItemNaoFornecido nItem=\"{}\"><vIBS>{vi}</vIBS><vCBS>{vc}</vCBS>\
             <gControleEstoque><qNaoFornecida>{qtd}</qNaoFornecida><uNaoFornecida>{u}</uNaoFornecida></gControleEstoque>\
             </gItemNaoFornecido>",
            item.item
        ));
    }
    build_event_xml(
        access_key,
        event_types::RTC_FORNECIMENTO_NAO_REALIZADO,
        seq,
        tax_id,
        environment,
        &additional,
    )
}

/// Build RTC event: Atualização da data de previsão de entrega (tpEvento=112150).
pub fn build_rtc_atualizacao_data_entrega(
    access_key: &str,
    seq: u32,
    environment: SefazEnvironment,
    tax_id: &str,
    c_uf: &str,
    ver_aplic: &str,
    data_prevista: &str,
) -> String {
    let c_orgao = get_state_code(c_uf)
        .map(|c| c.to_string())
        .unwrap_or_default();
    let additional = format!(
        "<cOrgaoAutor>{c_orgao}</cOrgaoAutor>\
         <tpAutor>1</tpAutor>\
         <verAplic>{ver_aplic}</verAplic>\
         <dPrevEntrega>{data_prevista}</dPrevEntrega>"
    );
    build_event_xml(
        access_key,
        event_types::RTC_ATUALIZACAO_DATA_ENTREGA,
        seq,
        tax_id,
        environment,
        &additional,
    )
}

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

    // ── EPEC request ─────────────────────────────────────────────────

    /// Sample NF-e XML for EPEC extraction tests.
    fn sample_nfe_xml() -> String {
        format!(
            concat!(
                r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe">"#,
                r#"<infNFe versao="4.00" Id="NFe{key}">"#,
                r#"<ide><tpNF>1</tpNF><dhEmi>2026-03-12T10:00:00-03:00</dhEmi></ide>"#,
                r#"<emit><CNPJ>12345678000199</CNPJ><IE>123456789</IE></emit>"#,
                r#"<dest><CNPJ>98765432000188</CNPJ><UF>RJ</UF><IE>987654321</IE></dest>"#,
                r#"<total><ICMSTot><vNF>1500.00</vNF><vICMS>270.00</vICMS><vST>0.00</vST></ICMSTot></total>"#,
                r#"<infAdic/>"#,
                r#"<infRespTec><verProc>fiscal-rs 0.1.0</verProc></infRespTec>"#,
                r#"</infNFe></NFe>"#,
            ),
            key = TEST_KEY
        )
    }

    #[test]
    fn extract_epec_data_from_nfe_xml() {
        let xml = sample_nfe_xml();
        let data = extract_epec_data(&xml, None).unwrap();

        assert_eq!(data.access_key, TEST_KEY);
        assert_eq!(data.c_orgao_autor, "35");
        assert_eq!(data.ver_aplic, "fiscal-rs 0.1.0");
        assert_eq!(data.dh_emi, "2026-03-12T10:00:00-03:00");
        assert_eq!(data.tp_nf, "1");
        assert_eq!(data.emit_ie, "123456789");
        assert_eq!(data.dest_uf, "RJ");
        assert_eq!(data.dest_id_tag, "<CNPJ>98765432000188</CNPJ>");
        assert_eq!(data.dest_ie.as_deref(), Some("987654321"));
        assert_eq!(data.v_nf, "1500.00");
        assert_eq!(data.v_icms, "270.00");
        assert_eq!(data.v_st, "0.00");
        assert_eq!(data.tax_id, "12345678000199");
    }

    #[test]
    fn extract_epec_data_with_ver_aplic_override() {
        let xml = sample_nfe_xml();
        let data = extract_epec_data(&xml, Some("MyApp 2.0")).unwrap();
        assert_eq!(data.ver_aplic, "MyApp 2.0");
    }

    #[test]
    fn extract_epec_data_with_cpf_dest() {
        let xml = format!(
            concat!(
                r#"<NFe><infNFe versao="4.00" Id="NFe{key}">"#,
                r#"<ide><tpNF>1</tpNF><dhEmi>2026-03-12T10:00:00-03:00</dhEmi></ide>"#,
                r#"<emit><CNPJ>12345678000199</CNPJ><IE>123456789</IE></emit>"#,
                r#"<dest><CPF>12345678909</CPF><UF>SP</UF></dest>"#,
                r#"<total><ICMSTot><vNF>100.00</vNF><vICMS>18.00</vICMS><vST>0.00</vST></ICMSTot></total>"#,
                r#"<infRespTec><verProc>test</verProc></infRespTec>"#,
                r#"</infNFe></NFe>"#,
            ),
            key = TEST_KEY
        );
        let data = extract_epec_data(&xml, None).unwrap();
        assert_eq!(data.dest_id_tag, "<CPF>12345678909</CPF>");
        assert_eq!(data.dest_ie, None);
    }

    #[test]
    fn extract_epec_data_with_id_estrangeiro() {
        let xml = format!(
            concat!(
                r#"<NFe><infNFe versao="4.00" Id="NFe{key}">"#,
                r#"<ide><tpNF>1</tpNF><dhEmi>2026-03-12T10:00:00-03:00</dhEmi></ide>"#,
                r#"<emit><CNPJ>12345678000199</CNPJ><IE>123456789</IE></emit>"#,
                r#"<dest><idEstrangeiro>ABC123</idEstrangeiro><UF>EX</UF></dest>"#,
                r#"<total><ICMSTot><vNF>500.00</vNF><vICMS>0.00</vICMS><vST>0.00</vST></ICMSTot></total>"#,
                r#"<infRespTec><verProc>test</verProc></infRespTec>"#,
                r#"</infNFe></NFe>"#,
            ),
            key = TEST_KEY
        );
        let data = extract_epec_data(&xml, None).unwrap();
        assert_eq!(data.dest_id_tag, "<idEstrangeiro>ABC123</idEstrangeiro>");
    }

    #[test]
    fn extract_epec_data_rejects_missing_inf_nfe() {
        let xml = "<NFe><ide/></NFe>";
        let err = extract_epec_data(xml, None).unwrap_err();
        assert!(matches!(err, fiscal_core::FiscalError::XmlParsing(_)));
    }

    #[test]
    fn build_epec_request_structure() {
        let xml = sample_nfe_xml();
        let data = extract_epec_data(&xml, None).unwrap();
        let request = build_epec_request(&data, SefazEnvironment::Homologation);

        // Event type
        assert!(
            request.contains("<tpEvento>110140</tpEvento>"),
            "EPEC must have tpEvento=110140"
        );
        // Description
        assert!(
            request.contains("<descEvento>EPEC</descEvento>"),
            "EPEC must have descEvento=EPEC"
        );
        // cOrgao in envelope should be 91 (AN)
        assert!(
            request.contains("<cOrgao>91</cOrgao>"),
            "EPEC envelope must use cOrgao=91 (AN)"
        );
        // cOrgaoAutor in detEvento should be the issuer's UF code
        assert!(
            request.contains("<cOrgaoAutor>35</cOrgaoAutor>"),
            "EPEC detEvento must contain cOrgaoAutor from issuer UF"
        );
        // tpAutor=1
        assert!(request.contains("<tpAutor>1</tpAutor>"));
        // verAplic
        assert!(request.contains("<verAplic>fiscal-rs 0.1.0</verAplic>"));
        // dhEmi
        assert!(request.contains("<dhEmi>2026-03-12T10:00:00-03:00</dhEmi>"));
        // tpNF
        assert!(request.contains("<tpNF>1</tpNF>"));
        // Issuer IE
        assert!(request.contains("<IE>123456789</IE>"));
        // Dest section
        assert!(request.contains("<dest>"));
        assert!(request.contains("<UF>RJ</UF>"));
        assert!(request.contains("<CNPJ>98765432000188</CNPJ>"));
        assert!(
            request.contains("<IE>987654321</IE>"),
            "Dest IE should be present"
        );
        assert!(request.contains("<vNF>1500.00</vNF>"));
        assert!(request.contains("<vICMS>270.00</vICMS>"));
        assert!(request.contains("<vST>0.00</vST>"));
        assert!(request.contains("</dest>"));
        // nSeqEvento=1 always
        assert!(request.contains("<nSeqEvento>1</nSeqEvento>"));
        // envEvento wrapper
        assert!(request.contains("<envEvento"));
        assert!(request.contains("</envEvento>"));
        // Access key
        assert!(request.contains(&format!("<chNFe>{TEST_KEY}</chNFe>")));
        // Event ID
        assert!(request.contains(&format!("Id=\"ID110140{TEST_KEY}01\"")));
        // tpAmb=2 (homologation)
        assert!(request.contains("<tpAmb>2</tpAmb>"));
        // Issuer tax ID in envelope
        assert!(request.contains("<CNPJ>12345678000199</CNPJ>"));
    }

    #[test]
    fn build_epec_request_without_dest_ie() {
        let xml = format!(
            concat!(
                r#"<NFe><infNFe versao="4.00" Id="NFe{key}">"#,
                r#"<ide><tpNF>0</tpNF><dhEmi>2026-01-01T08:00:00-03:00</dhEmi></ide>"#,
                r#"<emit><CNPJ>12345678000199</CNPJ><IE>111222333</IE></emit>"#,
                r#"<dest><CPF>12345678909</CPF><UF>MG</UF></dest>"#,
                r#"<total><ICMSTot><vNF>200.00</vNF><vICMS>36.00</vICMS><vST>5.00</vST></ICMSTot></total>"#,
                r#"<infRespTec><verProc>test</verProc></infRespTec>"#,
                r#"</infNFe></NFe>"#,
            ),
            key = TEST_KEY
        );
        let data = extract_epec_data(&xml, None).unwrap();
        let request = build_epec_request(&data, SefazEnvironment::Production);

        // Should NOT contain dest IE (CPF dest, no IE)
        // The dest section should have CPF but no IE
        assert!(request.contains("<CPF>12345678909</CPF>"));
        assert!(request.contains("<tpAmb>1</tpAmb>")); // Production
        assert!(request.contains("<tpNF>0</tpNF>")); // Entrada
        assert!(request.contains("<vST>5.00</vST>"));
    }

    // ── Cancelamento por substituição (110112) ────────────────────

    #[test]
    fn cancel_substituicao_request_structure() {
        let ref_key = "35240112345678000195650010000000021000000028";
        let xml = build_cancel_substituicao_request(
            TEST_KEY,
            ref_key,
            "135220000009921",
            "Erro na emissao da NFCe",
            "FISCAL-RS-1.0",
            1,
            SefazEnvironment::Homologation,
            "12345678000199",
        );
        assert!(xml.contains("<tpEvento>110112</tpEvento>"));
        assert!(xml.contains("<descEvento>Cancelamento por substituicao</descEvento>"));
        assert!(xml.contains("<cOrgaoAutor>35</cOrgaoAutor>"));
        assert!(xml.contains("<tpAutor>1</tpAutor>"));
        assert!(xml.contains("<verAplic>FISCAL-RS-1.0</verAplic>"));
        assert!(xml.contains("<nProt>135220000009921</nProt>"));
        assert!(xml.contains("<xJust>Erro na emissao da NFCe</xJust>"));
        assert!(xml.contains(&format!("<chNFeRef>{ref_key}</chNFeRef>")));
        assert!(xml.contains("<cOrgao>35</cOrgao>"));
    }

    #[test]
    fn cancel_substituicao_event_id_format() {
        let xml = build_cancel_substituicao_request(
            TEST_KEY,
            "35240112345678000195650010000000021000000028",
            "135220000009921",
            "Justificativa teste",
            "APP-1.0",
            1,
            SefazEnvironment::Homologation,
            "12345678000199",
        );
        let expected_id = format!("ID110112{TEST_KEY}01");
        assert!(xml.contains(&format!("Id=\"{expected_id}\"")));
    }

    // ── Ator interessado (110150) ─────────────────────────────────

    #[test]
    fn ator_interessado_request_with_cnpj() {
        let xml = build_ator_interessado_request(
            TEST_KEY,
            1,
            "FISCAL-RS-1.0",
            Some("12345678000199"),
            None,
            0,
            "SP",
            1,
            SefazEnvironment::Homologation,
            TEST_CNPJ,
        );
        assert!(xml.contains("<tpEvento>110150</tpEvento>"));
        assert!(xml.contains("<descEvento>Ator interessado na NF-e</descEvento>"));
        assert!(xml.contains("<cOrgao>91</cOrgao>"));
        assert!(xml.contains("<cOrgaoAutor>35</cOrgaoAutor>"));
        assert!(xml.contains("<tpAutor>1</tpAutor>"));
        assert!(xml.contains("<autXML><CNPJ>12345678000199</CNPJ></autXML>"));
        assert!(xml.contains("<tpAutorizacao>0</tpAutorizacao>"));
        assert!(!xml.contains("<xCondUso>"));
    }

    #[test]
    fn ator_interessado_request_with_cpf_and_cond_uso() {
        let xml = build_ator_interessado_request(
            TEST_KEY,
            2,
            "APP-2.0",
            None,
            Some("12345678901"),
            1,
            "SP",
            1,
            SefazEnvironment::Homologation,
            TEST_CNPJ,
        );
        assert!(xml.contains("<autXML><CPF>12345678901</CPF></autXML>"));
        assert!(xml.contains("<tpAutorizacao>1</tpAutorizacao>"));
        assert!(xml.contains("<xCondUso>"));
        assert!(xml.contains("transportador declarado no campo CNPJ/CPF"));
    }

    // ── Comprovante de entrega (110130) ───────────────────────────

    #[test]
    fn comprovante_entrega_request_structure() {
        let xml = build_comprovante_entrega_request(
            TEST_KEY,
            "FISCAL-RS-1.0",
            "2024-01-15T10:30:00-03:00",
            "12345678901",
            "Joao da Silva",
            Some("-23.5505"),
            Some("-46.6333"),
            "abc123hashbase64==",
            "2024-01-15T10:31:00-03:00",
            "SP",
            1,
            SefazEnvironment::Homologation,
            "12345678000199",
        );
        assert!(xml.contains("<tpEvento>110130</tpEvento>"));
        assert!(xml.contains("<descEvento>Comprovante de Entrega da NF-e</descEvento>"));
        assert!(xml.contains("<cOrgao>91</cOrgao>"));
        assert!(xml.contains("<cOrgaoAutor>35</cOrgaoAutor>"));
        assert!(xml.contains("<tpAutor>1</tpAutor>"));
        assert!(xml.contains("<dhEntrega>2024-01-15T10:30:00-03:00</dhEntrega>"));
        assert!(xml.contains("<nDoc>12345678901</nDoc>"));
        assert!(xml.contains("<xNome>Joao da Silva</xNome>"));
        assert!(xml.contains("<latGPS>-23.5505</latGPS>"));
        assert!(xml.contains("<longGPS>-46.6333</longGPS>"));
        assert!(xml.contains("<hashComprovante>abc123hashbase64==</hashComprovante>"));
        assert!(xml.contains("<dhHashComprovante>2024-01-15T10:31:00-03:00</dhHashComprovante>"));
    }

    #[test]
    fn comprovante_entrega_without_gps() {
        let xml = build_comprovante_entrega_request(
            TEST_KEY,
            "APP-1.0",
            "2024-01-15T10:30:00-03:00",
            "12345678901",
            "Maria Santos",
            None,
            None,
            "hashvalue==",
            "2024-01-15T10:31:00-03:00",
            "SP",
            1,
            SefazEnvironment::Homologation,
            "12345678000199",
        );
        assert!(!xml.contains("<latGPS>"));
        assert!(!xml.contains("<longGPS>"));
        assert!(xml.contains("<hashComprovante>hashvalue==</hashComprovante>"));
    }

    // ── Cancelamento comprovante de entrega (110131) ──────────────

    #[test]
    fn cancel_comprovante_entrega_request_structure() {
        let xml = build_cancel_comprovante_entrega_request(
            TEST_KEY,
            "FISCAL-RS-1.0",
            "135220000009999",
            "SP",
            1,
            SefazEnvironment::Homologation,
            "12345678000199",
        );
        assert!(xml.contains("<tpEvento>110131</tpEvento>"));
        assert!(
            xml.contains("<descEvento>Cancelamento Comprovante de Entrega da NF-e</descEvento>")
        );
        assert!(xml.contains("<cOrgao>91</cOrgao>"));
        assert!(xml.contains("<cOrgaoAutor>35</cOrgaoAutor>"));
        assert!(xml.contains("<tpAutor>1</tpAutor>"));
        assert!(xml.contains("<verAplic>FISCAL-RS-1.0</verAplic>"));
        assert!(xml.contains("<nProtEvento>135220000009999</nProtEvento>"));
    }

    // ── Insucesso na entrega (110192) ─────────────────────────────

    #[test]
    fn insucesso_entrega_request_structure() {
        let xml = build_insucesso_entrega_request(
            TEST_KEY,
            "FISCAL-RS-1.0",
            "2024-01-15T14:00:00-03:00",
            Some(3),
            1,
            None,
            Some("-23.5505"),
            Some("-46.6333"),
            "hashinsucesso==",
            "2024-01-15T14:01:00-03:00",
            "SP",
            1,
            SefazEnvironment::Homologation,
            "12345678000199",
        );
        assert!(xml.contains("<tpEvento>110192</tpEvento>"));
        assert!(xml.contains("<descEvento>Insucesso na Entrega da NF-e</descEvento>"));
        assert!(xml.contains("<cOrgao>91</cOrgao>"));
        assert!(xml.contains("<cOrgaoAutor>35</cOrgaoAutor>"));
        assert!(xml.contains("<dhTentativaEntrega>2024-01-15T14:00:00-03:00</dhTentativaEntrega>"));
        assert!(xml.contains("<nTentativa>3</nTentativa>"));
        assert!(xml.contains("<tpMotivo>1</tpMotivo>"));
        assert!(!xml.contains("<xJustMotivo>"));
        assert!(xml.contains("<latGPS>-23.5505</latGPS>"));
        assert!(xml.contains("<longGPS>-46.6333</longGPS>"));
        assert!(xml.contains("<hashTentativaEntrega>hashinsucesso==</hashTentativaEntrega>"));
        assert!(xml.contains(
            "<dhHashTentativaEntrega>2024-01-15T14:01:00-03:00</dhHashTentativaEntrega>"
        ));
    }

    #[test]
    fn insucesso_entrega_with_reason_type_4_includes_justification() {
        let xml = build_insucesso_entrega_request(
            TEST_KEY,
            "APP-1.0",
            "2024-01-15T14:00:00-03:00",
            None,
            4,
            Some("Destinatario mudou de endereco"),
            None,
            None,
            "hashval==",
            "2024-01-15T14:01:00-03:00",
            "SP",
            1,
            SefazEnvironment::Homologation,
            "12345678000199",
        );
        assert!(xml.contains("<tpMotivo>4</tpMotivo>"));
        assert!(xml.contains("<xJustMotivo>Destinatario mudou de endereco</xJustMotivo>"));
        assert!(!xml.contains("<nTentativa>"));
        assert!(!xml.contains("<latGPS>"));
    }

    // ── Cancelamento insucesso entrega (110193) ───────────────────

    #[test]
    fn cancel_insucesso_entrega_request_structure() {
        let xml = build_cancel_insucesso_entrega_request(
            TEST_KEY,
            "FISCAL-RS-1.0",
            "135220000009888",
            "SP",
            1,
            SefazEnvironment::Homologation,
            "12345678000199",
        );
        assert!(xml.contains("<tpEvento>110193</tpEvento>"));
        assert!(xml.contains("<descEvento>Cancelamento Insucesso na Entrega da NF-e</descEvento>"));
        assert!(xml.contains("<cOrgao>91</cOrgao>"));
        assert!(xml.contains("<cOrgaoAutor>35</cOrgaoAutor>"));
        assert!(xml.contains("<verAplic>FISCAL-RS-1.0</verAplic>"));
        assert!(xml.contains("<nProtEvento>135220000009888</nProtEvento>"));
    }

    // ── Prorrogação ICMS (111500/111501) ────────────────────────────

    #[test]
    fn prorrogacao_first_term_request_structure() {
        let items = vec![
            ProrrogacaoItem {
                num_item: 1,
                qtde: 10.0,
            },
            ProrrogacaoItem {
                num_item: 2,
                qtde: 5.5,
            },
        ];
        let xml = build_prorrogacao_request(
            TEST_KEY,
            "135220000009921",
            &items,
            false,
            1,
            SefazEnvironment::Homologation,
            TEST_CNPJ,
        );
        assert!(
            xml.contains("<tpEvento>111500</tpEvento>"),
            "First-term prorrogacao must use tpEvento=111500"
        );
        assert!(xml.contains("<descEvento>Pedido de Prorrogacao</descEvento>"));
        assert!(xml.contains("<nProt>135220000009921</nProt>"));
        assert!(xml.contains("<itemPedido numItem=\"1\"><qtdeItem>10</qtdeItem></itemPedido>"));
        assert!(xml.contains("<itemPedido numItem=\"2\"><qtdeItem>5.5</qtdeItem></itemPedido>"));
        assert!(xml.contains("<cOrgao>35</cOrgao>"));
        assert!(xml.contains(&format!("<chNFe>{TEST_KEY}</chNFe>")));
        let expected_id = format!("ID111500{TEST_KEY}01");
        assert!(xml.contains(&format!("Id=\"{expected_id}\"")));
    }

    #[test]
    fn prorrogacao_second_term_request_structure() {
        let items = vec![ProrrogacaoItem {
            num_item: 1,
            qtde: 3.0,
        }];
        let xml = build_prorrogacao_request(
            TEST_KEY,
            "135220000009921",
            &items,
            true,
            1,
            SefazEnvironment::Homologation,
            TEST_CNPJ,
        );
        assert!(
            xml.contains("<tpEvento>111501</tpEvento>"),
            "Second-term prorrogacao must use tpEvento=111501"
        );
        assert!(xml.contains("<descEvento>Pedido de Prorrogacao</descEvento>"));
    }

    // ── Cancelamento de prorrogação ICMS (111502/111503) ────────────

    #[test]
    fn cancel_prorrogacao_first_term_request_structure() {
        let xml = build_cancel_prorrogacao_request(
            TEST_KEY,
            "135220000009921",
            false,
            1,
            SefazEnvironment::Homologation,
            TEST_CNPJ,
        );
        assert!(
            xml.contains("<tpEvento>111502</tpEvento>"),
            "First-term cancel prorrogacao must use tpEvento=111502"
        );
        assert!(xml.contains("<descEvento>Cancelamento de Pedido de Prorrogacao</descEvento>"));
        let expected_id_cancelado = format!("ID111500{TEST_KEY}01");
        assert!(
            xml.contains(&format!(
                "<idPedidoCancelado>{expected_id_cancelado}</idPedidoCancelado>"
            )),
            "Must contain idPedidoCancelado referencing the original prorrogacao event"
        );
        assert!(xml.contains("<nProt>135220000009921</nProt>"));
        assert!(xml.contains("<cOrgao>35</cOrgao>"));
    }

    #[test]
    fn cancel_prorrogacao_second_term_request_structure() {
        let xml = build_cancel_prorrogacao_request(
            TEST_KEY,
            "135220000009921",
            true,
            2,
            SefazEnvironment::Homologation,
            TEST_CNPJ,
        );
        assert!(
            xml.contains("<tpEvento>111503</tpEvento>"),
            "Second-term cancel prorrogacao must use tpEvento=111503"
        );
        assert!(xml.contains("<descEvento>Cancelamento de Pedido de Prorrogacao</descEvento>"));
        // second term: orig_event = 111501
        let expected_id_cancelado = format!("ID111501{TEST_KEY}02");
        assert!(
            xml.contains(&format!(
                "<idPedidoCancelado>{expected_id_cancelado}</idPedidoCancelado>"
            )),
            "Must contain idPedidoCancelado referencing the 2nd-term prorrogacao event"
        );
    }

    // ── CSC request (admCscNFCe) ───────────────────────────────────

    #[test]
    fn csc_request_query_structure() {
        let xml = build_csc_request(
            SefazEnvironment::Homologation,
            1,
            "12345678000195",
            None,
            None,
        );
        assert!(xml.contains("<admCscNFCe versao=\"1.00\""));
        assert!(xml.contains(&format!("xmlns=\"{NFE_NAMESPACE}\"")));
        assert!(xml.contains("<tpAmb>2</tpAmb>"));
        assert!(xml.contains("<indOp>1</indOp>"));
        assert!(xml.contains("<raizCNPJ>12345678</raizCNPJ>"));
        assert!(!xml.contains("<dadosCsc>"));
    }

    #[test]
    fn csc_request_new_csc_structure() {
        let xml = build_csc_request(
            SefazEnvironment::Production,
            2,
            "12345678000195",
            None,
            None,
        );
        assert!(xml.contains("<tpAmb>1</tpAmb>"));
        assert!(xml.contains("<indOp>2</indOp>"));
        assert!(xml.contains("<raizCNPJ>12345678</raizCNPJ>"));
        assert!(!xml.contains("<dadosCsc>"));
    }

    #[test]
    fn csc_request_revoke_includes_dados_csc() {
        let xml = build_csc_request(
            SefazEnvironment::Homologation,
            3,
            "12345678000195",
            Some("000001"),
            Some("ABC123DEF456"),
        );
        assert!(xml.contains("<indOp>3</indOp>"));
        assert!(xml.contains("<dadosCsc>"));
        assert!(xml.contains("<idCsc>000001</idCsc>"));
        assert!(xml.contains("<codigoCsc>ABC123DEF456</codigoCsc>"));
        assert!(xml.contains("</dadosCsc>"));
    }

    #[test]
    fn csc_request_raiz_cnpj_is_first_8_digits() {
        let xml = build_csc_request(
            SefazEnvironment::Homologation,
            1,
            "98765432000188",
            None,
            None,
        );
        assert!(xml.contains("<raizCNPJ>98765432</raizCNPJ>"));
    }

    // ── Event batch request ─────────────────────────────────────────

    #[test]
    fn event_batch_request_structure() {
        let events = vec![
            EventItem {
                access_key: TEST_KEY.to_string(),
                event_type: event_types::CONFIRMATION,
                seq: 1,
                tax_id: TEST_CNPJ.to_string(),
                additional_tags: String::new(),
            },
            EventItem {
                access_key: TEST_KEY.to_string(),
                event_type: event_types::AWARENESS,
                seq: 1,
                tax_id: TEST_CNPJ.to_string(),
                additional_tags: String::new(),
            },
        ];
        let xml = build_event_batch_request(
            "AN",
            &events,
            Some("202401151030001"),
            SefazEnvironment::Homologation,
        );

        assert!(xml.contains("<envEvento"));
        assert!(xml.contains("<idLote>202401151030001</idLote>"));
        // Two <evento> elements
        let evento_count = xml.matches("<evento ").count();
        assert_eq!(
            evento_count, 2,
            "Should have 2 <evento> elements, got {evento_count}"
        );
        // Both event types present
        assert!(xml.contains("<tpEvento>210200</tpEvento>"));
        assert!(xml.contains("<tpEvento>210210</tpEvento>"));
        // cOrgao=91 for AN
        assert!(xml.contains("<cOrgao>91</cOrgao>"));
        // Descriptions
        assert!(xml.contains("<descEvento>Confirmacao da Operacao</descEvento>"));
        assert!(xml.contains("<descEvento>Ciencia da Operacao</descEvento>"));
    }

    #[test]
    fn event_batch_skips_epec() {
        let events = vec![
            EventItem {
                access_key: TEST_KEY.to_string(),
                event_type: event_types::EPEC,
                seq: 1,
                tax_id: TEST_CNPJ.to_string(),
                additional_tags: String::new(),
            },
            EventItem {
                access_key: TEST_KEY.to_string(),
                event_type: event_types::AWARENESS,
                seq: 1,
                tax_id: TEST_CNPJ.to_string(),
                additional_tags: String::new(),
            },
        ];
        let xml =
            build_event_batch_request("AN", &events, Some("123"), SefazEnvironment::Homologation);

        // EPEC should be skipped
        assert!(!xml.contains("<tpEvento>110140</tpEvento>"));
        // Ciencia should be present
        assert!(xml.contains("<tpEvento>210210</tpEvento>"));
        // Only one <evento> element
        let evento_count = xml.matches("<evento ").count();
        assert_eq!(
            evento_count, 1,
            "EPEC should be skipped, got {evento_count} events"
        );
    }

    #[test]
    fn event_batch_with_additional_tags() {
        let events = vec![EventItem {
            access_key: TEST_KEY.to_string(),
            event_type: event_types::OPERATION_NOT_PERFORMED,
            seq: 1,
            tax_id: TEST_CNPJ.to_string(),
            additional_tags: "<xJust>Motivo teste</xJust>".to_string(),
        }];
        let xml =
            build_event_batch_request("AN", &events, Some("456"), SefazEnvironment::Homologation);
        assert!(xml.contains("<xJust>Motivo teste</xJust>"));
        assert!(xml.contains("<tpEvento>210240</tpEvento>"));
        assert!(xml.contains("<descEvento>Operacao nao Realizada</descEvento>"));
    }

    #[test]
    #[should_panic(expected = "limited to 20")]
    fn event_batch_rejects_more_than_20() {
        let events: Vec<EventItem> = (0..21)
            .map(|i| EventItem {
                access_key: TEST_KEY.to_string(),
                event_type: event_types::AWARENESS,
                seq: i + 1,
                tax_id: TEST_CNPJ.to_string(),
                additional_tags: String::new(),
            })
            .collect();
        let _ = build_event_batch_request("AN", &events, None, SefazEnvironment::Homologation);
    }

    #[test]
    fn event_batch_with_cpf_tax_id() {
        let events = vec![EventItem {
            access_key: TEST_KEY.to_string(),
            event_type: event_types::AWARENESS,
            seq: 1,
            tax_id: TEST_CPF.to_string(),
            additional_tags: String::new(),
        }];
        let xml =
            build_event_batch_request("AN", &events, Some("789"), SefazEnvironment::Homologation);
        assert!(xml.contains(&format!("<CPF>{TEST_CPF}</CPF>")));
    }

    #[test]
    fn event_batch_event_id_format() {
        let events = vec![EventItem {
            access_key: TEST_KEY.to_string(),
            event_type: event_types::CONFIRMATION,
            seq: 3,
            tax_id: TEST_CNPJ.to_string(),
            additional_tags: String::new(),
        }];
        let xml =
            build_event_batch_request("SP", &events, Some("111"), SefazEnvironment::Homologation);
        let expected_id = format!("ID210200{TEST_KEY}03");
        assert!(
            xml.contains(&format!("Id=\"{expected_id}\"")),
            "Event ID must follow ID{{tpEvento}}{{chNFe}}{{nSeqEvento:02}} format"
        );
        // cOrgao=35 for SP
        assert!(xml.contains("<cOrgao>35</cOrgao>"));
    }

    // ── Conciliação financeira (110750/110751) ────────────────────────

    #[test]
    fn conciliacao_request_structure() {
        let det_pag = vec![ConciliacaoDetPag {
            ind_pag: Some("0".to_string()),
            t_pag: "01".to_string(),
            x_pag: Some("Dinheiro".to_string()),
            v_pag: "150.00".to_string(),
            d_pag: "2024-06-15".to_string(),
            cnpj_pag: None,
            uf_pag: None,
            cnpj_if: None,
            t_band: None,
            c_aut: None,
            cnpj_receb: None,
            uf_receb: None,
        }];
        let xml = build_conciliacao_request(
            TEST_KEY,
            "FISCAL-RS-1.0",
            &det_pag,
            false,
            None,
            1,
            SefazEnvironment::Homologation,
            TEST_CNPJ,
        );

        assert!(xml.contains("<tpEvento>110750</tpEvento>"));
        assert!(xml.contains("<descEvento>ECONF</descEvento>"));
        assert!(xml.contains("<verAplic>FISCAL-RS-1.0</verAplic>"));
        assert!(xml.contains("<detPag>"));
        assert!(xml.contains("<indPag>0</indPag>"));
        assert!(xml.contains("<tPag>01</tPag>"));
        assert!(xml.contains("<xPag>Dinheiro</xPag>"));
        assert!(xml.contains("<vPag>150.00</vPag>"));
        assert!(xml.contains("<dPag>2024-06-15</dPag>"));
        assert!(xml.contains("</detPag>"));
        assert!(xml.contains("<cOrgao>35</cOrgao>"));
    }

    #[test]
    fn conciliacao_request_with_card_payment() {
        let det_pag = vec![ConciliacaoDetPag {
            ind_pag: Some("0".to_string()),
            t_pag: "03".to_string(),
            x_pag: None,
            v_pag: "250.50".to_string(),
            d_pag: "2024-06-15".to_string(),
            cnpj_pag: Some("11222333000144".to_string()),
            uf_pag: Some("SP".to_string()),
            cnpj_if: Some("99888777000166".to_string()),
            t_band: Some("01".to_string()),
            c_aut: Some("AUTH123".to_string()),
            cnpj_receb: Some("55444333000122".to_string()),
            uf_receb: Some("RJ".to_string()),
        }];
        let xml = build_conciliacao_request(
            TEST_KEY,
            "APP-2.0",
            &det_pag,
            false,
            None,
            1,
            SefazEnvironment::Production,
            TEST_CNPJ,
        );

        assert!(xml.contains("<tPag>03</tPag>"));
        assert!(xml.contains("<CNPJPag>11222333000144</CNPJPag>"));
        assert!(xml.contains("<UFPag>SP</UFPag>"));
        assert!(xml.contains("<CNPJIF>99888777000166</CNPJIF>"));
        assert!(xml.contains("<tBand>01</tBand>"));
        assert!(xml.contains("<cAut>AUTH123</cAut>"));
        assert!(xml.contains("<CNPJReceb>55444333000122</CNPJReceb>"));
        assert!(xml.contains("<UFReceb>RJ</UFReceb>"));
        assert!(xml.contains("<tpAmb>1</tpAmb>"));
    }

    #[test]
    fn conciliacao_cancel_request_structure() {
        let xml = build_conciliacao_request(
            TEST_KEY,
            "FISCAL-RS-1.0",
            &[],
            true,
            Some("135220000009999"),
            1,
            SefazEnvironment::Homologation,
            TEST_CNPJ,
        );

        assert!(xml.contains("<tpEvento>110751</tpEvento>"));
        assert!(xml.contains("<descEvento>Cancelamento Conciliacao Financeira</descEvento>"));
        assert!(xml.contains("<verAplic>FISCAL-RS-1.0</verAplic>"));
        assert!(xml.contains("<nProtEvento>135220000009999</nProtEvento>"));
        assert!(!xml.contains("<detPag>"));
    }

    #[test]
    fn conciliacao_multiple_payments() {
        let det_pag = vec![
            ConciliacaoDetPag {
                ind_pag: Some("0".to_string()),
                t_pag: "01".to_string(),
                x_pag: None,
                v_pag: "100.00".to_string(),
                d_pag: "2024-06-15".to_string(),
                cnpj_pag: None,
                uf_pag: None,
                cnpj_if: None,
                t_band: None,
                c_aut: None,
                cnpj_receb: None,
                uf_receb: None,
            },
            ConciliacaoDetPag {
                ind_pag: Some("1".to_string()),
                t_pag: "03".to_string(),
                x_pag: None,
                v_pag: "200.00".to_string(),
                d_pag: "2024-07-15".to_string(),
                cnpj_pag: None,
                uf_pag: None,
                cnpj_if: None,
                t_band: None,
                c_aut: None,
                cnpj_receb: None,
                uf_receb: None,
            },
        ];
        let xml = build_conciliacao_request(
            TEST_KEY,
            "APP-1.0",
            &det_pag,
            false,
            None,
            1,
            SefazEnvironment::Homologation,
            TEST_CNPJ,
        );

        let det_pag_count = xml.matches("<detPag>").count();
        assert_eq!(det_pag_count, 2, "Should have 2 <detPag> elements");
        assert!(xml.contains("<vPag>100.00</vPag>"));
        assert!(xml.contains("<vPag>200.00</vPag>"));
        assert!(xml.contains("<dPag>2024-06-15</dPag>"));
        assert!(xml.contains("<dPag>2024-07-15</dPag>"));
    }

    // ── Event type constants ────────────────────────────────────────

    #[test]
    fn conciliacao_event_type_values() {
        assert_eq!(event_types::CONCILIACAO, 110750);
        assert_eq!(event_types::CANCEL_CONCILIACAO, 110751);
    }

    // ── Download (dist_dfe by access key) ───────────────────────────

    #[test]
    fn download_uses_cons_ch_nfe() {
        // sefazDownload builds a distDFeInt with consChNFe
        let xml = build_dist_dfe_request(
            "SP",
            TEST_CNPJ,
            None,
            Some(TEST_KEY),
            SefazEnvironment::Homologation,
        );
        assert!(xml.contains(&format!("<consChNFe><chNFe>{TEST_KEY}</chNFe></consChNFe>")));
        assert!(xml.contains("<tpAmb>2</tpAmb>"));
        assert!(xml.contains("<cUFAutor>35</cUFAutor>"));
        assert!(xml.contains(&format!("<CNPJ>{TEST_CNPJ}</CNPJ>")));
    }
}

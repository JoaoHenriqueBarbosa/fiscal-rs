use fiscal_core::state_codes::get_state_code;
use fiscal_core::types::SefazEnvironment;

use super::event_core::event_types;
use super::helpers::{build_event_xml, build_event_xml_with_org};

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

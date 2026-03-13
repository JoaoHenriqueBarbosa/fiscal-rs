use fiscal_core::constants::NFE_NAMESPACE;
use fiscal_core::state_codes::get_state_code;
use fiscal_core::types::SefazEnvironment;

use super::event_core::{build_event_id, event_description, event_types};
use super::helpers::{build_event_xml, tax_id_xml_tag};

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

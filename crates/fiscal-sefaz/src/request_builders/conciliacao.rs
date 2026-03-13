use fiscal_core::types::SefazEnvironment;

use super::event_core::event_types;
use super::helpers::build_event_xml_with_org;

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
    org_code_override: Option<&str>,
) -> String {
    if cancel {
        let protocol = cancel_protocol.unwrap_or("");
        let additional = format!(
            "<verAplic>{ver_aplic}</verAplic>\
             <nProtEvento>{protocol}</nProtEvento>"
        );
        build_event_xml_with_org(
            access_key,
            event_types::CANCEL_CONCILIACAO,
            seq,
            tax_id,
            environment,
            &additional,
            org_code_override,
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
        build_event_xml_with_org(
            access_key,
            event_types::CONCILIACAO,
            seq,
            tax_id,
            environment,
            &additional,
            org_code_override,
        )
    }
}

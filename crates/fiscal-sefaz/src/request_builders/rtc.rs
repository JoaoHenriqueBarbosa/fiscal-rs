use fiscal_core::state_codes::get_state_code;
use fiscal_core::types::SefazEnvironment;

use super::event_core::event_types;
use super::helpers::build_event_xml_with_org;

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
    org_code_override: Option<&str>,
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
    build_event_xml_with_org(
        access_key,
        event_types::RTC_INFO_PAGTO_INTEGRAL,
        seq,
        tax_id,
        environment,
        &additional,
        org_code_override,
    )
}

/// Build RTC event: Solicitação de apropriação de crédito presumido (tpEvento=211110).
#[allow(clippy::too_many_arguments)]
pub fn build_rtc_sol_aprop_cred_presumido(
    access_key: &str,
    seq: u32,
    environment: SefazEnvironment,
    tax_id: &str,
    c_uf: &str,
    ver_aplic: &str,
    itens: &[RtcCredPresItem],
    org_code_override: Option<&str>,
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
    build_event_xml_with_org(
        access_key,
        event_types::RTC_SOL_APROP_CRED_PRESUMIDO,
        seq,
        tax_id,
        environment,
        &additional,
        org_code_override,
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
    org_code_override: Option<&str>,
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
    build_event_xml_with_org(
        access_key,
        event_types::RTC_DESTINO_CONSUMO_PESSOAL,
        seq,
        tax_id,
        environment,
        &additional,
        org_code_override,
    )
}

/// Build RTC event: Aceite de débito na apuração (tpEvento=211128).
#[allow(clippy::too_many_arguments)]
pub fn build_rtc_aceite_debito(
    access_key: &str,
    seq: u32,
    environment: SefazEnvironment,
    tax_id: &str,
    c_uf: &str,
    ver_aplic: &str,
    ind_aceitacao: u8,
    org_code_override: Option<&str>,
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
    build_event_xml_with_org(
        access_key,
        event_types::RTC_ACEITE_DEBITO,
        seq,
        tax_id,
        environment,
        &additional,
        org_code_override,
    )
}

/// Build RTC event: Imobilização de item (tpEvento=211130).
#[allow(clippy::too_many_arguments)]
pub fn build_rtc_imobilizacao_item(
    access_key: &str,
    seq: u32,
    environment: SefazEnvironment,
    tax_id: &str,
    c_uf: &str,
    ver_aplic: &str,
    itens: &[RtcItem],
    org_code_override: Option<&str>,
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
    build_event_xml_with_org(
        access_key,
        event_types::RTC_IMOBILIZACAO_ITEM,
        seq,
        tax_id,
        environment,
        &additional,
        org_code_override,
    )
}

/// Build RTC event: Apropriação de crédito combustível (tpEvento=211140).
#[allow(clippy::too_many_arguments)]
pub fn build_rtc_apropriacao_credito_comb(
    access_key: &str,
    seq: u32,
    environment: SefazEnvironment,
    tax_id: &str,
    c_uf: &str,
    ver_aplic: &str,
    itens: &[RtcItem],
    org_code_override: Option<&str>,
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
    build_event_xml_with_org(
        access_key,
        event_types::RTC_APROPRIACAO_CREDITO_COMB,
        seq,
        tax_id,
        environment,
        &additional,
        org_code_override,
    )
}

/// Build RTC event: Apropriação de crédito bens/serviços (tpEvento=211150).
#[allow(clippy::too_many_arguments)]
pub fn build_rtc_apropriacao_credito_bens(
    access_key: &str,
    seq: u32,
    environment: SefazEnvironment,
    tax_id: &str,
    c_uf: &str,
    ver_aplic: &str,
    itens: &[RtcItem],
    org_code_override: Option<&str>,
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
    build_event_xml_with_org(
        access_key,
        event_types::RTC_APROPRIACAO_CREDITO_BENS,
        seq,
        tax_id,
        environment,
        &additional,
        org_code_override,
    )
}

/// Build RTC event: Manifestação transferência crédito IBS (tpEvento=212110).
#[allow(clippy::too_many_arguments)]
pub fn build_rtc_manif_transf_cred_ibs(
    access_key: &str,
    seq: u32,
    environment: SefazEnvironment,
    tax_id: &str,
    c_uf: &str,
    ver_aplic: &str,
    ind_aceitacao: u8,
    org_code_override: Option<&str>,
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
    build_event_xml_with_org(
        access_key,
        event_types::RTC_MANIF_TRANSF_CRED_IBS,
        seq,
        tax_id,
        environment,
        &additional,
        org_code_override,
    )
}

/// Build RTC event: Manifestação transferência crédito CBS (tpEvento=212120).
#[allow(clippy::too_many_arguments)]
pub fn build_rtc_manif_transf_cred_cbs(
    access_key: &str,
    seq: u32,
    environment: SefazEnvironment,
    tax_id: &str,
    c_uf: &str,
    ver_aplic: &str,
    ind_aceitacao: u8,
    org_code_override: Option<&str>,
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
    build_event_xml_with_org(
        access_key,
        event_types::RTC_MANIF_TRANSF_CRED_CBS,
        seq,
        tax_id,
        environment,
        &additional,
        org_code_override,
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
    org_code_override: Option<&str>,
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
    build_event_xml_with_org(
        access_key,
        event_types::RTC_CANCELA_EVENTO,
        seq,
        tax_id,
        environment,
        &additional,
        org_code_override,
    )
}

/// Build RTC event: Importação ZFM não convertida em isenção (tpEvento=112120).
#[allow(clippy::too_many_arguments)]
pub fn build_rtc_importacao_zfm(
    access_key: &str,
    seq: u32,
    environment: SefazEnvironment,
    tax_id: &str,
    c_uf: &str,
    ver_aplic: &str,
    itens: &[RtcItem],
    org_code_override: Option<&str>,
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
    build_event_xml_with_org(
        access_key,
        event_types::RTC_IMPORTACAO_ZFM,
        seq,
        tax_id,
        environment,
        &additional,
        org_code_override,
    )
}

/// Build RTC event: Perecimento/roubo transporte adquirente (tpEvento=211124).
#[allow(clippy::too_many_arguments)]
pub fn build_rtc_roubo_perda_adquirente(
    access_key: &str,
    seq: u32,
    environment: SefazEnvironment,
    tax_id: &str,
    c_uf: &str,
    ver_aplic: &str,
    itens: &[RtcItem],
    org_code_override: Option<&str>,
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
    build_event_xml_with_org(
        access_key,
        event_types::RTC_ROUBO_PERDA_ADQUIRENTE,
        seq,
        tax_id,
        environment,
        &additional,
        org_code_override,
    )
}

/// Build RTC event: Perecimento/roubo transporte fornecedor (tpEvento=112130).
#[allow(clippy::too_many_arguments)]
pub fn build_rtc_roubo_perda_fornecedor(
    access_key: &str,
    seq: u32,
    environment: SefazEnvironment,
    tax_id: &str,
    c_uf: &str,
    ver_aplic: &str,
    itens: &[RtcItem],
    org_code_override: Option<&str>,
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
    build_event_xml_with_org(
        access_key,
        event_types::RTC_ROUBO_PERDA_FORNECEDOR,
        seq,
        tax_id,
        environment,
        &additional,
        org_code_override,
    )
}

/// Build RTC event: Fornecimento não realizado (tpEvento=112140).
#[allow(clippy::too_many_arguments)]
pub fn build_rtc_fornecimento_nao_realizado(
    access_key: &str,
    seq: u32,
    environment: SefazEnvironment,
    tax_id: &str,
    c_uf: &str,
    ver_aplic: &str,
    itens: &[RtcItem],
    org_code_override: Option<&str>,
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
    build_event_xml_with_org(
        access_key,
        event_types::RTC_FORNECIMENTO_NAO_REALIZADO,
        seq,
        tax_id,
        environment,
        &additional,
        org_code_override,
    )
}

/// Build RTC event: Atualização da data de previsão de entrega (tpEvento=112150).
#[allow(clippy::too_many_arguments)]
pub fn build_rtc_atualizacao_data_entrega(
    access_key: &str,
    seq: u32,
    environment: SefazEnvironment,
    tax_id: &str,
    c_uf: &str,
    ver_aplic: &str,
    data_prevista: &str,
    org_code_override: Option<&str>,
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
    build_event_xml_with_org(
        access_key,
        event_types::RTC_ATUALIZACAO_DATA_ENTREGA,
        seq,
        tax_id,
        environment,
        &additional,
        org_code_override,
    )
}

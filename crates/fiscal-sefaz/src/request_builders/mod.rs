// ── Sub-modules ─────────────────────────────────────────────────────────────

mod autorizacao;
mod conciliacao;
mod epec;
mod event_batch;
mod event_core;
mod eventos;
mod helpers;
mod rtc;

// ── Re-exports (public API) ─────────────────────────────────────────────────

pub use event_core::event_types;

pub use autorizacao::{
    build_autorizacao_batch_request, build_autorizacao_request, build_cadastro_request,
    build_consulta_recibo_request, build_consulta_request, build_csc_request,
    build_dist_dfe_request, build_inutilizacao_request, build_status_request,
};

pub use eventos::{
    build_ator_interessado_request, build_cancel_comprovante_entrega_request,
    build_cancel_insucesso_entrega_request, build_cancel_substituicao_request,
    build_cancela_request, build_cce_request, build_comprovante_entrega_request,
    build_insucesso_entrega_request, build_manifesta_request,
};

pub use epec::{
    EpecData, EpecNfceData, build_epec_nfce_request, build_epec_nfce_status_request,
    build_epec_request, extract_epec_data, extract_epec_nfce_data,
};

pub use event_batch::{
    EventItem, ProrrogacaoItem, build_cancel_prorrogacao_request, build_event_batch_request,
    build_prorrogacao_request,
};

pub use conciliacao::{ConciliacaoDetPag, build_conciliacao_request};

pub use rtc::{
    RtcCredPresItem, RtcCredPresSub, RtcItem, build_rtc_aceite_debito,
    build_rtc_apropriacao_credito_bens, build_rtc_apropriacao_credito_comb,
    build_rtc_atualizacao_data_entrega, build_rtc_cancela_evento,
    build_rtc_destino_consumo_pessoal, build_rtc_fornecimento_nao_realizado,
    build_rtc_imobilizacao_item, build_rtc_importacao_zfm, build_rtc_info_pagto_integral,
    build_rtc_manif_transf_cred_cbs, build_rtc_manif_transf_cred_ibs,
    build_rtc_roubo_perda_adquirente, build_rtc_roubo_perda_fornecedor,
    build_rtc_sol_aprop_cred_presumido,
};

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests;

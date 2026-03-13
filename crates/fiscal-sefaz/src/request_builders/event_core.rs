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
pub(super) fn event_description(event_type: u32) -> &'static str {
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
        110751 => "Cancelamento Conciliação Financeira",
        // RTC events — descriptions must match PHP sped-nfe exactly
        110001 => "Cancelamento de Evento",
        112110 => {
            "Informação de efetivo pagamento integral para liberar crédito presumido do adquirente"
        }
        112120 => "Importação em ALC/ZFM não convertida em isenção",
        112130 => {
            "Perecimento, perda, roubo ou furto durante o transporte contratado pelo fornecedor"
        }
        112140 => "Fornecimento não realizado com pagamento antecipado",
        112150 => "Atualização da Data de Previsão de Entrega",
        211110 => "Solicitação de Apropriação de crédito presumido",
        211120 => "Destinação de item para consumo pessoal",
        211124 => {
            "Perecimento, perda, roubo ou furto durante o transporte contratado pelo adquirente"
        }
        211128 => "Aceite de débito na apuração por emissão de nota de crédito",
        211130 => "Imobilização de Item",
        211140 => "Solicitação de Apropriação de Crédito de Combustível",
        211150 => {
            "Solicitação de Apropriação de Crédito para bens e serviços que dependem de atividade do adquirente"
        }
        212110 => {
            "Manifestação sobre Pedido de Transferência de Crédito de IBS em Operação de Sucessão"
        }
        212120 => {
            "Manifestação sobre Pedido de Transferência de Crédito de CBS em Operação de Sucessão"
        }
        _ => "",
    }
}

/// Build the event ID string: `ID{tpEvento}{chNFe}{nSeqEvento:02}`.
pub(super) fn build_event_id(event_type: u32, access_key: &str, seq: u32) -> String {
    format!("ID{event_type}{access_key}{seq:02}")
}

use crate::newtypes::Cents;

// ── Cana-de-açúcar (Grupo ZC01) ────────────────────────────────────────────

/// Daily sugarcane supply entry (`<forDia>`, Grupo ZC04).
///
/// Each entry represents the quantity supplied on a specific day of the month.
/// Up to 31 entries are allowed (one per day).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ForDiaData {
    /// Day of the month (1–31).
    pub dia: u8,
    /// Quantity supplied on this day (10 decimal places).
    pub qtde: Cents,
}

impl ForDiaData {
    /// Create a new daily supply entry.
    pub fn new(dia: u8, qtde: Cents) -> Self {
        Self { dia, qtde }
    }
}

/// Deduction entry (`<deduc>`, Grupo ZC10).
///
/// Represents a deduction (taxes, contributions) on the sugarcane supply.
/// Up to 10 entries are allowed.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DeducData {
    /// Description of the deduction.
    pub x_ded: String,
    /// Value of the deduction.
    pub v_ded: Cents,
}

impl DeducData {
    /// Create a new deduction entry.
    pub fn new(x_ded: impl Into<String>, v_ded: Cents) -> Self {
        Self {
            x_ded: x_ded.into(),
            v_ded,
        }
    }
}

/// Sugarcane supply data (`<cana>`, Grupo ZC01).
///
/// Used for NF-e invoices related to sugarcane (cana-de-açúcar) supply.
/// Placed inside `<infNFe>` after `<compra>` and before `<infRespTec>`.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct CanaData {
    /// Crop identification (e.g. "2025/2026").
    pub safra: String,
    /// Reference month/year (e.g. "03/2026").
    pub referencia: String,
    /// Daily supply entries (up to 31).
    pub for_dia: Vec<ForDiaData>,
    /// Total quantity for the month.
    pub q_tot_mes: Cents,
    /// Total quantity from previous months.
    pub q_tot_ant: Cents,
    /// Grand total quantity.
    pub q_tot_ger: Cents,
    /// Deduction entries (up to 10, optional).
    pub deducoes: Option<Vec<DeducData>>,
    /// Total supply value.
    pub v_for: Cents,
    /// Total deduction value.
    pub v_tot_ded: Cents,
    /// Net supply value (vFor - vTotDed).
    pub v_liq_for: Cents,
}

impl CanaData {
    /// Create a new sugarcane supply data entry.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        safra: impl Into<String>,
        referencia: impl Into<String>,
        for_dia: Vec<ForDiaData>,
        q_tot_mes: Cents,
        q_tot_ant: Cents,
        q_tot_ger: Cents,
        v_for: Cents,
        v_tot_ded: Cents,
        v_liq_for: Cents,
    ) -> Self {
        Self {
            safra: safra.into(),
            referencia: referencia.into(),
            for_dia,
            q_tot_mes,
            q_tot_ant,
            q_tot_ger,
            deducoes: None,
            v_for,
            v_tot_ded,
            v_liq_for,
        }
    }

    /// Set deduction entries.
    pub fn deducoes(mut self, d: Vec<DeducData>) -> Self {
        self.deducoes = Some(d);
        self
    }
}

// ── Agropecuário (Grupo ZF01) ──────────────────────────────────────────────

/// Guia de trânsito agropecuário (`<guiaTransito>`, Grupo ZF04).
///
/// Informações de produtos da agricultura, pecuária e produção florestal.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct AgropecuarioGuiaData {
    /// Tipo da guia (tpGuia).
    pub tp_guia: String,
    /// UF de emissão (UFGuia, opcional).
    pub uf_guia: Option<String>,
    /// Série da guia (serieGuia, opcional).
    pub serie_guia: Option<String>,
    /// Número da guia (nGuia).
    pub n_guia: String,
}

impl AgropecuarioGuiaData {
    /// Create a new guia de trânsito.
    pub fn new(tp_guia: impl Into<String>, n_guia: impl Into<String>) -> Self {
        Self {
            tp_guia: tp_guia.into(),
            uf_guia: None,
            serie_guia: None,
            n_guia: n_guia.into(),
        }
    }

    /// Set the UF (state code) of the guia.
    pub fn uf_guia(mut self, v: impl Into<String>) -> Self {
        self.uf_guia = Some(v.into());
        self
    }

    /// Set the series of the guia.
    pub fn serie_guia(mut self, v: impl Into<String>) -> Self {
        self.serie_guia = Some(v.into());
        self
    }
}

/// Defensivo agrícola (`<defensivo>`, Grupo ZF02).
///
/// Informação de receituário de agrotóxico/defensivo agrícola.
/// Up to 20 entries allowed.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct AgropecuarioDefensivoData {
    /// Número da receita ou receituário (nReceituario).
    pub n_receituario: String,
    /// CPF do responsável técnico (CPFRespTec).
    pub cpf_resp_tec: String,
}

impl AgropecuarioDefensivoData {
    /// Create a new defensivo entry.
    pub fn new(n_receituario: impl Into<String>, cpf_resp_tec: impl Into<String>) -> Self {
        Self {
            n_receituario: n_receituario.into(),
            cpf_resp_tec: cpf_resp_tec.into(),
        }
    }
}

/// Agropecuário data wrapper.
///
/// Contains either a guia de trânsito or a list of defensivos (up to 20).
/// Placed inside `<infNFe>` after `<infRespTec>`.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum AgropecuarioData {
    /// Guia de trânsito (guiaTransito).
    Guia(AgropecuarioGuiaData),
    /// Lista de defensivos (defensivo), up to 20 entries.
    Defensivos(Vec<AgropecuarioDefensivoData>),
}

// ── Compra governamental (Grupo B31 / PL_010) ──────────────────────────────

/// Informação de compras governamentais (`<gCompraGov>`, Grupo B31).
///
/// Placed inside `<ide>` after `<NFref>` elements (PL_010+).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct CompraGovData {
    /// Tipo de ente governamental (tpEnteGov).
    pub tp_ente_gov: String,
    /// Percentual de redução de alíquota (pRedutor, 4 decimal places).
    pub p_redutor: String,
    /// Tipo de operação com o ente governamental (tpOperGov).
    pub tp_oper_gov: String,
}

impl CompraGovData {
    /// Create a new `CompraGovData`.
    pub fn new(
        tp_ente_gov: impl Into<String>,
        p_redutor: impl Into<String>,
        tp_oper_gov: impl Into<String>,
    ) -> Self {
        Self {
            tp_ente_gov: tp_ente_gov.into(),
            p_redutor: p_redutor.into(),
            tp_oper_gov: tp_oper_gov.into(),
        }
    }
}

/// Pagamento antecipado (`<gPagAntecipado>`, Grupo B34).
///
/// Contém chave(s) de acesso da NF-e de antecipação de pagamento.
/// Placed inside `<ide>` after `<gCompraGov>` (PL_010+).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct PagAntecipadoData {
    /// Chave(s) de acesso da NF-e de antecipação (refNFe).
    pub ref_nfe: Vec<String>,
}

impl PagAntecipadoData {
    /// Create a new `PagAntecipadoData` with one or more access keys.
    pub fn new(ref_nfe: Vec<String>) -> Self {
        Self { ref_nfe }
    }
}

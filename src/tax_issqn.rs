use crate::format_utils::{format_cents_2, format_rate_4};
use crate::tax_element::{filter_fields, optional_field, serialize_tax_element, TaxElement, TaxField};

/// ISSQN (ISS - Imposto Sobre Servicos) input data.
/// All monetary amounts in cents, rates as hundredths.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct IssqnData {
    /// Base de calculo in cents
    pub v_bc: i64,
    /// ISS rate as hundredths (500 = 5.00%)
    pub v_aliq: i64,
    /// ISSQN value in cents
    pub v_issqn: i64,
    /// Municipality code of taxable event (IBGE 7 digits)
    pub c_mun_fg: String,
    /// Service list item (LC 116/2003)
    pub c_list_serv: String,
    /// Deduction value (optional) in cents
    pub v_deducao: Option<i64>,
    /// Other retention value (optional) in cents
    pub v_outro: Option<i64>,
    /// Unconditional discount (optional) in cents
    pub v_desc_incond: Option<i64>,
    /// Conditional discount (optional) in cents
    pub v_desc_cond: Option<i64>,
    /// ISS retention value (optional) in cents
    pub v_iss_ret: Option<i64>,
    /// ISS enforceability indicator: 1-7
    pub ind_iss: Option<String>,
    /// Municipal service code (optional)
    pub c_servico: Option<String>,
    /// Municipality of incidence (optional, IBGE)
    pub c_mun: Option<String>,
    /// Country code (optional)
    pub c_pais: Option<String>,
    /// Judicial process number (optional)
    pub n_processo: Option<String>,
    /// Tax incentive indicator: 1=yes, 2=no
    pub ind_incentivo: Option<String>,
}

/// ISSQN totals accumulator (mirrors PHP stdISSQNTot).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct IssqnTotals {
    /// Total ISSQN base value
    pub v_bc: i64,
    /// Total ISS value
    pub v_iss: i64,
    /// Total ISS retained value
    pub v_iss_ret: i64,
    /// Total deduction value
    pub v_deducao: i64,
    /// Total other retention value
    pub v_outro: i64,
    /// Total unconditional discount
    pub v_desc_incond: i64,
    /// Total conditional discount
    pub v_desc_cond: i64,
}

/// Create a new zeroed ISSQN totals accumulator.
pub fn create_issqn_totals() -> IssqnTotals {
    IssqnTotals::default()
}

/// Calculate ISSQN tax element (domain logic, no XML).
///
/// Builds a `TaxElement` from the ISSQN input data and optionally
/// accumulates into the provided totals (only when vBC > 0, matching
/// PHP behavior).
fn calculate_issqn(data: &IssqnData, totals: Option<&mut IssqnTotals>) -> TaxElement {
    // Accumulate totals only when vBC > 0 (matching PHP behavior)
    if let Some(t) = totals {
        if data.v_bc > 0 {
            t.v_bc += data.v_bc;
            t.v_iss += data.v_issqn;
            t.v_iss_ret += data.v_iss_ret.unwrap_or(0);
            t.v_deducao += data.v_deducao.unwrap_or(0);
            t.v_outro += data.v_outro.unwrap_or(0);
            t.v_desc_incond += data.v_desc_incond.unwrap_or(0);
            t.v_desc_cond += data.v_desc_cond.unwrap_or(0);
        }
    }

    let fields: Vec<Option<TaxField>> = vec![
        Some(TaxField::new("vBC", format_cents_2(data.v_bc))),
        Some(TaxField::new("vAliq", format_rate_4(data.v_aliq))),
        Some(TaxField::new("vISSQN", format_cents_2(data.v_issqn))),
        Some(TaxField::new("cMunFG", &data.c_mun_fg)),
        Some(TaxField::new("cListServ", &data.c_list_serv)),
        optional_field("vDeducao", data.v_deducao.map(|v| format_cents_2(v)).as_deref()),
        optional_field("vOutro", data.v_outro.map(|v| format_cents_2(v)).as_deref()),
        optional_field("vDescIncond", data.v_desc_incond.map(|v| format_cents_2(v)).as_deref()),
        optional_field("vDescCond", data.v_desc_cond.map(|v| format_cents_2(v)).as_deref()),
        optional_field("vISSRet", data.v_iss_ret.map(|v| format_cents_2(v)).as_deref()),
        optional_field("indISS", data.ind_iss.as_deref()),
        optional_field("cServico", data.c_servico.as_deref()),
        optional_field("cMun", data.c_mun.as_deref()),
        optional_field("cPais", data.c_pais.as_deref()),
        optional_field("nProcesso", data.n_processo.as_deref()),
        optional_field("indIncentivo", data.ind_incentivo.as_deref()),
    ];

    TaxElement {
        outer_tag: None,
        outer_fields: vec![],
        variant_tag: "ISSQN".into(),
        fields: filter_fields(fields),
    }
}

/// Build ISSQN XML string (without totals accumulation).
pub fn build_issqn_xml(data: &IssqnData) -> String {
    serialize_tax_element(&calculate_issqn(data, None))
}

/// Build ISSQN XML string and accumulate into totals.
pub fn build_issqn_xml_with_totals(data: &IssqnData, totals: &mut IssqnTotals) -> String {
    serialize_tax_element(&calculate_issqn(data, Some(totals)))
}

/// Calculate impostoDevol element (domain logic, no XML).
///
/// `p_devol` is in cents (10000 = 100.00%), `v_ipi_devol` in cents.
fn calculate_imposto_devol(p_devol: i64, v_ipi_devol: i64) -> TaxElement {
    TaxElement {
        outer_tag: Some("impostoDevol".into()),
        outer_fields: vec![TaxField::new(
            "pDevol",
            format!("{:.2}", p_devol as f64 / 100.0),
        )],
        variant_tag: "IPI".into(),
        fields: vec![TaxField::new("vIPIDevol", format_cents_2(v_ipi_devol))],
    }
}

/// Build impostoDevol XML fragment.
///
/// `p_devol` is in cents (10000 = 100.00%), `v_ipi_devol` in cents.
pub fn build_imposto_devol(p_devol: i64, v_ipi_devol: i64) -> String {
    serialize_tax_element(&calculate_imposto_devol(p_devol, v_ipi_devol))
}

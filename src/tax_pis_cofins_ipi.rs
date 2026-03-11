use crate::format_utils::{format_cents_or_zero, format_rate4_or_zero};
use crate::newtypes::{Cents, Rate, Rate4};
use crate::tax_element::{serialize_tax_element, TaxElement, TaxField};

// ── CST classification sets ─────────────────────────────────────────────────

/// PIS/COFINS CSTs that use percentage-based calculation (Aliq variant).
const ALIQ_CSTS: &[&str] = &["01", "02"];

/// PIS/COFINS CSTs that use quantity-based calculation (Qtde variant).
const QTDE_CSTS: &[&str] = &["03"];

/// PIS/COFINS CSTs that are not taxed (NT variant).
const NT_CSTS: &[&str] = &["04", "05", "06", "07", "08", "09"];

/// PIS/COFINS CSTs that use "other operations" (Outr variant).
const OUTR_CSTS: &[&str] = &[
    "49", "50", "51", "52", "53", "54", "55", "56",
    "60", "61", "62", "63", "64", "65", "66", "67",
    "70", "71", "72", "73", "74", "75",
    "98", "99",
];

/// IPI CSTs that are taxed (Trib variant): 00, 49, 50, 99.
const IPI_TRIB_CSTS: &[&str] = &["00", "49", "50", "99"];

// ── Types ───────────────────────────────────────────────────────────────────

/// PIS tax input data. Monetary fields use [`Cents`], rates use [`Rate4`].
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct PisData {
    pub cst: String,
    pub v_bc: Option<Cents>,
    pub p_pis: Option<Rate4>,
    pub v_pis: Option<Cents>,
    pub q_bc_prod: Option<i64>,
    pub v_aliq_prod: Option<i64>,
}

impl PisData {
    /// Create a new `PisData` with the required CST.
    pub fn new(cst: impl Into<String>) -> Self {
        Self { cst: cst.into(), ..Default::default() }
    }
    /// Set the base value.
    pub fn v_bc(mut self, v: Cents) -> Self { self.v_bc = Some(v); self }
    /// Set the PIS rate.
    pub fn p_pis(mut self, v: Rate4) -> Self { self.p_pis = Some(v); self }
    /// Set the PIS value.
    pub fn v_pis(mut self, v: Cents) -> Self { self.v_pis = Some(v); self }
    /// Set the quantity base.
    pub fn q_bc_prod(mut self, v: i64) -> Self { self.q_bc_prod = Some(v); self }
    /// Set the quantity rate.
    pub fn v_aliq_prod(mut self, v: i64) -> Self { self.v_aliq_prod = Some(v); self }
}

/// PIS-ST input data
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct PisStData {
    pub v_bc: Option<Cents>,
    pub p_pis: Option<Rate4>,
    pub q_bc_prod: Option<i64>,
    pub v_aliq_prod: Option<i64>,
    pub v_pis: Cents,
    pub ind_soma_pis_st: Option<i64>,
}

impl PisStData {
    /// Create a new `PisStData` with the required PIS value.
    pub fn new(v_pis: Cents) -> Self {
        Self { v_pis, ..Default::default() }
    }
    /// Set the base value.
    pub fn v_bc(mut self, v: Cents) -> Self { self.v_bc = Some(v); self }
    /// Set the PIS rate.
    pub fn p_pis(mut self, v: Rate4) -> Self { self.p_pis = Some(v); self }
    /// Set the quantity base.
    pub fn q_bc_prod(mut self, v: i64) -> Self { self.q_bc_prod = Some(v); self }
    /// Set the quantity rate.
    pub fn v_aliq_prod(mut self, v: i64) -> Self { self.v_aliq_prod = Some(v); self }
    /// Set the ST indicator.
    pub fn ind_soma_pis_st(mut self, v: i64) -> Self { self.ind_soma_pis_st = Some(v); self }
}

/// COFINS tax input data. Monetary fields use [`Cents`], rates use [`Rate4`].
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct CofinsData {
    pub cst: String,
    pub v_bc: Option<Cents>,
    pub p_cofins: Option<Rate4>,
    pub v_cofins: Option<Cents>,
    pub q_bc_prod: Option<i64>,
    pub v_aliq_prod: Option<i64>,
}

impl CofinsData {
    /// Create a new `CofinsData` with the required CST.
    pub fn new(cst: impl Into<String>) -> Self {
        Self { cst: cst.into(), ..Default::default() }
    }
    /// Set the base value.
    pub fn v_bc(mut self, v: Cents) -> Self { self.v_bc = Some(v); self }
    /// Set the COFINS rate.
    pub fn p_cofins(mut self, v: Rate4) -> Self { self.p_cofins = Some(v); self }
    /// Set the COFINS value.
    pub fn v_cofins(mut self, v: Cents) -> Self { self.v_cofins = Some(v); self }
    /// Set the quantity base.
    pub fn q_bc_prod(mut self, v: i64) -> Self { self.q_bc_prod = Some(v); self }
    /// Set the quantity rate.
    pub fn v_aliq_prod(mut self, v: i64) -> Self { self.v_aliq_prod = Some(v); self }
}

/// COFINS-ST input data
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct CofinsStData {
    pub v_bc: Option<Cents>,
    pub p_cofins: Option<Rate4>,
    pub q_bc_prod: Option<i64>,
    pub v_aliq_prod: Option<i64>,
    pub v_cofins: Cents,
    pub ind_soma_cofins_st: Option<i64>,
}

impl CofinsStData {
    /// Create a new `CofinsStData` with the required COFINS value.
    pub fn new(v_cofins: Cents) -> Self {
        Self { v_cofins, ..Default::default() }
    }
    /// Set the base value.
    pub fn v_bc(mut self, v: Cents) -> Self { self.v_bc = Some(v); self }
    /// Set the COFINS rate.
    pub fn p_cofins(mut self, v: Rate4) -> Self { self.p_cofins = Some(v); self }
    /// Set the quantity base.
    pub fn q_bc_prod(mut self, v: i64) -> Self { self.q_bc_prod = Some(v); self }
    /// Set the quantity rate.
    pub fn v_aliq_prod(mut self, v: i64) -> Self { self.v_aliq_prod = Some(v); self }
    /// Set the ST indicator.
    pub fn ind_soma_cofins_st(mut self, v: i64) -> Self { self.ind_soma_cofins_st = Some(v); self }
}

/// IPI input data
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct IpiData {
    pub cst: String,
    pub c_enq: String,
    pub cnpj_prod: Option<String>,
    pub c_selo: Option<String>,
    pub q_selo: Option<i64>,
    pub v_bc: Option<Cents>,
    pub p_ipi: Option<Rate>,
    pub q_unid: Option<i64>,
    pub v_unid: Option<i64>,
    pub v_ipi: Option<Cents>,
}

impl IpiData {
    /// Create a new `IpiData` with required CST and enquadramento code.
    pub fn new(cst: impl Into<String>, c_enq: impl Into<String>) -> Self {
        Self { cst: cst.into(), c_enq: c_enq.into(), ..Default::default() }
    }
    /// Set the CNPJ of the producer.
    pub fn cnpj_prod(mut self, v: impl Into<String>) -> Self { self.cnpj_prod = Some(v.into()); self }
    /// Set the seal code.
    pub fn c_selo(mut self, v: impl Into<String>) -> Self { self.c_selo = Some(v.into()); self }
    /// Set the seal quantity.
    pub fn q_selo(mut self, v: i64) -> Self { self.q_selo = Some(v); self }
    /// Set the base value.
    pub fn v_bc(mut self, v: Cents) -> Self { self.v_bc = Some(v); self }
    /// Set the IPI rate.
    pub fn p_ipi(mut self, v: Rate) -> Self { self.p_ipi = Some(v); self }
    /// Set the unit quantity.
    pub fn q_unid(mut self, v: i64) -> Self { self.q_unid = Some(v); self }
    /// Set the unit value.
    pub fn v_unid(mut self, v: i64) -> Self { self.v_unid = Some(v); self }
    /// Set the IPI value.
    pub fn v_ipi(mut self, v: Cents) -> Self { self.v_ipi = Some(v); self }
}

/// II (import tax) input data
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct IiData {
    pub v_bc: Cents,
    pub v_desp_adu: Cents,
    pub v_ii: Cents,
    pub v_iof: Cents,
}

impl IiData {
    /// Create a new `IiData` with all required fields.
    pub fn new(v_bc: Cents, v_desp_adu: Cents, v_ii: Cents, v_iof: Cents) -> Self {
        Self { v_bc, v_desp_adu, v_ii, v_iof }
    }
}

// ── Contribution tax (PIS / COFINS) generic engine ─────────────────────────
//
// PIS and COFINS are two variants of the same "contribuicao social" domain
// concept. They share identical calculation logic -- only the XML tag names
// and field names differ. A ContributionTaxConfig captures those differences
// so a single set of private functions handles both taxes.

/// Configuration that captures the naming differences between PIS and COFINS.
struct ContributionTaxConfig {
    /// Tax name used as outer tag and variant prefix ("PIS" or "COFINS")
    tax_name: &'static str,
    /// Rate field name ("pPIS" or "pCOFINS")
    rate_field: &'static str,
    /// Value field name ("vPIS" or "vCOFINS")
    value_field: &'static str,
    /// ST variant tag ("PISST" or "COFINSST")
    st_tag: &'static str,
    /// ST indicator field name ("indSomaPISST" or "indSomaCOFINSST")
    st_indicator: &'static str,
}

const PIS_CONFIG: ContributionTaxConfig = ContributionTaxConfig {
    tax_name: "PIS",
    rate_field: "pPIS",
    value_field: "vPIS",
    st_tag: "PISST",
    st_indicator: "indSomaPISST",
};

const COFINS_CONFIG: ContributionTaxConfig = ContributionTaxConfig {
    tax_name: "COFINS",
    rate_field: "pCOFINS",
    value_field: "vCOFINS",
    st_tag: "COFINSST",
    st_indicator: "indSomaCOFINSST",
};

/// Normalized input for the contribution tax engine.
struct ContributionTaxInput<'a> {
    cst: &'a str,
    v_bc: Option<Cents>,
    rate: Option<Rate4>,
    value: Option<Cents>,
    q_bc_prod: Option<i64>,
    v_aliq_prod: Option<i64>,
}

/// Normalized input for the contribution tax ST engine.
struct ContributionTaxStInput {
    v_bc: Option<Cents>,
    rate: Option<Rate4>,
    q_bc_prod: Option<i64>,
    v_aliq_prod: Option<i64>,
    value: Cents,
    st_indicator: Option<i64>,
}

/// Build the "Outr" variant fields for PIS/COFINS.
fn build_contribution_outr_fields(
    d: &ContributionTaxInput,
    cfg: &ContributionTaxConfig,
) -> Vec<TaxField> {
    let mut fields = vec![TaxField::new("CST", d.cst)];

    if d.q_bc_prod.is_some() {
        fields.push(TaxField::new("qBCProd", format_rate4_or_zero(d.q_bc_prod)));
        fields.push(TaxField::new("vAliqProd", format_rate4_or_zero(d.v_aliq_prod)));
    } else {
        fields.push(TaxField::new("vBC", format_cents_or_zero(d.v_bc.map(|c| c.0), 2)));
        fields.push(TaxField::new(cfg.rate_field, format_rate4_or_zero(d.rate.map(|r| r.0))));
    }

    fields.push(TaxField::new(cfg.value_field, format_cents_or_zero(d.value.map(|c| c.0), 2)));
    fields
}

/// Calculate a contribution tax (PIS or COFINS) element.
fn calculate_contribution_tax(
    d: &ContributionTaxInput,
    cfg: &ContributionTaxConfig,
) -> TaxElement {
    let cst = d.cst;

    let (variant_tag, fields) = if ALIQ_CSTS.contains(&cst) {
        let tag = format!("{}Aliq", cfg.tax_name);
        let fields = vec![
            TaxField::new("CST", cst),
            TaxField::new("vBC", format_cents_or_zero(d.v_bc.map(|c| c.0), 2)),
            TaxField::new(cfg.rate_field, format_rate4_or_zero(d.rate.map(|r| r.0))),
            TaxField::new(cfg.value_field, format_cents_or_zero(d.value.map(|c| c.0), 2)),
        ];
        (tag, fields)
    } else if QTDE_CSTS.contains(&cst) {
        let tag = format!("{}Qtde", cfg.tax_name);
        let fields = vec![
            TaxField::new("CST", cst),
            TaxField::new("qBCProd", format_rate4_or_zero(d.q_bc_prod)),
            TaxField::new("vAliqProd", format_rate4_or_zero(d.v_aliq_prod)),
            TaxField::new(cfg.value_field, format_cents_or_zero(d.value.map(|c| c.0), 2)),
        ];
        (tag, fields)
    } else if NT_CSTS.contains(&cst) {
        let tag = format!("{}NT", cfg.tax_name);
        let fields = vec![TaxField::new("CST", cst)];
        (tag, fields)
    } else if OUTR_CSTS.contains(&cst) {
        let tag = format!("{}Outr", cfg.tax_name);
        let fields = build_contribution_outr_fields(d, cfg);
        (tag, fields)
    } else {
        // Fallback: treat unknown CSTs as NT
        let tag = format!("{}NT", cfg.tax_name);
        let fields = vec![TaxField::new("CST", cst)];
        (tag, fields)
    };

    TaxElement {
        outer_tag: Some(cfg.tax_name.to_string()),
        outer_fields: vec![],
        variant_tag,
        fields,
    }
}

/// Calculate a contribution tax ST (PIS-ST or COFINS-ST) element.
fn calculate_contribution_tax_st(
    d: &ContributionTaxStInput,
    cfg: &ContributionTaxConfig,
) -> TaxElement {
    let mut fields = Vec::new();

    if d.q_bc_prod.is_some() {
        fields.push(TaxField::new("qBCProd", format_rate4_or_zero(d.q_bc_prod)));
        fields.push(TaxField::new("vAliqProd", format_rate4_or_zero(d.v_aliq_prod)));
    } else {
        fields.push(TaxField::new("vBC", format_cents_or_zero(d.v_bc.map(|c| c.0), 2)));
        fields.push(TaxField::new(cfg.rate_field, format_rate4_or_zero(d.rate.map(|r| r.0))));
    }

    fields.push(TaxField::new(cfg.value_field, format_cents_or_zero(Some(d.value.0), 2)));

    if let Some(indicator) = d.st_indicator {
        fields.push(TaxField::new(cfg.st_indicator, indicator.to_string()));
    }

    TaxElement {
        outer_tag: None,
        outer_fields: vec![],
        variant_tag: cfg.st_tag.to_string(),
        fields,
    }
}

// ── PIS (public API) ───────────────────────────────────────────────────────

/// Build PIS XML string.
///
/// Generates the `<PIS>` element with the appropriate variant tag based on CST:
/// - CST 01, 02: `<PISAliq>` (percentage-based)
/// - CST 03: `<PISQtde>` (quantity-based)
/// - CST 04-09: `<PISNT>` (not taxed)
/// - CST 49, 50-75, 98, 99: `<PISOutr>` (other operations)
pub fn build_pis_xml(data: &PisData) -> String {
    let input = ContributionTaxInput {
        cst: &data.cst,
        v_bc: data.v_bc,
        rate: data.p_pis,
        value: data.v_pis,
        q_bc_prod: data.q_bc_prod,
        v_aliq_prod: data.v_aliq_prod,
    };
    let element = calculate_contribution_tax(&input, &PIS_CONFIG);
    serialize_tax_element(&element)
}

/// Build PIS-ST XML string.
///
/// Generates the `<PISST>` element for PIS substituicao tributaria.
/// Supports both percentage-based (vBC/pPIS) and quantity-based (qBCProd/vAliqProd) modes.
pub fn build_pis_st_xml(data: &PisStData) -> String {
    let input = ContributionTaxStInput {
        v_bc: data.v_bc,
        rate: data.p_pis,
        q_bc_prod: data.q_bc_prod,
        v_aliq_prod: data.v_aliq_prod,
        value: data.v_pis,
        st_indicator: data.ind_soma_pis_st,
    };
    let element = calculate_contribution_tax_st(&input, &PIS_CONFIG);
    serialize_tax_element(&element)
}

// ── COFINS (public API) ────────────────────────────────────────────────────

/// Build COFINS XML string.
///
/// Generates the `<COFINS>` element with the appropriate variant tag based on CST:
/// - CST 01, 02: `<COFINSAliq>` (percentage-based)
/// - CST 03: `<COFINSQtde>` (quantity-based)
/// - CST 04-09: `<COFINSNT>` (not taxed)
/// - CST 49, 50-75, 98, 99: `<COFINSOutr>` (other operations)
pub fn build_cofins_xml(data: &CofinsData) -> String {
    let input = ContributionTaxInput {
        cst: &data.cst,
        v_bc: data.v_bc,
        rate: data.p_cofins,
        value: data.v_cofins,
        q_bc_prod: data.q_bc_prod,
        v_aliq_prod: data.v_aliq_prod,
    };
    let element = calculate_contribution_tax(&input, &COFINS_CONFIG);
    serialize_tax_element(&element)
}

/// Build COFINS-ST XML string.
///
/// Generates the `<COFINSST>` element for COFINS substituicao tributaria.
/// Supports both percentage-based (vBC/pCOFINS) and quantity-based (qBCProd/vAliqProd) modes.
pub fn build_cofins_st_xml(data: &CofinsStData) -> String {
    let input = ContributionTaxStInput {
        v_bc: data.v_bc,
        rate: data.p_cofins,
        q_bc_prod: data.q_bc_prod,
        v_aliq_prod: data.v_aliq_prod,
        value: data.v_cofins,
        st_indicator: data.ind_soma_cofins_st,
    };
    let element = calculate_contribution_tax_st(&input, &COFINS_CONFIG);
    serialize_tax_element(&element)
}

// ── IPI ─────────────────────────────────────────────────────────────────────

/// Build IPI XML string.
///
/// Generates the `<IPI>` element with outer fields (CNPJProd, cSelo, qSelo, cEnq)
/// and the appropriate variant tag:
/// - CST 00, 49, 50, 99: `<IPITrib>` (taxed) with vBC/pIPI or qUnid/vUnid
/// - All other CSTs: `<IPINT>` (not taxed)
pub fn build_ipi_xml(data: &IpiData) -> String {
    let mut outer_fields = Vec::new();

    if let Some(ref cnpj) = data.cnpj_prod {
        outer_fields.push(TaxField::new("CNPJProd", cnpj.as_str()));
    }
    if let Some(ref selo) = data.c_selo {
        outer_fields.push(TaxField::new("cSelo", selo.as_str()));
    }
    if let Some(q) = data.q_selo {
        outer_fields.push(TaxField::new("qSelo", q.to_string()));
    }
    outer_fields.push(TaxField::new("cEnq", &data.c_enq));

    let (variant_tag, fields) = if IPI_TRIB_CSTS.contains(&data.cst.as_str()) {
        let mut fields = vec![TaxField::new("CST", &data.cst)];

        if data.v_bc.is_some() && data.p_ipi.is_some() {
            fields.push(TaxField::new("vBC", format_cents_or_zero(data.v_bc.map(|c| c.0), 2)));
            fields.push(TaxField::new("pIPI", format_rate4_or_zero(data.p_ipi.map(|r| r.0))));
        } else {
            fields.push(TaxField::new("qUnid", format_rate4_or_zero(data.q_unid)));
            fields.push(TaxField::new("vUnid", format_rate4_or_zero(data.v_unid)));
        }

        fields.push(TaxField::new("vIPI", format_cents_or_zero(data.v_ipi.map(|c| c.0), 2)));
        ("IPITrib".to_string(), fields)
    } else {
        let fields = vec![TaxField::new("CST", &data.cst)];
        ("IPINT".to_string(), fields)
    };

    let element = TaxElement {
        outer_tag: Some("IPI".to_string()),
        outer_fields,
        variant_tag,
        fields,
    };
    serialize_tax_element(&element)
}

// ── II (Imposto de Importacao) ──────────────────────────────────────────────

/// Build II (import tax) XML string.
///
/// Generates the `<II>` element with four required fields:
/// vBC, vDespAdu, vII, and vIOF -- all formatted as cents to 2 decimal places.
pub fn build_ii_xml(data: &IiData) -> String {
    let element = TaxElement {
        outer_tag: None,
        outer_fields: vec![],
        variant_tag: "II".to_string(),
        fields: vec![
            TaxField::new("vBC", format_cents_or_zero(Some(data.v_bc.0), 2)),
            TaxField::new("vDespAdu", format_cents_or_zero(Some(data.v_desp_adu.0), 2)),
            TaxField::new("vII", format_cents_or_zero(Some(data.v_ii.0), 2)),
            TaxField::new("vIOF", format_cents_or_zero(Some(data.v_iof.0), 2)),
        ],
    };
    serialize_tax_element(&element)
}

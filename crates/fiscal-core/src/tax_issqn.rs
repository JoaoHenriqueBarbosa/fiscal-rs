//! ISSQN (ISS — Imposto Sobre Serviços) XML generation for NF-e service items.
//!
//! Public entry points:
//! - [`build_issqn_xml`] — generate `<ISSQN>` element without total accumulation.
//! - [`build_issqn_xml_with_totals`] — generate `<ISSQN>` element and accumulate into [`IssqnTotals`].
//! - [`build_imposto_devol`] — generate `<impostoDevol>` element for return invoices.
//! - [`create_issqn_totals`] — create a zeroed [`IssqnTotals`] accumulator.

use serde::{Deserialize, Serialize};

use crate::format_utils::{format_cents_2, format_rate_4};
use crate::tax_element::{
    TaxElement, TaxField, filter_fields, optional_field, serialize_tax_element,
};

/// ISSQN (ISS - Imposto Sobre Servicos) input data.
/// All monetary amounts in cents, rates as hundredths.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
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

impl IssqnData {
    /// Create a new `IssqnData` with all required fields.
    ///
    /// `v_bc` is the calculation base in cents; `v_aliq` is the ISS rate in
    /// hundredths of percent (e.g. `500` = 5.00%); `v_issqn` is the tax value
    /// in cents; `c_mun_fg` is the 7-digit IBGE municipality code of the
    /// taxable event; `c_list_serv` is the service list item code (LC 116/2003).
    pub fn new(
        v_bc: i64,
        v_aliq: i64,
        v_issqn: i64,
        c_mun_fg: impl Into<String>,
        c_list_serv: impl Into<String>,
    ) -> Self {
        Self {
            v_bc,
            v_aliq,
            v_issqn,
            c_mun_fg: c_mun_fg.into(),
            c_list_serv: c_list_serv.into(),
            ..Default::default()
        }
    }
    /// Set the deduction value (`vDeducao`) in cents.
    pub fn v_deducao(mut self, v: i64) -> Self {
        self.v_deducao = Some(v);
        self
    }
    /// Set the other retention value (`vOutro`) in cents.
    pub fn v_outro(mut self, v: i64) -> Self {
        self.v_outro = Some(v);
        self
    }
    /// Set the unconditional discount (`vDescIncond`) in cents.
    pub fn v_desc_incond(mut self, v: i64) -> Self {
        self.v_desc_incond = Some(v);
        self
    }
    /// Set the conditional discount (`vDescCond`) in cents.
    pub fn v_desc_cond(mut self, v: i64) -> Self {
        self.v_desc_cond = Some(v);
        self
    }
    /// Set the ISS retention value (`vISSRet`) in cents.
    pub fn v_iss_ret(mut self, v: i64) -> Self {
        self.v_iss_ret = Some(v);
        self
    }
    /// Set the ISS enforceability indicator (`indISS`), values 1–7.
    pub fn ind_iss(mut self, v: impl Into<String>) -> Self {
        self.ind_iss = Some(v.into());
        self
    }
    /// Set the municipal service code (`cServico`).
    pub fn c_servico(mut self, v: impl Into<String>) -> Self {
        self.c_servico = Some(v.into());
        self
    }
    /// Set the municipality of incidence (`cMun`, 7-digit IBGE code).
    pub fn c_mun(mut self, v: impl Into<String>) -> Self {
        self.c_mun = Some(v.into());
        self
    }
    /// Set the country code (`cPais`).
    pub fn c_pais(mut self, v: impl Into<String>) -> Self {
        self.c_pais = Some(v.into());
        self
    }
    /// Set the judicial process number (`nProcesso`).
    pub fn n_processo(mut self, v: impl Into<String>) -> Self {
        self.n_processo = Some(v.into());
        self
    }
    /// Set the tax incentive indicator (`indIncentivo`): `"1"` = yes, `"2"` = no.
    pub fn ind_incentivo(mut self, v: impl Into<String>) -> Self {
        self.ind_incentivo = Some(v.into());
        self
    }
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
        optional_field("vDeducao", data.v_deducao.map(format_cents_2).as_deref()),
        optional_field("vOutro", data.v_outro.map(format_cents_2).as_deref()),
        optional_field(
            "vDescIncond",
            data.v_desc_incond.map(format_cents_2).as_deref(),
        ),
        optional_field("vDescCond", data.v_desc_cond.map(format_cents_2).as_deref()),
        optional_field("vISSRet", data.v_iss_ret.map(format_cents_2).as_deref()),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn issqn_xml_required_fields_only() {
        let data = IssqnData::new(10000, 500, 500, "3550308", "14.01");
        let xml = build_issqn_xml(&data);

        assert_eq!(
            xml,
            "<ISSQN>\
                <vBC>100.00</vBC>\
                <vAliq>5.0000</vAliq>\
                <vISSQN>5.00</vISSQN>\
                <cMunFG>3550308</cMunFG>\
                <cListServ>14.01</cListServ>\
            </ISSQN>"
        );
    }

    #[test]
    fn issqn_xml_with_all_optional_fields() {
        let data = IssqnData::new(20000, 300, 600, "3304557", "07.02")
            .v_deducao(1000)
            .v_outro(500)
            .v_desc_incond(200)
            .v_desc_cond(100)
            .v_iss_ret(300)
            .ind_iss("1")
            .c_servico("1234")
            .c_mun("3304557")
            .c_pais("1058")
            .n_processo("PROC-2026")
            .ind_incentivo("2");

        let xml = build_issqn_xml(&data);

        assert_eq!(
            xml,
            "<ISSQN>\
                <vBC>200.00</vBC>\
                <vAliq>3.0000</vAliq>\
                <vISSQN>6.00</vISSQN>\
                <cMunFG>3304557</cMunFG>\
                <cListServ>07.02</cListServ>\
                <vDeducao>10.00</vDeducao>\
                <vOutro>5.00</vOutro>\
                <vDescIncond>2.00</vDescIncond>\
                <vDescCond>1.00</vDescCond>\
                <vISSRet>3.00</vISSRet>\
                <indISS>1</indISS>\
                <cServico>1234</cServico>\
                <cMun>3304557</cMun>\
                <cPais>1058</cPais>\
                <nProcesso>PROC-2026</nProcesso>\
                <indIncentivo>2</indIncentivo>\
            </ISSQN>"
        );
    }

    #[test]
    fn issqn_totals_accumulate_when_vbc_positive() {
        let mut totals = create_issqn_totals();

        let data1 = IssqnData::new(10000, 500, 500, "3550308", "14.01")
            .v_iss_ret(100)
            .v_deducao(200)
            .v_outro(50)
            .v_desc_incond(30)
            .v_desc_cond(20);
        let _ = build_issqn_xml_with_totals(&data1, &mut totals);

        let data2 = IssqnData::new(20000, 300, 600, "3304557", "07.02")
            .v_iss_ret(150)
            .v_deducao(100);
        let _ = build_issqn_xml_with_totals(&data2, &mut totals);

        assert_eq!(totals.v_bc, 30000);
        assert_eq!(totals.v_iss, 1100);
        assert_eq!(totals.v_iss_ret, 250);
        assert_eq!(totals.v_deducao, 300);
        assert_eq!(totals.v_outro, 50);
        assert_eq!(totals.v_desc_incond, 30);
        assert_eq!(totals.v_desc_cond, 20);
    }

    #[test]
    fn issqn_totals_skip_when_vbc_is_zero() {
        let mut totals = create_issqn_totals();

        let data = IssqnData::new(0, 500, 0, "3550308", "14.01").v_iss_ret(100);
        let _ = build_issqn_xml_with_totals(&data, &mut totals);

        assert_eq!(totals.v_bc, 0);
        assert_eq!(totals.v_iss, 0);
        assert_eq!(totals.v_iss_ret, 0);
    }

    #[test]
    fn imposto_devol_xml() {
        let xml = build_imposto_devol(10000, 5000);

        assert_eq!(
            xml,
            "<impostoDevol>\
                <pDevol>100.00</pDevol>\
                <IPI>\
                    <vIPIDevol>50.00</vIPIDevol>\
                </IPI>\
            </impostoDevol>"
        );
    }
}

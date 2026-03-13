//! Build the `<total>` group of the NF-e XML.

use crate::format_utils::format_cents;
use crate::newtypes::Cents;
use crate::tax_ibs_cbs::{self, IbsCbsTotData, IsTotData};
use crate::tax_icms::IcmsTotals;
use crate::types::{CalculationMethod, IssqnTotData, RetTribData, SchemaVersion};
use crate::xml_utils::{TagContent, tag};

/// Accumulated non-ICMS totals for the invoice total calculation.
#[derive(Debug, Clone)]
pub struct OtherTotals {
    /// Total IPI value in cents.
    pub v_ipi: i64,
    /// Total PIS value in cents.
    pub v_pis: i64,
    /// Total COFINS value in cents.
    pub v_cofins: i64,
    /// Total II (import tax) value in cents.
    pub v_ii: i64,
    /// Total freight value in cents (accumulated from items).
    pub v_frete: i64,
    /// Total insurance value in cents (accumulated from items).
    pub v_seg: i64,
    /// Total discount value in cents (accumulated from items).
    pub v_desc: i64,
    /// Total other expenses value in cents (accumulated from items).
    pub v_outro: i64,
    /// Total approximate tax value in cents (vTotTrib, optional).
    pub v_tot_trib: i64,
    /// Total IPI devolution value in cents (vIPIDevol).
    pub v_ipi_devol: i64,
    /// Total PIS-ST value in cents (vPISST, accumulated from items with indSomaPISST=1).
    pub v_pis_st: i64,
    /// Total COFINS-ST value in cents (vCOFINSST, accumulated from items with indSomaCOFINSST=1).
    pub v_cofins_st: i64,
}

/// Calculate `vNF` using the V1 method (from accumulated struct values).
///
/// Matches the PHP `buildTotalICMS()` formula:
/// ```text
/// vNF = vProd - vDesc - (vICMSDeson * indDeduzDeson)
///     + vST + vFCPST + vICMSMonoReten
///     + vFrete + vSeg + vOutro
///     + vII + vIPI + vIPIDevol
///     + vServ + vPISST + vCOFINSST
/// ```
fn calculate_v_nf_v1(
    total_products: i64,
    icms: &IcmsTotals,
    other: &OtherTotals,
    v_serv: i64,
) -> i64 {
    let deson_deduction = if icms.ind_deduz_deson {
        icms.v_icms_deson.0
    } else {
        0
    };

    total_products - other.v_desc - deson_deduction
        + icms.v_st.0
        + icms.v_fcp_st.0
        + icms.v_icms_mono_reten.0
        + other.v_frete
        + other.v_seg
        + other.v_outro
        + other.v_ii
        + other.v_ipi
        + other.v_ipi_devol
        + other.v_pis_st
        + other.v_cofins_st
        + v_serv
}

/// Calculate `vNF` using the V2 method (from built XML tag values).
///
/// In the Rust implementation, since we accumulate from the same source
/// data, V2 produces the same result as V1.  The function exists for API
/// parity with PHP `sped-nfe` and for future divergence.
fn calculate_v_nf_v2(
    total_products: i64,
    icms: &IcmsTotals,
    other: &OtherTotals,
    v_serv: i64,
) -> i64 {
    // V2 uses the same formula — the PHP difference is only in _where_
    // the input values come from (DOM tags vs. accumulated array).
    calculate_v_nf_v1(total_products, icms, other, v_serv)
}

/// Parse a decimal string (e.g. "100.50") into cents (i64).
///
/// Returns 0 when the string is empty or cannot be parsed.
/// Used internally to extract numeric values from `IbsCbsTotData` / `IsTotData`
/// for the `vNFTot` auto-calculation.
fn parse_decimal_to_cents(s: &str) -> i64 {
    if s.is_empty() {
        return 0;
    }
    s.parse::<f64>()
        .map(|v| (v * 100.0).round() as i64)
        .unwrap_or(0)
}

/// Build the `<total>` element with ICMSTot, optional ISSQNtot, retTrib, ISTot, IBSCBSTot,
/// and optional vNFTot (PL_010 only).
///
/// `schema_version` controls whether PL_010-exclusive totals (ISTot, IBSCBSTot, vNFTot) are
/// emitted.
///
/// `v_nf_tot_override` allows the caller to supply a manual override for `vNFTot`, matching
/// the PHP `tagTotal(vNFTot)` API.  When `None`, `vNFTot` is auto-calculated as
/// `vNF + vIBS + vCBS + vIS` (only emitted when IBSCBSTot is present and the result > 0).
#[allow(clippy::too_many_arguments)]
pub fn build_total(
    total_products: i64,
    icms: &IcmsTotals,
    other: &OtherTotals,
    ret_trib: Option<&RetTribData>,
    issqn_tot: Option<&IssqnTotData>,
    is_tot: Option<&IsTotData>,
    ibs_cbs_tot: Option<&IbsCbsTotData>,
    schema_version: SchemaVersion,
    calculation_method: CalculationMethod,
    v_nf_tot_override: Option<Cents>,
) -> String {
    let fc2 = |c: i64| format_cents(c, 2);

    // Extract vServ from ISSQNTot for the vNF formula
    let v_serv = issqn_tot.and_then(|iq| iq.v_serv).map(|c| c.0).unwrap_or(0);

    // Calculate vNF using the selected method.
    let v_nf = match calculation_method {
        CalculationMethod::V1 => calculate_v_nf_v1(total_products, icms, other, v_serv),
        CalculationMethod::V2 => calculate_v_nf_v2(total_products, icms, other, v_serv),
    };

    // Optional ICMSTot fields — PHP sped-nfe omits these when <= 0
    let mut icms_children = vec![
        tag("vBC", &[], TagContent::Text(&fc2(icms.v_bc.0))),
        tag("vICMS", &[], TagContent::Text(&fc2(icms.v_icms.0))),
        tag(
            "vICMSDeson",
            &[],
            TagContent::Text(&fc2(icms.v_icms_deson.0)),
        ),
    ];
    // vFCPUFDest, vICMSUFDest, vICMSUFRemet: only included when > 0 (matches PHP)
    if icms.v_fcp_uf_dest.0 > 0 {
        icms_children.push(tag(
            "vFCPUFDest",
            &[],
            TagContent::Text(&fc2(icms.v_fcp_uf_dest.0)),
        ));
    }
    if icms.v_icms_uf_dest.0 > 0 {
        icms_children.push(tag(
            "vICMSUFDest",
            &[],
            TagContent::Text(&fc2(icms.v_icms_uf_dest.0)),
        ));
    }
    if icms.v_icms_uf_remet.0 > 0 {
        icms_children.push(tag(
            "vICMSUFRemet",
            &[],
            TagContent::Text(&fc2(icms.v_icms_uf_remet.0)),
        ));
    }
    icms_children.push(tag("vFCP", &[], TagContent::Text(&fc2(icms.v_fcp.0))));
    icms_children.extend([
        tag("vBCST", &[], TagContent::Text(&fc2(icms.v_bc_st.0))),
        tag("vST", &[], TagContent::Text(&fc2(icms.v_st.0))),
        tag("vFCPST", &[], TagContent::Text(&fc2(icms.v_fcp_st.0))),
        tag(
            "vFCPSTRet",
            &[],
            TagContent::Text(&fc2(icms.v_fcp_st_ret.0)),
        ),
    ]);

    // ICMS monofásico (NT 2023.001): only included when > 0 (matches PHP)
    if icms.q_bc_mono > 0 {
        icms_children.push(tag("qBCMono", &[], TagContent::Text(&fc2(icms.q_bc_mono))));
    }
    if icms.v_icms_mono.0 > 0 {
        icms_children.push(tag(
            "vICMSMono",
            &[],
            TagContent::Text(&fc2(icms.v_icms_mono.0)),
        ));
    }
    if icms.q_bc_mono_reten > 0 {
        icms_children.push(tag(
            "qBCMonoReten",
            &[],
            TagContent::Text(&fc2(icms.q_bc_mono_reten)),
        ));
    }
    if icms.v_icms_mono_reten.0 > 0 {
        icms_children.push(tag(
            "vICMSMonoReten",
            &[],
            TagContent::Text(&fc2(icms.v_icms_mono_reten.0)),
        ));
    }
    if icms.q_bc_mono_ret > 0 {
        icms_children.push(tag(
            "qBCMonoRet",
            &[],
            TagContent::Text(&fc2(icms.q_bc_mono_ret)),
        ));
    }
    if icms.v_icms_mono_ret.0 > 0 {
        icms_children.push(tag(
            "vICMSMonoRet",
            &[],
            TagContent::Text(&fc2(icms.v_icms_mono_ret.0)),
        ));
    }

    icms_children.extend([
        tag("vProd", &[], TagContent::Text(&fc2(total_products))),
        tag("vFrete", &[], TagContent::Text(&fc2(other.v_frete))),
        tag("vSeg", &[], TagContent::Text(&fc2(other.v_seg))),
        tag("vDesc", &[], TagContent::Text(&fc2(other.v_desc))),
        tag("vII", &[], TagContent::Text(&fc2(other.v_ii))),
        tag("vIPI", &[], TagContent::Text(&fc2(other.v_ipi))),
        tag("vIPIDevol", &[], TagContent::Text(&fc2(other.v_ipi_devol))),
        tag("vPIS", &[], TagContent::Text(&fc2(other.v_pis))),
        tag("vCOFINS", &[], TagContent::Text(&fc2(other.v_cofins))),
        tag("vOutro", &[], TagContent::Text(&fc2(other.v_outro))),
        tag("vNF", &[], TagContent::Text(&fc2(v_nf))),
    ]);

    // vTotTrib: only included when > 0 (matches PHP)
    if other.v_tot_trib > 0 {
        icms_children.push(tag(
            "vTotTrib",
            &[],
            TagContent::Text(&fc2(other.v_tot_trib)),
        ));
    }

    let icms_tot = tag("ICMSTot", &[], TagContent::Children(icms_children));

    let mut total_children = vec![icms_tot];

    // ISSQNtot — emitted when the invoice has service items subject to ISSQN
    if let Some(iqt) = issqn_tot {
        total_children.push(build_issqn_tot(iqt));
    }

    // ISTot, IBSCBSTot, and vNFTot — only emitted when schema is PL_010 or later
    // (matching PHP: $this->schema > 9)
    if schema_version.is_pl010() {
        // ISTot — emitted when IS (Imposto Seletivo) is present
        if let Some(ist) = is_tot {
            total_children.push(tax_ibs_cbs::build_is_tot_xml(ist));
        }

        // IBSCBSTot — emitted when IBS/CBS is present
        if let Some(ibst) = ibs_cbs_tot {
            total_children.push(tax_ibs_cbs::build_ibs_cbs_tot_xml(ibst));

            // vNFTot — emitted after IBSCBSTot (matching PHP addTagTotal position).
            // When override is provided and non-zero, use it directly.
            // Otherwise, auto-calculate as vNF + vIBS + vCBS + vIS.
            let v_nf_tot_value = if let Some(ov) = v_nf_tot_override {
                // Override provided — use it (PHP errors on zero, we skip emission)
                ov.0
            } else {
                // Auto-calculate: vNFTot = vNF + vIBS + vCBS + vIS
                let v_ibs = ibst
                    .g_ibs_v_ibs
                    .as_deref()
                    .map(parse_decimal_to_cents)
                    .unwrap_or(0);
                let v_cbs = ibst
                    .g_cbs_v_cbs
                    .as_deref()
                    .map(parse_decimal_to_cents)
                    .unwrap_or(0);
                let v_is = is_tot
                    .map(|ist| parse_decimal_to_cents(&ist.v_is))
                    .unwrap_or(0);
                v_nf + v_ibs + v_cbs + v_is
            };

            // Only emit when > 0 (matches PHP: empty check)
            if v_nf_tot_value > 0 {
                total_children.push(tag("vNFTot", &[], TagContent::Text(&fc2(v_nf_tot_value))));
            }
        }
    }

    if let Some(rt) = ret_trib {
        let opt_tag = |name: &str, val: Option<crate::newtypes::Cents>| -> Option<String> {
            val.map(|v| tag(name, &[], TagContent::Text(&fc2(v.0))))
        };
        let ret_children: Vec<String> = [
            opt_tag("vRetPIS", rt.v_ret_pis),
            opt_tag("vRetCOFINS", rt.v_ret_cofins),
            opt_tag("vRetCSLL", rt.v_ret_csll),
            opt_tag("vBCIRRF", rt.v_bc_irrf),
            opt_tag("vIRRF", rt.v_irrf),
            opt_tag("vBCRetPrev", rt.v_bc_ret_prev),
            opt_tag("vRetPrev", rt.v_ret_prev),
        ]
        .into_iter()
        .flatten()
        .collect();

        if !ret_children.is_empty() {
            total_children.push(tag("retTrib", &[], TagContent::Children(ret_children)));
        }
    }

    tag("total", &[], TagContent::Children(total_children))
}

/// Build the `<ISSQNtot>` element.
///
/// Matches PHP sped-nfe `tagISSQNTot`: monetary fields are only emitted when > 0;
/// `dCompet` is always emitted; `cRegTrib` is optional.
fn build_issqn_tot(data: &IssqnTotData) -> String {
    let fc2 = |c: i64| format_cents(c, 2);

    // Helper: emit a tag only when the Cents value is > 0
    let opt_cents = |name: &str, val: Option<Cents>| -> Option<String> {
        val.and_then(|c| {
            if c.0 > 0 {
                Some(tag(name, &[], TagContent::Text(&fc2(c.0))))
            } else {
                None
            }
        })
    };

    let mut children: Vec<String> = Vec::new();

    if let Some(t) = opt_cents("vServ", data.v_serv) {
        children.push(t);
    }
    if let Some(t) = opt_cents("vBC", data.v_bc) {
        children.push(t);
    }
    if let Some(t) = opt_cents("vISS", data.v_iss) {
        children.push(t);
    }
    if let Some(t) = opt_cents("vPIS", data.v_pis) {
        children.push(t);
    }
    if let Some(t) = opt_cents("vCOFINS", data.v_cofins) {
        children.push(t);
    }

    // dCompet is always present (required)
    children.push(tag("dCompet", &[], TagContent::Text(&data.d_compet)));

    if let Some(t) = opt_cents("vDeducao", data.v_deducao) {
        children.push(t);
    }
    if let Some(t) = opt_cents("vOutro", data.v_outro) {
        children.push(t);
    }
    if let Some(t) = opt_cents("vDescIncond", data.v_desc_incond) {
        children.push(t);
    }
    if let Some(t) = opt_cents("vDescCond", data.v_desc_cond) {
        children.push(t);
    }
    if let Some(t) = opt_cents("vISSRet", data.v_iss_ret) {
        children.push(t);
    }

    if let Some(ref reg) = data.c_reg_trib {
        children.push(tag("cRegTrib", &[], TagContent::Text(reg)));
    }

    tag("ISSQNtot", &[], TagContent::Children(children))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::newtypes::Cents;
    use crate::tax_icms::IcmsTotals;
    use crate::types::IssqnTotData;

    fn zero_icms() -> IcmsTotals {
        IcmsTotals::default()
    }

    fn zero_other() -> OtherTotals {
        OtherTotals {
            v_ipi: 0,
            v_pis: 0,
            v_cofins: 0,
            v_ii: 0,
            v_frete: 0,
            v_seg: 0,
            v_desc: 0,
            v_outro: 0,
            v_tot_trib: 0,
            v_ipi_devol: 0,
            v_pis_st: 0,
            v_cofins_st: 0,
        }
    }

    /// Default calculation method for tests (V2, matching PHP default).
    fn default_method() -> CalculationMethod {
        CalculationMethod::V2
    }

    #[test]
    fn issqn_tot_minimal_only_dcompet() {
        let data = IssqnTotData::new("2026-03-12");
        let xml = build_issqn_tot(&data);

        assert_eq!(xml, "<ISSQNtot><dCompet>2026-03-12</dCompet></ISSQNtot>");
    }

    #[test]
    fn issqn_tot_with_all_positive_values() {
        let data = IssqnTotData::new("2026-03-12")
            .v_serv(Cents(100000))
            .v_bc(Cents(100000))
            .v_iss(Cents(5000))
            .v_pis(Cents(1650))
            .v_cofins(Cents(7600))
            .v_deducao(Cents(2000))
            .v_outro(Cents(500))
            .v_desc_incond(Cents(300))
            .v_desc_cond(Cents(200))
            .v_iss_ret(Cents(1000))
            .c_reg_trib("6");

        let xml = build_issqn_tot(&data);

        assert_eq!(
            xml,
            "<ISSQNtot>\
                <vServ>1000.00</vServ>\
                <vBC>1000.00</vBC>\
                <vISS>50.00</vISS>\
                <vPIS>16.50</vPIS>\
                <vCOFINS>76.00</vCOFINS>\
                <dCompet>2026-03-12</dCompet>\
                <vDeducao>20.00</vDeducao>\
                <vOutro>5.00</vOutro>\
                <vDescIncond>3.00</vDescIncond>\
                <vDescCond>2.00</vDescCond>\
                <vISSRet>10.00</vISSRet>\
                <cRegTrib>6</cRegTrib>\
            </ISSQNtot>"
        );
    }

    #[test]
    fn issqn_tot_zero_values_omitted() {
        let data = IssqnTotData::new("2026-01-01")
            .v_serv(Cents(0))
            .v_bc(Cents(0))
            .v_iss(Cents(0));

        let xml = build_issqn_tot(&data);

        // Zero values should NOT appear (matching PHP behavior)
        assert_eq!(xml, "<ISSQNtot><dCompet>2026-01-01</dCompet></ISSQNtot>");
    }

    #[test]
    fn issqn_tot_in_total_element() {
        let data = IssqnTotData::new("2026-03-12")
            .v_serv(Cents(50000))
            .v_bc(Cents(50000))
            .v_iss(Cents(2500));

        let xml = build_total(
            0,
            &zero_icms(),
            &zero_other(),
            None,
            Some(&data),
            None,
            None,
            SchemaVersion::PL009,
            default_method(),
            None,
        );

        // ISSQNtot should appear after ICMSTot
        let icms_end = xml.find("</ICMSTot>").expect("</ICMSTot> must exist");
        let issqn_start = xml.find("<ISSQNtot>").expect("<ISSQNtot> must exist");
        assert!(issqn_start > icms_end, "ISSQNtot must come after ICMSTot");

        assert!(xml.contains("<vServ>500.00</vServ>"));
        assert!(xml.contains("<vBC>500.00</vBC>"));
        assert!(xml.contains("<vISS>25.00</vISS>"));
        assert!(xml.contains("<dCompet>2026-03-12</dCompet>"));
    }

    #[test]
    fn no_issqn_tot_when_none() {
        let xml = build_total(
            0,
            &zero_icms(),
            &zero_other(),
            None,
            None,
            None,
            None,
            SchemaVersion::PL009,
            default_method(),
            None,
        );
        assert!(!xml.contains("<ISSQNtot>"));
    }

    #[test]
    fn icms_mono_fields_emitted_when_positive() {
        let mut icms = zero_icms();
        icms.q_bc_mono = 10000;
        icms.v_icms_mono = Cents(5000);
        icms.q_bc_mono_reten = 8000;
        icms.v_icms_mono_reten = Cents(4000);
        icms.q_bc_mono_ret = 6000;
        icms.v_icms_mono_ret = Cents(3000);

        let xml = build_total(
            100000,
            &icms,
            &zero_other(),
            None,
            None,
            None,
            None,
            SchemaVersion::PL009,
            default_method(),
            None,
        );

        // All monophasic fields must be present
        assert!(xml.contains("<qBCMono>100.00</qBCMono>"));
        assert!(xml.contains("<vICMSMono>50.00</vICMSMono>"));
        assert!(xml.contains("<qBCMonoReten>80.00</qBCMonoReten>"));
        assert!(xml.contains("<vICMSMonoReten>40.00</vICMSMonoReten>"));
        assert!(xml.contains("<qBCMonoRet>60.00</qBCMonoRet>"));
        assert!(xml.contains("<vICMSMonoRet>30.00</vICMSMonoRet>"));

        // Verify position: monophasic fields must come after vFCPSTRet and before vProd
        let fcp_st_ret_end = xml.find("</vFCPSTRet>").expect("vFCPSTRet must exist");
        let q_bc_mono_pos = xml.find("<qBCMono>").expect("qBCMono must exist");
        let v_prod_pos = xml.find("<vProd>").expect("vProd must exist");
        assert!(
            fcp_st_ret_end < q_bc_mono_pos,
            "qBCMono must come after vFCPSTRet"
        );
        assert!(q_bc_mono_pos < v_prod_pos, "qBCMono must come before vProd");
    }

    #[test]
    fn icms_mono_fields_omitted_when_zero() {
        let xml = build_total(
            100000,
            &zero_icms(),
            &zero_other(),
            None,
            None,
            None,
            None,
            SchemaVersion::PL009,
            default_method(),
            None,
        );

        assert!(
            !xml.contains("<qBCMono>"),
            "qBCMono must be omitted when zero"
        );
        assert!(
            !xml.contains("<vICMSMono>"),
            "vICMSMono must be omitted when zero"
        );
        assert!(
            !xml.contains("<qBCMonoReten>"),
            "qBCMonoReten must be omitted when zero"
        );
        assert!(
            !xml.contains("<vICMSMonoReten>"),
            "vICMSMonoReten must be omitted when zero"
        );
        assert!(
            !xml.contains("<qBCMonoRet>"),
            "qBCMonoRet must be omitted when zero"
        );
        assert!(
            !xml.contains("<vICMSMonoRet>"),
            "vICMSMonoRet must be omitted when zero"
        );
    }

    #[test]
    fn icms_mono_partial_fields_emitted() {
        let mut icms = zero_icms();
        // Only set vICMSMono and qBCMonoRet
        icms.v_icms_mono = Cents(2500);
        icms.q_bc_mono_ret = 5000;

        let xml = build_total(
            100000,
            &icms,
            &zero_other(),
            None,
            None,
            None,
            None,
            SchemaVersion::PL009,
            default_method(),
            None,
        );

        // q_bc_mono is 0, should be omitted
        assert!(!xml.contains("<qBCMono>"));
        // v_icms_mono > 0, should be present
        assert!(xml.contains("<vICMSMono>25.00</vICMSMono>"));
        // q_bc_mono_reten is 0, should be omitted
        assert!(!xml.contains("<qBCMonoReten>"));
        // v_icms_mono_reten is 0, should be omitted
        assert!(!xml.contains("<vICMSMonoReten>"));
        // q_bc_mono_ret > 0, should be present
        assert!(xml.contains("<qBCMonoRet>50.00</qBCMonoRet>"));
        // v_icms_mono_ret is 0, should be omitted
        assert!(!xml.contains("<vICMSMonoRet>"));
    }

    #[test]
    fn issqn_tot_creg_trib_without_monetary_values() {
        let data = IssqnTotData::new("2026-06-15").c_reg_trib("1");
        let xml = build_issqn_tot(&data);

        assert_eq!(
            xml,
            "<ISSQNtot>\
                <dCompet>2026-06-15</dCompet>\
                <cRegTrib>1</cRegTrib>\
            </ISSQNtot>"
        );
    }

    /// Quando indDeduzDeson=1 e vICMSDeson=100.00 (10000 cents),
    /// vNF deve ser reduzido em 100.00.
    #[test]
    fn vnf_reduced_by_vicms_deson_when_ind_deduz_deson_is_true() {
        // vProd = 1000.00
        let total_products: i64 = 100_000;
        let mut icms = zero_icms();
        icms.v_icms_deson = Cents(10_000); // 100.00
        icms.ind_deduz_deson = true;

        let xml = build_total(
            total_products,
            &icms,
            &zero_other(),
            None,
            None,
            None,
            None,
            SchemaVersion::PL009,
            default_method(),
            None,
        );

        // vNF = vProd - vDesc - vICMSDeson + ... = 1000.00 - 0 - 100.00 = 900.00
        assert_eq!(
            xml.contains("<vNF>900.00</vNF>"),
            true,
            "vNF deve ser 900.00 quando indDeduzDeson=1 e vICMSDeson=100.00. XML: {}",
            xml
        );
    }

    /// Quando indDeduzDeson=false (padrão), vICMSDeson NÃO é subtraído de vNF.
    #[test]
    fn vnf_unchanged_without_desoneracao() {
        let total_products: i64 = 100_000;
        let mut icms = zero_icms();
        icms.v_icms_deson = Cents(10_000); // 100.00 — presente mas não deduzido
        icms.ind_deduz_deson = false;

        let xml = build_total(
            total_products,
            &icms,
            &zero_other(),
            None,
            None,
            None,
            None,
            SchemaVersion::PL009,
            default_method(),
            None,
        );

        // vNF = vProd = 1000.00 (desoneração NÃO deduzida)
        assert_eq!(
            xml.contains("<vNF>1000.00</vNF>"),
            true,
            "vNF deve ser 1000.00 quando indDeduzDeson=false. XML: {}",
            xml
        );
    }

    /// vServ do ISSQNTot deve ser somado ao vNF.
    #[test]
    fn vnf_includes_vserv_from_issqn() {
        let total_products: i64 = 100_000; // vProd = 1000.00
        let issqn = IssqnTotData::new("2026-03-12").v_serv(Cents(50_000)); // vServ = 500.00

        let xml = build_total(
            total_products,
            &zero_icms(),
            &zero_other(),
            None,
            Some(&issqn),
            None,
            None,
            SchemaVersion::PL009,
            default_method(),
            None,
        );

        // vNF = vProd + vServ = 1000.00 + 500.00 = 1500.00
        assert_eq!(
            xml.contains("<vNF>1500.00</vNF>"),
            true,
            "vNF deve ser 1500.00 incluindo vServ=500.00. XML: {}",
            xml
        );
    }

    // ── vFCPUFDest, vICMSUFDest, vICMSUFRemet emitted when > 0 ─────────

    #[test]
    fn icms_uf_dest_fields_emitted_when_positive() {
        let mut icms = zero_icms();
        icms.v_fcp_uf_dest = Cents(1000);
        icms.v_icms_uf_dest = Cents(2000);
        icms.v_icms_uf_remet = Cents(3000);

        let xml = build_total(
            100000,
            &icms,
            &zero_other(),
            None,
            None,
            None,
            None,
            SchemaVersion::PL009,
            default_method(),
            None,
        );

        assert!(
            xml.contains("<vFCPUFDest>10.00</vFCPUFDest>"),
            "vFCPUFDest must be present when > 0. XML: {xml}"
        );
        assert!(
            xml.contains("<vICMSUFDest>20.00</vICMSUFDest>"),
            "vICMSUFDest must be present when > 0. XML: {xml}"
        );
        assert!(
            xml.contains("<vICMSUFRemet>30.00</vICMSUFRemet>"),
            "vICMSUFRemet must be present when > 0. XML: {xml}"
        );

        // Verify ordering: these fields come after vICMSDeson and before vFCP
        let deson_end = xml.find("</vICMSDeson>").expect("vICMSDeson must exist");
        let fcp_uf_start = xml.find("<vFCPUFDest>").expect("vFCPUFDest must exist");
        let fcp_start = xml.find("<vFCP>").expect("vFCP must exist");
        assert!(
            fcp_uf_start > deson_end,
            "vFCPUFDest must come after vICMSDeson"
        );
        assert!(fcp_start > fcp_uf_start, "vFCP must come after vFCPUFDest");
    }

    #[test]
    fn icms_uf_dest_fields_omitted_when_zero() {
        let xml = build_total(
            100000,
            &zero_icms(),
            &zero_other(),
            None,
            None,
            None,
            None,
            SchemaVersion::PL009,
            default_method(),
            None,
        );

        assert!(
            !xml.contains("<vFCPUFDest>"),
            "vFCPUFDest must be omitted when zero"
        );
        assert!(
            !xml.contains("<vICMSUFDest>"),
            "vICMSUFDest must be omitted when zero"
        );
        assert!(
            !xml.contains("<vICMSUFRemet>"),
            "vICMSUFRemet must be omitted when zero"
        );
    }

    // ── vNFTot (PL_010 only) ─────────────────────────────────────────────

    #[test]
    fn v_nf_tot_override_emitted_on_pl010_with_ibs_cbs() {
        use crate::tax_ibs_cbs::IbsCbsTotData;

        let ibs_cbs = IbsCbsTotData::new("1000.00");
        let xml = build_total(
            100_000, // vProd = 1000.00
            &zero_icms(),
            &zero_other(),
            None,
            None,
            None,
            Some(&ibs_cbs),
            SchemaVersion::PL010,
            default_method(),
            Some(Cents(150_000)), // override = 1500.00
        );

        assert!(
            xml.contains("<vNFTot>1500.00</vNFTot>"),
            "vNFTot override deve ser emitido com valor 1500.00. XML: {xml}"
        );

        // vNFTot deve vir depois de </IBSCBSTot>
        let ibs_end = xml.find("</IBSCBSTot>").expect("</IBSCBSTot> must exist");
        let vnftot_start = xml.find("<vNFTot>").expect("<vNFTot> must exist");
        assert!(
            vnftot_start > ibs_end,
            "vNFTot deve vir depois de IBSCBSTot"
        );
    }

    #[test]
    fn v_nf_tot_auto_calculated_on_pl010() {
        use crate::tax_ibs_cbs::{IbsCbsTotData, IsTotData};

        // vProd = 1000.00, vNF será 1000.00
        // vIBS = 50.00, vCBS = 30.00, vIS = 10.00
        // vNFTot auto = 1000.00 + 50.00 + 30.00 + 10.00 = 1090.00
        let mut ibs_cbs = IbsCbsTotData::new("1000.00");
        ibs_cbs.g_ibs_v_ibs = Some("50.00".to_string());
        ibs_cbs.g_cbs_v_cbs = Some("30.00".to_string());

        let is_tot = IsTotData::new("10.00");

        let xml = build_total(
            100_000, // vProd = 1000.00
            &zero_icms(),
            &zero_other(),
            None,
            None,
            Some(&is_tot),
            Some(&ibs_cbs),
            SchemaVersion::PL010,
            default_method(),
            None, // sem override — auto-cálculo
        );

        assert!(
            xml.contains("<vNFTot>1090.00</vNFTot>"),
            "vNFTot auto-calculado deve ser 1090.00. XML: {xml}"
        );
    }

    #[test]
    fn v_nf_tot_not_emitted_on_pl009() {
        use crate::tax_ibs_cbs::IbsCbsTotData;

        let ibs_cbs = IbsCbsTotData::new("1000.00");
        let xml = build_total(
            100_000,
            &zero_icms(),
            &zero_other(),
            None,
            None,
            None,
            Some(&ibs_cbs),
            SchemaVersion::PL009,
            default_method(),
            Some(Cents(150_000)),
        );

        assert!(
            !xml.contains("<vNFTot>"),
            "vNFTot não deve ser emitido no PL009. XML: {xml}"
        );
    }

    #[test]
    fn v_nf_tot_not_emitted_without_ibs_cbs() {
        let xml = build_total(
            100_000,
            &zero_icms(),
            &zero_other(),
            None,
            None,
            None,
            None, // sem IBSCBSTot
            SchemaVersion::PL010,
            default_method(),
            Some(Cents(150_000)),
        );

        assert!(
            !xml.contains("<vNFTot>"),
            "vNFTot não deve ser emitido sem IBSCBSTot. XML: {xml}"
        );
    }

    #[test]
    fn v_nf_tot_zero_override_not_emitted() {
        use crate::tax_ibs_cbs::IbsCbsTotData;

        let ibs_cbs = IbsCbsTotData::new("1000.00");
        let xml = build_total(
            100_000,
            &zero_icms(),
            &zero_other(),
            None,
            None,
            None,
            Some(&ibs_cbs),
            SchemaVersion::PL010,
            default_method(),
            Some(Cents(0)), // override = zero
        );

        assert!(
            !xml.contains("<vNFTot>"),
            "vNFTot não deve ser emitido quando override é zero. XML: {xml}"
        );
    }

    #[test]
    fn v_nf_tot_auto_without_is() {
        use crate::tax_ibs_cbs::IbsCbsTotData;

        // vProd = 500.00, vNF será 500.00
        // vIBS = 25.00, vCBS = 15.00, sem IS
        // vNFTot auto = 500.00 + 25.00 + 15.00 = 540.00
        let mut ibs_cbs = IbsCbsTotData::new("500.00");
        ibs_cbs.g_ibs_v_ibs = Some("25.00".to_string());
        ibs_cbs.g_cbs_v_cbs = Some("15.00".to_string());

        let xml = build_total(
            50_000, // vProd = 500.00
            &zero_icms(),
            &zero_other(),
            None,
            None,
            None,
            Some(&ibs_cbs),
            SchemaVersion::PL010,
            default_method(),
            None,
        );

        assert!(
            xml.contains("<vNFTot>540.00</vNFTot>"),
            "vNFTot auto-calculado sem IS deve ser 540.00. XML: {xml}"
        );
    }
}

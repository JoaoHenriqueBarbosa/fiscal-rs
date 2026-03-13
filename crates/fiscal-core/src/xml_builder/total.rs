//! Build the `<total>` group of the NF-e XML.

use crate::format_utils::format_cents;
use crate::newtypes::Cents;
use crate::tax_icms::IcmsTotals;
use crate::types::{IssqnTotData, RetTribData};
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
}

/// Build the `<total>` element with ICMSTot, optional ISSQNtot, and retTrib.
pub fn build_total(
    total_products: i64,
    icms: &IcmsTotals,
    other: &OtherTotals,
    ret_trib: Option<&RetTribData>,
    issqn_tot: Option<&IssqnTotData>,
) -> String {
    let fc2 = |c: i64| format_cents(c, 2);

    // Calculate vNF per PHP formula:
    // vNF = vProd - vDesc - (vICMSDeson * indDeduzDeson) + vST + vFCPST
    //       + vFrete + vSeg + vOutro + vII + vIPI + vIPIDevol + vServ
    // Note: indDeduzDeson and vServ are not currently implemented,
    // so we omit those terms for now.
    let v_nf = total_products - other.v_desc
        + icms.v_st.0
        + icms.v_fcp_st.0
        + other.v_frete
        + other.v_seg
        + other.v_outro
        + other.v_ii
        + other.v_ipi
        + other.v_ipi_devol;

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
        }
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

        let xml = build_total(0, &zero_icms(), &zero_other(), None, Some(&data));

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
        let xml = build_total(0, &zero_icms(), &zero_other(), None, None);
        assert!(!xml.contains("<ISSQNtot>"));
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
}

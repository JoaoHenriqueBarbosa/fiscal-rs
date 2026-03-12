//! Build the `<total>` group of the NF-e XML.

use crate::format_utils::format_cents;
use crate::tax_icms::IcmsTotals;
use crate::types::RetTribData;
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
}

/// Build the `<total>` element with ICMSTot, optional ISSQNTot, and retTrib.
pub fn build_total(
    total_products: i64,
    icms: &IcmsTotals,
    other: &OtherTotals,
    ret_trib: Option<&RetTribData>,
) -> String {
    let fc2 = |c: i64| format_cents(c, 2);

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
        tag("vFrete", &[], TagContent::Text("0.00")),
        tag("vSeg", &[], TagContent::Text("0.00")),
        tag("vDesc", &[], TagContent::Text("0.00")),
        tag("vII", &[], TagContent::Text(&fc2(other.v_ii))),
        tag("vIPI", &[], TagContent::Text(&fc2(other.v_ipi))),
        tag("vIPIDevol", &[], TagContent::Text("0.00")),
        tag("vPIS", &[], TagContent::Text(&fc2(other.v_pis))),
        tag("vCOFINS", &[], TagContent::Text(&fc2(other.v_cofins))),
        tag("vOutro", &[], TagContent::Text("0.00")),
        tag("vNF", &[], TagContent::Text(&fc2(total_products))),
    ]);

    let icms_tot = tag("ICMSTot", &[], TagContent::Children(icms_children));

    let mut total_children = vec![icms_tot];

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

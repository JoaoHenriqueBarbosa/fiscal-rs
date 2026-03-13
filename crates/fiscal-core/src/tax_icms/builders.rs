//! XML builders for IcmsVariant, ICMSPart, ICMSST, and ICMSUFDest.

use crate::FiscalError;
use crate::newtypes::Cents;
use crate::tax_element::{
    TaxElement, TaxField, filter_fields, optional_field, serialize_tax_element,
};

use super::csosn::build_icms_csosn_xml;
use super::cst_xml::build_icms_cst_xml;
use super::data::{IcmsPartData, IcmsStData, IcmsUfDestData};
use super::totals::{IcmsTotals, create_icms_totals};
use super::{IcmsVariant, accum, fc2, fc4};

// ── Main builders ───────────────────────────────────────────────────────────

/// Build ICMS XML string from a typed [`IcmsVariant`].
///
/// Delegates to [`build_icms_cst_xml`] or [`build_icms_csosn_xml`] depending
/// on the variant, then wraps the result in an `<ICMS>` element and
/// accumulates totals.
///
/// # Errors
///
/// Returns [`FiscalError`] if XML field serialization fails (should not happen
/// when the enum is correctly constructed).
pub fn build_icms_xml(
    variant: &IcmsVariant,
    totals: &mut IcmsTotals,
) -> Result<String, FiscalError> {
    let (variant_tag, fields) = match variant {
        IcmsVariant::Cst(cst) => build_icms_cst_xml(cst, totals)?,
        IcmsVariant::Csosn(csosn) => build_icms_csosn_xml(csosn, totals)?,
    };

    let element = TaxElement {
        outer_tag: Some("ICMS".to_string()),
        outer_fields: vec![],
        variant_tag,
        fields,
    };

    Ok(serialize_tax_element(&element))
}

/// Build the ICMSPart XML group (partition between states).
///
/// Used inside `<ICMS>` for CST 10 or 90 with interstate partition.
///
/// # Errors
///
/// Returns [`FiscalError::MissingRequiredField`] if any required field is
/// missing in the data.
pub fn build_icms_part_xml(data: &IcmsPartData) -> Result<(String, IcmsTotals), FiscalError> {
    let mut totals = create_icms_totals();
    totals.v_bc = accum(totals.v_bc, Some(data.v_bc));
    totals.v_icms = accum(totals.v_icms, Some(data.v_icms));
    totals.v_bc_st = accum(totals.v_bc_st, Some(data.v_bc_st));
    totals.v_st = accum(totals.v_st, Some(data.v_icms_st));
    if data.ind_deduz_deson.as_deref() == Some("1") {
        totals.ind_deduz_deson = true;
    }

    let mut fields_opt: Vec<Option<TaxField>> = vec![
        Some(TaxField::new("orig", data.orig.as_str())),
        Some(TaxField::new("CST", data.cst.as_str())),
        Some(TaxField::new("modBC", data.mod_bc.as_str())),
        Some(TaxField::new("vBC", fc2(Some(data.v_bc)).unwrap())),
        optional_field("pRedBC", fc4(data.p_red_bc).as_deref()),
        Some(TaxField::new("pICMS", fc4(Some(data.p_icms)).unwrap())),
        Some(TaxField::new("vICMS", fc2(Some(data.v_icms)).unwrap())),
    ];

    // ST fields
    fields_opt.push(Some(TaxField::new("modBCST", data.mod_bc_st.as_str())));
    if let Some(v) = data.p_mva_st {
        fields_opt.push(Some(TaxField::new("pMVAST", fc4(Some(v)).unwrap())));
    }
    if let Some(v) = data.p_red_bc_st {
        fields_opt.push(Some(TaxField::new("pRedBCST", fc4(Some(v)).unwrap())));
    }
    fields_opt.push(Some(TaxField::new(
        "vBCST",
        fc2(Some(data.v_bc_st)).unwrap(),
    )));
    fields_opt.push(Some(TaxField::new(
        "pICMSST",
        fc4(Some(data.p_icms_st)).unwrap(),
    )));
    fields_opt.push(Some(TaxField::new(
        "vICMSST",
        fc2(Some(data.v_icms_st)).unwrap(),
    )));

    // FCP ST fields
    fields_opt.push(optional_field("vBCFCPST", fc2(data.v_bc_fcp_st).as_deref()));
    fields_opt.push(optional_field("pFCPST", fc4(data.p_fcp_st).as_deref()));
    fields_opt.push(optional_field("vFCPST", fc2(data.v_fcp_st).as_deref()));

    // pBCOp, UFST
    fields_opt.push(Some(TaxField::new(
        "pBCOp",
        fc4(Some(data.p_bc_op)).unwrap(),
    )));
    fields_opt.push(Some(TaxField::new("UFST", data.uf_st.as_str())));

    // Desoneration
    fields_opt.push(optional_field(
        "vICMSDeson",
        fc2(data.v_icms_deson).as_deref(),
    ));
    fields_opt.push(optional_field("motDesICMS", data.mot_des_icms.as_deref()));
    fields_opt.push(optional_field(
        "indDeduzDeson",
        data.ind_deduz_deson.as_deref(),
    ));

    let fields = filter_fields(fields_opt);

    let element = TaxElement {
        outer_tag: Some("ICMS".to_string()),
        outer_fields: vec![],
        variant_tag: "ICMSPart".to_string(),
        fields,
    };

    Ok((serialize_tax_element(&element), totals))
}

/// Build the ICMSST XML group (ST repasse).
///
/// Used inside `<ICMS>` for CST 41 or 60 with interstate ST repasse.
///
/// # Errors
///
/// Returns [`FiscalError`] if serialization fails.
pub fn build_icms_st_xml(data: &IcmsStData) -> Result<(String, IcmsTotals), FiscalError> {
    let mut totals = create_icms_totals();
    totals.v_fcp_st_ret = accum(totals.v_fcp_st_ret, data.v_fcp_st_ret);

    let fields_opt: Vec<Option<TaxField>> = vec![
        Some(TaxField::new("orig", data.orig.as_str())),
        Some(TaxField::new("CST", data.cst.as_str())),
        Some(TaxField::new(
            "vBCSTRet",
            fc2(Some(data.v_bc_st_ret)).unwrap(),
        )),
        optional_field("pST", fc4(data.p_st).as_deref()),
        optional_field("vICMSSubstituto", fc2(data.v_icms_substituto).as_deref()),
        Some(TaxField::new(
            "vICMSSTRet",
            fc2(Some(data.v_icms_st_ret)).unwrap(),
        )),
        optional_field("vBCFCPSTRet", fc2(data.v_bc_fcp_st_ret).as_deref()),
        optional_field("pFCPSTRet", fc4(data.p_fcp_st_ret).as_deref()),
        optional_field("vFCPSTRet", fc2(data.v_fcp_st_ret).as_deref()),
        Some(TaxField::new(
            "vBCSTDest",
            fc2(Some(data.v_bc_st_dest)).unwrap(),
        )),
        Some(TaxField::new(
            "vICMSSTDest",
            fc2(Some(data.v_icms_st_dest)).unwrap(),
        )),
        optional_field("pRedBCEfet", fc4(data.p_red_bc_efet).as_deref()),
        optional_field("vBCEfet", fc2(data.v_bc_efet).as_deref()),
        optional_field("pICMSEfet", fc4(data.p_icms_efet).as_deref()),
        optional_field("vICMSEfet", fc2(data.v_icms_efet).as_deref()),
    ];

    let fields = filter_fields(fields_opt);

    let element = TaxElement {
        outer_tag: Some("ICMS".to_string()),
        outer_fields: vec![],
        variant_tag: "ICMSST".to_string(),
        fields,
    };

    Ok((serialize_tax_element(&element), totals))
}

/// Build the ICMSUFDest XML group (interstate destination).
///
/// This is a sibling of `<ICMS>`, placed directly inside `<imposto>`.
///
/// # Errors
///
/// Returns [`FiscalError`] if serialization fails.
pub fn build_icms_uf_dest_xml(data: &IcmsUfDestData) -> Result<(String, IcmsTotals), FiscalError> {
    let mut totals = create_icms_totals();
    totals.v_icms_uf_dest = accum(totals.v_icms_uf_dest, Some(data.v_icms_uf_dest));
    totals.v_fcp_uf_dest = accum(totals.v_fcp_uf_dest, data.v_fcp_uf_dest);
    totals.v_icms_uf_remet = accum(totals.v_icms_uf_remet, data.v_icms_uf_remet);

    let fields_opt: Vec<Option<TaxField>> = vec![
        Some(TaxField::new(
            "vBCUFDest",
            fc2(Some(data.v_bc_uf_dest)).unwrap(),
        )),
        optional_field("vBCFCPUFDest", fc2(data.v_bc_fcp_uf_dest).as_deref()),
        optional_field("pFCPUFDest", fc4(data.p_fcp_uf_dest).as_deref()),
        Some(TaxField::new(
            "pICMSUFDest",
            fc4(Some(data.p_icms_uf_dest)).unwrap(),
        )),
        Some(TaxField::new(
            "pICMSInter",
            fc4(Some(data.p_icms_inter)).unwrap(),
        )),
        Some(TaxField::new("pICMSInterPart", "100.0000")),
        optional_field("vFCPUFDest", fc2(data.v_fcp_uf_dest).as_deref()),
        Some(TaxField::new(
            "vICMSUFDest",
            fc2(Some(data.v_icms_uf_dest)).unwrap(),
        )),
        Some(TaxField::new(
            "vICMSUFRemet",
            fc2(Some(data.v_icms_uf_remet.unwrap_or(Cents(0)))).unwrap(),
        )),
    ];

    let fields = filter_fields(fields_opt);

    let element = TaxElement {
        outer_tag: None,
        outer_fields: vec![],
        variant_tag: "ICMSUFDest".to_string(),
        fields,
    };

    Ok((serialize_tax_element(&element), totals))
}

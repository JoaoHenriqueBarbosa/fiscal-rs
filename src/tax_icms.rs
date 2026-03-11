use crate::format_utils::format_cents_or_none;
use crate::newtypes::{Cents, Rate};
use crate::tax_element::{
    filter_fields, optional_field, required_field, serialize_tax_element, TaxElement, TaxField,
};
use crate::FiscalError;

/// Accumulate a value into a totals field.
fn accum(current: Cents, value: Option<Cents>) -> Cents {
    current + value.unwrap_or(Cents(0))
}

/// Accumulate a raw i64 quantity into a totals field.
fn accum_raw(current: i64, value: Option<i64>) -> i64 {
    current + value.unwrap_or(0)
}

// ── Types ───────────────────────────────────────────────────────────────────

/// Unified input data for all ICMS variations.
/// Monetary fields use [`Cents`], rate fields use [`Rate`] (hundredths).
#[derive(Debug, Clone, Default)]
pub struct IcmsData {
    pub tax_regime: u8,
    pub orig: String,
    pub cst: Option<String>,
    pub csosn: Option<String>,
    pub mod_bc: Option<String>,
    pub v_bc: Option<Cents>,
    pub p_red_bc: Option<Rate>,
    pub p_icms: Option<Rate>,
    pub v_icms: Option<Cents>,
    pub v_bc_fcp: Option<Cents>,
    pub p_fcp: Option<Rate>,
    pub v_fcp: Option<Cents>,
    pub mod_bc_st: Option<String>,
    pub p_mva_st: Option<Rate>,
    pub p_red_bc_st: Option<Rate>,
    pub v_bc_st: Option<Cents>,
    pub p_icms_st: Option<Rate>,
    pub v_icms_st: Option<Cents>,
    pub v_bc_fcp_st: Option<Cents>,
    pub p_fcp_st: Option<Rate>,
    pub v_fcp_st: Option<Cents>,
    pub v_icms_deson: Option<Cents>,
    pub mot_des_icms: Option<String>,
    pub ind_deduz_deson: Option<String>,
    pub v_icms_st_deson: Option<Cents>,
    pub mot_des_icms_st: Option<String>,
    pub v_bc_st_ret: Option<Cents>,
    pub p_st: Option<Rate>,
    pub v_icms_substituto: Option<Cents>,
    pub v_icms_st_ret: Option<Cents>,
    pub v_bc_fcp_st_ret: Option<Cents>,
    pub p_fcp_st_ret: Option<Rate>,
    pub v_fcp_st_ret: Option<Cents>,
    pub p_red_bc_efet: Option<Rate>,
    pub v_bc_efet: Option<Cents>,
    pub p_icms_efet: Option<Rate>,
    pub v_icms_efet: Option<Cents>,
    pub v_icms_op: Option<Cents>,
    pub p_dif: Option<Rate>,
    pub v_icms_dif: Option<Cents>,
    pub p_fcp_dif: Option<Rate>,
    pub v_fcp_dif: Option<Cents>,
    pub v_fcp_efet: Option<Cents>,
    pub q_bc_mono: Option<i64>,
    pub ad_rem_icms: Option<Rate>,
    pub v_icms_mono: Option<Cents>,
    pub v_icms_mono_op: Option<Cents>,
    pub ad_rem_icms_reten: Option<Rate>,
    pub q_bc_mono_reten: Option<i64>,
    pub v_icms_mono_reten: Option<Cents>,
    pub v_icms_mono_dif: Option<Cents>,
    pub q_bc_mono_ret: Option<i64>,
    pub ad_rem_icms_ret: Option<Rate>,
    pub v_icms_mono_ret: Option<Cents>,
    pub p_red_ad_rem: Option<Rate>,
    pub mot_red_ad_rem: Option<String>,
    pub c_benef_rbc: Option<String>,
    pub p_cred_sn: Option<Rate>,
    pub v_cred_icms_sn: Option<Cents>,
    pub p_bc_op: Option<Rate>,
    pub uf_st: Option<String>,
    pub v_bc_st_dest: Option<Cents>,
    pub v_icms_st_dest: Option<Cents>,
    pub v_bc_uf_dest: Option<Cents>,
    pub v_bc_fcp_uf_dest: Option<Cents>,
    pub p_fcp_uf_dest: Option<Rate>,
    pub p_icms_uf_dest: Option<Rate>,
    pub p_icms_inter: Option<Rate>,
    pub p_icms_inter_part: Option<Rate>,
    pub v_fcp_uf_dest: Option<Cents>,
    pub v_icms_uf_dest: Option<Cents>,
    pub v_icms_uf_remet: Option<Cents>,
}

/// Accumulated ICMS totals across all items.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IcmsTotals {
    pub v_bc: Cents,
    pub v_icms: Cents,
    pub v_icms_deson: Cents,
    pub v_bc_st: Cents,
    pub v_st: Cents,
    pub v_fcp: Cents,
    pub v_fcp_st: Cents,
    pub v_fcp_st_ret: Cents,
    pub v_fcp_uf_dest: Cents,
    pub v_icms_uf_dest: Cents,
    pub v_icms_uf_remet: Cents,
    pub q_bc_mono: i64,
    pub v_icms_mono: Cents,
    pub q_bc_mono_reten: i64,
    pub v_icms_mono_reten: Cents,
    pub q_bc_mono_ret: i64,
    pub v_icms_mono_ret: Cents,
}

/// Create a zeroed-out ICMS totals.
pub fn create_icms_totals() -> IcmsTotals {
    IcmsTotals::default()
}

// ── Helper: format field values ─────────────────────────────────────────────

/// Format a monetary [`Cents`] value (2 decimal places) returning `Option<String>`.
fn fc2(v: Option<Cents>) -> Option<String> {
    format_cents_or_none(v.map(|c| c.0), 2)
}

/// Format a [`Rate`] value (4 decimal places) returning `Option<String>`.
fn fc4(v: Option<Rate>) -> Option<String> {
    format_cents_or_none(v.map(|r| r.0), 4)
}

/// Format a raw i64 quantity (4 decimal places) returning `Option<String>`.
fn fc4_raw(v: Option<i64>) -> Option<String> {
    format_cents_or_none(v, 4)
}

// ── Domain field-block helpers ──────────────────────────────────────────────

/// FCP (Fundo de Combate a Pobreza): vBCFCP, pFCP, vFCP
fn fcp_fields(d: &IcmsData) -> Vec<Option<TaxField>> {
    vec![
        optional_field("vBCFCP", fc2(d.v_bc_fcp).as_deref()),
        optional_field("pFCP", fc4(d.p_fcp).as_deref()),
        optional_field("vFCP", fc2(d.v_fcp).as_deref()),
    ]
}

/// FCP-ST: vBCFCPST, pFCPST, vFCPST
fn fcp_st_fields(d: &IcmsData) -> Vec<Option<TaxField>> {
    vec![
        optional_field("vBCFCPST", fc2(d.v_bc_fcp_st).as_deref()),
        optional_field("pFCPST", fc4(d.p_fcp_st).as_deref()),
        optional_field("vFCPST", fc2(d.v_fcp_st).as_deref()),
    ]
}

/// ST base (required modBCST): modBCST ... vICMSST
fn st_fields(d: &IcmsData) -> Vec<Result<TaxField, FiscalError>> {
    vec![
        required_field("modBCST", d.mod_bc_st.as_deref()),
        Ok(optional_field("pMVAST", fc4(d.p_mva_st).as_deref())).and_then(|f| match f {
            Some(f) => Ok(f),
            None => Err(FiscalError::MissingRequiredField {
                field: "_skip_".to_string(),
            }),
        }),
        Ok(optional_field("pRedBCST", fc4(d.p_red_bc_st).as_deref())).and_then(|f| match f {
            Some(f) => Ok(f),
            None => Err(FiscalError::MissingRequiredField {
                field: "_skip_".to_string(),
            }),
        }),
        required_field("vBCST", fc2(d.v_bc_st).as_deref()),
        required_field("pICMSST", fc4(d.p_icms_st).as_deref()),
        required_field("vICMSST", fc2(d.v_icms_st).as_deref()),
    ]
}

/// Collect ST fields: required fields cause error, optional skips return None
fn collect_st_fields(d: &IcmsData) -> Result<Vec<TaxField>, FiscalError> {
    let mut result = Vec::new();
    // modBCST - required
    result.push(required_field("modBCST", d.mod_bc_st.as_deref())?);
    // pMVAST - optional
    if let Some(f) = optional_field("pMVAST", fc4(d.p_mva_st).as_deref()) {
        result.push(f);
    }
    // pRedBCST - optional
    if let Some(f) = optional_field("pRedBCST", fc4(d.p_red_bc_st).as_deref()) {
        result.push(f);
    }
    // vBCST - required
    result.push(required_field("vBCST", fc2(d.v_bc_st).as_deref())?);
    // pICMSST - required
    result.push(required_field("pICMSST", fc4(d.p_icms_st).as_deref())?);
    // vICMSST - required
    result.push(required_field("vICMSST", fc2(d.v_icms_st).as_deref())?);
    Ok(result)
}

/// Desoneration: vICMSDeson, motDesICMS, indDeduzDeson
fn desoneration_fields(d: &IcmsData) -> Vec<Option<TaxField>> {
    vec![
        optional_field("vICMSDeson", fc2(d.v_icms_deson).as_deref()),
        optional_field("motDesICMS", d.mot_des_icms.as_deref()),
        optional_field("indDeduzDeson", d.ind_deduz_deson.as_deref()),
    ]
}

/// ST desoneration: vICMSSTDeson, motDesICMSST
fn st_desoneration_fields(d: &IcmsData) -> Vec<Option<TaxField>> {
    vec![
        optional_field("vICMSSTDeson", fc2(d.v_icms_st_deson).as_deref()),
        optional_field("motDesICMSST", d.mot_des_icms_st.as_deref()),
    ]
}

// ── Main builder ────────────────────────────────────────────────────────────

/// Build ICMS XML string.
///
/// Dispatches by tax_regime and CST/CSOSN to produce the correct variant tag
/// and accumulate totals. Returns `(xml_string, totals)`.
///
/// # Errors
///
/// Returns [`FiscalError::MissingRequiredField`] if a required tax field
/// (CST, CSOSN, or any inner field like `modBC`, `vBC`, etc.) is `None`.
/// Returns [`FiscalError::UnsupportedIcmsCst`] or
/// [`FiscalError::UnsupportedIcmsCsosn`] for unrecognized CST/CSOSN codes.
pub fn build_icms_xml(data: &IcmsData) -> Result<(String, IcmsTotals), FiscalError> {
    let mut totals = create_icms_totals();

    let (variant_tag, fields) = if data.tax_regime == 1 || data.tax_regime == 2 {
        let csosn = data.csosn.as_deref().ok_or_else(|| {
            FiscalError::MissingRequiredField {
                field: "CSOSN".to_string(),
            }
        })?;
        // Generic SN totals
        totals.v_fcp_st = accum(totals.v_fcp_st, data.v_fcp_st);
        totals.v_fcp_st_ret = accum(totals.v_fcp_st_ret, data.v_fcp_st_ret);
        calculate_csosn(data, &mut totals, csosn)?
    } else {
        let cst = data.cst.as_deref().ok_or_else(|| {
            FiscalError::MissingRequiredField {
                field: "CST".to_string(),
            }
        })?;
        calculate_cst(data, &mut totals, cst)?
    };

    let element = TaxElement {
        outer_tag: Some("ICMS".to_string()),
        outer_fields: vec![],
        variant_tag,
        fields,
    };

    Ok((serialize_tax_element(&element), totals))
}

/// Build the ICMSPart XML group (partition between states).
///
/// Used inside `<ICMS>` for CST 10 or 90 with interstate partition.
///
/// # Errors
///
/// Returns [`FiscalError::MissingRequiredField`] if any required field
/// (e.g. `orig`, `CST`, `modBC`, `vBC`, `pICMS`, `vICMS`, `modBCST`,
/// `vBCST`, `pICMSST`, `vICMSST`, `pBCOp`, `UFST`) is `None`.
pub fn build_icms_part_xml(data: &IcmsData) -> Result<(String, IcmsTotals), FiscalError> {
    let mut totals = create_icms_totals();
    totals.v_bc = accum(totals.v_bc, data.v_bc);
    totals.v_icms = accum(totals.v_icms, data.v_icms);
    totals.v_bc_st = accum(totals.v_bc_st, data.v_bc_st);
    totals.v_st = accum(totals.v_st, data.v_icms_st);

    let mut fields_opt: Vec<Option<TaxField>> = vec![
        Some(required_field("orig", Some(data.orig.as_str()))?),
        Some(required_field("CST", data.cst.as_deref())?),
        Some(required_field("modBC", data.mod_bc.as_deref())?),
        Some(required_field("vBC", fc2(data.v_bc).as_deref())?),
        optional_field("pRedBC", fc4(data.p_red_bc).as_deref()),
        Some(required_field("pICMS", fc4(data.p_icms).as_deref())?),
        Some(required_field("vICMS", fc2(data.v_icms).as_deref())?),
    ];

    // ST fields
    let st = collect_st_fields(data)?;
    for f in st {
        fields_opt.push(Some(f));
    }

    // FCP ST fields
    fields_opt.extend(fcp_st_fields(data));

    // pBCOp, UFST
    fields_opt.push(Some(required_field(
        "pBCOp",
        fc4(data.p_bc_op).as_deref(),
    )?));
    fields_opt.push(Some(required_field("UFST", data.uf_st.as_deref())?));

    // Desoneration
    fields_opt.extend(desoneration_fields(data));

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
/// Returns [`FiscalError::MissingRequiredField`] if any required field
/// (e.g. `orig`, `CST`, `vBCSTRet`, `vICMSSTRet`, `vBCSTDest`,
/// `vICMSSTDest`) is `None`.
pub fn build_icms_st_xml(data: &IcmsData) -> Result<(String, IcmsTotals), FiscalError> {
    let mut totals = create_icms_totals();
    totals.v_fcp_st_ret = accum(totals.v_fcp_st_ret, data.v_fcp_st_ret);

    let fields_opt: Vec<Option<TaxField>> = vec![
        Some(required_field("orig", Some(data.orig.as_str()))?),
        Some(required_field("CST", data.cst.as_deref())?),
        Some(required_field("vBCSTRet", fc2(data.v_bc_st_ret).as_deref())?),
        optional_field("pST", fc4(data.p_st).as_deref()),
        optional_field("vICMSSubstituto", fc2(data.v_icms_substituto).as_deref()),
        Some(required_field(
            "vICMSSTRet",
            fc2(data.v_icms_st_ret).as_deref(),
        )?),
        optional_field("vBCFCPSTRet", fc2(data.v_bc_fcp_st_ret).as_deref()),
        optional_field("pFCPSTRet", fc4(data.p_fcp_st_ret).as_deref()),
        optional_field("vFCPSTRet", fc2(data.v_fcp_st_ret).as_deref()),
        Some(required_field(
            "vBCSTDest",
            fc2(data.v_bc_st_dest).as_deref(),
        )?),
        Some(required_field(
            "vICMSSTDest",
            fc2(data.v_icms_st_dest).as_deref(),
        )?),
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
/// Returns [`FiscalError::MissingRequiredField`] if any required field
/// (e.g. `vBCUFDest`, `pICMSUFDest`, `pICMSInter`, `vICMSUFDest`,
/// `vICMSUFRemet`) is `None`.
pub fn build_icms_uf_dest_xml(data: &IcmsData) -> Result<(String, IcmsTotals), FiscalError> {
    let mut totals = create_icms_totals();
    totals.v_icms_uf_dest = accum(totals.v_icms_uf_dest, data.v_icms_uf_dest);
    totals.v_fcp_uf_dest = accum(totals.v_fcp_uf_dest, data.v_fcp_uf_dest);
    totals.v_icms_uf_remet = accum(totals.v_icms_uf_remet, data.v_icms_uf_remet);

    let fields_opt: Vec<Option<TaxField>> = vec![
        Some(required_field(
            "vBCUFDest",
            fc2(data.v_bc_uf_dest).as_deref(),
        )?),
        optional_field("vBCFCPUFDest", fc2(data.v_bc_fcp_uf_dest).as_deref()),
        optional_field("pFCPUFDest", fc4(data.p_fcp_uf_dest).as_deref()),
        Some(required_field(
            "pICMSUFDest",
            fc4(data.p_icms_uf_dest).as_deref(),
        )?),
        Some(required_field(
            "pICMSInter",
            fc4(data.p_icms_inter).as_deref(),
        )?),
        Some(TaxField::new("pICMSInterPart", "100.0000")),
        optional_field("vFCPUFDest", fc2(data.v_fcp_uf_dest).as_deref()),
        Some(required_field(
            "vICMSUFDest",
            fc2(data.v_icms_uf_dest).as_deref(),
        )?),
        Some(required_field(
            "vICMSUFRemet",
            fc2(Some(data.v_icms_uf_remet.unwrap_or(Cents(0)))).as_deref(),
        )?),
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

/// Merge item-level ICMS totals into an accumulator.
pub fn merge_icms_totals(target: &mut IcmsTotals, source: &IcmsTotals) {
    target.v_bc += source.v_bc;
    target.v_icms += source.v_icms;
    target.v_icms_deson += source.v_icms_deson;
    target.v_bc_st += source.v_bc_st;
    target.v_st += source.v_st;
    target.v_fcp += source.v_fcp;
    target.v_fcp_st += source.v_fcp_st;
    target.v_fcp_st_ret += source.v_fcp_st_ret;
    target.v_fcp_uf_dest += source.v_fcp_uf_dest;
    target.v_icms_uf_dest += source.v_icms_uf_dest;
    target.v_icms_uf_remet += source.v_icms_uf_remet;
    target.q_bc_mono += source.q_bc_mono;
    target.v_icms_mono += source.v_icms_mono;
    target.q_bc_mono_reten += source.q_bc_mono_reten;
    target.v_icms_mono_reten += source.v_icms_mono_reten;
    target.q_bc_mono_ret += source.q_bc_mono_ret;
    target.v_icms_mono_ret += source.v_icms_mono_ret;
}

// ── CST builders (regime Normal) ────────────────────────────────────────────

fn calculate_cst(
    data: &IcmsData,
    totals: &mut IcmsTotals,
    cst: &str,
) -> Result<(String, Vec<TaxField>), FiscalError> {
    match cst {
        "00" => calc_cst_00(data, totals),
        "02" => calc_cst_02(data, totals),
        "10" => calc_cst_10(data, totals),
        "15" => calc_cst_15(data, totals),
        "20" => calc_cst_20(data, totals),
        "30" => calc_cst_30(data, totals),
        "40" | "41" | "50" => calc_cst_40(data, totals),
        "51" => calc_cst_51(data, totals),
        "53" => calc_cst_53(data, totals),
        "60" => calc_cst_60(data, totals),
        "61" => calc_cst_61(data, totals),
        "70" => calc_cst_70(data, totals),
        "90" => calc_cst_90(data, totals),
        _ => Err(FiscalError::UnsupportedIcmsCst(cst.to_string())),
    }
}

/// CST 00 - Tributada integralmente
fn calc_cst_00(d: &IcmsData, t: &mut IcmsTotals) -> Result<(String, Vec<TaxField>), FiscalError> {
    t.v_bc = accum(t.v_bc, d.v_bc);
    t.v_icms = accum(t.v_icms, d.v_icms);
    t.v_fcp = accum(t.v_fcp, d.v_fcp);

    let fields = filter_fields(vec![
        Some(required_field("orig", Some(d.orig.as_str()))?),
        Some(required_field("CST", d.cst.as_deref())?),
        Some(required_field("modBC", d.mod_bc.as_deref())?),
        Some(required_field("vBC", fc2(d.v_bc).as_deref())?),
        Some(required_field("pICMS", fc4(d.p_icms).as_deref())?),
        Some(required_field("vICMS", fc2(d.v_icms).as_deref())?),
        optional_field("pFCP", fc4(d.p_fcp).as_deref()),
        optional_field("vFCP", fc2(d.v_fcp).as_deref()),
    ]);

    Ok(("ICMS00".to_string(), fields))
}

/// CST 02 - Tributacao monofasica propria sobre combustiveis
fn calc_cst_02(d: &IcmsData, t: &mut IcmsTotals) -> Result<(String, Vec<TaxField>), FiscalError> {
    t.q_bc_mono = accum_raw(t.q_bc_mono, d.q_bc_mono);
    t.v_icms_mono = accum(t.v_icms_mono, d.v_icms_mono);

    let fields = filter_fields(vec![
        Some(required_field("orig", Some(d.orig.as_str()))?),
        Some(required_field("CST", d.cst.as_deref())?),
        optional_field("qBCMono", fc4_raw(d.q_bc_mono).as_deref()),
        Some(required_field("adRemICMS", fc4(d.ad_rem_icms).as_deref())?),
        Some(required_field("vICMSMono", fc2(d.v_icms_mono).as_deref())?),
    ]);

    Ok(("ICMS02".to_string(), fields))
}

/// CST 10 - Tributada e com cobranca do ICMS por ST
fn calc_cst_10(d: &IcmsData, t: &mut IcmsTotals) -> Result<(String, Vec<TaxField>), FiscalError> {
    t.v_bc = accum(t.v_bc, d.v_bc);
    t.v_icms = accum(t.v_icms, d.v_icms);
    t.v_bc_st = accum(t.v_bc_st, d.v_bc_st);
    t.v_st = accum(t.v_st, d.v_icms_st);
    t.v_fcp_st = accum(t.v_fcp_st, d.v_fcp_st);
    t.v_fcp = accum(t.v_fcp, d.v_fcp);

    let mut fields_opt: Vec<Option<TaxField>> = vec![
        Some(required_field("orig", Some(d.orig.as_str()))?),
        Some(required_field("CST", d.cst.as_deref())?),
        Some(required_field("modBC", d.mod_bc.as_deref())?),
        Some(required_field("vBC", fc2(d.v_bc).as_deref())?),
        Some(required_field("pICMS", fc4(d.p_icms).as_deref())?),
        Some(required_field("vICMS", fc2(d.v_icms).as_deref())?),
    ];

    // FCP fields
    fields_opt.extend(fcp_fields(d));
    // ST fields
    let st = collect_st_fields(d)?;
    for f in st {
        fields_opt.push(Some(f));
    }
    // FCP ST fields
    fields_opt.extend(fcp_st_fields(d));
    // ST desoneration fields
    fields_opt.extend(st_desoneration_fields(d));

    Ok(("ICMS10".to_string(), filter_fields(fields_opt)))
}

/// CST 15 - Tributacao monofasica propria e com responsabilidade pela retencao sobre combustiveis
fn calc_cst_15(d: &IcmsData, t: &mut IcmsTotals) -> Result<(String, Vec<TaxField>), FiscalError> {
    t.q_bc_mono = accum_raw(t.q_bc_mono, d.q_bc_mono);
    t.v_icms_mono = accum(t.v_icms_mono, d.v_icms_mono);
    t.q_bc_mono_reten = accum_raw(t.q_bc_mono_reten, d.q_bc_mono_reten);
    t.v_icms_mono_reten = accum(t.v_icms_mono_reten, d.v_icms_mono_reten);

    let mut fields = filter_fields(vec![
        Some(required_field("orig", Some(d.orig.as_str()))?),
        Some(required_field("CST", d.cst.as_deref())?),
        optional_field("qBCMono", fc4_raw(d.q_bc_mono).as_deref()),
        Some(required_field("adRemICMS", fc4(d.ad_rem_icms).as_deref())?),
        Some(required_field("vICMSMono", fc2(d.v_icms_mono).as_deref())?),
        optional_field("qBCMonoReten", fc4_raw(d.q_bc_mono_reten).as_deref()),
        Some(required_field(
            "adRemICMSReten",
            fc4(d.ad_rem_icms_reten).as_deref(),
        )?),
        Some(required_field(
            "vICMSMonoReten",
            fc2(d.v_icms_mono_reten).as_deref(),
        )?),
    ]);

    if d.p_red_ad_rem.is_some() {
        fields.push(required_field("pRedAdRem", fc4(d.p_red_ad_rem).as_deref())?);
        fields.push(required_field("motRedAdRem", d.mot_red_ad_rem.as_deref())?);
    }

    Ok(("ICMS15".to_string(), fields))
}

/// CST 20 - Com reducao de base de calculo
fn calc_cst_20(d: &IcmsData, t: &mut IcmsTotals) -> Result<(String, Vec<TaxField>), FiscalError> {
    t.v_icms_deson = accum(t.v_icms_deson, d.v_icms_deson);
    t.v_bc = accum(t.v_bc, d.v_bc);
    t.v_icms = accum(t.v_icms, d.v_icms);
    t.v_fcp = accum(t.v_fcp, d.v_fcp);

    let mut fields_opt: Vec<Option<TaxField>> = vec![
        Some(required_field("orig", Some(d.orig.as_str()))?),
        Some(required_field("CST", d.cst.as_deref())?),
        Some(required_field("modBC", d.mod_bc.as_deref())?),
        Some(required_field("pRedBC", fc4(d.p_red_bc).as_deref())?),
        Some(required_field("vBC", fc2(d.v_bc).as_deref())?),
        Some(required_field("pICMS", fc4(d.p_icms).as_deref())?),
        Some(required_field("vICMS", fc2(d.v_icms).as_deref())?),
    ];

    // FCP fields
    fields_opt.extend(fcp_fields(d));
    // Desoneration fields
    fields_opt.extend(desoneration_fields(d));

    Ok(("ICMS20".to_string(), filter_fields(fields_opt)))
}

/// CST 30 - Isenta ou nao tributada e com cobranca do ICMS por ST
fn calc_cst_30(d: &IcmsData, t: &mut IcmsTotals) -> Result<(String, Vec<TaxField>), FiscalError> {
    t.v_icms_deson = accum(t.v_icms_deson, d.v_icms_deson);
    t.v_bc_st = accum(t.v_bc_st, d.v_bc_st);
    t.v_st = accum(t.v_st, d.v_icms_st);
    t.v_fcp_st = accum(t.v_fcp_st, d.v_fcp_st);

    let mut fields_opt: Vec<Option<TaxField>> = vec![
        Some(required_field("orig", Some(d.orig.as_str()))?),
        Some(required_field("CST", d.cst.as_deref())?),
    ];

    // ST fields
    let st = collect_st_fields(d)?;
    for f in st {
        fields_opt.push(Some(f));
    }
    // FCP ST fields
    fields_opt.extend(fcp_st_fields(d));
    // Desoneration fields
    fields_opt.extend(desoneration_fields(d));

    Ok(("ICMS30".to_string(), filter_fields(fields_opt)))
}

/// CST 40/41/50 - Isenta / Nao tributada / Suspensao
fn calc_cst_40(d: &IcmsData, t: &mut IcmsTotals) -> Result<(String, Vec<TaxField>), FiscalError> {
    t.v_icms_deson = accum(t.v_icms_deson, d.v_icms_deson);

    let mut fields_opt: Vec<Option<TaxField>> = vec![
        Some(required_field("orig", Some(d.orig.as_str()))?),
        Some(required_field("CST", d.cst.as_deref())?),
    ];

    // Desoneration fields
    fields_opt.extend(desoneration_fields(d));

    Ok(("ICMS40".to_string(), filter_fields(fields_opt)))
}

/// CST 51 - Diferimento
fn calc_cst_51(d: &IcmsData, t: &mut IcmsTotals) -> Result<(String, Vec<TaxField>), FiscalError> {
    t.v_bc = accum(t.v_bc, d.v_bc);
    t.v_icms = accum(t.v_icms, d.v_icms);
    t.v_fcp = accum(t.v_fcp, d.v_fcp);

    let fields_opt: Vec<Option<TaxField>> = vec![
        Some(required_field("orig", Some(d.orig.as_str()))?),
        Some(required_field("CST", d.cst.as_deref())?),
        optional_field("modBC", d.mod_bc.as_deref()),
        optional_field("pRedBC", fc4(d.p_red_bc).as_deref()),
        optional_field("cBenefRBC", d.c_benef_rbc.as_deref()),
        optional_field("vBC", fc2(d.v_bc).as_deref()),
        optional_field("pICMS", fc4(d.p_icms).as_deref()),
        optional_field("vICMSOp", fc2(d.v_icms_op).as_deref()),
        optional_field("pDif", fc4(d.p_dif).as_deref()),
        optional_field("vICMSDif", fc2(d.v_icms_dif).as_deref()),
        optional_field("vICMS", fc2(d.v_icms).as_deref()),
        optional_field("vBCFCP", fc2(d.v_bc_fcp).as_deref()),
        optional_field("pFCP", fc4(d.p_fcp).as_deref()),
        optional_field("vFCP", fc2(d.v_fcp).as_deref()),
        optional_field("pFCPDif", fc4(d.p_fcp_dif).as_deref()),
        optional_field("vFCPDif", fc2(d.v_fcp_dif).as_deref()),
        optional_field("vFCPEfet", fc2(d.v_fcp_efet).as_deref()),
    ];

    Ok(("ICMS51".to_string(), filter_fields(fields_opt)))
}

/// CST 53 - Tributacao monofasica sobre combustiveis com recolhimento diferido
fn calc_cst_53(d: &IcmsData, t: &mut IcmsTotals) -> Result<(String, Vec<TaxField>), FiscalError> {
    t.q_bc_mono = accum_raw(t.q_bc_mono, d.q_bc_mono);
    t.v_icms_mono = accum(t.v_icms_mono, d.v_icms_mono);
    t.q_bc_mono_reten = accum_raw(t.q_bc_mono_reten, d.q_bc_mono_reten);
    t.v_icms_mono_reten = accum(t.v_icms_mono_reten, d.v_icms_mono_reten);

    let fields_opt: Vec<Option<TaxField>> = vec![
        Some(required_field("orig", Some(d.orig.as_str()))?),
        Some(required_field("CST", d.cst.as_deref())?),
        optional_field("qBCMono", fc4_raw(d.q_bc_mono).as_deref()),
        optional_field("adRemICMS", fc4(d.ad_rem_icms).as_deref()),
        optional_field("vICMSMonoOp", fc2(d.v_icms_mono_op).as_deref()),
        optional_field("pDif", fc4(d.p_dif).as_deref()),
        optional_field("vICMSMonoDif", fc2(d.v_icms_mono_dif).as_deref()),
        optional_field("vICMSMono", fc2(d.v_icms_mono).as_deref()),
    ];

    Ok(("ICMS53".to_string(), filter_fields(fields_opt)))
}

/// CST 60 - ICMS cobrado anteriormente por ST
fn calc_cst_60(d: &IcmsData, t: &mut IcmsTotals) -> Result<(String, Vec<TaxField>), FiscalError> {
    t.v_fcp_st_ret = accum(t.v_fcp_st_ret, d.v_fcp_st_ret);

    let fields_opt: Vec<Option<TaxField>> = vec![
        Some(required_field("orig", Some(d.orig.as_str()))?),
        Some(required_field("CST", d.cst.as_deref())?),
        optional_field("vBCSTRet", fc2(d.v_bc_st_ret).as_deref()),
        optional_field("pST", fc4(d.p_st).as_deref()),
        optional_field("vICMSSubstituto", fc2(d.v_icms_substituto).as_deref()),
        optional_field("vICMSSTRet", fc2(d.v_icms_st_ret).as_deref()),
        optional_field("vBCFCPSTRet", fc2(d.v_bc_fcp_st_ret).as_deref()),
        optional_field("pFCPSTRet", fc4(d.p_fcp_st_ret).as_deref()),
        optional_field("vFCPSTRet", fc2(d.v_fcp_st_ret).as_deref()),
        optional_field("pRedBCEfet", fc4(d.p_red_bc_efet).as_deref()),
        optional_field("vBCEfet", fc2(d.v_bc_efet).as_deref()),
        optional_field("pICMSEfet", fc4(d.p_icms_efet).as_deref()),
        optional_field("vICMSEfet", fc2(d.v_icms_efet).as_deref()),
    ];

    Ok(("ICMS60".to_string(), filter_fields(fields_opt)))
}

/// CST 61 - Tributacao monofasica sobre combustiveis cobrada anteriormente
fn calc_cst_61(d: &IcmsData, t: &mut IcmsTotals) -> Result<(String, Vec<TaxField>), FiscalError> {
    t.q_bc_mono_ret = accum_raw(t.q_bc_mono_ret, d.q_bc_mono_ret);
    t.v_icms_mono_ret = accum(t.v_icms_mono_ret, d.v_icms_mono_ret);

    let fields = filter_fields(vec![
        Some(required_field("orig", Some(d.orig.as_str()))?),
        Some(required_field("CST", d.cst.as_deref())?),
        optional_field("qBCMonoRet", fc4_raw(d.q_bc_mono_ret).as_deref()),
        Some(required_field(
            "adRemICMSRet",
            fc4(d.ad_rem_icms_ret).as_deref(),
        )?),
        Some(required_field(
            "vICMSMonoRet",
            fc2(d.v_icms_mono_ret).as_deref(),
        )?),
    ]);

    Ok(("ICMS61".to_string(), fields))
}

/// CST 70 - Reducao de BC e cobranca do ICMS por ST
fn calc_cst_70(d: &IcmsData, t: &mut IcmsTotals) -> Result<(String, Vec<TaxField>), FiscalError> {
    t.v_icms_deson = accum(t.v_icms_deson, d.v_icms_deson);
    t.v_bc = accum(t.v_bc, d.v_bc);
    t.v_icms = accum(t.v_icms, d.v_icms);
    t.v_bc_st = accum(t.v_bc_st, d.v_bc_st);
    t.v_st = accum(t.v_st, d.v_icms_st);
    t.v_fcp_st = accum(t.v_fcp_st, d.v_fcp_st);
    t.v_fcp = accum(t.v_fcp, d.v_fcp);

    let mut fields_opt: Vec<Option<TaxField>> = vec![
        Some(required_field("orig", Some(d.orig.as_str()))?),
        Some(required_field("CST", d.cst.as_deref())?),
        Some(required_field("modBC", d.mod_bc.as_deref())?),
        Some(required_field("pRedBC", fc4(d.p_red_bc).as_deref())?),
        Some(required_field("vBC", fc2(d.v_bc).as_deref())?),
        Some(required_field("pICMS", fc4(d.p_icms).as_deref())?),
        Some(required_field("vICMS", fc2(d.v_icms).as_deref())?),
    ];

    // FCP fields
    fields_opt.extend(fcp_fields(d));
    // ST fields
    let st = collect_st_fields(d)?;
    for f in st {
        fields_opt.push(Some(f));
    }
    // FCP ST fields
    fields_opt.extend(fcp_st_fields(d));
    // Desoneration fields
    fields_opt.extend(desoneration_fields(d));
    // ST desoneration fields
    fields_opt.extend(st_desoneration_fields(d));

    Ok(("ICMS70".to_string(), filter_fields(fields_opt)))
}

/// CST 90 - Outros
fn calc_cst_90(d: &IcmsData, t: &mut IcmsTotals) -> Result<(String, Vec<TaxField>), FiscalError> {
    t.v_icms_deson = accum(t.v_icms_deson, d.v_icms_deson);
    t.v_bc = accum(t.v_bc, d.v_bc);
    t.v_icms = accum(t.v_icms, d.v_icms);
    t.v_bc_st = accum(t.v_bc_st, d.v_bc_st);
    t.v_st = accum(t.v_st, d.v_icms_st);
    t.v_fcp_st = accum(t.v_fcp_st, d.v_fcp_st);
    t.v_fcp = accum(t.v_fcp, d.v_fcp);

    let mut fields_opt: Vec<Option<TaxField>> = vec![
        Some(required_field("orig", Some(d.orig.as_str()))?),
        Some(required_field("CST", d.cst.as_deref())?),
        optional_field("modBC", d.mod_bc.as_deref()),
        optional_field("vBC", fc2(d.v_bc).as_deref()),
        optional_field("pRedBC", fc4(d.p_red_bc).as_deref()),
        optional_field("cBenefRBC", d.c_benef_rbc.as_deref()),
        optional_field("pICMS", fc4(d.p_icms).as_deref()),
        optional_field("vICMSOp", fc2(d.v_icms_op).as_deref()),
        optional_field("pDif", fc4(d.p_dif).as_deref()),
        optional_field("vICMSDif", fc2(d.v_icms_dif).as_deref()),
        optional_field("vICMS", fc2(d.v_icms).as_deref()),
    ];

    // FCP fields
    fields_opt.extend(fcp_fields(d));

    // FCP deferral fields
    fields_opt.push(optional_field("pFCPDif", fc4(d.p_fcp_dif).as_deref()));
    fields_opt.push(optional_field("vFCPDif", fc2(d.v_fcp_dif).as_deref()));
    fields_opt.push(optional_field("vFCPEfet", fc2(d.v_fcp_efet).as_deref()));

    // ST fields (all optional for CST 90)
    fields_opt.push(optional_field("modBCST", d.mod_bc_st.as_deref()));
    fields_opt.push(optional_field("pMVAST", fc4(d.p_mva_st).as_deref()));
    fields_opt.push(optional_field("pRedBCST", fc4(d.p_red_bc_st).as_deref()));
    fields_opt.push(optional_field("vBCST", fc2(d.v_bc_st).as_deref()));
    fields_opt.push(optional_field("pICMSST", fc4(d.p_icms_st).as_deref()));
    fields_opt.push(optional_field("vICMSST", fc2(d.v_icms_st).as_deref()));

    // FCP ST fields
    fields_opt.extend(fcp_st_fields(d));
    // Desoneration fields
    fields_opt.extend(desoneration_fields(d));
    // ST desoneration fields
    fields_opt.extend(st_desoneration_fields(d));

    Ok(("ICMS90".to_string(), filter_fields(fields_opt)))
}

// ── CSOSN builders (Simples Nacional) ───────────────────────────────────────

fn calculate_csosn(
    data: &IcmsData,
    totals: &mut IcmsTotals,
    csosn: &str,
) -> Result<(String, Vec<TaxField>), FiscalError> {
    match csosn {
        "101" => calc_csosn_101(data, totals),
        "102" | "103" | "300" | "400" => calc_csosn_102(data, totals),
        "201" => calc_csosn_201(data, totals),
        "202" | "203" => calc_csosn_202(data, totals),
        "500" => calc_csosn_500(data, totals),
        "900" => calc_csosn_900(data, totals),
        _ => Err(FiscalError::UnsupportedIcmsCsosn(csosn.to_string())),
    }
}

/// CSOSN 101 - Tributada pelo Simples Nacional com permissao de credito
fn calc_csosn_101(d: &IcmsData, _t: &mut IcmsTotals) -> Result<(String, Vec<TaxField>), FiscalError> {
    let fields = filter_fields(vec![
        Some(required_field("orig", Some(d.orig.as_str()))?),
        Some(required_field("CSOSN", d.csosn.as_deref())?),
        Some(required_field("pCredSN", fc4(d.p_cred_sn).as_deref())?),
        Some(required_field(
            "vCredICMSSN",
            fc2(d.v_cred_icms_sn).as_deref(),
        )?),
    ]);

    Ok(("ICMSSN101".to_string(), fields))
}

/// CSOSN 102/103/300/400 - Tributada sem permissao de credito / Imune / Nao tributada
fn calc_csosn_102(d: &IcmsData, _t: &mut IcmsTotals) -> Result<(String, Vec<TaxField>), FiscalError> {
    let orig = if d.orig.is_empty() { None } else { Some(d.orig.as_str()) };
    let fields = filter_fields(vec![
        optional_field("orig", orig), // may be null for CRT=4
        Some(required_field("CSOSN", d.csosn.as_deref())?),
    ]);

    Ok(("ICMSSN102".to_string(), fields))
}

/// CSOSN 201 - Tributada com permissao de credito e com cobranca do ICMS por ST
fn calc_csosn_201(d: &IcmsData, t: &mut IcmsTotals) -> Result<(String, Vec<TaxField>), FiscalError> {
    t.v_bc_st = accum(t.v_bc_st, d.v_bc_st);
    t.v_st = accum(t.v_st, d.v_icms_st);

    let mut fields_opt: Vec<Option<TaxField>> = vec![
        Some(required_field("orig", Some(d.orig.as_str()))?),
        Some(required_field("CSOSN", d.csosn.as_deref())?),
    ];

    // ST fields
    let st = collect_st_fields(d)?;
    for f in st {
        fields_opt.push(Some(f));
    }
    // FCP ST fields
    fields_opt.extend(fcp_st_fields(d));
    // SN credit fields
    fields_opt.push(optional_field("pCredSN", fc4(d.p_cred_sn).as_deref()));
    fields_opt.push(optional_field(
        "vCredICMSSN",
        fc2(d.v_cred_icms_sn).as_deref(),
    ));

    Ok(("ICMSSN201".to_string(), filter_fields(fields_opt)))
}

/// CSOSN 202/203 - Tributada sem permissao de credito e com cobranca do ICMS por ST
fn calc_csosn_202(d: &IcmsData, t: &mut IcmsTotals) -> Result<(String, Vec<TaxField>), FiscalError> {
    t.v_bc_st = accum(t.v_bc_st, d.v_bc_st);
    t.v_st = accum(t.v_st, d.v_icms_st);

    let mut fields_opt: Vec<Option<TaxField>> = vec![
        Some(required_field("orig", Some(d.orig.as_str()))?),
        Some(required_field("CSOSN", d.csosn.as_deref())?),
    ];

    // ST fields
    let st = collect_st_fields(d)?;
    for f in st {
        fields_opt.push(Some(f));
    }
    // FCP ST fields
    fields_opt.extend(fcp_st_fields(d));

    Ok(("ICMSSN202".to_string(), filter_fields(fields_opt)))
}

/// CSOSN 500 - ICMS cobrado anteriormente por ST ou por antecipacao
fn calc_csosn_500(d: &IcmsData, _t: &mut IcmsTotals) -> Result<(String, Vec<TaxField>), FiscalError> {
    let fields = filter_fields(vec![
        Some(required_field("orig", Some(d.orig.as_str()))?),
        Some(required_field("CSOSN", d.csosn.as_deref())?),
        optional_field("vBCSTRet", fc2(d.v_bc_st_ret).as_deref()),
        optional_field("pST", fc4(d.p_st).as_deref()),
        optional_field("vICMSSubstituto", fc2(d.v_icms_substituto).as_deref()),
        optional_field("vICMSSTRet", fc2(d.v_icms_st_ret).as_deref()),
        optional_field("vBCFCPSTRet", fc2(d.v_bc_fcp_st_ret).as_deref()),
        optional_field("pFCPSTRet", fc4(d.p_fcp_st_ret).as_deref()),
        optional_field("vFCPSTRet", fc2(d.v_fcp_st_ret).as_deref()),
        optional_field("pRedBCEfet", fc4(d.p_red_bc_efet).as_deref()),
        optional_field("vBCEfet", fc2(d.v_bc_efet).as_deref()),
        optional_field("pICMSEfet", fc4(d.p_icms_efet).as_deref()),
        optional_field("vICMSEfet", fc2(d.v_icms_efet).as_deref()),
    ]);

    Ok(("ICMSSN500".to_string(), fields))
}

/// CSOSN 900 - Outros
fn calc_csosn_900(d: &IcmsData, t: &mut IcmsTotals) -> Result<(String, Vec<TaxField>), FiscalError> {
    t.v_bc = accum(t.v_bc, d.v_bc);
    t.v_icms = accum(t.v_icms, d.v_icms);
    t.v_bc_st = accum(t.v_bc_st, d.v_bc_st);
    t.v_st = accum(t.v_st, d.v_icms_st);

    let orig = if d.orig.is_empty() { None } else { Some(d.orig.as_str()) };
    let mut fields_opt: Vec<Option<TaxField>> = vec![
        optional_field("orig", orig), // may be null for CRT=4
        Some(required_field("CSOSN", d.csosn.as_deref())?),
        optional_field("modBC", d.mod_bc.as_deref()),
        optional_field("vBC", fc2(d.v_bc).as_deref()),
        optional_field("pRedBC", fc4(d.p_red_bc).as_deref()),
        optional_field("pICMS", fc4(d.p_icms).as_deref()),
        optional_field("vICMS", fc2(d.v_icms).as_deref()),
        // ST fields are all optional for CSOSN 900
        optional_field("modBCST", d.mod_bc_st.as_deref()),
        optional_field("pMVAST", fc4(d.p_mva_st).as_deref()),
        optional_field("pRedBCST", fc4(d.p_red_bc_st).as_deref()),
        optional_field("vBCST", fc2(d.v_bc_st).as_deref()),
        optional_field("pICMSST", fc4(d.p_icms_st).as_deref()),
        optional_field("vICMSST", fc2(d.v_icms_st).as_deref()),
    ];

    // FCP ST fields
    fields_opt.extend(fcp_st_fields(d));
    // SN credit fields
    fields_opt.push(optional_field("pCredSN", fc4(d.p_cred_sn).as_deref()));
    fields_opt.push(optional_field(
        "vCredICMSSN",
        fc2(d.v_cred_icms_sn).as_deref(),
    ));

    Ok(("ICMSSN900".to_string(), filter_fields(fields_opt)))
}

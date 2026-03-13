//! ICMS tax computation and XML generation for NF-e / NFC-e documents.
//!
//! This module provides two main enum types:
//! - [`IcmsCst`] — for normal tax regime (Lucro Real / Lucro Presumido), covering
//!   CSTs 00, 02, 10, 15, 20, 30, 40, 41, 50, 51, 53, 60, 61, 70, and 90.
//! - [`IcmsCsosn`] — for Simples Nacional regime (CRT 1/2), covering CSOSNs
//!   101, 102, 103, 201, 202, 203, 300, 400, 500, and 900.
//!
//! Both are wrapped by the [`IcmsVariant`] enum, which is consumed by
//! [`build_icms_cst_xml`] / [`build_icms_csosn_xml`] to produce the `<ICMS>`
//! XML fragment and accumulate [`IcmsTotals`].
//!
//! There are also three auxiliary data structs for special ICMS groups:
//! [`IcmsPartData`] (partition), [`IcmsStData`] (ST repasse), and
//! [`IcmsUfDestData`] (interstate destination differential).

mod builders;
mod csosn;
mod cst;
mod cst_xml;
mod data;
mod totals;

use crate::format_utils::format_cents_or_none;
use crate::newtypes::{Cents, Rate};

// ── Internal helpers ────────────────────────────────────────────────────────

/// Accumulate a value into a totals field.
fn accum(current: Cents, value: Option<Cents>) -> Cents {
    current + value.unwrap_or(Cents(0))
}

/// Accumulate a raw i64 quantity into a totals field.
fn accum_raw(current: i64, value: Option<i64>) -> i64 {
    current + value.unwrap_or(0)
}

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

// ── IcmsVariant ─────────────────────────────────────────────────────────────

/// Unified ICMS variant wrapping both normal-regime CSTs and Simples Nacional
/// CSOSNs. Pass one of these to [`build_icms_xml`] for compile-time-safe XML
/// generation.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum IcmsVariant {
    /// Normal tax regime (Lucro Real / Presumido).
    Cst(Box<IcmsCst>),
    /// Simples Nacional tax regime (CRT 1/2).
    Csosn(Box<IcmsCsosn>),
}

impl From<IcmsCst> for IcmsVariant {
    fn from(cst: IcmsCst) -> Self {
        Self::Cst(Box::new(cst))
    }
}

impl From<IcmsCsosn> for IcmsVariant {
    fn from(csosn: IcmsCsosn) -> Self {
        Self::Csosn(Box::new(csosn))
    }
}

// ── Public re-exports ───────────────────────────────────────────────────────

pub use builders::{
    build_icms_part_xml, build_icms_st_xml, build_icms_uf_dest_xml, build_icms_xml,
};
pub use csosn::{IcmsCsosn, build_icms_csosn_xml};
pub use cst::IcmsCst;
pub use cst_xml::build_icms_cst_xml;
pub use data::{IcmsPartData, IcmsStData, IcmsUfDestData};
pub use totals::{IcmsTotals, create_icms_totals, merge_icms_totals};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::newtypes::{Cents, Rate};

    // ── IcmsTotals builder methods ──────────────────────────────────

    #[test]
    fn icms_totals_builder_v_icms_uf_remet() {
        let t = IcmsTotals::new().v_icms_uf_remet(Cents(500));
        assert_eq!(t.v_icms_uf_remet, Cents(500));
    }

    #[test]
    fn icms_totals_builder_v_icms_mono() {
        let t = IcmsTotals::new().v_icms_mono(Cents(300));
        assert_eq!(t.v_icms_mono, Cents(300));
    }

    #[test]
    fn icms_totals_builder_v_icms_mono_reten() {
        let t = IcmsTotals::new().v_icms_mono_reten(Cents(200));
        assert_eq!(t.v_icms_mono_reten, Cents(200));
    }

    #[test]
    fn icms_totals_builder_v_icms_mono_ret() {
        let t = IcmsTotals::new().v_icms_mono_ret(Cents(100));
        assert_eq!(t.v_icms_mono_ret, Cents(100));
    }

    #[test]
    fn icms_totals_builder_ind_deduz_deson() {
        let t = IcmsTotals::new().ind_deduz_deson(true);
        assert!(t.ind_deduz_deson);
        let t2 = IcmsTotals::new().ind_deduz_deson(false);
        assert!(!t2.ind_deduz_deson);
    }

    // ── IcmsCst::cst_code() for uncovered variants ─────────────────

    #[test]
    fn cst_code_cst30() {
        let cst = IcmsCst::Cst30 {
            orig: "0".into(),
            mod_bc_st: "4".into(),
            p_mva_st: None,
            p_red_bc_st: None,
            v_bc_st: Cents(1000),
            p_icms_st: Rate(1800),
            v_icms_st: Cents(180),
            v_bc_fcp_st: None,
            p_fcp_st: None,
            v_fcp_st: None,
            v_icms_deson: None,
            mot_des_icms: None,
            ind_deduz_deson: None,
        };
        assert_eq!(cst.cst_code(), "30");
    }

    #[test]
    fn cst_code_cst51() {
        let cst = IcmsCst::Cst51 {
            orig: "0".into(),
            mod_bc: None,
            p_red_bc: None,
            c_benef_rbc: None,
            v_bc: None,
            p_icms: None,
            v_icms_op: None,
            p_dif: None,
            v_icms_dif: None,
            v_icms: None,
            v_bc_fcp: None,
            p_fcp: None,
            v_fcp: None,
            p_fcp_dif: None,
            v_fcp_dif: None,
            v_fcp_efet: None,
        };
        assert_eq!(cst.cst_code(), "51");
    }

    #[test]
    fn cst_code_cst53() {
        let cst = IcmsCst::Cst53 {
            orig: "0".into(),
            q_bc_mono: None,
            ad_rem_icms: None,
            v_icms_mono_op: None,
            p_dif: None,
            v_icms_mono_dif: None,
            v_icms_mono: None,
        };
        assert_eq!(cst.cst_code(), "53");
    }

    #[test]
    fn cst_code_cst61() {
        let cst = IcmsCst::Cst61 {
            orig: "0".into(),
            q_bc_mono_ret: None,
            ad_rem_icms_ret: Rate(100),
            v_icms_mono_ret: Cents(50),
        };
        assert_eq!(cst.cst_code(), "61");
    }

    // ── IcmsCsosn::csosn_code() for uncovered variants ─────────────

    #[test]
    fn csosn_code_102() {
        let c = IcmsCsosn::Csosn102 {
            orig: "0".into(),
            csosn: "102".into(),
        };
        assert_eq!(c.csosn_code(), "102");
    }

    #[test]
    fn csosn_code_103() {
        let c = IcmsCsosn::Csosn103 {
            orig: "0".into(),
            csosn: "103".into(),
        };
        assert_eq!(c.csosn_code(), "103");
    }

    #[test]
    fn csosn_code_201() {
        let c = IcmsCsosn::Csosn201 {
            orig: "0".into(),
            csosn: "201".into(),
            mod_bc_st: "4".into(),
            p_mva_st: None,
            p_red_bc_st: None,
            v_bc_st: Cents(0),
            p_icms_st: Rate(0),
            v_icms_st: Cents(0),
            v_bc_fcp_st: None,
            p_fcp_st: None,
            v_fcp_st: None,
            p_cred_sn: None,
            v_cred_icms_sn: None,
        };
        assert_eq!(c.csosn_code(), "201");
    }

    #[test]
    fn csosn_code_202() {
        let c = IcmsCsosn::Csosn202 {
            orig: "0".into(),
            csosn: "202".into(),
            mod_bc_st: "4".into(),
            p_mva_st: None,
            p_red_bc_st: None,
            v_bc_st: Cents(0),
            p_icms_st: Rate(0),
            v_icms_st: Cents(0),
            v_bc_fcp_st: None,
            p_fcp_st: None,
            v_fcp_st: None,
        };
        assert_eq!(c.csosn_code(), "202");
    }

    #[test]
    fn csosn_code_203() {
        let c = IcmsCsosn::Csosn203 {
            orig: "0".into(),
            csosn: "203".into(),
            mod_bc_st: "4".into(),
            p_mva_st: None,
            p_red_bc_st: None,
            v_bc_st: Cents(0),
            p_icms_st: Rate(0),
            v_icms_st: Cents(0),
            v_bc_fcp_st: None,
            p_fcp_st: None,
            v_fcp_st: None,
        };
        assert_eq!(c.csosn_code(), "203");
    }

    #[test]
    fn csosn_code_300() {
        let c = IcmsCsosn::Csosn300 {
            orig: "0".into(),
            csosn: "300".into(),
        };
        assert_eq!(c.csosn_code(), "300");
    }

    #[test]
    fn csosn_code_400() {
        let c = IcmsCsosn::Csosn400 {
            orig: "0".into(),
            csosn: "400".into(),
        };
        assert_eq!(c.csosn_code(), "400");
    }

    #[test]
    fn csosn_code_500() {
        let c = IcmsCsosn::Csosn500 {
            orig: "0".into(),
            csosn: "500".into(),
            v_bc_st_ret: None,
            p_st: None,
            v_icms_substituto: None,
            v_icms_st_ret: None,
            v_bc_fcp_st_ret: None,
            p_fcp_st_ret: None,
            v_fcp_st_ret: None,
            p_red_bc_efet: None,
            v_bc_efet: None,
            p_icms_efet: None,
            v_icms_efet: None,
        };
        assert_eq!(c.csosn_code(), "500");
    }

    #[test]
    fn csosn_code_900() {
        let c = IcmsCsosn::Csosn900 {
            orig: "0".into(),
            csosn: "900".into(),
            mod_bc: None,
            v_bc: None,
            p_red_bc: None,
            p_icms: None,
            v_icms: None,
            mod_bc_st: None,
            p_mva_st: None,
            p_red_bc_st: None,
            v_bc_st: None,
            p_icms_st: None,
            v_icms_st: None,
            v_bc_fcp_st: None,
            p_fcp_st: None,
            v_fcp_st: None,
            p_cred_sn: None,
            v_cred_icms_sn: None,
        };
        assert_eq!(c.csosn_code(), "900");
    }

    // ── empty orig in Csosn102/103/300/400 ──────────────────────────

    #[test]
    fn csosn102_empty_orig_omits_orig_field() {
        let c = IcmsCsosn::Csosn102 {
            orig: String::new(),
            csosn: "102".into(),
        };
        let mut totals = IcmsTotals::new();
        let (tag, fields) = build_icms_csosn_xml(&c, &mut totals).unwrap();
        assert_eq!(tag, "ICMSSN102");
        // When orig is empty, the orig field should not be present
        assert!(fields.iter().all(|f| f.name != "orig"));
    }

    // ── merge_icms_totals with ind_deduz_deson=true ─────────────────

    #[test]
    fn merge_icms_totals_propagates_ind_deduz_deson() {
        let mut target = IcmsTotals::new();
        assert!(!target.ind_deduz_deson);

        let source = IcmsTotals::new()
            .v_bc(Cents(1000))
            .v_icms(Cents(180))
            .v_icms_mono(Cents(50))
            .v_icms_mono_reten(Cents(25))
            .v_icms_mono_ret(Cents(10))
            .v_icms_uf_remet(Cents(30))
            .ind_deduz_deson(true);

        merge_icms_totals(&mut target, &source);
        assert!(target.ind_deduz_deson);
        assert_eq!(target.v_bc, Cents(1000));
        assert_eq!(target.v_icms, Cents(180));
        assert_eq!(target.v_icms_mono, Cents(50));
        assert_eq!(target.v_icms_mono_reten, Cents(25));
        assert_eq!(target.v_icms_mono_ret, Cents(10));
        assert_eq!(target.v_icms_uf_remet, Cents(30));
    }

    #[test]
    fn merge_icms_totals_does_not_set_false_on_target() {
        let mut target = IcmsTotals::new().ind_deduz_deson(true);
        let source = IcmsTotals::new(); // ind_deduz_deson = false
        merge_icms_totals(&mut target, &source);
        // target should remain true
        assert!(target.ind_deduz_deson);
    }

    // ── IcmsCst::cst_code() for previously uncovered variants ────────────

    #[test]
    fn cst_code_cst00() {
        let cst = IcmsCst::Cst00 {
            orig: "0".into(),
            mod_bc: "3".into(),
            v_bc: Cents(10000),
            p_icms: Rate(1800),
            v_icms: Cents(1800),
            p_fcp: None,
            v_fcp: None,
        };
        assert_eq!(cst.cst_code(), "00");
    }

    #[test]
    fn cst_code_cst02() {
        let cst = IcmsCst::Cst02 {
            orig: "0".into(),
            q_bc_mono: None,
            ad_rem_icms: Rate(100),
            v_icms_mono: Cents(50),
        };
        assert_eq!(cst.cst_code(), "02");
    }

    #[test]
    fn cst_code_cst10() {
        let cst = IcmsCst::Cst10 {
            orig: "0".into(),
            mod_bc: "3".into(),
            v_bc: Cents(10000),
            p_icms: Rate(1800),
            v_icms: Cents(1800),
            v_bc_fcp: None,
            p_fcp: None,
            v_fcp: None,
            mod_bc_st: "4".into(),
            p_mva_st: None,
            p_red_bc_st: None,
            v_bc_st: Cents(10000),
            p_icms_st: Rate(1800),
            v_icms_st: Cents(1800),
            v_bc_fcp_st: None,
            p_fcp_st: None,
            v_fcp_st: None,
            v_icms_st_deson: None,
            mot_des_icms_st: None,
        };
        assert_eq!(cst.cst_code(), "10");
    }

    #[test]
    fn cst_code_cst15() {
        let cst = IcmsCst::Cst15 {
            orig: "0".into(),
            q_bc_mono: None,
            ad_rem_icms: Rate(100),
            v_icms_mono: Cents(50),
            q_bc_mono_reten: None,
            ad_rem_icms_reten: Rate(80),
            v_icms_mono_reten: Cents(40),
            p_red_ad_rem: None,
            mot_red_ad_rem: None,
        };
        assert_eq!(cst.cst_code(), "15");
    }

    #[test]
    fn cst_code_cst20() {
        let cst = IcmsCst::Cst20 {
            orig: "0".into(),
            mod_bc: "3".into(),
            p_red_bc: Rate(5000),
            v_bc: Cents(5000),
            p_icms: Rate(1800),
            v_icms: Cents(900),
            v_bc_fcp: None,
            p_fcp: None,
            v_fcp: None,
            v_icms_deson: None,
            mot_des_icms: None,
            ind_deduz_deson: None,
        };
        assert_eq!(cst.cst_code(), "20");
    }

    #[test]
    fn cst_code_cst60() {
        let cst = IcmsCst::Cst60 {
            orig: "0".into(),
            v_bc_st_ret: None,
            p_st: None,
            v_icms_substituto: None,
            v_icms_st_ret: None,
            v_bc_fcp_st_ret: None,
            p_fcp_st_ret: None,
            v_fcp_st_ret: None,
            p_red_bc_efet: None,
            v_bc_efet: None,
            p_icms_efet: None,
            v_icms_efet: None,
        };
        assert_eq!(cst.cst_code(), "60");
    }

    #[test]
    fn cst_code_cst70() {
        let cst = IcmsCst::Cst70 {
            orig: "0".into(),
            mod_bc: "3".into(),
            p_red_bc: Rate(5000),
            v_bc: Cents(5000),
            p_icms: Rate(1800),
            v_icms: Cents(900),
            v_bc_fcp: None,
            p_fcp: None,
            v_fcp: None,
            mod_bc_st: "4".into(),
            p_mva_st: None,
            p_red_bc_st: None,
            v_bc_st: Cents(5000),
            p_icms_st: Rate(1800),
            v_icms_st: Cents(900),
            v_bc_fcp_st: None,
            p_fcp_st: None,
            v_fcp_st: None,
            v_icms_deson: None,
            mot_des_icms: None,
            ind_deduz_deson: None,
            v_icms_st_deson: None,
            mot_des_icms_st: None,
        };
        assert_eq!(cst.cst_code(), "70");
    }

    #[test]
    fn cst_code_cst90() {
        let cst = IcmsCst::Cst90 {
            orig: "0".into(),
            mod_bc: None,
            v_bc: None,
            p_red_bc: None,
            c_benef_rbc: None,
            p_icms: None,
            v_icms_op: None,
            p_dif: None,
            v_icms_dif: None,
            v_icms: None,
            v_bc_fcp: None,
            p_fcp: None,
            v_fcp: None,
            p_fcp_dif: None,
            v_fcp_dif: None,
            v_fcp_efet: None,
            mod_bc_st: None,
            p_mva_st: None,
            p_red_bc_st: None,
            v_bc_st: None,
            p_icms_st: None,
            v_icms_st: None,
            v_bc_fcp_st: None,
            p_fcp_st: None,
            v_fcp_st: None,
            v_icms_deson: None,
            mot_des_icms: None,
            ind_deduz_deson: None,
            v_icms_st_deson: None,
            mot_des_icms_st: None,
        };
        assert_eq!(cst.cst_code(), "90");
    }

    // ── IcmsCsosn::csosn_code() for Csosn101 (line 1752) ────────────────

    #[test]
    fn csosn_code_101() {
        let c = IcmsCsosn::Csosn101 {
            orig: "0".into(),
            csosn: "101".into(),
            p_cred_sn: Rate(150),
            v_cred_icms_sn: Cents(30),
        };
        assert_eq!(c.csosn_code(), "101");
    }
}

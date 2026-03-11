use fiscal::tax_icms::{build_icms_xml, build_icms_uf_dest_xml, create_icms_totals, merge_icms_totals, IcmsData};
use rstest::rstest;
use fiscal::newtypes::{Cents, Rate};

// ── buildIcmsXml — Regime Normal (CST) ──────────────────────────────────────

mod regime_normal_cst {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn cst_00_regular_icms() {
        let (xml, totals) = build_icms_xml(&IcmsData {
            tax_regime: 3,
            orig: "0".to_string(),
            cst: Some("00".to_string()),
            mod_bc: Some("0".to_string()),
            v_bc: Some(Cents(10000)),
            p_icms: Some(Rate(1800)),
            v_icms: Some(Cents(1800)),
            ..Default::default()
        }).unwrap();
        assert!(xml.contains("<ICMS00>"));
        assert!(xml.contains("<orig>0</orig>"));
        assert!(xml.contains("<CST>00</CST>"));
        assert!(xml.contains("<modBC>0</modBC>"));
        assert!(xml.contains("<vBC>100.00</vBC>"));
        assert!(xml.contains("<pICMS>18.0000</pICMS>"));
        assert!(xml.contains("<vICMS>18.00</vICMS>"));
        assert_eq!(totals.v_bc, Cents(10000));
        assert_eq!(totals.v_icms, Cents(1800));
    }

    #[test]
    fn cst_00_with_fcp() {
        let (xml, totals) = build_icms_xml(&IcmsData {
            tax_regime: 3,
            orig: "0".to_string(),
            cst: Some("00".to_string()),
            mod_bc: Some("0".to_string()),
            v_bc: Some(Cents(10000)),
            p_icms: Some(Rate(1800)),
            v_icms: Some(Cents(1800)),
            v_bc_fcp: Some(Cents(10000)),
            p_fcp: Some(Rate(200)),
            v_fcp: Some(Cents(200)),
            ..Default::default()
        }).unwrap();
        assert!(xml.contains("<pFCP>"));
        assert!(xml.contains("<vFCP>2.00</vFCP>"));
        assert_eq!(totals.v_fcp, Cents(200));
    }

    #[test]
    fn cst_10_icms_with_st() {
        let (xml, totals) = build_icms_xml(&IcmsData {
            tax_regime: 3,
            orig: "0".to_string(),
            cst: Some("10".to_string()),
            mod_bc: Some("0".to_string()),
            v_bc: Some(Cents(10000)),
            p_icms: Some(Rate(1800)),
            v_icms: Some(Cents(1800)),
            mod_bc_st: Some("4".to_string()),
            p_mva_st: Some(Rate(4000)),
            v_bc_st: Some(Cents(14000)),
            p_icms_st: Some(Rate(1800)),
            v_icms_st: Some(Cents(720)),
            ..Default::default()
        }).unwrap();
        assert!(xml.contains("<ICMS10>"));
        assert!(xml.contains("<modBCST>4</modBCST>"));
        assert!(xml.contains("<vBCST>140.00</vBCST>"));
        assert!(xml.contains("<vICMSST>7.20</vICMSST>"));
        assert_eq!(totals.v_bc_st, Cents(14000));
        assert_eq!(totals.v_st, Cents(720));
    }

    #[test]
    fn cst_20_icms_with_base_reduction() {
        let (xml, _totals) = build_icms_xml(&IcmsData {
            tax_regime: 3,
            orig: "0".to_string(),
            cst: Some("20".to_string()),
            mod_bc: Some("0".to_string()),
            p_red_bc: Some(Rate(5000)),
            v_bc: Some(Cents(5000)),
            p_icms: Some(Rate(1800)),
            v_icms: Some(Cents(900)),
            ..Default::default()
        }).unwrap();
        assert!(xml.contains("<ICMS20>"));
        assert!(xml.contains("<pRedBC>50.0000</pRedBC>"));
    }

    #[test]
    fn cst_30_exempt_with_st() {
        let (xml, totals) = build_icms_xml(&IcmsData {
            tax_regime: 3,
            orig: "0".to_string(),
            cst: Some("30".to_string()),
            mod_bc_st: Some("4".to_string()),
            v_bc_st: Some(Cents(10000)),
            p_icms_st: Some(Rate(1800)),
            v_icms_st: Some(Cents(1800)),
            ..Default::default()
        }).unwrap();
        assert!(xml.contains("<ICMS30>"));
        assert_eq!(totals.v_st, Cents(1800));
    }

    /// CST 40 (exempt), 41 (not taxed), and 50 (suspended) all use the ICMS40 tag
    /// and produce zero ICMS. Parametrize to avoid repetition.
    #[rstest]
    #[case("40", "<ICMS40>", "<CST>40</CST>")]
    #[case("41", "<ICMS40>", "<CST>41</CST>")]
    #[case("50", "<ICMS40>", "<CST>50</CST>")]
    fn cst_40_41_50_exempt_variants(
        #[case] cst: &str,
        #[case] expected_tag: &str,
        #[case] expected_cst: &str,
    ) {
        let (xml, totals) = build_icms_xml(&IcmsData {
            tax_regime: 3,
            orig: "0".to_string(),
            cst: Some(cst.to_string()),
            ..Default::default()
        }).unwrap();
        assert!(xml.contains(expected_tag), "CST {cst} should produce {expected_tag}");
        assert!(xml.contains(expected_cst), "CST {cst} should contain {expected_cst}");
        assert_eq!(totals.v_icms, Cents(0), "CST {cst} should have zero ICMS");
    }

    #[test]
    fn cst_51_deferred() {
        let (xml, _totals) = build_icms_xml(&IcmsData {
            tax_regime: 3,
            orig: "0".to_string(),
            cst: Some("51".to_string()),
            mod_bc: Some("0".to_string()),
            v_bc: Some(Cents(10000)),
            p_icms: Some(Rate(1800)),
            v_icms: Some(Cents(1800)),
            ..Default::default()
        }).unwrap();
        assert!(xml.contains("<ICMS51>"));
    }

    #[test]
    fn cst_60_previously_charged_by_st() {
        let (xml, _totals) = build_icms_xml(&IcmsData {
            tax_regime: 3,
            orig: "0".to_string(),
            cst: Some("60".to_string()),
            ..Default::default()
        }).unwrap();
        assert!(xml.contains("<ICMS60>"));
    }

    #[test]
    fn cst_70_reduction_with_st() {
        let (xml, totals) = build_icms_xml(&IcmsData {
            tax_regime: 3,
            orig: "0".to_string(),
            cst: Some("70".to_string()),
            mod_bc: Some("0".to_string()),
            p_red_bc: Some(Rate(3000)),
            v_bc: Some(Cents(7000)),
            p_icms: Some(Rate(1800)),
            v_icms: Some(Cents(1260)),
            mod_bc_st: Some("4".to_string()),
            v_bc_st: Some(Cents(10000)),
            p_icms_st: Some(Rate(1800)),
            v_icms_st: Some(Cents(540)),
            ..Default::default()
        }).unwrap();
        assert!(xml.contains("<ICMS70>"));
        assert_eq!(totals.v_bc, Cents(7000));
        assert_eq!(totals.v_st, Cents(540));
    }

    #[test]
    fn cst_90_other() {
        let (xml, _totals) = build_icms_xml(&IcmsData {
            tax_regime: 3,
            orig: "0".to_string(),
            cst: Some("90".to_string()),
            mod_bc: Some("0".to_string()),
            v_bc: Some(Cents(10000)),
            p_icms: Some(Rate(1200)),
            v_icms: Some(Cents(1200)),
            ..Default::default()
        }).unwrap();
        assert!(xml.contains("<ICMS90>"));
    }

    #[test]
    fn throws_on_unknown_cst() {
        let result = build_icms_xml(&IcmsData {
            tax_regime: 3,
            orig: "0".to_string(),
            cst: Some("99".to_string()),
            ..Default::default()
        });
        assert!(result.is_err());
    }
}

// ── buildIcmsXml — Simples Nacional (CSOSN) ─────────────────────────────────

mod simples_nacional_csosn {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn csosn_101_with_sn_credit() {
        let (xml, _totals) = build_icms_xml(&IcmsData {
            tax_regime: 1,
            orig: "0".to_string(),
            csosn: Some("101".to_string()),
            p_cred_sn: Some(Rate(350)),
            v_cred_icms_sn: Some(Cents(350)),
            ..Default::default()
        }).unwrap();
        assert!(xml.contains("<CSOSN>101</CSOSN>"));
        assert!(xml.contains("<pCredSN>"));
        assert!(xml.contains("<vCredICMSSN>"));
    }

    /// CSOSN 102 and 103 both produce a simple CSOSN tag with no credit fields.
    #[rstest]
    #[case("102")]
    #[case("103")]
    fn csosn_102_103_no_credit(#[case] csosn: &str) {
        let (xml, _totals) = build_icms_xml(&IcmsData {
            tax_regime: 1,
            orig: "0".to_string(),
            csosn: Some(csosn.to_string()),
            ..Default::default()
        }).unwrap();
        assert!(xml.contains(&format!("<CSOSN>{csosn}</CSOSN>")));
        assert!(!xml.contains("<pCredSN>"), "CSOSN {csosn} should not have credit");
    }

    #[test]
    fn csosn_201_sn_with_st_and_credit() {
        let (xml, totals) = build_icms_xml(&IcmsData {
            tax_regime: 1,
            orig: "0".to_string(),
            csosn: Some("201".to_string()),
            mod_bc_st: Some("4".to_string()),
            v_bc_st: Some(Cents(10000)),
            p_icms_st: Some(Rate(1800)),
            v_icms_st: Some(Cents(1800)),
            p_cred_sn: Some(Rate(350)),
            v_cred_icms_sn: Some(Cents(350)),
            ..Default::default()
        }).unwrap();
        assert!(xml.contains("<CSOSN>201</CSOSN>"));
        assert!(xml.contains("<vICMSST>"));
        assert!(xml.contains("<pCredSN>"));
        assert_eq!(totals.v_st, Cents(1800));
    }

    #[test]
    fn csosn_202_sn_with_st_no_credit() {
        let (xml, _totals) = build_icms_xml(&IcmsData {
            tax_regime: 1,
            orig: "0".to_string(),
            csosn: Some("202".to_string()),
            mod_bc_st: Some("4".to_string()),
            v_bc_st: Some(Cents(10000)),
            p_icms_st: Some(Rate(1800)),
            v_icms_st: Some(Cents(1800)),
            ..Default::default()
        }).unwrap();
        assert!(xml.contains("<CSOSN>202</CSOSN>"));
    }

    #[test]
    fn csosn_500_previously_charged_by_st() {
        let (xml, _totals) = build_icms_xml(&IcmsData {
            tax_regime: 1,
            orig: "0".to_string(),
            csosn: Some("500".to_string()),
            ..Default::default()
        }).unwrap();
        assert!(xml.contains("<CSOSN>500</CSOSN>"));
    }

    #[test]
    fn csosn_900_other_sn() {
        let (xml, _totals) = build_icms_xml(&IcmsData {
            tax_regime: 1,
            orig: "0".to_string(),
            csosn: Some("900".to_string()),
            mod_bc: Some("0".to_string()),
            v_bc: Some(Cents(10000)),
            p_icms: Some(Rate(1800)),
            v_icms: Some(Cents(1800)),
            ..Default::default()
        }).unwrap();
        assert!(xml.contains("<CSOSN>900</CSOSN>"));
    }

    #[test]
    fn throws_on_unknown_csosn() {
        let result = build_icms_xml(&IcmsData {
            tax_regime: 1,
            orig: "0".to_string(),
            csosn: Some("999".to_string()),
            ..Default::default()
        });
        assert!(result.is_err());
    }
}

// ── buildIcmsUfDestXml ──────────────────────────────────────────────────────

mod icms_uf_dest {
    use super::*;

    #[test]
    fn builds_interstate_destination_icms() {
        let (xml, _totals) = build_icms_uf_dest_xml(&IcmsData {
            tax_regime: 3,
            orig: "0".to_string(),
            v_bc_uf_dest: Some(Cents(10000)),
            p_fcp_uf_dest: Some(Rate(200)),
            v_fcp_uf_dest: Some(Cents(200)),
            p_icms_uf_dest: Some(Rate(1800)),
            v_icms_uf_dest: Some(Cents(600)),
            p_icms_inter: Some(Rate(1200)),
            p_icms_inter_part: Some(Rate(10000)),
            v_icms_uf_remet: Some(Cents(1200)),
            ..Default::default()
        }).unwrap();
        assert!(xml.contains("<ICMSUFDest>"));
        assert!(xml.contains("<vBCUFDest>100.00</vBCUFDest>"));
    }
}

// ── totals accumulation ─────────────────────────────────────────────────────

mod totals_accumulation {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn create_icms_totals_returns_zeroed_object() {
        let t = create_icms_totals();
        assert_eq!(t.v_bc, Cents(0));
        assert_eq!(t.v_icms, Cents(0));
        assert_eq!(t.v_st, Cents(0));
        assert_eq!(t.v_fcp, Cents(0));
    }

    #[test]
    fn merge_icms_totals_accumulates_correctly() {
        let mut target = create_icms_totals();
        let mut source1 = create_icms_totals();
        source1.v_bc = Cents(5000);
        source1.v_icms = Cents(900);
        let mut source2 = create_icms_totals();
        source2.v_bc = Cents(3000);
        source2.v_icms = Cents(540);
        source2.v_st = Cents(200);

        merge_icms_totals(&mut target, &source1);
        merge_icms_totals(&mut target, &source2);

        assert_eq!(target.v_bc, Cents(8000));
        assert_eq!(target.v_icms, Cents(1440));
        assert_eq!(target.v_st, Cents(200));
    }
}

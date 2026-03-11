use fiscal::newtypes::{Cents, Rate};
use fiscal::tax_icms::{
    IcmsCsosn, IcmsCst, IcmsTotals, IcmsUfDestData, IcmsVariant, build_icms_uf_dest_xml,
    build_icms_xml, create_icms_totals, merge_icms_totals,
};
use rstest::rstest;

// ── buildIcmsXml — Regime Normal (CST) ──────────────────────────────────────

mod regime_normal_cst {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn cst_00_regular_icms() {
        let variant = IcmsVariant::from(IcmsCst::Cst00 {
            orig: "0".into(),
            mod_bc: "0".into(),
            v_bc: Cents(10000),
            p_icms: Rate(1800),
            v_icms: Cents(1800),
            p_fcp: None,
            v_fcp: None,
        });
        let mut totals = IcmsTotals::default();
        let xml = build_icms_xml(&variant, &mut totals).unwrap();
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
        let variant = IcmsVariant::from(IcmsCst::Cst00 {
            orig: "0".into(),
            mod_bc: "0".into(),
            v_bc: Cents(10000),
            p_icms: Rate(1800),
            v_icms: Cents(1800),
            p_fcp: Some(Rate(200)),
            v_fcp: Some(Cents(200)),
        });
        let mut totals = IcmsTotals::default();
        let xml = build_icms_xml(&variant, &mut totals).unwrap();
        assert!(xml.contains("<pFCP>"));
        assert!(xml.contains("<vFCP>2.00</vFCP>"));
        assert_eq!(totals.v_fcp, Cents(200));
    }

    #[test]
    fn cst_10_icms_with_st() {
        let variant = IcmsVariant::from(IcmsCst::Cst10 {
            orig: "0".into(),
            mod_bc: "0".into(),
            v_bc: Cents(10000),
            p_icms: Rate(1800),
            v_icms: Cents(1800),
            v_bc_fcp: None,
            p_fcp: None,
            v_fcp: None,
            mod_bc_st: "4".into(),
            p_mva_st: Some(Rate(4000)),
            p_red_bc_st: None,
            v_bc_st: Cents(14000),
            p_icms_st: Rate(1800),
            v_icms_st: Cents(720),
            v_bc_fcp_st: None,
            p_fcp_st: None,
            v_fcp_st: None,
            v_icms_st_deson: None,
            mot_des_icms_st: None,
        });
        let mut totals = IcmsTotals::default();
        let xml = build_icms_xml(&variant, &mut totals).unwrap();
        assert!(xml.contains("<ICMS10>"));
        assert!(xml.contains("<modBCST>4</modBCST>"));
        assert!(xml.contains("<vBCST>140.00</vBCST>"));
        assert!(xml.contains("<vICMSST>7.20</vICMSST>"));
        assert_eq!(totals.v_bc_st, Cents(14000));
        assert_eq!(totals.v_st, Cents(720));
    }

    #[test]
    fn cst_20_icms_with_base_reduction() {
        let variant = IcmsVariant::from(IcmsCst::Cst20 {
            orig: "0".into(),
            mod_bc: "0".into(),
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
        });
        let mut totals = IcmsTotals::default();
        let xml = build_icms_xml(&variant, &mut totals).unwrap();
        assert!(xml.contains("<ICMS20>"));
        assert!(xml.contains("<pRedBC>50.0000</pRedBC>"));
    }

    #[test]
    fn cst_30_exempt_with_st() {
        let variant = IcmsVariant::from(IcmsCst::Cst30 {
            orig: "0".into(),
            mod_bc_st: "4".into(),
            p_mva_st: None,
            p_red_bc_st: None,
            v_bc_st: Cents(10000),
            p_icms_st: Rate(1800),
            v_icms_st: Cents(1800),
            v_bc_fcp_st: None,
            p_fcp_st: None,
            v_fcp_st: None,
            v_icms_deson: None,
            mot_des_icms: None,
            ind_deduz_deson: None,
        });
        let mut totals = IcmsTotals::default();
        let xml = build_icms_xml(&variant, &mut totals).unwrap();
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
        let variant = match cst {
            "40" => IcmsVariant::from(IcmsCst::Cst40 {
                orig: "0".into(),
                v_icms_deson: None,
                mot_des_icms: None,
                ind_deduz_deson: None,
            }),
            "41" => IcmsVariant::from(IcmsCst::Cst41 {
                orig: "0".into(),
                v_icms_deson: None,
                mot_des_icms: None,
                ind_deduz_deson: None,
            }),
            "50" => IcmsVariant::from(IcmsCst::Cst50 {
                orig: "0".into(),
                v_icms_deson: None,
                mot_des_icms: None,
                ind_deduz_deson: None,
            }),
            _ => unreachable!(),
        };
        let mut totals = IcmsTotals::default();
        let xml = build_icms_xml(&variant, &mut totals).unwrap();
        assert!(
            xml.contains(expected_tag),
            "CST {cst} should produce {expected_tag}"
        );
        assert!(
            xml.contains(expected_cst),
            "CST {cst} should contain {expected_cst}"
        );
        assert_eq!(totals.v_icms, Cents(0), "CST {cst} should have zero ICMS");
    }

    #[test]
    fn cst_51_deferred() {
        let variant = IcmsVariant::from(IcmsCst::Cst51 {
            orig: "0".into(),
            mod_bc: Some("0".into()),
            p_red_bc: None,
            c_benef_rbc: None,
            v_bc: Some(Cents(10000)),
            p_icms: Some(Rate(1800)),
            v_icms_op: None,
            p_dif: None,
            v_icms_dif: None,
            v_icms: Some(Cents(1800)),
            v_bc_fcp: None,
            p_fcp: None,
            v_fcp: None,
            p_fcp_dif: None,
            v_fcp_dif: None,
            v_fcp_efet: None,
        });
        let mut totals = IcmsTotals::default();
        let xml = build_icms_xml(&variant, &mut totals).unwrap();
        assert!(xml.contains("<ICMS51>"));
    }

    #[test]
    fn cst_60_previously_charged_by_st() {
        let variant = IcmsVariant::from(IcmsCst::Cst60 {
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
        });
        let mut totals = IcmsTotals::default();
        let xml = build_icms_xml(&variant, &mut totals).unwrap();
        assert!(xml.contains("<ICMS60>"));
    }

    #[test]
    fn cst_70_reduction_with_st() {
        let variant = IcmsVariant::from(IcmsCst::Cst70 {
            orig: "0".into(),
            mod_bc: "0".into(),
            p_red_bc: Rate(3000),
            v_bc: Cents(7000),
            p_icms: Rate(1800),
            v_icms: Cents(1260),
            v_bc_fcp: None,
            p_fcp: None,
            v_fcp: None,
            mod_bc_st: "4".into(),
            p_mva_st: None,
            p_red_bc_st: None,
            v_bc_st: Cents(10000),
            p_icms_st: Rate(1800),
            v_icms_st: Cents(540),
            v_bc_fcp_st: None,
            p_fcp_st: None,
            v_fcp_st: None,
            v_icms_deson: None,
            mot_des_icms: None,
            ind_deduz_deson: None,
            v_icms_st_deson: None,
            mot_des_icms_st: None,
        });
        let mut totals = IcmsTotals::default();
        let xml = build_icms_xml(&variant, &mut totals).unwrap();
        assert!(xml.contains("<ICMS70>"));
        assert_eq!(totals.v_bc, Cents(7000));
        assert_eq!(totals.v_st, Cents(540));
    }

    #[test]
    fn cst_90_other() {
        let variant = IcmsVariant::from(IcmsCst::Cst90 {
            orig: "0".into(),
            mod_bc: Some("0".into()),
            v_bc: Some(Cents(10000)),
            p_red_bc: None,
            c_benef_rbc: None,
            p_icms: Some(Rate(1200)),
            v_icms_op: None,
            p_dif: None,
            v_icms_dif: None,
            v_icms: Some(Cents(1200)),
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
        });
        let mut totals = IcmsTotals::default();
        let xml = build_icms_xml(&variant, &mut totals).unwrap();
        assert!(xml.contains("<ICMS90>"));
    }

    #[test]
    fn throws_on_unknown_cst() {
        // With the algebraic API, unknown CSTs are a compile-time error.
        // This test verifies that UnsupportedIcmsCst still works at the error level.
        let err = fiscal::FiscalError::UnsupportedIcmsCst("99".into());
        assert!(format!("{err:?}").contains("99"));
    }
}

// ── buildIcmsXml — Simples Nacional (CSOSN) ─────────────────────────────────

mod simples_nacional_csosn {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn csosn_101_with_sn_credit() {
        let variant = IcmsVariant::from(IcmsCsosn::Csosn101 {
            orig: "0".into(),
            csosn: "101".into(),
            p_cred_sn: Rate(350),
            v_cred_icms_sn: Cents(350),
        });
        let mut totals = IcmsTotals::default();
        let xml = build_icms_xml(&variant, &mut totals).unwrap();
        assert!(xml.contains("<CSOSN>101</CSOSN>"));
        assert!(xml.contains("<pCredSN>"));
        assert!(xml.contains("<vCredICMSSN>"));
    }

    /// CSOSN 102 and 103 both produce a simple CSOSN tag with no credit fields.
    #[rstest]
    #[case("102")]
    #[case("103")]
    fn csosn_102_103_no_credit(#[case] csosn: &str) {
        let variant = IcmsVariant::from(IcmsCsosn::Csosn102 {
            orig: "0".into(),
            csosn: csosn.into(),
        });
        let mut totals = IcmsTotals::default();
        let xml = build_icms_xml(&variant, &mut totals).unwrap();
        assert!(xml.contains(&format!("<CSOSN>{csosn}</CSOSN>")));
        assert!(
            !xml.contains("<pCredSN>"),
            "CSOSN {csosn} should not have credit"
        );
    }

    #[test]
    fn csosn_201_sn_with_st_and_credit() {
        let variant = IcmsVariant::from(IcmsCsosn::Csosn201 {
            orig: "0".into(),
            csosn: "201".into(),
            mod_bc_st: "4".into(),
            p_mva_st: None,
            p_red_bc_st: None,
            v_bc_st: Cents(10000),
            p_icms_st: Rate(1800),
            v_icms_st: Cents(1800),
            v_bc_fcp_st: None,
            p_fcp_st: None,
            v_fcp_st: None,
            p_cred_sn: Some(Rate(350)),
            v_cred_icms_sn: Some(Cents(350)),
        });
        let mut totals = IcmsTotals::default();
        let xml = build_icms_xml(&variant, &mut totals).unwrap();
        assert!(xml.contains("<CSOSN>201</CSOSN>"));
        assert!(xml.contains("<vICMSST>"));
        assert!(xml.contains("<pCredSN>"));
        assert_eq!(totals.v_st, Cents(1800));
    }

    #[test]
    fn csosn_202_sn_with_st_no_credit() {
        let variant = IcmsVariant::from(IcmsCsosn::Csosn202 {
            orig: "0".into(),
            csosn: "202".into(),
            mod_bc_st: "4".into(),
            p_mva_st: None,
            p_red_bc_st: None,
            v_bc_st: Cents(10000),
            p_icms_st: Rate(1800),
            v_icms_st: Cents(1800),
            v_bc_fcp_st: None,
            p_fcp_st: None,
            v_fcp_st: None,
        });
        let mut totals = IcmsTotals::default();
        let xml = build_icms_xml(&variant, &mut totals).unwrap();
        assert!(xml.contains("<CSOSN>202</CSOSN>"));
    }

    #[test]
    fn csosn_500_previously_charged_by_st() {
        let variant = IcmsVariant::from(IcmsCsosn::Csosn500 {
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
        });
        let mut totals = IcmsTotals::default();
        let xml = build_icms_xml(&variant, &mut totals).unwrap();
        assert!(xml.contains("<CSOSN>500</CSOSN>"));
    }

    #[test]
    fn csosn_900_other_sn() {
        let variant = IcmsVariant::from(IcmsCsosn::Csosn900 {
            orig: "0".into(),
            csosn: "900".into(),
            mod_bc: Some("0".into()),
            v_bc: Some(Cents(10000)),
            p_red_bc: None,
            p_icms: Some(Rate(1800)),
            v_icms: Some(Cents(1800)),
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
        });
        let mut totals = IcmsTotals::default();
        let xml = build_icms_xml(&variant, &mut totals).unwrap();
        assert!(xml.contains("<CSOSN>900</CSOSN>"));
    }

    #[test]
    fn throws_on_unknown_csosn() {
        // With the algebraic API, unknown CSOSNs are a compile-time error.
        let err = fiscal::FiscalError::UnsupportedIcmsCsosn("999".into());
        assert!(format!("{err:?}").contains("999"));
    }
}

// ── buildIcmsUfDestXml ──────────────────────────────────────────────────────

mod icms_uf_dest {
    use super::*;

    #[test]
    fn builds_interstate_destination_icms() {
        let (xml, _totals) = build_icms_uf_dest_xml(
            &IcmsUfDestData::new(Cents(10000), Rate(1800), Rate(1200), Cents(600))
                .p_fcp_uf_dest(Rate(200))
                .v_fcp_uf_dest(Cents(200))
                .v_icms_uf_remet(Cents(1200)),
        )
        .unwrap();
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

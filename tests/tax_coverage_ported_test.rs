// Ported from TypeScript tax-coverage-ported.test.ts — ICMS sections only (first half).
//
// Each TypeScript describe()/it() block becomes a Rust mod/#[test].
// All tests use the algebraic IcmsCst/IcmsCsosn/IcmsVariant types.
//
// Covers: ICMS CST (00–90), ICMSPart, ICMSST, ICMSSN CSOSN (101–900),
//         ICMSUFDest, and ICMS totals.

mod common;

use fiscal::newtypes::{Cents, Rate};
use fiscal::tax_icms::{
    IcmsCsosn, IcmsCst, IcmsPartData, IcmsStData, IcmsTotals, IcmsUfDestData, IcmsVariant,
    build_icms_part_xml, build_icms_st_xml, build_icms_uf_dest_xml, build_icms_xml,
    create_icms_totals, merge_icms_totals,
};

use common::{expect_xml_contains, expect_xml_not_contains};

// =============================================================================
// TaxCoverageTest — ICMS CST
// =============================================================================

mod icms_cst {
    use super::*;

    #[test]
    fn test_icms00_cst_00_with_fcp() {
        let v = IcmsVariant::from(IcmsCst::Cst00 {
            orig: "0".into(),
            mod_bc: "3".into(),
            v_bc: Cents(10000),
            p_icms: Rate(1800),
            v_icms: Cents(1800),
            p_fcp: Some(Rate(200)),
            v_fcp: Some(Cents(200)),
        });
        let mut t = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut t).unwrap();
        expect_xml_contains(&xml, &["<ICMS00>", "<vFCP>"]);
    }

    #[test]
    fn test_icms00_without_fcp_cst_00_without_fcp() {
        let v = IcmsVariant::from(IcmsCst::Cst00 {
            orig: "0".into(),
            mod_bc: "3".into(),
            v_bc: Cents(10000),
            p_icms: Rate(1800),
            v_icms: Cents(1800),
            p_fcp: None,
            v_fcp: None,
        });
        let mut t = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut t).unwrap();
        expect_xml_contains(&xml, &["<ICMS00>"]);
        expect_xml_not_contains(&xml, &["<vFCP>"]);
    }

    #[test]
    fn test_icms02_cst_02_monofasico() {
        let v = IcmsVariant::from(IcmsCst::Cst02 {
            orig: "0".into(),
            q_bc_mono: Some(1000000),
            ad_rem_icms: Rate(15000),
            v_icms_mono: Cents(15000),
        });
        let mut t = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut t).unwrap();
        expect_xml_contains(&xml, &["<ICMS02>", "<adRemICMS>"]);
    }

    #[test]
    fn test_icms10_cst_10_with_st_and_fcp() {
        let v = IcmsVariant::from(IcmsCst::Cst10 {
            orig: "0".into(),
            mod_bc: "3".into(),
            v_bc: Cents(10000),
            p_icms: Rate(1800),
            v_icms: Cents(1800),
            v_bc_fcp: Some(Cents(10000)),
            p_fcp: Some(Rate(200)),
            v_fcp: Some(Cents(200)),
            mod_bc_st: "4".into(),
            p_mva_st: Some(Rate(4000)),
            p_red_bc_st: Some(Rate(0)),
            v_bc_st: Cents(14000),
            p_icms_st: Rate(1800),
            v_icms_st: Cents(720),
            v_bc_fcp_st: Some(Cents(14000)),
            p_fcp_st: Some(Rate(200)),
            v_fcp_st: Some(Cents(280)),
            v_icms_st_deson: Some(Cents(100)),
            mot_des_icms_st: Some("3".into()),
        });
        let mut t = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut t).unwrap();
        expect_xml_contains(
            &xml,
            &["<ICMS10>", "<vFCPST>", "<vICMSSTDeson>", "<motDesICMSST>"],
        );
    }

    #[test]
    fn test_icms15_cst_15_monofasico_with_retention() {
        let v = IcmsVariant::from(IcmsCst::Cst15 {
            orig: "0".into(),
            q_bc_mono: Some(1000000),
            ad_rem_icms: Rate(15000),
            v_icms_mono: Cents(15000),
            q_bc_mono_reten: Some(500000),
            ad_rem_icms_reten: Rate(10000),
            v_icms_mono_reten: Cents(5000),
            p_red_ad_rem: None,
            mot_red_ad_rem: None,
        });
        let mut t = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut t).unwrap();
        expect_xml_contains(&xml, &["<ICMS15>", "<qBCMonoReten>"]);
        expect_xml_not_contains(&xml, &["<pRedAdRem>"]);
    }

    #[test]
    fn test_icms15_with_p_red_ad_rem_cst_15_with_ad_rem_reduction() {
        let v = IcmsVariant::from(IcmsCst::Cst15 {
            orig: "0".into(),
            q_bc_mono: Some(1000000),
            ad_rem_icms: Rate(15000),
            v_icms_mono: Cents(15000),
            q_bc_mono_reten: Some(500000),
            ad_rem_icms_reten: Rate(10000),
            v_icms_mono_reten: Cents(5000),
            p_red_ad_rem: Some(Rate(1000)),
            mot_red_ad_rem: Some("1".into()),
        });
        let mut t = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut t).unwrap();
        expect_xml_contains(&xml, &["<ICMS15>", "<pRedAdRem>", "<motRedAdRem>"]);
    }

    #[test]
    fn test_icms20_cst_20_with_fcp_and_desoneration() {
        let v = IcmsVariant::from(IcmsCst::Cst20 {
            orig: "0".into(),
            mod_bc: "3".into(),
            p_red_bc: Rate(1000),
            v_bc: Cents(9000),
            p_icms: Rate(1800),
            v_icms: Cents(1620),
            v_bc_fcp: Some(Cents(9000)),
            p_fcp: Some(Rate(200)),
            v_fcp: Some(Cents(180)),
            v_icms_deson: Some(Cents(180)),
            mot_des_icms: Some("9".into()),
            ind_deduz_deson: Some("1".into()),
        });
        let mut t = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut t).unwrap();
        expect_xml_contains(&xml, &["<ICMS20>", "<vICMSDeson>", "<indDeduzDeson>"]);
    }

    #[test]
    fn test_icms30_cst_30_with_st_fcp() {
        let v = IcmsVariant::from(IcmsCst::Cst30 {
            orig: "0".into(),
            mod_bc_st: "4".into(),
            p_mva_st: Some(Rate(4000)),
            p_red_bc_st: Some(Rate(0)),
            v_bc_st: Cents(14000),
            p_icms_st: Rate(1800),
            v_icms_st: Cents(720),
            v_bc_fcp_st: Some(Cents(14000)),
            p_fcp_st: Some(Rate(200)),
            v_fcp_st: Some(Cents(280)),
            v_icms_deson: Some(Cents(180)),
            mot_des_icms: Some("9".into()),
            ind_deduz_deson: Some("1".into()),
        });
        let mut t = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut t).unwrap();
        expect_xml_contains(&xml, &["<ICMS30>", "<vBCFCPST>", "<indDeduzDeson>"]);
    }

    #[test]
    fn test_icms40_cst_40_isento_with_desoneration() {
        let v = IcmsVariant::from(IcmsCst::Cst40 {
            orig: "0".into(),
            v_icms_deson: Some(Cents(1800)),
            mot_des_icms: Some("1".into()),
            ind_deduz_deson: Some("1".into()),
        });
        let mut t = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut t).unwrap();
        expect_xml_contains(&xml, &["<ICMS40>", "<vICMSDeson>"]);
    }

    #[test]
    fn test_icms41_cst_41_uses_icms40_wrapper() {
        let v = IcmsVariant::from(IcmsCst::Cst41 {
            orig: "0".into(),
            v_icms_deson: None,
            mot_des_icms: None,
            ind_deduz_deson: None,
        });
        let mut t = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut t).unwrap();
        expect_xml_contains(&xml, &["<ICMS40>", "<CST>41</CST>"]);
    }

    #[test]
    fn test_icms50_cst_50_uses_icms40_wrapper() {
        let v = IcmsVariant::from(IcmsCst::Cst50 {
            orig: "0".into(),
            v_icms_deson: Some(Cents(500)),
            mot_des_icms: Some("9".into()),
            ind_deduz_deson: None,
        });
        let mut t = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut t).unwrap();
        expect_xml_contains(&xml, &["<ICMS40>", "<CST>50</CST>"]);
    }

    #[test]
    fn test_icms51_with_dif_cst_51_with_deferral_and_fcp_deferral() {
        let v = IcmsVariant::from(IcmsCst::Cst51 {
            orig: "0".into(),
            mod_bc: Some("3".into()),
            p_red_bc: Some(Rate(1000)),
            c_benef_rbc: Some("SP999999".into()),
            v_bc: Some(Cents(9000)),
            p_icms: Some(Rate(1800)),
            v_icms_op: Some(Cents(1620)),
            p_dif: Some(Rate(3333)),
            v_icms_dif: Some(Cents(540)),
            v_icms: Some(Cents(1080)),
            v_bc_fcp: Some(Cents(9000)),
            p_fcp: Some(Rate(200)),
            v_fcp: Some(Cents(180)),
            p_fcp_dif: Some(Rate(3333)),
            v_fcp_dif: Some(Cents(60)),
            v_fcp_efet: Some(Cents(120)),
        });
        let mut t = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut t).unwrap();
        expect_xml_contains(
            &xml,
            &[
                "<ICMS51>",
                "<pDif>33.3300</pDif>",
                "<vICMSDif>5.40</vICMSDif>",
                "<cBenefRBC>",
                "<pFCPDif>33.3300</pFCPDif>",
                "<vFCPDif>0.60</vFCPDif>",
                "<vFCPEfet>1.20</vFCPEfet>",
            ],
        );
    }

    #[test]
    fn test_icms51_minimal_cst_51_minimal() {
        let v = IcmsVariant::from(IcmsCst::Cst51 {
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
        });
        let mut t = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut t).unwrap();
        expect_xml_contains(&xml, &["<ICMS51>"]);
        expect_xml_not_contains(&xml, &["<pDif>"]);
    }

    #[test]
    fn test_icms53_cst_53_monofasico_with_deferral() {
        let v = IcmsVariant::from(IcmsCst::Cst53 {
            orig: "0".into(),
            q_bc_mono: Some(1000000),
            ad_rem_icms: Some(Rate(15000)),
            v_icms_mono_op: Some(Cents(15000)),
            p_dif: Some(Rate(3333)),
            v_icms_mono_dif: Some(Cents(5000)),
            v_icms_mono: Some(Cents(10000)),
        });
        let mut t = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut t).unwrap();
        expect_xml_contains(&xml, &["<ICMS53>", "<vICMSMonoOp>", "<vICMSMonoDif>"]);
    }

    #[test]
    fn test_icms60_cst_60_with_st_retained_and_effective_values() {
        let v = IcmsVariant::from(IcmsCst::Cst60 {
            orig: "0".into(),
            v_bc_st_ret: Some(Cents(10000)),
            p_st: Some(Rate(1800)),
            v_icms_substituto: Some(Cents(1000)),
            v_icms_st_ret: Some(Cents(800)),
            v_bc_fcp_st_ret: Some(Cents(10000)),
            p_fcp_st_ret: Some(Rate(200)),
            v_fcp_st_ret: Some(Cents(200)),
            p_red_bc_efet: Some(Rate(1000)),
            v_bc_efet: Some(Cents(9000)),
            p_icms_efet: Some(Rate(1800)),
            v_icms_efet: Some(Cents(1620)),
        });
        let mut t = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut t).unwrap();
        expect_xml_contains(
            &xml,
            &[
                "<ICMS60>",
                "<vICMSSubstituto>",
                "<pRedBCEfet>",
                "<vICMSEfet>",
            ],
        );
    }

    #[test]
    fn test_icms60_minimal_cst_60_minimal() {
        let v = IcmsVariant::from(IcmsCst::Cst60 {
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
        let mut t = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut t).unwrap();
        expect_xml_contains(&xml, &["<ICMS60>"]);
        expect_xml_not_contains(&xml, &["<vICMSSubstituto>"]);
    }

    #[test]
    fn test_icms61_cst_61_monofasico_retained() {
        let v = IcmsVariant::from(IcmsCst::Cst61 {
            orig: "0".into(),
            q_bc_mono_ret: Some(1000000),
            ad_rem_icms_ret: Rate(15000),
            v_icms_mono_ret: Cents(15000),
        });
        let mut t = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut t).unwrap();
        expect_xml_contains(&xml, &["<ICMS61>", "<adRemICMSRet>"]);
    }

    #[test]
    fn test_icms70_full_cst_70_with_all_st_and_desoneration_fields() {
        let v = IcmsVariant::from(IcmsCst::Cst70 {
            orig: "0".into(),
            mod_bc: "3".into(),
            p_red_bc: Rate(1000),
            v_bc: Cents(9000),
            p_icms: Rate(1800),
            v_icms: Cents(1620),
            v_bc_fcp: Some(Cents(9000)),
            p_fcp: Some(Rate(200)),
            v_fcp: Some(Cents(180)),
            mod_bc_st: "4".into(),
            p_mva_st: Some(Rate(4000)),
            p_red_bc_st: Some(Rate(0)),
            v_bc_st: Cents(12600),
            p_icms_st: Rate(1800),
            v_icms_st: Cents(648),
            v_bc_fcp_st: Some(Cents(12600)),
            p_fcp_st: Some(Rate(200)),
            v_fcp_st: Some(Cents(252)),
            v_icms_deson: Some(Cents(180)),
            mot_des_icms: Some("9".into()),
            ind_deduz_deson: Some("1".into()),
            v_icms_st_deson: Some(Cents(100)),
            mot_des_icms_st: Some("3".into()),
        });
        let mut t = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut t).unwrap();
        expect_xml_contains(
            &xml,
            &[
                "<ICMS70>",
                "<vICMSSTDeson>",
                "<motDesICMSST>",
                "<indDeduzDeson>",
            ],
        );
    }

    #[test]
    fn test_icms70_without_st_deson_cst_70_without_st_desoneration() {
        let v = IcmsVariant::from(IcmsCst::Cst70 {
            orig: "0".into(),
            mod_bc: "3".into(),
            p_red_bc: Rate(1000),
            v_bc: Cents(9000),
            p_icms: Rate(1800),
            v_icms: Cents(1620),
            v_bc_fcp: None,
            p_fcp: None,
            v_fcp: None,
            mod_bc_st: "4".into(),
            p_mva_st: None,
            p_red_bc_st: None,
            v_bc_st: Cents(12600),
            p_icms_st: Rate(1800),
            v_icms_st: Cents(648),
            v_bc_fcp_st: None,
            p_fcp_st: None,
            v_fcp_st: None,
            v_icms_deson: None,
            mot_des_icms: None,
            ind_deduz_deson: None,
            v_icms_st_deson: None,
            mot_des_icms_st: None,
        });
        let mut t = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut t).unwrap();
        expect_xml_contains(&xml, &["<ICMS70>"]);
        expect_xml_not_contains(&xml, &["<vICMSSTDeson>"]);
    }

    #[test]
    fn test_icms90_full_cst_90_with_deferral_fcp_deferral_st_desoneration() {
        let v = IcmsVariant::from(IcmsCst::Cst90 {
            orig: "0".into(),
            mod_bc: Some("3".into()),
            v_bc: Some(Cents(10000)),
            p_red_bc: Some(Rate(1000)),
            c_benef_rbc: Some("SP999999".into()),
            p_icms: Some(Rate(1800)),
            v_icms_op: Some(Cents(1620)),
            p_dif: Some(Rate(3333)),
            v_icms_dif: Some(Cents(540)),
            v_icms: Some(Cents(1080)),
            v_bc_fcp: Some(Cents(10000)),
            p_fcp: Some(Rate(200)),
            v_fcp: Some(Cents(200)),
            p_fcp_dif: Some(Rate(3333)),
            v_fcp_dif: Some(Cents(67)),
            v_fcp_efet: Some(Cents(133)),
            mod_bc_st: Some("4".into()),
            p_mva_st: Some(Rate(4000)),
            p_red_bc_st: Some(Rate(0)),
            v_bc_st: Some(Cents(14000)),
            p_icms_st: Some(Rate(1800)),
            v_icms_st: Some(Cents(720)),
            v_bc_fcp_st: Some(Cents(14000)),
            p_fcp_st: Some(Rate(200)),
            v_fcp_st: Some(Cents(280)),
            v_icms_deson: Some(Cents(180)),
            mot_des_icms: Some("9".into()),
            ind_deduz_deson: Some("1".into()),
            v_icms_st_deson: Some(Cents(100)),
            mot_des_icms_st: Some("3".into()),
        });
        let mut t = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut t).unwrap();
        expect_xml_contains(
            &xml,
            &[
                "<ICMS90>",
                "<cBenefRBC>",
                "<vICMSOp>",
                "<pDif>33.3300</pDif>",
                "<vICMSDif>5.40</vICMSDif>",
                "<pFCPDif>33.3300</pFCPDif>",
                "<vFCPDif>0.67</vFCPDif>",
                "<vFCPEfet>1.33</vFCPEfet>",
                "<vICMSSTDeson>",
                "<motDesICMSST>",
            ],
        );
    }

    #[test]
    fn test_icms90_minimal_cst_90_minimal() {
        let v = IcmsVariant::from(IcmsCst::Cst90 {
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
        });
        let mut t = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut t).unwrap();
        expect_xml_contains(&xml, &["<ICMS90>"]);
        expect_xml_not_contains(&xml, &["<cBenefRBC>"]);
    }
}

// =============================================================================
// TaxCoverageTest — ICMSPart
// =============================================================================

mod icms_part {
    use super::*;

    #[test]
    fn test_icms_part_partition_between_states() {
        let (xml, _totals) = build_icms_part_xml(
            &IcmsPartData::new(
                "0",
                "10",
                "3",
                Cents(10000),
                Rate(1800),
                Cents(1800),
                "4",
                Cents(14000),
                Rate(1800),
                Cents(720),
                Rate(10000),
                "SP",
            )
            .p_red_bc(Rate(0))
            .p_mva_st(Rate(4000))
            .p_red_bc_st(Rate(0))
            .v_bc_fcp_st(Cents(14000))
            .p_fcp_st(Rate(200))
            .v_fcp_st(Cents(280))
            .v_icms_deson(Cents(100))
            .mot_des_icms("9")
            .ind_deduz_deson("1"),
        )
        .unwrap();
        expect_xml_contains(&xml, &["<ICMSPart>", "<UFST>", "<vICMSDeson>"]);
    }
}

// =============================================================================
// TaxCoverageTest — ICMSST
// =============================================================================

mod icms_st {
    use super::*;

    #[test]
    fn test_icms_st_repasse_with_effective_values() {
        let (xml, _totals) = build_icms_st_xml(
            &IcmsStData::new(
                "0",
                "41",
                Cents(10000),
                Cents(800),
                Cents(8000),
                Cents(1440),
            )
            .p_st(Rate(1800))
            .v_icms_substituto(Cents(1000))
            .v_bc_fcp_st_ret(Cents(10000))
            .p_fcp_st_ret(Rate(200))
            .v_fcp_st_ret(Cents(200))
            .p_red_bc_efet(Rate(1000))
            .v_bc_efet(Cents(9000))
            .p_icms_efet(Rate(1800))
            .v_icms_efet(Cents(1620)),
        )
        .unwrap();
        expect_xml_contains(&xml, &["<ICMSST>", "<vICMSSubstituto>", "<vICMSEfet>"]);
    }
}

// =============================================================================
// TaxCoverageTest — ICMSSN (Simples Nacional) CSOSN
// =============================================================================

mod icmssn_csosn {
    use super::*;

    #[test]
    fn test_icmssn101_csosn_101() {
        let v = IcmsVariant::from(IcmsCsosn::Csosn101 {
            orig: "0".into(),
            csosn: "101".into(),
            p_cred_sn: Rate(200),
            v_cred_icms_sn: Cents(200),
        });
        let mut t = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut t).unwrap();
        expect_xml_contains(&xml, &["<ICMSSN101>", "<pCredSN>2.0000</pCredSN>"]);
    }

    #[test]
    fn test_icmssn102_csosn_102() {
        let v = IcmsVariant::from(IcmsCsosn::Csosn102 {
            orig: "0".into(),
            csosn: "102".into(),
        });
        let mut t = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut t).unwrap();
        expect_xml_contains(&xml, &["<ICMSSN102>"]);
    }

    #[test]
    fn test_icmssn103_csosn_103_uses_icmssn102_wrapper() {
        let v = IcmsVariant::from(IcmsCsosn::Csosn102 {
            orig: "0".into(),
            csosn: "103".into(),
        });
        let mut t = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut t).unwrap();
        expect_xml_contains(&xml, &["<ICMSSN102>", "<CSOSN>103</CSOSN>"]);
    }

    #[test]
    fn test_icmssn300_csosn_300_uses_icmssn102_wrapper() {
        let v = IcmsVariant::from(IcmsCsosn::Csosn102 {
            orig: "0".into(),
            csosn: "300".into(),
        });
        let mut t = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut t).unwrap();
        expect_xml_contains(&xml, &["<ICMSSN102>", "<CSOSN>300</CSOSN>"]);
    }

    #[test]
    fn test_icmssn400_csosn_400_uses_icmssn102_wrapper() {
        let v = IcmsVariant::from(IcmsCsosn::Csosn102 {
            orig: "0".into(),
            csosn: "400".into(),
        });
        let mut t = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut t).unwrap();
        expect_xml_contains(&xml, &["<ICMSSN102>", "<CSOSN>400</CSOSN>"]);
    }

    #[test]
    fn test_icmssn201_full_csosn_201_with_fcp_st() {
        let v = IcmsVariant::from(IcmsCsosn::Csosn201 {
            orig: "0".into(),
            csosn: "201".into(),
            mod_bc_st: "4".into(),
            p_mva_st: Some(Rate(4000)),
            p_red_bc_st: Some(Rate(0)),
            v_bc_st: Cents(14000),
            p_icms_st: Rate(1800),
            v_icms_st: Cents(720),
            v_bc_fcp_st: Some(Cents(14000)),
            p_fcp_st: Some(Rate(200)),
            v_fcp_st: Some(Cents(280)),
            p_cred_sn: Some(Rate(200)),
            v_cred_icms_sn: Some(Cents(200)),
        });
        let mut t = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut t).unwrap();
        expect_xml_contains(
            &xml,
            &[
                "<ICMSSN201>",
                "<vBCFCPST>",
                "<pFCPST>",
                "<vFCPST>",
                "<pCredSN>",
                "<vCredICMSSN>",
            ],
        );
    }

    #[test]
    fn test_icmssn201_minimal_csosn_201_minimal() {
        let v = IcmsVariant::from(IcmsCsosn::Csosn201 {
            orig: "0".into(),
            csosn: "201".into(),
            mod_bc_st: "4".into(),
            p_mva_st: None,
            p_red_bc_st: None,
            v_bc_st: Cents(14000),
            p_icms_st: Rate(1800),
            v_icms_st: Cents(720),
            v_bc_fcp_st: None,
            p_fcp_st: None,
            v_fcp_st: None,
            p_cred_sn: None,
            v_cred_icms_sn: None,
        });
        let mut t = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut t).unwrap();
        expect_xml_contains(&xml, &["<ICMSSN201>"]);
        expect_xml_not_contains(&xml, &["<vBCFCPST>"]);
    }

    #[test]
    fn test_icmssn202_csosn_202_with_fcp_st() {
        let v = IcmsVariant::from(IcmsCsosn::Csosn202 {
            orig: "0".into(),
            csosn: "202".into(),
            mod_bc_st: "4".into(),
            p_mva_st: Some(Rate(4000)),
            p_red_bc_st: Some(Rate(0)),
            v_bc_st: Cents(14000),
            p_icms_st: Rate(1800),
            v_icms_st: Cents(720),
            v_bc_fcp_st: Some(Cents(14000)),
            p_fcp_st: Some(Rate(200)),
            v_fcp_st: Some(Cents(280)),
        });
        let mut t = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut t).unwrap();
        expect_xml_contains(&xml, &["<ICMSSN202>", "<vFCPST>"]);
    }

    #[test]
    fn test_icmssn203_csosn_203_uses_icmssn202_wrapper() {
        let v = IcmsVariant::from(IcmsCsosn::Csosn202 {
            orig: "0".into(),
            csosn: "203".into(),
            mod_bc_st: "4".into(),
            p_mva_st: None,
            p_red_bc_st: None,
            v_bc_st: Cents(14000),
            p_icms_st: Rate(1800),
            v_icms_st: Cents(720),
            v_bc_fcp_st: None,
            p_fcp_st: None,
            v_fcp_st: None,
        });
        let mut t = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut t).unwrap();
        expect_xml_contains(&xml, &["<ICMSSN202>", "<CSOSN>203</CSOSN>"]);
    }

    #[test]
    fn test_icmssn500_full_csosn_500_with_all_effective_values() {
        let v = IcmsVariant::from(IcmsCsosn::Csosn500 {
            orig: "0".into(),
            csosn: "500".into(),
            v_bc_st_ret: Some(Cents(10000)),
            p_st: Some(Rate(1800)),
            v_icms_substituto: Some(Cents(1000)),
            v_icms_st_ret: Some(Cents(800)),
            v_bc_fcp_st_ret: Some(Cents(10000)),
            p_fcp_st_ret: Some(Rate(200)),
            v_fcp_st_ret: Some(Cents(200)),
            p_red_bc_efet: Some(Rate(1000)),
            v_bc_efet: Some(Cents(9000)),
            p_icms_efet: Some(Rate(1800)),
            v_icms_efet: Some(Cents(1620)),
        });
        let mut t = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut t).unwrap();
        expect_xml_contains(
            &xml,
            &[
                "<ICMSSN500>",
                "<vICMSSubstituto>",
                "<vBCFCPSTRet>",
                "<pRedBCEfet>",
                "<vICMSEfet>",
            ],
        );
    }

    #[test]
    fn test_icmssn500_minimal_csosn_500_minimal() {
        let v = IcmsVariant::from(IcmsCsosn::Csosn500 {
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
        let mut t = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut t).unwrap();
        expect_xml_contains(&xml, &["<ICMSSN500>"]);
        expect_xml_not_contains(&xml, &["<vICMSSubstituto>"]);
    }

    #[test]
    fn test_icmssn900_full_csosn_900_full() {
        let v = IcmsVariant::from(IcmsCsosn::Csosn900 {
            orig: "0".into(),
            csosn: "900".into(),
            mod_bc: Some("3".into()),
            v_bc: Some(Cents(10000)),
            p_red_bc: Some(Rate(1000)),
            p_icms: Some(Rate(1800)),
            v_icms: Some(Cents(1620)),
            mod_bc_st: Some("4".into()),
            p_mva_st: Some(Rate(4000)),
            p_red_bc_st: Some(Rate(0)),
            v_bc_st: Some(Cents(14000)),
            p_icms_st: Some(Rate(1800)),
            v_icms_st: Some(Cents(720)),
            v_bc_fcp_st: Some(Cents(14000)),
            p_fcp_st: Some(Rate(200)),
            v_fcp_st: Some(Cents(280)),
            p_cred_sn: Some(Rate(200)),
            v_cred_icms_sn: Some(Cents(200)),
        });
        let mut t = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut t).unwrap();
        expect_xml_contains(
            &xml,
            &[
                "<ICMSSN900>",
                "<modBC>",
                "<pRedBC>",
                "<vBCFCPST>",
                "<pCredSN>",
            ],
        );
    }

    #[test]
    fn test_icmssn900_minimal_csosn_900_minimal() {
        let v = IcmsVariant::from(IcmsCsosn::Csosn900 {
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
        });
        let mut t = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut t).unwrap();
        expect_xml_contains(&xml, &["<ICMSSN900>"]);
        expect_xml_not_contains(&xml, &["<modBC>"]);
    }
}

// =============================================================================
// TaxCoverageTest — ICMSUFDest
// =============================================================================

mod icms_uf_dest {
    use super::*;

    #[test]
    fn test_icms_uf_dest_interstate_destination() {
        let (xml, _totals) = build_icms_uf_dest_xml(
            &IcmsUfDestData::new(Cents(10000), Rate(1800), Rate(1200), Cents(600))
                .v_bc_fcp_uf_dest(Cents(10000))
                .p_fcp_uf_dest(Rate(200))
                .v_fcp_uf_dest(Cents(200))
                .v_icms_uf_remet(Cents(0)),
        )
        .unwrap();
        expect_xml_contains(&xml, &["<ICMSUFDest>", "<vBCUFDest>"]);
    }
}

// =============================================================================
// TaxCoverageTest — ICMS Totals
// =============================================================================

mod icms_totals {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_icms00_totals_cst_00_accumulates_vbc_and_vicms() {
        let v = IcmsVariant::from(IcmsCst::Cst00 {
            orig: "0".into(),
            mod_bc: "3".into(),
            v_bc: Cents(10000),
            p_icms: Rate(1800),
            v_icms: Cents(1800),
            p_fcp: None,
            v_fcp: None,
        });
        let mut totals = IcmsTotals::default();
        let _xml = build_icms_xml(&v, &mut totals).unwrap();
        assert_eq!(totals.v_bc, Cents(10000));
        assert_eq!(totals.v_icms, Cents(1800));
    }

    #[test]
    fn test_icms10_totals_cst_10_accumulates_st_totals() {
        let v = IcmsVariant::from(IcmsCst::Cst10 {
            orig: "0".into(),
            mod_bc: "3".into(),
            v_bc: Cents(10000),
            p_icms: Rate(1800),
            v_icms: Cents(1800),
            v_bc_fcp: None,
            p_fcp: None,
            v_fcp: Some(Cents(200)),
            mod_bc_st: "4".into(),
            p_mva_st: None,
            p_red_bc_st: None,
            v_bc_st: Cents(14000),
            p_icms_st: Rate(1800),
            v_icms_st: Cents(720),
            v_bc_fcp_st: None,
            p_fcp_st: None,
            v_fcp_st: Some(Cents(280)),
            v_icms_st_deson: None,
            mot_des_icms_st: None,
        });
        let mut totals = IcmsTotals::default();
        let _xml = build_icms_xml(&v, &mut totals).unwrap();
        assert_eq!(totals.v_bc_st, Cents(14000));
        assert_eq!(totals.v_st, Cents(720));
        assert_eq!(totals.v_fcp_st, Cents(280));
    }

    #[test]
    fn test_icms20_totals_cst_20_accumulates_vicms_deson() {
        let v = IcmsVariant::from(IcmsCst::Cst20 {
            orig: "0".into(),
            mod_bc: "3".into(),
            p_red_bc: Rate(1000),
            v_bc: Cents(9000),
            p_icms: Rate(1800),
            v_icms: Cents(1620),
            v_bc_fcp: None,
            p_fcp: None,
            v_fcp: None,
            v_icms_deson: Some(Cents(180)),
            mot_des_icms: Some("9".into()),
            ind_deduz_deson: None,
        });
        let mut totals = IcmsTotals::default();
        let _xml = build_icms_xml(&v, &mut totals).unwrap();
        assert_eq!(totals.v_icms_deson, Cents(180));
    }

    #[test]
    fn test_icms02_totals_cst_02_accumulates_mono_totals() {
        let v = IcmsVariant::from(IcmsCst::Cst02 {
            orig: "0".into(),
            q_bc_mono: Some(1000000),
            ad_rem_icms: Rate(15000),
            v_icms_mono: Cents(15000),
        });
        let mut totals = IcmsTotals::default();
        let _xml = build_icms_xml(&v, &mut totals).unwrap();
        assert_eq!(totals.q_bc_mono, 1000000);
        assert_eq!(totals.v_icms_mono, Cents(15000));
    }

    #[test]
    fn test_icms61_totals_cst_61_accumulates_mono_retained_totals() {
        let v = IcmsVariant::from(IcmsCst::Cst61 {
            orig: "0".into(),
            q_bc_mono_ret: Some(1000000),
            ad_rem_icms_ret: Rate(15000),
            v_icms_mono_ret: Cents(15000),
        });
        let mut totals = IcmsTotals::default();
        let _xml = build_icms_xml(&v, &mut totals).unwrap();
        assert_eq!(totals.q_bc_mono_ret, 1000000);
        assert_eq!(totals.v_icms_mono_ret, Cents(15000));
    }

    #[test]
    fn test_create_icms_totals_returns_zeroed() {
        let t = create_icms_totals();
        assert_eq!(t.v_bc, Cents(0));
        assert_eq!(t.v_icms, Cents(0));
        assert_eq!(t.v_icms_deson, Cents(0));
        assert_eq!(t.v_bc_st, Cents(0));
        assert_eq!(t.v_st, Cents(0));
        assert_eq!(t.v_fcp, Cents(0));
        assert_eq!(t.v_fcp_st, Cents(0));
        assert_eq!(t.v_fcp_st_ret, Cents(0));
        assert_eq!(t.v_fcp_uf_dest, Cents(0));
        assert_eq!(t.v_icms_uf_dest, Cents(0));
        assert_eq!(t.v_icms_uf_remet, Cents(0));
        assert_eq!(t.q_bc_mono, 0);
        assert_eq!(t.v_icms_mono, Cents(0));
        assert_eq!(t.q_bc_mono_reten, 0);
        assert_eq!(t.v_icms_mono_reten, Cents(0));
        assert_eq!(t.q_bc_mono_ret, 0);
        assert_eq!(t.v_icms_mono_ret, Cents(0));
    }

    #[test]
    fn test_merge_icms_totals_accumulates_correctly() {
        let mut target = create_icms_totals();

        let mut source1 = create_icms_totals();
        source1.v_bc = Cents(5000);
        source1.v_icms = Cents(900);
        source1.v_fcp = Cents(100);

        let mut source2 = create_icms_totals();
        source2.v_bc = Cents(3000);
        source2.v_icms = Cents(540);
        source2.v_st = Cents(200);
        source2.v_bc_st = Cents(4000);
        source2.v_fcp_st = Cents(80);

        merge_icms_totals(&mut target, &source1);
        merge_icms_totals(&mut target, &source2);

        assert_eq!(target.v_bc, Cents(8000));
        assert_eq!(target.v_icms, Cents(1440));
        assert_eq!(target.v_st, Cents(200));
        assert_eq!(target.v_bc_st, Cents(4000));
        assert_eq!(target.v_fcp, Cents(100));
        assert_eq!(target.v_fcp_st, Cents(80));
    }
}

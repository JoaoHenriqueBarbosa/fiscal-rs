use fiscal::tax_pis_cofins_ipi::{
    build_cofins_st_xml, build_cofins_xml, build_ii_xml, build_ipi_xml, build_pis_st_xml,
    build_pis_xml, CofinsData, CofinsStData, IiData, IpiData, PisData, PisStData,
};
use rstest::rstest;
use fiscal::newtypes::{Cents, Rate, Rate4};

// ── buildPisXml ─────────────────────────────────────────────────────────────

mod build_pis_xml_tests {
    use super::*;

    #[test]
    fn cst_01_pis_aliq_percentage_based() {
        // pPIS=16500 means 1.6500%, vBC=10000 cents = R$100, vPIS=165 cents = R$1.65
        let xml = build_pis_xml(&PisData {
            cst: "01".into(),
            v_bc: Some(Cents(10000)),
            p_pis: Some(Rate4(16500)),
            v_pis: Some(Cents(165)),
            ..Default::default()
        });
        assert!(xml.contains("<PISAliq>"));
        assert!(xml.contains("<CST>01</CST>"));
        assert!(xml.contains("<vBC>100.00</vBC>"));
        assert!(xml.contains("<pPIS>1.6500</pPIS>"));
        assert!(xml.contains("<vPIS>1.65</vPIS>"));
    }

    #[test]
    fn cst_02_pis_aliq() {
        let xml = build_pis_xml(&PisData {
            cst: "02".into(),
            v_bc: Some(Cents(5000)),
            p_pis: Some(Rate4(16500)),
            v_pis: Some(Cents(83)),
            ..Default::default()
        });
        assert!(xml.contains("<PISAliq>"));
        assert!(xml.contains("<CST>02</CST>"));
    }

    #[test]
    fn cst_03_pis_qtde_quantity_based() {
        let xml = build_pis_xml(&PisData {
            cst: "03".into(),
            q_bc_prod: Some(10000),
            v_aliq_prod: Some(500000),
            v_pis: Some(Cents(500)),
            ..Default::default()
        });
        assert!(xml.contains("<PISQtde>"));
        assert!(xml.contains("<qBCProd>"));
        assert!(xml.contains("<vAliqProd>"));
    }

    /// CST 04 through 09 all produce PISNT (not taxed) with no value fields.
    #[rstest]
    #[case("04")]
    #[case("05")]
    #[case("06")]
    #[case("07")]
    #[case("08")]
    #[case("09")]
    fn cst_04_to_09_pis_nt(#[case] cst: &str) {
        let xml = build_pis_xml(&PisData {
            cst: cst.into(),
            ..Default::default()
        });
        assert!(xml.contains("<PISNT>"), "CST {cst} should produce <PISNT>");
        assert!(
            xml.contains(&format!("<CST>{cst}</CST>")),
            "CST {cst} should appear in XML"
        );
        assert!(!xml.contains("<vBC>"), "CST {cst} should not have <vBC>");
    }

    #[test]
    fn cst_49_pis_outr_percentage() {
        let xml = build_pis_xml(&PisData {
            cst: "49".into(),
            v_bc: Some(Cents(10000)),
            p_pis: Some(Rate4(16500)),
            v_pis: Some(Cents(165)),
            ..Default::default()
        });
        assert!(xml.contains("<PISOutr>"));
        assert!(xml.contains("<vBC>"));
        assert!(xml.contains("<pPIS>"));
    }

    #[test]
    fn cst_99_pis_outr_percentage() {
        let xml = build_pis_xml(&PisData {
            cst: "99".into(),
            v_bc: Some(Cents(0)),
            p_pis: Some(Rate4(0)),
            v_pis: Some(Cents(0)),
            ..Default::default()
        });
        assert!(xml.contains("<PISOutr>"));
    }

    #[test]
    fn cst_99_pis_outr_quantity_based() {
        let xml = build_pis_xml(&PisData {
            cst: "99".into(),
            q_bc_prod: Some(5000),
            v_aliq_prod: Some(100),
            v_pis: Some(Cents(500)),
            ..Default::default()
        });
        assert!(xml.contains("<PISOutr>"));
        assert!(xml.contains("<qBCProd>"));
    }
}

// ── buildPisStXml ───────────────────────────────────────────────────────────

mod build_pis_st_xml_tests {
    use super::*;

    #[test]
    fn builds_pisst_with_percentage() {
        let xml = build_pis_st_xml(&PisStData {
            v_bc: Some(Cents(10000)),
            p_pis: Some(Rate4(16500)),
            v_pis: Cents(165),
            ..Default::default()
        });
        assert!(xml.contains("<PISST>"));
        assert!(xml.contains("<vBC>"));
    }
}

// ── buildCofinsXml ──────────────────────────────────────────────────────────

mod build_cofins_xml_tests {
    use super::*;

    #[test]
    fn cst_01_cofins_aliq() {
        // pCOFINS=76000 means 7.6000%
        let xml = build_cofins_xml(&CofinsData {
            cst: "01".into(),
            v_bc: Some(Cents(10000)),
            p_cofins: Some(Rate4(76000)),
            v_cofins: Some(Cents(760)),
            ..Default::default()
        });
        assert!(xml.contains("<COFINSAliq>"));
        assert!(xml.contains("<CST>01</CST>"));
        assert!(xml.contains("<pCOFINS>7.6000</pCOFINS>"));
    }

    #[test]
    fn cst_03_cofins_qtde() {
        let xml = build_cofins_xml(&CofinsData {
            cst: "03".into(),
            q_bc_prod: Some(10000),
            v_aliq_prod: Some(500000),
            v_cofins: Some(Cents(500)),
            ..Default::default()
        });
        assert!(xml.contains("<COFINSQtde>"));
    }

    #[test]
    fn cst_04_cofins_nt() {
        let xml = build_cofins_xml(&CofinsData {
            cst: "04".into(),
            ..Default::default()
        });
        assert!(xml.contains("<COFINSNT>"));
    }

    #[test]
    fn cst_99_cofins_outr() {
        let xml = build_cofins_xml(&CofinsData {
            cst: "99".into(),
            v_bc: Some(Cents(0)),
            p_cofins: Some(Rate4(0)),
            v_cofins: Some(Cents(0)),
            ..Default::default()
        });
        assert!(xml.contains("<COFINSOutr>"));
    }
}

// ── buildCofinsStXml ────────────────────────────────────────────────────────

mod build_cofins_st_xml_tests {
    use super::*;

    #[test]
    fn builds_cofinsst() {
        let xml = build_cofins_st_xml(&CofinsStData {
            v_bc: Some(Cents(10000)),
            p_cofins: Some(Rate4(76000)),
            v_cofins: Cents(760),
            ..Default::default()
        });
        assert!(xml.contains("<COFINSST>"));
    }
}

// ── buildIpiXml ─────────────────────────────────────────────────────────────

mod build_ipi_xml_tests {
    use super::*;

    #[test]
    fn cst_50_ipi_trib_percentage_based() {
        // pIPI=50000 means 5.0000%
        let xml = build_ipi_xml(&IpiData {
            cst: "50".into(),
            c_enq: "999".into(),
            v_bc: Some(Cents(10000)),
            p_ipi: Some(Rate(50000)),
            v_ipi: Some(Cents(500)),
            ..Default::default()
        });
        assert!(xml.contains("<IPI>"));
        assert!(xml.contains("<IPITrib>"));
        assert!(xml.contains("<CST>50</CST>"));
        assert!(xml.contains("<cEnq>999</cEnq>"));
        assert!(xml.contains("<vBC>100.00</vBC>"));
        assert!(xml.contains("<pIPI>5.0000</pIPI>"));
        assert!(xml.contains("<vIPI>5.00</vIPI>"));
    }

    #[test]
    fn cst_00_ipi_trib() {
        let xml = build_ipi_xml(&IpiData {
            cst: "00".into(),
            c_enq: "999".into(),
            v_bc: Some(Cents(5000)),
            p_ipi: Some(Rate(100000)),
            v_ipi: Some(Cents(500)),
            ..Default::default()
        });
        assert!(xml.contains("<IPITrib>"));
    }

    #[test]
    fn cst_99_ipi_trib_quantity_based() {
        let xml = build_ipi_xml(&IpiData {
            cst: "99".into(),
            c_enq: "999".into(),
            q_unid: Some(10000),
            v_unid: Some(500000),
            v_ipi: Some(Cents(500)),
            ..Default::default()
        });
        assert!(xml.contains("<IPITrib>"));
        assert!(xml.contains("<qUnid>"));
        assert!(xml.contains("<vUnid>"));
    }

    /// IPINT (not taxed) CST variants: 01-05 and 51-55 all produce the IPINT tag.
    #[rstest]
    #[case("01")]
    #[case("02")]
    #[case("03")]
    #[case("04")]
    #[case("05")]
    #[case("51")]
    #[case("52")]
    #[case("53")]
    #[case("54")]
    #[case("55")]
    fn ipint_cst_variants(#[case] cst: &str) {
        let xml = build_ipi_xml(&IpiData {
            cst: cst.into(),
            c_enq: "999".into(),
            ..Default::default()
        });
        assert!(xml.contains("<IPINT>"), "CST {cst} should produce <IPINT>");
        assert!(
            xml.contains(&format!("<CST>{cst}</CST>")),
            "CST {cst} should appear in XML"
        );
        assert!(!xml.contains("<IPITrib>"), "CST {cst} should not produce <IPITrib>");
    }

    #[test]
    fn includes_optional_fields() {
        let xml = build_ipi_xml(&IpiData {
            cst: "50".into(),
            c_enq: "999".into(),
            cnpj_prod: Some("12345678000199".into()),
            c_selo: Some("ABC".into()),
            q_selo: Some(10),
            v_bc: Some(Cents(10000)),
            p_ipi: Some(Rate(50000)),
            v_ipi: Some(Cents(500)),
            ..Default::default()
        });
        assert!(xml.contains("<CNPJProd>12345678000199</CNPJProd>"));
        assert!(xml.contains("<cSelo>ABC</cSelo>"));
        assert!(xml.contains("<qSelo>10</qSelo>"));
    }
}

// ── buildIiXml ──────────────────────────────────────────────────────────────

mod build_ii_xml_tests {
    use super::*;

    #[test]
    fn builds_import_tax() {
        let xml = build_ii_xml(&IiData {
            v_bc: Cents(50000),
            v_desp_adu: Cents(5000),
            v_ii: Cents(7500),
            v_iof: Cents(0),
        });
        assert!(xml.contains("<II>"));
        assert!(xml.contains("<vBC>500.00</vBC>"));
        assert!(xml.contains("<vDespAdu>50.00</vDespAdu>"));
        assert!(xml.contains("<vII>75.00</vII>"));
        assert!(xml.contains("<vIOF>0.00</vIOF>"));
    }
}

use fiscal::newtypes::{Cents, Rate, Rate4};
use fiscal::tax_pis_cofins_ipi::{
    CofinsData, CofinsStData, IiData, IpiData, PisData, PisStData, build_cofins_st_xml,
    build_cofins_xml, build_ii_xml, build_ipi_xml, build_pis_st_xml, build_pis_xml,
};
use rstest::rstest;

// ── buildPisXml ─────────────────────────────────────────────────────────────

mod build_pis_xml_tests {
    use super::*;

    #[test]
    fn cst_01_pis_aliq_percentage_based() {
        // pPIS=16500 means 1.6500%, vBC=10000 cents = R$100, vPIS=165 cents = R$1.65
        let xml = build_pis_xml(
            &PisData::new("01")
                .v_bc(Cents(10000))
                .p_pis(Rate4(16500))
                .v_pis(Cents(165)),
        );
        assert!(xml.contains("<PISAliq>"));
        assert!(xml.contains("<CST>01</CST>"));
        assert!(xml.contains("<vBC>100.00</vBC>"));
        assert!(xml.contains("<pPIS>1.6500</pPIS>"));
        assert!(xml.contains("<vPIS>1.65</vPIS>"));
    }

    #[test]
    fn cst_02_pis_aliq() {
        let xml = build_pis_xml(
            &PisData::new("02")
                .v_bc(Cents(5000))
                .p_pis(Rate4(16500))
                .v_pis(Cents(83)),
        );
        assert!(xml.contains("<PISAliq>"));
        assert!(xml.contains("<CST>02</CST>"));
    }

    #[test]
    fn cst_03_pis_qtde_quantity_based() {
        let xml = build_pis_xml(
            &PisData::new("03")
                .q_bc_prod(10000)
                .v_aliq_prod(500000)
                .v_pis(Cents(500)),
        );
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
        let xml = build_pis_xml(&PisData::new(cst));
        assert!(xml.contains("<PISNT>"), "CST {cst} should produce <PISNT>");
        assert!(
            xml.contains(&format!("<CST>{cst}</CST>")),
            "CST {cst} should appear in XML"
        );
        assert!(!xml.contains("<vBC>"), "CST {cst} should not have <vBC>");
    }

    #[test]
    fn cst_49_pis_outr_percentage() {
        let xml = build_pis_xml(
            &PisData::new("49")
                .v_bc(Cents(10000))
                .p_pis(Rate4(16500))
                .v_pis(Cents(165)),
        );
        assert!(xml.contains("<PISOutr>"));
        assert!(xml.contains("<vBC>"));
        assert!(xml.contains("<pPIS>"));
    }

    #[test]
    fn cst_99_pis_outr_percentage() {
        let xml = build_pis_xml(
            &PisData::new("99")
                .v_bc(Cents(0))
                .p_pis(Rate4(0))
                .v_pis(Cents(0)),
        );
        assert!(xml.contains("<PISOutr>"));
    }

    #[test]
    fn cst_99_pis_outr_quantity_based() {
        let xml = build_pis_xml(
            &PisData::new("99")
                .q_bc_prod(5000)
                .v_aliq_prod(100)
                .v_pis(Cents(500)),
        );
        assert!(xml.contains("<PISOutr>"));
        assert!(xml.contains("<qBCProd>"));
    }
}

// ── buildPisStXml ───────────────────────────────────────────────────────────

mod build_pis_st_xml_tests {
    use super::*;

    #[test]
    fn builds_pisst_with_percentage() {
        let xml = build_pis_st_xml(
            &PisStData::new(Cents(165))
                .v_bc(Cents(10000))
                .p_pis(Rate4(16500)),
        );
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
        let xml = build_cofins_xml(
            &CofinsData::new("01")
                .v_bc(Cents(10000))
                .p_cofins(Rate4(76000))
                .v_cofins(Cents(760)),
        );
        assert!(xml.contains("<COFINSAliq>"));
        assert!(xml.contains("<CST>01</CST>"));
        assert!(xml.contains("<pCOFINS>7.6000</pCOFINS>"));
    }

    #[test]
    fn cst_03_cofins_qtde() {
        let xml = build_cofins_xml(
            &CofinsData::new("03")
                .q_bc_prod(10000)
                .v_aliq_prod(500000)
                .v_cofins(Cents(500)),
        );
        assert!(xml.contains("<COFINSQtde>"));
    }

    #[test]
    fn cst_04_cofins_nt() {
        let xml = build_cofins_xml(&CofinsData::new("04"));
        assert!(xml.contains("<COFINSNT>"));
    }

    #[test]
    fn cst_99_cofins_outr() {
        let xml = build_cofins_xml(
            &CofinsData::new("99")
                .v_bc(Cents(0))
                .p_cofins(Rate4(0))
                .v_cofins(Cents(0)),
        );
        assert!(xml.contains("<COFINSOutr>"));
    }
}

// ── buildCofinsStXml ────────────────────────────────────────────────────────

mod build_cofins_st_xml_tests {
    use super::*;

    #[test]
    fn builds_cofinsst() {
        let xml = build_cofins_st_xml(
            &CofinsStData::new(Cents(760))
                .v_bc(Cents(10000))
                .p_cofins(Rate4(76000)),
        );
        assert!(xml.contains("<COFINSST>"));
    }
}

// ── buildIpiXml ─────────────────────────────────────────────────────────────

mod build_ipi_xml_tests {
    use super::*;

    #[test]
    fn cst_50_ipi_trib_percentage_based() {
        // pIPI=50000 means 5.0000%
        let xml = build_ipi_xml(
            &IpiData::new("50", "999")
                .v_bc(Cents(10000))
                .p_ipi(Rate(50000))
                .v_ipi(Cents(500)),
        );
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
        let xml = build_ipi_xml(
            &IpiData::new("00", "999")
                .v_bc(Cents(5000))
                .p_ipi(Rate(100000))
                .v_ipi(Cents(500)),
        );
        assert!(xml.contains("<IPITrib>"));
    }

    #[test]
    fn cst_99_ipi_trib_quantity_based() {
        let xml = build_ipi_xml(
            &IpiData::new("99", "999")
                .q_unid(10000)
                .v_unid(500000)
                .v_ipi(Cents(500)),
        );
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
        let xml = build_ipi_xml(&IpiData::new(cst, "999"));
        assert!(xml.contains("<IPINT>"), "CST {cst} should produce <IPINT>");
        assert!(
            xml.contains(&format!("<CST>{cst}</CST>")),
            "CST {cst} should appear in XML"
        );
        assert!(
            !xml.contains("<IPITrib>"),
            "CST {cst} should not produce <IPITrib>"
        );
    }

    #[test]
    fn includes_optional_fields() {
        let xml = build_ipi_xml(
            &IpiData::new("50", "999")
                .cnpj_prod("12345678000199")
                .c_selo("ABC")
                .q_selo(10)
                .v_bc(Cents(10000))
                .p_ipi(Rate(50000))
                .v_ipi(Cents(500)),
        );
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
        let xml = build_ii_xml(&IiData::new(
            Cents(50000),
            Cents(5000),
            Cents(7500),
            Cents(0),
        ));
        assert!(xml.contains("<II>"));
        assert!(xml.contains("<vBC>500.00</vBC>"));
        assert!(xml.contains("<vDespAdu>50.00</vDespAdu>"));
        assert!(xml.contains("<vII>75.00</vII>"));
        assert!(xml.contains("<vIOF>0.00</vIOF>"));
    }
}

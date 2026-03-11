// Ported from TypeScript tax-coverage-ported.test.ts — SECOND HALF
//
// Covers: PIS, PISST, COFINS, COFINSST, PIS/COFINS totalizer,
//         II, ISSQN, IS (IBSCBS), Cana, Compra, Exporta.
//
// Each TypeScript describe()/it() block becomes a Rust mod/test.
// All tests compile but will fail at runtime (implementations use todo!()).

use fiscal::tax_pis_cofins_ipi::{
    build_cofins_st_xml, build_cofins_xml, build_ii_xml, build_pis_st_xml, build_pis_xml,
    CofinsData, CofinsStData, IiData, PisData, PisStData,
};
use fiscal::tax_issqn::{
    build_issqn_xml, build_issqn_xml_with_totals, create_issqn_totals, IssqnData,
};
use fiscal::tax_is::{build_is_xml, IsData};
use fiscal::xml_utils::{tag, TagContent};
use fiscal::newtypes::{Cents, Rate4};

// =============================================================================
// TaxCoverageTest.php -- PIS Tests
// =============================================================================

mod pis {
    use super::*;

    #[test]
    fn test_pis_aliq_cst_01() {
        let xml = build_pis_xml(&PisData {
            cst: "01".into(),
            v_bc: Some(Cents(10000)),
            p_pis: Some(Rate4(16500)),
            v_pis: Some(Cents(165)),
            ..Default::default()
        });
        assert!(xml.contains("<PISAliq>"));
        assert!(xml.contains("<CST>01</CST>"));
        assert!(xml.contains("<vBC>"));
        assert!(xml.contains("<pPIS>"));
        assert!(xml.contains("<vPIS>"));
    }

    #[test]
    fn test_pis_aliq_cst_02() {
        let xml = build_pis_xml(&PisData {
            cst: "02".into(),
            v_bc: Some(Cents(10000)),
            p_pis: Some(Rate4(16500)),
            v_pis: Some(Cents(165)),
            ..Default::default()
        });
        assert!(xml.contains("<PISAliq>"));
        assert!(xml.contains("<CST>02</CST>"));
    }

    #[test]
    fn test_pis_qtde_cst_03() {
        let xml = build_pis_xml(&PisData {
            cst: "03".into(),
            q_bc_prod: Some(1000000),
            v_aliq_prod: Some(165),
            v_pis: Some(Cents(165)),
            ..Default::default()
        });
        assert!(xml.contains("<PISQtde>"));
        assert!(xml.contains("<qBCProd>"));
        assert!(xml.contains("<vAliqProd>"));
    }

    #[test]
    fn test_pis_nt_cst_04() {
        let xml = build_pis_xml(&PisData {
            cst: "04".into(),
            v_pis: Some(Cents(0)),
            ..Default::default()
        });
        assert!(xml.contains("<PISNT>"));
        assert!(xml.contains("<CST>04</CST>"));
    }

    #[test]
    fn test_pis_nt_cst_05() {
        let xml = build_pis_xml(&PisData {
            cst: "05".into(),
            v_pis: Some(Cents(0)),
            ..Default::default()
        });
        assert!(xml.contains("<PISNT>"));
        assert!(xml.contains("<CST>05</CST>"));
    }

    #[test]
    fn test_pis_nt_cst_06() {
        let xml = build_pis_xml(&PisData {
            cst: "06".into(),
            v_pis: Some(Cents(0)),
            ..Default::default()
        });
        assert!(xml.contains("<PISNT>"));
        assert!(xml.contains("<CST>06</CST>"));
    }

    #[test]
    fn test_pis_nt_cst_07() {
        let xml = build_pis_xml(&PisData {
            cst: "07".into(),
            v_pis: Some(Cents(0)),
            ..Default::default()
        });
        assert!(xml.contains("<PISNT>"));
        assert!(xml.contains("<CST>07</CST>"));
    }

    #[test]
    fn test_pis_nt_cst_08() {
        let xml = build_pis_xml(&PisData {
            cst: "08".into(),
            v_pis: Some(Cents(0)),
            ..Default::default()
        });
        assert!(xml.contains("<PISNT>"));
        assert!(xml.contains("<CST>08</CST>"));
    }

    #[test]
    fn test_pis_nt_cst_09() {
        let xml = build_pis_xml(&PisData {
            cst: "09".into(),
            v_pis: Some(Cents(0)),
            ..Default::default()
        });
        assert!(xml.contains("<PISNT>"));
        assert!(xml.contains("<CST>09</CST>"));
    }

    #[test]
    fn test_pis_outr_with_vbc_cst_49() {
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
        assert!(!xml.contains("<qBCProd>"));
    }

    #[test]
    fn test_pis_outr_with_qbc_prod_cst_99() {
        let xml = build_pis_xml(&PisData {
            cst: "99".into(),
            q_bc_prod: Some(1000000),
            v_aliq_prod: Some(165),
            v_pis: Some(Cents(165)),
            ..Default::default()
        });
        assert!(xml.contains("<PISOutr>"));
        assert!(xml.contains("<qBCProd>"));
        assert!(xml.contains("<vAliqProd>"));
        assert!(!xml.contains("<vBC>"));
    }

    #[test]
    fn test_pis_outr_cst_50() {
        let xml = build_pis_xml(&PisData {
            cst: "50".into(),
            v_bc: Some(Cents(10000)),
            p_pis: Some(Rate4(16500)),
            v_pis: Some(Cents(165)),
            ..Default::default()
        });
        assert!(xml.contains("<PISOutr>"));
        assert!(xml.contains("<CST>50</CST>"));
    }

    #[test]
    fn test_pis_outr_cst_51() {
        let xml = build_pis_xml(&PisData {
            cst: "51".into(),
            v_bc: Some(Cents(10000)),
            p_pis: Some(Rate4(16500)),
            v_pis: Some(Cents(165)),
            ..Default::default()
        });
        assert!(xml.contains("<PISOutr>"));
        assert!(xml.contains("<CST>51</CST>"));
    }

    #[test]
    fn test_pis_outr_cst_52() {
        let xml = build_pis_xml(&PisData {
            cst: "52".into(),
            v_bc: Some(Cents(10000)),
            p_pis: Some(Rate4(16500)),
            v_pis: Some(Cents(165)),
            ..Default::default()
        });
        assert!(xml.contains("<PISOutr>"));
        assert!(xml.contains("<CST>52</CST>"));
    }

    #[test]
    fn test_pis_outr_cst_53() {
        let xml = build_pis_xml(&PisData {
            cst: "53".into(),
            v_bc: Some(Cents(10000)),
            p_pis: Some(Rate4(16500)),
            v_pis: Some(Cents(165)),
            ..Default::default()
        });
        assert!(xml.contains("<PISOutr>"));
        assert!(xml.contains("<CST>53</CST>"));
    }

    #[test]
    fn test_pis_outr_cst_54() {
        let xml = build_pis_xml(&PisData {
            cst: "54".into(),
            v_bc: Some(Cents(10000)),
            p_pis: Some(Rate4(16500)),
            v_pis: Some(Cents(165)),
            ..Default::default()
        });
        assert!(xml.contains("<PISOutr>"));
        assert!(xml.contains("<CST>54</CST>"));
    }

    #[test]
    fn test_pis_outr_cst_55() {
        let xml = build_pis_xml(&PisData {
            cst: "55".into(),
            v_bc: Some(Cents(10000)),
            p_pis: Some(Rate4(16500)),
            v_pis: Some(Cents(165)),
            ..Default::default()
        });
        assert!(xml.contains("<PISOutr>"));
        assert!(xml.contains("<CST>55</CST>"));
    }

    #[test]
    fn test_pis_outr_cst_56() {
        let xml = build_pis_xml(&PisData {
            cst: "56".into(),
            v_bc: Some(Cents(10000)),
            p_pis: Some(Rate4(16500)),
            v_pis: Some(Cents(165)),
            ..Default::default()
        });
        assert!(xml.contains("<PISOutr>"));
        assert!(xml.contains("<CST>56</CST>"));
    }

    #[test]
    fn test_pis_outr_cst_60() {
        let xml = build_pis_xml(&PisData {
            cst: "60".into(),
            v_bc: Some(Cents(10000)),
            p_pis: Some(Rate4(16500)),
            v_pis: Some(Cents(165)),
            ..Default::default()
        });
        assert!(xml.contains("<PISOutr>"));
        assert!(xml.contains("<CST>60</CST>"));
    }

    #[test]
    fn test_pis_outr_cst_61() {
        let xml = build_pis_xml(&PisData {
            cst: "61".into(),
            v_bc: Some(Cents(10000)),
            p_pis: Some(Rate4(16500)),
            v_pis: Some(Cents(165)),
            ..Default::default()
        });
        assert!(xml.contains("<PISOutr>"));
        assert!(xml.contains("<CST>61</CST>"));
    }

    #[test]
    fn test_pis_outr_cst_62() {
        let xml = build_pis_xml(&PisData {
            cst: "62".into(),
            v_bc: Some(Cents(10000)),
            p_pis: Some(Rate4(16500)),
            v_pis: Some(Cents(165)),
            ..Default::default()
        });
        assert!(xml.contains("<PISOutr>"));
        assert!(xml.contains("<CST>62</CST>"));
    }

    #[test]
    fn test_pis_outr_cst_63() {
        let xml = build_pis_xml(&PisData {
            cst: "63".into(),
            v_bc: Some(Cents(10000)),
            p_pis: Some(Rate4(16500)),
            v_pis: Some(Cents(165)),
            ..Default::default()
        });
        assert!(xml.contains("<PISOutr>"));
        assert!(xml.contains("<CST>63</CST>"));
    }

    #[test]
    fn test_pis_outr_cst_64() {
        let xml = build_pis_xml(&PisData {
            cst: "64".into(),
            v_bc: Some(Cents(10000)),
            p_pis: Some(Rate4(16500)),
            v_pis: Some(Cents(165)),
            ..Default::default()
        });
        assert!(xml.contains("<PISOutr>"));
        assert!(xml.contains("<CST>64</CST>"));
    }

    #[test]
    fn test_pis_outr_cst_65() {
        let xml = build_pis_xml(&PisData {
            cst: "65".into(),
            v_bc: Some(Cents(10000)),
            p_pis: Some(Rate4(16500)),
            v_pis: Some(Cents(165)),
            ..Default::default()
        });
        assert!(xml.contains("<PISOutr>"));
        assert!(xml.contains("<CST>65</CST>"));
    }

    #[test]
    fn test_pis_outr_cst_66() {
        let xml = build_pis_xml(&PisData {
            cst: "66".into(),
            v_bc: Some(Cents(10000)),
            p_pis: Some(Rate4(16500)),
            v_pis: Some(Cents(165)),
            ..Default::default()
        });
        assert!(xml.contains("<PISOutr>"));
        assert!(xml.contains("<CST>66</CST>"));
    }

    #[test]
    fn test_pis_outr_cst_67() {
        let xml = build_pis_xml(&PisData {
            cst: "67".into(),
            v_bc: Some(Cents(10000)),
            p_pis: Some(Rate4(16500)),
            v_pis: Some(Cents(165)),
            ..Default::default()
        });
        assert!(xml.contains("<PISOutr>"));
        assert!(xml.contains("<CST>67</CST>"));
    }

    #[test]
    fn test_pis_outr_cst_70() {
        let xml = build_pis_xml(&PisData {
            cst: "70".into(),
            v_bc: Some(Cents(10000)),
            p_pis: Some(Rate4(16500)),
            v_pis: Some(Cents(165)),
            ..Default::default()
        });
        assert!(xml.contains("<PISOutr>"));
        assert!(xml.contains("<CST>70</CST>"));
    }

    #[test]
    fn test_pis_outr_cst_71() {
        let xml = build_pis_xml(&PisData {
            cst: "71".into(),
            v_bc: Some(Cents(10000)),
            p_pis: Some(Rate4(16500)),
            v_pis: Some(Cents(165)),
            ..Default::default()
        });
        assert!(xml.contains("<PISOutr>"));
        assert!(xml.contains("<CST>71</CST>"));
    }

    #[test]
    fn test_pis_outr_cst_72() {
        let xml = build_pis_xml(&PisData {
            cst: "72".into(),
            v_bc: Some(Cents(10000)),
            p_pis: Some(Rate4(16500)),
            v_pis: Some(Cents(165)),
            ..Default::default()
        });
        assert!(xml.contains("<PISOutr>"));
        assert!(xml.contains("<CST>72</CST>"));
    }

    #[test]
    fn test_pis_outr_cst_73() {
        let xml = build_pis_xml(&PisData {
            cst: "73".into(),
            v_bc: Some(Cents(10000)),
            p_pis: Some(Rate4(16500)),
            v_pis: Some(Cents(165)),
            ..Default::default()
        });
        assert!(xml.contains("<PISOutr>"));
        assert!(xml.contains("<CST>73</CST>"));
    }

    #[test]
    fn test_pis_outr_cst_74() {
        let xml = build_pis_xml(&PisData {
            cst: "74".into(),
            v_bc: Some(Cents(10000)),
            p_pis: Some(Rate4(16500)),
            v_pis: Some(Cents(165)),
            ..Default::default()
        });
        assert!(xml.contains("<PISOutr>"));
        assert!(xml.contains("<CST>74</CST>"));
    }

    #[test]
    fn test_pis_outr_cst_75() {
        let xml = build_pis_xml(&PisData {
            cst: "75".into(),
            v_bc: Some(Cents(10000)),
            p_pis: Some(Rate4(16500)),
            v_pis: Some(Cents(165)),
            ..Default::default()
        });
        assert!(xml.contains("<PISOutr>"));
        assert!(xml.contains("<CST>75</CST>"));
    }

    #[test]
    fn test_pis_outr_cst_98() {
        let xml = build_pis_xml(&PisData {
            cst: "98".into(),
            v_bc: Some(Cents(10000)),
            p_pis: Some(Rate4(16500)),
            v_pis: Some(Cents(165)),
            ..Default::default()
        });
        assert!(xml.contains("<PISOutr>"));
        assert!(xml.contains("<CST>98</CST>"));
    }

    #[test]
    fn test_pis_aliq_with_empty_vpis() {
        // PIS Aliq with null vPIS
        let xml = build_pis_xml(&PisData {
            cst: "01".into(),
            v_bc: Some(Cents(10000)),
            p_pis: Some(Rate4(16500)),
            v_pis: None,
            ..Default::default()
        });
        assert!(xml.contains("<PISAliq>"));
    }

    #[test]
    fn test_pis_outr_with_null_vpis() {
        // PIS Outr with null vPIS
        let xml = build_pis_xml(&PisData {
            cst: "99".into(),
            v_bc: Some(Cents(10000)),
            p_pis: Some(Rate4(16500)),
            v_pis: None,
            ..Default::default()
        });
        assert!(xml.contains("<PISOutr>"));
    }
}

// =============================================================================
// TaxCoverageTest.php -- PISST
// =============================================================================

mod pisst {
    use super::*;

    #[test]
    fn test_pisst_with_vbc() {
        let xml = build_pis_st_xml(&PisStData {
            v_bc: Some(Cents(10000)),
            p_pis: Some(Rate4(16500)),
            v_pis: Cents(165),
            ind_soma_pis_st: Some(1),
            ..Default::default()
        });
        assert!(xml.contains("<PISST>"));
        assert!(xml.contains("<vBC>"));
        assert!(xml.contains("<pPIS>"));
        assert!(xml.contains("<indSomaPISST>"));
    }

    #[test]
    fn test_pisst_with_qbc_prod() {
        let xml = build_pis_st_xml(&PisStData {
            q_bc_prod: Some(1000000),
            v_aliq_prod: Some(165),
            v_pis: Cents(165),
            ind_soma_pis_st: Some(0),
            ..Default::default()
        });
        assert!(xml.contains("<PISST>"));
        assert!(xml.contains("<qBCProd>"));
        assert!(xml.contains("<vAliqProd>"));
        assert!(!xml.contains("<vBC>"));
    }
}

// =============================================================================
// TaxCoverageTest.php -- COFINS Tests
// =============================================================================

mod cofins {
    use super::*;

    #[test]
    fn test_cofins_aliq_cst_01() {
        let xml = build_cofins_xml(&CofinsData {
            cst: "01".into(),
            v_bc: Some(Cents(10000)),
            p_cofins: Some(Rate4(76000)),
            v_cofins: Some(Cents(760)),
            ..Default::default()
        });
        assert!(xml.contains("<COFINSAliq>"));
        assert!(xml.contains("<CST>01</CST>"));
        assert!(xml.contains("<vBC>"));
        assert!(xml.contains("<pCOFINS>"));
        assert!(xml.contains("<vCOFINS>"));
    }

    #[test]
    fn test_cofins_aliq_cst_02() {
        let xml = build_cofins_xml(&CofinsData {
            cst: "02".into(),
            v_bc: Some(Cents(10000)),
            p_cofins: Some(Rate4(76000)),
            v_cofins: Some(Cents(760)),
            ..Default::default()
        });
        assert!(xml.contains("<COFINSAliq>"));
        assert!(xml.contains("<CST>02</CST>"));
    }

    #[test]
    fn test_cofins_qtde_cst_03() {
        let xml = build_cofins_xml(&CofinsData {
            cst: "03".into(),
            q_bc_prod: Some(1000000),
            v_aliq_prod: Some(760),
            v_cofins: Some(Cents(760)),
            ..Default::default()
        });
        assert!(xml.contains("<COFINSQtde>"));
        assert!(xml.contains("<qBCProd>"));
        assert!(xml.contains("<vAliqProd>"));
    }

    #[test]
    fn test_cofins_nt_cst_04() {
        let xml = build_cofins_xml(&CofinsData {
            cst: "04".into(),
            v_cofins: Some(Cents(0)),
            ..Default::default()
        });
        assert!(xml.contains("<COFINSNT>"));
        assert!(xml.contains("<CST>04</CST>"));
    }

    #[test]
    fn test_cofins_nt_cst_05() {
        let xml = build_cofins_xml(&CofinsData {
            cst: "05".into(),
            v_cofins: Some(Cents(0)),
            ..Default::default()
        });
        assert!(xml.contains("<COFINSNT>"));
        assert!(xml.contains("<CST>05</CST>"));
    }

    #[test]
    fn test_cofins_nt_cst_06() {
        let xml = build_cofins_xml(&CofinsData {
            cst: "06".into(),
            v_cofins: Some(Cents(0)),
            ..Default::default()
        });
        assert!(xml.contains("<COFINSNT>"));
        assert!(xml.contains("<CST>06</CST>"));
    }

    #[test]
    fn test_cofins_nt_cst_07() {
        let xml = build_cofins_xml(&CofinsData {
            cst: "07".into(),
            v_cofins: Some(Cents(0)),
            ..Default::default()
        });
        assert!(xml.contains("<COFINSNT>"));
        assert!(xml.contains("<CST>07</CST>"));
    }

    #[test]
    fn test_cofins_nt_cst_08() {
        let xml = build_cofins_xml(&CofinsData {
            cst: "08".into(),
            v_cofins: Some(Cents(0)),
            ..Default::default()
        });
        assert!(xml.contains("<COFINSNT>"));
        assert!(xml.contains("<CST>08</CST>"));
    }

    #[test]
    fn test_cofins_nt_cst_09() {
        let xml = build_cofins_xml(&CofinsData {
            cst: "09".into(),
            v_cofins: Some(Cents(0)),
            ..Default::default()
        });
        assert!(xml.contains("<COFINSNT>"));
        assert!(xml.contains("<CST>09</CST>"));
    }

    #[test]
    fn test_cofins_outr_with_vbc_cst_49() {
        let xml = build_cofins_xml(&CofinsData {
            cst: "49".into(),
            v_bc: Some(Cents(10000)),
            p_cofins: Some(Rate4(76000)),
            v_cofins: Some(Cents(760)),
            ..Default::default()
        });
        assert!(xml.contains("<COFINSOutr>"));
        assert!(xml.contains("<vBC>"));
        assert!(xml.contains("<pCOFINS>"));
        assert!(!xml.contains("<qBCProd>"));
    }

    #[test]
    fn test_cofins_outr_with_qbc_prod_cst_99() {
        let xml = build_cofins_xml(&CofinsData {
            cst: "99".into(),
            q_bc_prod: Some(1000000),
            v_aliq_prod: Some(760),
            v_cofins: Some(Cents(760)),
            ..Default::default()
        });
        assert!(xml.contains("<COFINSOutr>"));
        assert!(xml.contains("<qBCProd>"));
        assert!(xml.contains("<vAliqProd>"));
        assert!(!xml.contains("<vBC>"));
    }

    #[test]
    fn test_cofins_outr_cst_50() {
        let xml = build_cofins_xml(&CofinsData {
            cst: "50".into(),
            v_bc: Some(Cents(10000)),
            p_cofins: Some(Rate4(76000)),
            v_cofins: Some(Cents(760)),
            ..Default::default()
        });
        assert!(xml.contains("<COFINSOutr>"));
        assert!(xml.contains("<CST>50</CST>"));
    }

    #[test]
    fn test_cofins_outr_cst_51() {
        let xml = build_cofins_xml(&CofinsData {
            cst: "51".into(),
            v_bc: Some(Cents(10000)),
            p_cofins: Some(Rate4(76000)),
            v_cofins: Some(Cents(760)),
            ..Default::default()
        });
        assert!(xml.contains("<COFINSOutr>"));
        assert!(xml.contains("<CST>51</CST>"));
    }

    #[test]
    fn test_cofins_outr_cst_52() {
        let xml = build_cofins_xml(&CofinsData {
            cst: "52".into(),
            v_bc: Some(Cents(10000)),
            p_cofins: Some(Rate4(76000)),
            v_cofins: Some(Cents(760)),
            ..Default::default()
        });
        assert!(xml.contains("<COFINSOutr>"));
        assert!(xml.contains("<CST>52</CST>"));
    }

    #[test]
    fn test_cofins_outr_cst_53() {
        let xml = build_cofins_xml(&CofinsData {
            cst: "53".into(),
            v_bc: Some(Cents(10000)),
            p_cofins: Some(Rate4(76000)),
            v_cofins: Some(Cents(760)),
            ..Default::default()
        });
        assert!(xml.contains("<COFINSOutr>"));
        assert!(xml.contains("<CST>53</CST>"));
    }

    #[test]
    fn test_cofins_outr_cst_54() {
        let xml = build_cofins_xml(&CofinsData {
            cst: "54".into(),
            v_bc: Some(Cents(10000)),
            p_cofins: Some(Rate4(76000)),
            v_cofins: Some(Cents(760)),
            ..Default::default()
        });
        assert!(xml.contains("<COFINSOutr>"));
        assert!(xml.contains("<CST>54</CST>"));
    }

    #[test]
    fn test_cofins_outr_cst_55() {
        let xml = build_cofins_xml(&CofinsData {
            cst: "55".into(),
            v_bc: Some(Cents(10000)),
            p_cofins: Some(Rate4(76000)),
            v_cofins: Some(Cents(760)),
            ..Default::default()
        });
        assert!(xml.contains("<COFINSOutr>"));
        assert!(xml.contains("<CST>55</CST>"));
    }

    #[test]
    fn test_cofins_outr_cst_56() {
        let xml = build_cofins_xml(&CofinsData {
            cst: "56".into(),
            v_bc: Some(Cents(10000)),
            p_cofins: Some(Rate4(76000)),
            v_cofins: Some(Cents(760)),
            ..Default::default()
        });
        assert!(xml.contains("<COFINSOutr>"));
        assert!(xml.contains("<CST>56</CST>"));
    }

    #[test]
    fn test_cofins_outr_cst_60() {
        let xml = build_cofins_xml(&CofinsData {
            cst: "60".into(),
            v_bc: Some(Cents(10000)),
            p_cofins: Some(Rate4(76000)),
            v_cofins: Some(Cents(760)),
            ..Default::default()
        });
        assert!(xml.contains("<COFINSOutr>"));
        assert!(xml.contains("<CST>60</CST>"));
    }

    #[test]
    fn test_cofins_outr_cst_61() {
        let xml = build_cofins_xml(&CofinsData {
            cst: "61".into(),
            v_bc: Some(Cents(10000)),
            p_cofins: Some(Rate4(76000)),
            v_cofins: Some(Cents(760)),
            ..Default::default()
        });
        assert!(xml.contains("<COFINSOutr>"));
        assert!(xml.contains("<CST>61</CST>"));
    }

    #[test]
    fn test_cofins_outr_cst_62() {
        let xml = build_cofins_xml(&CofinsData {
            cst: "62".into(),
            v_bc: Some(Cents(10000)),
            p_cofins: Some(Rate4(76000)),
            v_cofins: Some(Cents(760)),
            ..Default::default()
        });
        assert!(xml.contains("<COFINSOutr>"));
        assert!(xml.contains("<CST>62</CST>"));
    }

    #[test]
    fn test_cofins_outr_cst_63() {
        let xml = build_cofins_xml(&CofinsData {
            cst: "63".into(),
            v_bc: Some(Cents(10000)),
            p_cofins: Some(Rate4(76000)),
            v_cofins: Some(Cents(760)),
            ..Default::default()
        });
        assert!(xml.contains("<COFINSOutr>"));
        assert!(xml.contains("<CST>63</CST>"));
    }

    #[test]
    fn test_cofins_outr_cst_64() {
        let xml = build_cofins_xml(&CofinsData {
            cst: "64".into(),
            v_bc: Some(Cents(10000)),
            p_cofins: Some(Rate4(76000)),
            v_cofins: Some(Cents(760)),
            ..Default::default()
        });
        assert!(xml.contains("<COFINSOutr>"));
        assert!(xml.contains("<CST>64</CST>"));
    }

    #[test]
    fn test_cofins_outr_cst_65() {
        let xml = build_cofins_xml(&CofinsData {
            cst: "65".into(),
            v_bc: Some(Cents(10000)),
            p_cofins: Some(Rate4(76000)),
            v_cofins: Some(Cents(760)),
            ..Default::default()
        });
        assert!(xml.contains("<COFINSOutr>"));
        assert!(xml.contains("<CST>65</CST>"));
    }

    #[test]
    fn test_cofins_outr_cst_66() {
        let xml = build_cofins_xml(&CofinsData {
            cst: "66".into(),
            v_bc: Some(Cents(10000)),
            p_cofins: Some(Rate4(76000)),
            v_cofins: Some(Cents(760)),
            ..Default::default()
        });
        assert!(xml.contains("<COFINSOutr>"));
        assert!(xml.contains("<CST>66</CST>"));
    }

    #[test]
    fn test_cofins_outr_cst_67() {
        let xml = build_cofins_xml(&CofinsData {
            cst: "67".into(),
            v_bc: Some(Cents(10000)),
            p_cofins: Some(Rate4(76000)),
            v_cofins: Some(Cents(760)),
            ..Default::default()
        });
        assert!(xml.contains("<COFINSOutr>"));
        assert!(xml.contains("<CST>67</CST>"));
    }

    #[test]
    fn test_cofins_outr_cst_70() {
        let xml = build_cofins_xml(&CofinsData {
            cst: "70".into(),
            v_bc: Some(Cents(10000)),
            p_cofins: Some(Rate4(76000)),
            v_cofins: Some(Cents(760)),
            ..Default::default()
        });
        assert!(xml.contains("<COFINSOutr>"));
        assert!(xml.contains("<CST>70</CST>"));
    }

    #[test]
    fn test_cofins_outr_cst_71() {
        let xml = build_cofins_xml(&CofinsData {
            cst: "71".into(),
            v_bc: Some(Cents(10000)),
            p_cofins: Some(Rate4(76000)),
            v_cofins: Some(Cents(760)),
            ..Default::default()
        });
        assert!(xml.contains("<COFINSOutr>"));
        assert!(xml.contains("<CST>71</CST>"));
    }

    #[test]
    fn test_cofins_outr_cst_72() {
        let xml = build_cofins_xml(&CofinsData {
            cst: "72".into(),
            v_bc: Some(Cents(10000)),
            p_cofins: Some(Rate4(76000)),
            v_cofins: Some(Cents(760)),
            ..Default::default()
        });
        assert!(xml.contains("<COFINSOutr>"));
        assert!(xml.contains("<CST>72</CST>"));
    }

    #[test]
    fn test_cofins_outr_cst_73() {
        let xml = build_cofins_xml(&CofinsData {
            cst: "73".into(),
            v_bc: Some(Cents(10000)),
            p_cofins: Some(Rate4(76000)),
            v_cofins: Some(Cents(760)),
            ..Default::default()
        });
        assert!(xml.contains("<COFINSOutr>"));
        assert!(xml.contains("<CST>73</CST>"));
    }

    #[test]
    fn test_cofins_outr_cst_74() {
        let xml = build_cofins_xml(&CofinsData {
            cst: "74".into(),
            v_bc: Some(Cents(10000)),
            p_cofins: Some(Rate4(76000)),
            v_cofins: Some(Cents(760)),
            ..Default::default()
        });
        assert!(xml.contains("<COFINSOutr>"));
        assert!(xml.contains("<CST>74</CST>"));
    }

    #[test]
    fn test_cofins_outr_cst_75() {
        let xml = build_cofins_xml(&CofinsData {
            cst: "75".into(),
            v_bc: Some(Cents(10000)),
            p_cofins: Some(Rate4(76000)),
            v_cofins: Some(Cents(760)),
            ..Default::default()
        });
        assert!(xml.contains("<COFINSOutr>"));
        assert!(xml.contains("<CST>75</CST>"));
    }

    #[test]
    fn test_cofins_outr_cst_98() {
        let xml = build_cofins_xml(&CofinsData {
            cst: "98".into(),
            v_bc: Some(Cents(10000)),
            p_cofins: Some(Rate4(76000)),
            v_cofins: Some(Cents(760)),
            ..Default::default()
        });
        assert!(xml.contains("<COFINSOutr>"));
        assert!(xml.contains("<CST>98</CST>"));
    }
}

// =============================================================================
// TaxCoverageTest.php -- COFINSST
// =============================================================================

mod cofinsst {
    use super::*;

    #[test]
    fn test_cofinsst_with_vbc() {
        let xml = build_cofins_st_xml(&CofinsStData {
            v_bc: Some(Cents(10000)),
            p_cofins: Some(Rate4(76000)),
            v_cofins: Cents(760),
            ind_soma_cofins_st: Some(1),
            ..Default::default()
        });
        assert!(xml.contains("<COFINSST>"));
        assert!(xml.contains("<vBC>"));
        assert!(xml.contains("<pCOFINS>"));
        assert!(xml.contains("<indSomaCOFINSST>"));
    }

    #[test]
    fn test_cofinsst_with_qbc_prod() {
        let xml = build_cofins_st_xml(&CofinsStData {
            q_bc_prod: Some(1000000),
            v_aliq_prod: Some(760),
            v_cofins: Cents(760),
            ind_soma_cofins_st: Some(0),
            ..Default::default()
        });
        assert!(xml.contains("<COFINSST>"));
        assert!(xml.contains("<qBCProd>"));
        assert!(xml.contains("<vAliqProd>"));
        assert!(!xml.contains("<vBC>"));
    }
}

// =============================================================================
// TaxCoverageTest.php -- PIS/COFINS totalizer branch coverage
// =============================================================================

mod pis_cofins_totalizer {
    use super::*;

    #[test]
    fn test_cofins_aliq_totalizer() {
        // COFINS Aliq CST 01 generates correct XML
        let xml = build_cofins_xml(&CofinsData {
            cst: "01".into(),
            v_bc: Some(Cents(10000)),
            p_cofins: Some(Rate4(76000)),
            v_cofins: Some(Cents(760)),
            ..Default::default()
        });
        assert!(xml.contains("<COFINSAliq>"));
        assert!(xml.contains("<vCOFINS>"));
    }

    #[test]
    fn test_cofins_qtde_totalizer() {
        // COFINS Qtde CST 03 generates correct XML
        let xml = build_cofins_xml(&CofinsData {
            cst: "03".into(),
            q_bc_prod: Some(1000000),
            v_aliq_prod: Some(760),
            v_cofins: Some(Cents(760)),
            ..Default::default()
        });
        assert!(xml.contains("<COFINSQtde>"));
        assert!(xml.contains("<vCOFINS>"));
    }

    #[test]
    fn test_cofins_outr_totalizer() {
        // COFINS Outr CST 99 generates correct XML
        let xml = build_cofins_xml(&CofinsData {
            cst: "99".into(),
            v_bc: Some(Cents(10000)),
            p_cofins: Some(Rate4(76000)),
            v_cofins: Some(Cents(760)),
            ..Default::default()
        });
        assert!(xml.contains("<COFINSOutr>"));
        assert!(xml.contains("<vCOFINS>"));
    }

    #[test]
    fn test_pisst_with_ind_soma_1() {
        // PISST with indSomaPISST=1 includes indicator
        let xml = build_pis_st_xml(&PisStData {
            v_bc: Some(Cents(10000)),
            p_pis: Some(Rate4(16500)),
            v_pis: Cents(165),
            ind_soma_pis_st: Some(1),
            ..Default::default()
        });
        assert!(xml.contains("<indSomaPISST>1</indSomaPISST>"));
    }

    #[test]
    fn test_pisst_with_ind_soma_0() {
        // PISST with indSomaPISST=0 includes indicator
        let xml = build_pis_st_xml(&PisStData {
            v_bc: Some(Cents(10000)),
            p_pis: Some(Rate4(16500)),
            v_pis: Cents(165),
            ind_soma_pis_st: Some(0),
            ..Default::default()
        });
        assert!(xml.contains("<indSomaPISST>0</indSomaPISST>"));
    }

    #[test]
    fn test_cofinsst_with_ind_soma_1() {
        // COFINSST with indSomaCOFINSST=1
        let xml = build_cofins_st_xml(&CofinsStData {
            v_bc: Some(Cents(10000)),
            p_cofins: Some(Rate4(76000)),
            v_cofins: Cents(760),
            ind_soma_cofins_st: Some(1),
            ..Default::default()
        });
        assert!(xml.contains("<indSomaCOFINSST>1</indSomaCOFINSST>"));
    }

    #[test]
    fn test_cofinsst_with_ind_soma_0() {
        // COFINSST with indSomaCOFINSST=0
        let xml = build_cofins_st_xml(&CofinsStData {
            v_bc: Some(Cents(10000)),
            p_cofins: Some(Rate4(76000)),
            v_cofins: Cents(760),
            ind_soma_cofins_st: Some(0),
            ..Default::default()
        });
        assert!(xml.contains("<indSomaCOFINSST>0</indSomaCOFINSST>"));
    }
}

// =============================================================================
// TraitsCoverageTest.php -- II (Imposto de Importacao)
// =============================================================================

mod ii {
    use super::*;

    #[test]
    fn test_tag_ii_all_fields() {
        let xml = build_ii_xml(&IiData {
            v_bc: Cents(100000),
            v_desp_adu: Cents(5000),
            v_ii: Cents(12000),
            v_iof: Cents(1500),
        });
        assert!(xml.contains("<II>"));
        assert!(xml.contains("<vBC>1000.00</vBC>"));
        assert!(xml.contains("<vDespAdu>50.00</vDespAdu>"));
        assert!(xml.contains("<vII>120.00</vII>"));
        assert!(xml.contains("<vIOF>15.00</vIOF>"));
    }
}

// =============================================================================
// TraitsCoverageTest.php -- ISSQN
// =============================================================================

mod issqn {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_tag_issqn_all_fields() {
        let mut totals = create_issqn_totals();
        let xml = build_issqn_xml_with_totals(
            &IssqnData {
                v_bc: 10000,       // 100.00
                v_aliq: 500,       // 5.0000
                v_issqn: 500,      // 5.00
                c_mun_fg: "3550308".into(),
                c_list_serv: "1401".into(),
                v_deducao: Some(1000),     // 10.00
                v_outro: Some(200),        // 2.00
                v_desc_incond: Some(300),  // 3.00
                v_desc_cond: Some(100),    // 1.00
                v_iss_ret: Some(50),       // 0.50
                ind_iss: Some("1".into()),
                c_servico: Some("1234".into()),
                c_mun: Some("3550308".into()),
                c_pais: Some("1058".into()),
                n_processo: Some("9999".into()),
                ind_incentivo: Some("1".into()),
            },
            &mut totals,
        );

        assert!(xml.contains("<ISSQN>"));
        assert!(xml.contains("<vBC>100.00</vBC>"));
        assert!(xml.contains("<vAliq>5.0000</vAliq>"));
        assert!(xml.contains("<vISSQN>5.00</vISSQN>"));
        assert!(xml.contains("<cMunFG>3550308</cMunFG>"));
        assert!(xml.contains("<cListServ>1401</cListServ>"));
        assert!(xml.contains("<vDeducao>10.00</vDeducao>"));
        assert!(xml.contains("<vOutro>2.00</vOutro>"));
        assert!(xml.contains("<vDescIncond>3.00</vDescIncond>"));
        assert!(xml.contains("<vDescCond>1.00</vDescCond>"));
        assert!(xml.contains("<vISSRet>0.50</vISSRet>"));
        assert!(xml.contains("<indISS>1</indISS>"));
        assert!(xml.contains("<cServico>1234</cServico>"));
        assert!(xml.contains("<cMun>3550308</cMun>"));
        assert!(xml.contains("<cPais>1058</cPais>"));
        assert!(xml.contains("<nProcesso>9999</nProcesso>"));
        assert!(xml.contains("<indIncentivo>1</indIncentivo>"));
        // Totals should be accumulated
        assert_eq!(totals.v_bc, 10000);
        assert_eq!(totals.v_iss, 500);
        assert_eq!(totals.v_iss_ret, 50);
    }

    #[test]
    fn test_tag_issqn_zero_vbc_does_not_accumulate_totals() {
        let mut totals = create_issqn_totals();
        let xml = build_issqn_xml_with_totals(
            &IssqnData {
                v_bc: 0,
                v_aliq: 500,
                v_issqn: 0,
                c_mun_fg: "3550308".into(),
                c_list_serv: "1401".into(),
                ind_iss: Some("1".into()),
                ind_incentivo: Some("2".into()),
                ..Default::default()
            },
            &mut totals,
        );

        assert!(xml.contains("<ISSQN>"));
        assert!(xml.contains("<vBC>0.00</vBC>"));
        // Totals should NOT be accumulated when vBC = 0
        assert_eq!(totals.v_bc, 0);
        assert_eq!(totals.v_iss, 0);
    }

    #[test]
    fn test_tag_issqn_optional_fields_null() {
        let xml = build_issqn_xml(&IssqnData {
            v_bc: 5000,       // 50.00
            v_aliq: 300,      // 3.0000
            v_issqn: 150,     // 1.50
            c_mun_fg: "3550308".into(),
            c_list_serv: "1401".into(),
            ind_iss: Some("2".into()),
            ind_incentivo: Some("2".into()),
            // all optional fields left unset
            ..Default::default()
        });

        assert!(xml.contains("<ISSQN>"));
        assert!(xml.contains("<vBC>50.00</vBC>"));
        assert!(!xml.contains("<cServico>"));
        assert!(!xml.contains("<vDeducao>"));
        assert!(!xml.contains("<vOutro>"));
        assert!(!xml.contains("<nProcesso>"));
    }
}

// =============================================================================
// TraitsCoverageTest.php -- IS (IBSCBS)
// =============================================================================

mod is_ibscbs {
    use super::*;

    #[test]
    fn test_tag_is_with_vbcis() {
        let xml = build_is_xml(&IsData {
            cst_is: "00".into(),
            c_class_trib_is: "001".into(),
            v_bc_is: Some("100.00".into()),
            p_is: Some("5.0000".into()),
            p_is_espec: Some("1.5000".into()),
            v_is: "5.00".into(),
            ..Default::default()
        });

        assert!(xml.contains("<IS>"));
        assert!(xml.contains("<CSTIS>00</CSTIS>"));
        assert!(xml.contains("<cClassTribIS>001</cClassTribIS>"));
        assert!(xml.contains("<vBCIS>100.00</vBCIS>"));
        assert!(xml.contains("<pIS>5.0000</pIS>"));
        assert!(xml.contains("<pISEspec>1.5000</pISEspec>"));
        assert!(xml.contains("<vIS>5.00</vIS>"));
    }

    #[test]
    fn test_tag_is_with_utrib_and_qtrib() {
        let xml = build_is_xml(&IsData {
            cst_is: "01".into(),
            c_class_trib_is: "002".into(),
            u_trib: Some("LT".into()),
            q_trib: Some("10.0000".into()),
            v_is: "8.00".into(),
            ..Default::default()
        });

        assert!(xml.contains("<IS>"));
        assert!(xml.contains("<uTrib>LT</uTrib>"));
        assert!(xml.contains("<qTrib>10.0000</qTrib>"));
    }

    #[test]
    fn test_tag_is_without_vbcis_or_utrib() {
        let xml = build_is_xml(&IsData {
            cst_is: "02".into(),
            c_class_trib_is: "003".into(),
            v_is: "3.00".into(),
            ..Default::default()
        });

        assert!(xml.contains("<IS>"));
        // vBCIS should not be present
        assert!(!xml.contains("<vBCIS>"));
        // uTrib should not be present
        assert!(!xml.contains("<uTrib>"));
    }
}

// =============================================================================
// TraitsCoverageTest.php -- Cana (sugarcane)
// =============================================================================

mod cana {
    use super::*;

    #[test]
    fn test_tagcana_and_tagfordia_and_tagdeduc() {
        let xml = tag(
            "cana",
            &[],
            TagContent::Children(vec![
                tag("safra", &[], TagContent::Text("2025/2026")),
                tag("ref", &[], TagContent::Text("01/2026")),
                tag(
                    "forDia",
                    &[("dia", "1")],
                    TagContent::Children(vec![tag(
                        "qtde",
                        &[],
                        TagContent::Text("500.0000000000"),
                    )]),
                ),
                tag(
                    "forDia",
                    &[("dia", "2")],
                    TagContent::Children(vec![tag(
                        "qtde",
                        &[],
                        TagContent::Text("600.0000000000"),
                    )]),
                ),
                tag("qTotMes", &[], TagContent::Text("10000.0000000000")),
                tag("qTotAnt", &[], TagContent::Text("5000.0000000000")),
                tag("qTotGer", &[], TagContent::Text("15000.0000000000")),
                tag(
                    "deduc",
                    &[],
                    TagContent::Children(vec![
                        tag("xDed", &[], TagContent::Text("DEDUCAO TESTE")),
                        tag("vDed", &[], TagContent::Text("500.00")),
                    ]),
                ),
                tag("vFor", &[], TagContent::Text("50000.00")),
                tag("vTotDed", &[], TagContent::Text("1000.00")),
                tag("vLiqFor", &[], TagContent::Text("49000.00")),
            ]),
        );

        assert!(xml.contains("<cana>"));
        assert!(xml.contains("<safra>2025/2026</safra>"));
        assert!(xml.contains("<forDia dia=\"1\">"));
        assert!(xml.contains("<forDia dia=\"2\">"));
        assert!(xml.contains("<xDed>DEDUCAO TESTE</xDed>"));
        assert!(xml.contains("<vDed>500.00</vDed>"));
        assert!(xml.contains("<vLiqFor>49000.00</vLiqFor>"));
    }
}

// =============================================================================
// TraitsCoverageTest.php -- Compra
// =============================================================================

mod compra {
    use super::*;

    #[test]
    fn test_tagcompra_all_fields() {
        let xml = tag(
            "compra",
            &[],
            TagContent::Children(vec![
                tag("xNEmp", &[], TagContent::Text("NE-2025-001")),
                tag("xPed", &[], TagContent::Text("PED-12345")),
                tag("xCont", &[], TagContent::Text("CONT-67890")),
            ]),
        );

        assert!(xml.contains("<compra>"));
        assert!(xml.contains("<xNEmp>NE-2025-001</xNEmp>"));
        assert!(xml.contains("<xPed>PED-12345</xPed>"));
        assert!(xml.contains("<xCont>CONT-67890</xCont>"));
    }

    #[test]
    fn test_tagcompra_optional_fields_null() {
        // builds compra with only xPed
        let xml = tag(
            "compra",
            &[],
            TagContent::Children(vec![tag(
                "xPed",
                &[],
                TagContent::Text("PED-99999"),
            )]),
        );

        assert!(xml.contains("<compra>"));
        assert!(xml.contains("<xPed>PED-99999</xPed>"));
        assert!(!xml.contains("<xNEmp>"));
        assert!(!xml.contains("<xCont>"));
    }
}

// =============================================================================
// TraitsCoverageTest.php -- Exporta
// =============================================================================

mod exporta {
    use super::*;

    #[test]
    fn test_tagexporta_all_fields() {
        let xml = tag(
            "exporta",
            &[],
            TagContent::Children(vec![
                tag("UFSaidaPais", &[], TagContent::Text("SP")),
                tag("xLocExporta", &[], TagContent::Text("Porto de Santos")),
                tag(
                    "xLocDespacho",
                    &[],
                    TagContent::Text("Aeroporto de Guarulhos"),
                ),
            ]),
        );

        assert!(xml.contains("<exporta>"));
        assert!(xml.contains("<UFSaidaPais>SP</UFSaidaPais>"));
        assert!(xml.contains("<xLocExporta>Porto de Santos</xLocExporta>"));
        assert!(xml.contains(
            "<xLocDespacho>Aeroporto de Guarulhos</xLocDespacho>"
        ));
    }

    #[test]
    fn test_tagexporta_without_xloc_despacho() {
        let xml = tag(
            "exporta",
            &[],
            TagContent::Children(vec![
                tag("UFSaidaPais", &[], TagContent::Text("RJ")),
                tag("xLocExporta", &[], TagContent::Text("Porto do Rio")),
            ]),
        );

        assert!(xml.contains("<exporta>"));
        assert!(xml.contains("<UFSaidaPais>RJ</UFSaidaPais>"));
        assert!(xml.contains("<xLocExporta>Porto do Rio</xLocExporta>"));
        assert!(!xml.contains("<xLocDespacho>"));
    }
}

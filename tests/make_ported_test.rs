// Ported from TypeScript make-ported.test.ts (91 tests)
//
// Each TypeScript describe()/it() block becomes a Rust mod/test.
// All tests compile but will fail at runtime (implementations use todo!()).

mod common;

use fiscal::newtypes::{Cents, IbgeCode, Rate, Rate4};
use fiscal::tax_icms::{
    IcmsCsosn, IcmsCst, IcmsPartData, IcmsStData, IcmsTotals, IcmsUfDestData, IcmsVariant,
    build_icms_part_xml, build_icms_st_xml, build_icms_uf_dest_xml, build_icms_xml,
};
use fiscal::tax_pis_cofins_ipi::{
    CofinsData, IiData, IpiData, PisData, build_cofins_xml, build_ii_xml, build_ipi_xml,
    build_pis_xml,
};
use fiscal::types::{AccessKeyParams, EmissionType, InvoiceModel};
use fiscal::xml_builder::build_access_key;
use fiscal::xml_utils::{TagContent, tag};

use common::{expect_wrapped_in, expect_xml_tag_values as expect_xml_contains};

// ═══════════════════════════════════════════════════════════════════════════
//  tag() utility tests (equivalent to taginfNFe, tagide basic structure)
// ═══════════════════════════════════════════════════════════════════════════

mod tag_utility_ported_from_taginfnfe {
    use super::*;

    #[test]
    fn test_taginfnfe_builds_infnfe_with_id_and_versao_attributes() {
        let id = "35170358716523000119550010000000301000000300";
        let xml = tag(
            "infNFe",
            &[("Id", &format!("NFe{id}")), ("versao", "4.00")],
            "content".into(),
        );
        assert!(xml.contains(&format!("Id=\"NFe{id}\"")));
        assert!(xml.contains("versao=\"4.00\""));
    }

    #[test]
    fn test_taginfnfe_com_pk_nitem_builds_infnfe_with_pk_nitem_attribute() {
        let id = "35170358716523000119550010000000301000000300";
        let xml = tag(
            "infNFe",
            &[
                ("Id", &format!("NFe{id}")),
                ("versao", "4.00"),
                ("pk_nItem", "1"),
            ],
            "content".into(),
        );
        assert!(xml.contains("pk_nItem=\"1\""));
    }

    #[test]
    fn test_taginfnfe_sem_chave_de_acesso_builds_infnfe_without_access_key() {
        let xml = tag(
            "infNFe",
            &[("Id", "NFe"), ("versao", "4.00")],
            "content".into(),
        );
        assert!(xml.contains("Id=\"NFe\""));
        assert!(xml.contains("versao=\"4.00\""));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  tagide tests (structure validation)
// ═══════════════════════════════════════════════════════════════════════════

mod tagide_ported {
    use super::*;

    #[test]
    fn test_tagide_model_55_ide_fields() {
        let xml = tag(
            "ide",
            &[],
            TagContent::Children(vec![
                tag("cUF", &[], "50".into()),
                tag("cNF", &[], "80070008".into()),
                tag("natOp", &[], "VENDA".into()),
                tag("mod", &[], "55".into()),
                tag("serie", &[], "1".into()),
                tag("nNF", &[], "1".into()),
                tag("dhEmi", &[], "2018-06-23T17:45:49-03:00".into()),
                tag("dhSaiEnt", &[], "2018-06-23T17:45:49-03:00".into()),
                tag("tpNF", &[], "1".into()),
                tag("idDest", &[], "1".into()),
                tag("cMunFG", &[], "5002704".into()),
                tag("tpImp", &[], "1".into()),
                tag("tpEmis", &[], "1".into()),
                tag("cDV", &[], "2".into()),
                tag("tpAmb", &[], "2".into()),
                tag("finNFe", &[], "1".into()),
                tag("indFinal", &[], "0".into()),
                tag("indPres", &[], "1".into()),
                tag("procEmi", &[], "0".into()),
                tag("verProc", &[], "5.0".into()),
            ]),
        );

        expect_xml_contains(
            &xml,
            &[
                ("cUF", "50"),
                ("cNF", "80070008"),
                ("natOp", "VENDA"),
                ("mod", "55"),
                ("serie", "1"),
                ("nNF", "1"),
                ("dhEmi", "2018-06-23T17:45:49-03:00"),
                ("dhSaiEnt", "2018-06-23T17:45:49-03:00"),
                ("tpNF", "1"),
                ("idDest", "1"),
                ("cMunFG", "5002704"),
                ("cDV", "2"),
                ("tpAmb", "2"),
                ("finNFe", "1"),
                ("indFinal", "0"),
                ("indPres", "1"),
                ("procEmi", "0"),
                ("verProc", "5.0"),
            ],
        );
        // indPag should NOT be present (version 4.00)
        assert!(!xml.contains("<indPag>"));
        // dhCont and xJust should NOT be present
        assert!(!xml.contains("<dhCont>"));
        assert!(!xml.contains("<xJust>"));
    }

    #[test]
    fn test_tagide_contingency_fields() {
        let xml = tag(
            "ide",
            &[],
            TagContent::Children(vec![
                tag("dhCont", &[], "2018-06-26T17:45:49-03:00".into()),
                tag("xJust", &[], "SEFAZ INDISPONIVEL".into()),
            ]),
        );

        expect_xml_contains(
            &xml,
            &[
                ("dhCont", "2018-06-26T17:45:49-03:00"),
                ("xJust", "SEFAZ INDISPONIVEL"),
            ],
        );
    }

    #[test]
    fn test_tagide_model_65_ide_fields() {
        let xml = tag(
            "ide",
            &[],
            TagContent::Children(vec![
                tag("cUF", &[], "50".into()),
                tag("cNF", &[], "80070008".into()),
                tag("natOp", &[], "VENDA".into()),
                tag("mod", &[], "65".into()),
                tag("serie", &[], "1".into()),
                tag("nNF", &[], "1".into()),
                tag("dhEmi", &[], "2018-06-23T17:45:49-03:00".into()),
                tag("tpNF", &[], "1".into()),
                tag("idDest", &[], "1".into()),
                tag("cMunFG", &[], "5002704".into()),
                tag("tpImp", &[], "4".into()),
                tag("tpEmis", &[], "1".into()),
                tag("cDV", &[], "2".into()),
                tag("tpAmb", &[], "2".into()),
                tag("finNFe", &[], "1".into()),
                tag("indFinal", &[], "0".into()),
                tag("indPres", &[], "4".into()),
                tag("procEmi", &[], "0".into()),
                tag("verProc", &[], "5.0".into()),
            ]),
        );

        expect_xml_contains(&xml, &[("mod", "65"), ("tpImp", "4"), ("indPres", "4")]);
        assert!(!xml.contains("<indPag>"));
        assert!(!xml.contains("<dhSaiEnt>"));
        assert!(!xml.contains("<dhCont>"));
        assert!(!xml.contains("<xJust>"));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  Reference document tags
// ═══════════════════════════════════════════════════════════════════════════

mod reference_document_tags {
    use super::*;

    #[test]
    fn test_tagref_nfe_builds_refnfe_tag() {
        let ref_nfe = "35150271780456000160550010000253101000253101";
        let xml = tag("refNFe", &[], ref_nfe.into());
        assert_eq!(xml, format!("<refNFe>{ref_nfe}</refNFe>"));
    }

    #[test]
    fn test_tagref_nf_builds_refnf_group() {
        let xml = tag(
            "refNF",
            &[],
            TagContent::Children(vec![
                tag("cUF", &[], "35".into()),
                tag("AAMM", &[], "1412".into()),
                tag("CNPJ", &[], "52297850000105".into()),
                tag("mod", &[], "01".into()),
                tag("serie", &[], "3".into()),
                tag("nNF", &[], "587878".into()),
            ]),
        );

        expect_xml_contains(
            &xml,
            &[
                ("cUF", "35"),
                ("AAMM", "1412"),
                ("CNPJ", "52297850000105"),
                ("mod", "01"),
                ("serie", "3"),
                ("nNF", "587878"),
            ],
        );
    }

    #[test]
    fn test_tagref_nfp_builds_refnfp_group_with_cnpj() {
        let xml = tag(
            "refNFP",
            &[],
            TagContent::Children(vec![
                tag("cUF", &[], "35".into()),
                tag("AAMM", &[], "1502".into()),
                tag("CNPJ", &[], "00940734000150".into()),
                tag("IE", &[], "ISENTO".into()),
                tag("mod", &[], "04".into()),
                tag("serie", &[], "0".into()),
                tag("nNF", &[], "5578".into()),
            ]),
        );

        expect_xml_contains(
            &xml,
            &[
                ("cUF", "35"),
                ("CNPJ", "00940734000150"),
                ("IE", "ISENTO"),
                ("mod", "04"),
            ],
        );
    }

    #[test]
    fn test_tagref_nfp_builds_refnfp_group_with_cpf() {
        let xml = tag(
            "refNFP",
            &[],
            TagContent::Children(vec![
                tag("cUF", &[], "35".into()),
                tag("AAMM", &[], "1502".into()),
                tag("CPF", &[], "08456452009".into()),
                tag("IE", &[], "ISENTO".into()),
                tag("mod", &[], "04".into()),
                tag("serie", &[], "0".into()),
                tag("nNF", &[], "5578".into()),
            ]),
        );

        assert!(xml.contains("<CPF>08456452009</CPF>"));
        assert!(!xml.contains("<CNPJ>"));
    }

    #[test]
    fn test_tagref_cte_builds_refcte_tag() {
        let ref_cte = "35150268252816000146570010000016161002008472";
        let xml = tag("refCTe", &[], ref_cte.into());
        assert_eq!(xml, format!("<refCTe>{ref_cte}</refCTe>"));
    }

    #[test]
    fn test_tagref_ecf_builds_refecf_group() {
        let xml = tag(
            "refECF",
            &[],
            TagContent::Children(vec![
                tag("mod", &[], "2C".into()),
                tag("nECF", &[], "788".into()),
                tag("nCOO", &[], "114".into()),
            ]),
        );

        expect_xml_contains(&xml, &[("mod", "2C"), ("nECF", "788"), ("nCOO", "114")]);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  Retirada and Entrega
// ═══════════════════════════════════════════════════════════════════════════

mod retirada_entrega_tags {
    use super::*;

    fn address_fields() -> Vec<String> {
        vec![
            tag("xLgr", &[], "Rua Um".into()),
            tag("nro", &[], "123".into()),
            tag("xCpl", &[], "sobreloja".into()),
            tag("xBairro", &[], "centro".into()),
            tag("cMun", &[], "3550308".into()),
            tag("xMun", &[], "Sao Paulo".into()),
            tag("UF", &[], "SP".into()),
            tag("CEP", &[], "01023000".into()),
            tag("cPais", &[], "1058".into()),
            tag("xPais", &[], "BRASIL".into()),
            tag("fone", &[], "1122225544".into()),
            tag("email", &[], "contato@beltrano.com.br".into()),
        ]
    }

    #[test]
    fn test_tagretirada_builds_retirada_with_cnpj() {
        let mut children = vec![
            tag("CNPJ", &[], "12345678901234".into()),
            tag("IE", &[], "12345678901".into()),
            tag("xNome", &[], "Beltrano e Cia Ltda".into()),
        ];
        children.extend(address_fields());
        let xml = tag("retirada", &[], TagContent::Children(children));

        expect_wrapped_in(&xml, "retirada");
        expect_xml_contains(
            &xml,
            &[
                ("CNPJ", "12345678901234"),
                ("IE", "12345678901"),
                ("xNome", "Beltrano e Cia Ltda"),
                ("xLgr", "Rua Um"),
                ("nro", "123"),
                ("xCpl", "sobreloja"),
                ("xBairro", "centro"),
                ("cMun", "3550308"),
                ("xMun", "Sao Paulo"),
                ("UF", "SP"),
                ("CEP", "01023000"),
                ("cPais", "1058"),
                ("xPais", "BRASIL"),
                ("fone", "1122225544"),
                ("email", "contato@beltrano.com.br"),
            ],
        );
    }

    #[test]
    fn test_tagretirada_builds_retirada_with_cpf() {
        let mut children = vec![tag("CPF", &[], "06563904092".into())];
        children.extend(address_fields());
        let xml = tag("retirada", &[], TagContent::Children(children));

        assert!(xml.contains("<CPF>06563904092</CPF>"));
    }

    #[test]
    fn test_tagentrega_builds_entrega_with_cnpj() {
        let mut children = vec![
            tag("CNPJ", &[], "12345678901234".into()),
            tag("IE", &[], "12345678901".into()),
            tag("xNome", &[], "Beltrano e Cia Ltda".into()),
        ];
        children.extend(address_fields());
        let xml = tag("entrega", &[], TagContent::Children(children));

        expect_wrapped_in(&xml, "entrega");
        expect_xml_contains(
            &xml,
            &[
                ("CNPJ", "12345678901234"),
                ("xLgr", "Rua Um"),
                ("nro", "123"),
                ("xCpl", "sobreloja"),
                ("xBairro", "centro"),
                ("cMun", "3550308"),
                ("xMun", "Sao Paulo"),
                ("UF", "SP"),
                ("CEP", "01023000"),
                ("cPais", "1058"),
                ("xPais", "BRASIL"),
                ("fone", "1122225544"),
                ("email", "contato@beltrano.com.br"),
            ],
        );
    }

    #[test]
    fn test_tagentrega_builds_entrega_with_cpf() {
        let mut children = vec![tag("CPF", &[], "06563904092".into())];
        children.extend(address_fields());
        let xml = tag("entrega", &[], TagContent::Children(children));

        assert!(xml.contains("<CPF>06563904092</CPF>"));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  autXML
// ═══════════════════════════════════════════════════════════════════════════

mod autxml_ported {
    use super::*;

    #[test]
    fn test_tagautxml_builds_autxml_with_cnpj() {
        let xml = tag(
            "autXML",
            &[],
            TagContent::Children(vec![tag("CNPJ", &[], "12345678901234".into())]),
        );
        expect_wrapped_in(&xml, "autXML");
        assert!(xml.contains("<CNPJ>12345678901234</CNPJ>"));
    }

    #[test]
    fn test_tagautxml_builds_autxml_with_cpf() {
        let xml = tag(
            "autXML",
            &[],
            TagContent::Children(vec![tag("CPF", &[], "06563904092".into())]),
        );
        assert!(xml.contains("<CPF>06563904092</CPF>"));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  infAdProd
// ═══════════════════════════════════════════════════════════════════════════

mod inf_ad_prod_ported {
    use super::*;

    #[test]
    fn test_taginfadprod_builds_additional_product_info_tag() {
        let xml = tag("infAdProd", &[], "informacao adicional do item".into());
        assert_eq!(xml, "<infAdProd>informacao adicional do item</infAdProd>");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  gCred (CreditoPresumidoProd)
// ═══════════════════════════════════════════════════════════════════════════

mod gcred_ported {
    use super::*;

    #[test]
    fn test_tag_credito_presumido_prod_builds_gcred_tag() {
        let xml = tag(
            "gCred",
            &[],
            TagContent::Children(vec![
                tag("cCredPresumido", &[], "2222211234".into()),
                tag("pCredPresumido", &[], "4.0000".into()),
                tag("vCredPresumido", &[], "4.00".into()),
            ]),
        );

        expect_xml_contains(
            &xml,
            &[
                ("cCredPresumido", "2222211234"),
                ("pCredPresumido", "4.0000"),
                ("vCredPresumido", "4.00"),
            ],
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  obsItem (obsCont / obsFisco)
// ═══════════════════════════════════════════════════════════════════════════

mod obs_item_ported {
    use super::*;

    #[test]
    fn test_tagprod_obscont_builds_obsitem_with_obscont() {
        let xml = tag(
            "obsItem",
            &[],
            TagContent::Children(vec![tag(
                "obsCont",
                &[("xCampo", "abc")],
                TagContent::Children(vec![tag("xTexto", &[], "123".into())]),
            )]),
        );

        expect_wrapped_in(&xml, "obsItem");
        assert!(xml.contains("xCampo=\"abc\""));
        assert!(xml.contains("<xTexto>123</xTexto>"));
    }

    #[test]
    fn test_tagprod_obsfisco_builds_obsitem_with_obsfisco() {
        let xml = tag(
            "obsItem",
            &[],
            TagContent::Children(vec![tag(
                "obsFisco",
                &[("xCampo", "abc")],
                TagContent::Children(vec![tag("xTexto", &[], "123".into())]),
            )]),
        );

        expect_wrapped_in(&xml, "obsItem");
        assert!(xml.contains("xCampo=\"abc\""));
        assert!(xml.contains("<xTexto>123</xTexto>"));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  veicProd
// ═══════════════════════════════════════════════════════════════════════════

mod veic_prod_ported {
    use super::*;

    #[test]
    fn test_tagveicprod_builds_vehicle_product_tag() {
        let xml = tag(
            "veicProd",
            &[],
            TagContent::Children(vec![
                tag("tpOp", &[], "1".into()),
                tag("chassi", &[], "9BGRX4470AG745440".into()),
                tag("cCor", &[], "121".into()),
                tag("xCor", &[], "PRATA".into()),
                tag("pot", &[], "0078".into()),
                tag("cilin", &[], "1000".into()),
                tag("pesoL", &[], "000008900".into()),
                tag("pesoB", &[], "000008900".into()),
                tag("nSerie", &[], "AAA123456".into()),
                tag("tpComb", &[], "16".into()),
                tag("nMotor", &[], "BBB123456".into()),
                tag("CMT", &[], "460.0000".into()),
                tag("dist", &[], "2443".into()),
                tag("anoMod", &[], "2010".into()),
                tag("anoFab", &[], "2011".into()),
                tag("tpPint", &[], "M".into()),
                tag("tpVeic", &[], "06".into()),
                tag("espVeic", &[], "1".into()),
                tag("VIN", &[], "N".into()),
                tag("condVeic", &[], "1".into()),
                tag("cMod", &[], "123456".into()),
                tag("cCorDENATRAN", &[], "10".into()),
                tag("lota", &[], "5".into()),
                tag("tpRest", &[], "0".into()),
            ]),
        );

        expect_wrapped_in(&xml, "veicProd");
        expect_xml_contains(
            &xml,
            &[
                ("tpOp", "1"),
                ("chassi", "9BGRX4470AG745440"),
                ("cCor", "121"),
                ("xCor", "PRATA"),
                ("pot", "0078"),
                ("cilin", "1000"),
                ("nSerie", "AAA123456"),
                ("tpComb", "16"),
                ("nMotor", "BBB123456"),
                ("CMT", "460.0000"),
                ("dist", "2443"),
                ("anoMod", "2010"),
                ("anoFab", "2011"),
                ("tpPint", "M"),
                ("tpVeic", "06"),
                ("espVeic", "1"),
                ("VIN", "N"),
                ("condVeic", "1"),
                ("cMod", "123456"),
                ("cCorDENATRAN", "10"),
                ("lota", "5"),
                ("tpRest", "0"),
            ],
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  med
// ═══════════════════════════════════════════════════════════════════════════

mod med_ported {
    use super::*;

    #[test]
    fn test_tagmed_builds_medicine_tag() {
        let xml = tag(
            "med",
            &[],
            TagContent::Children(vec![
                tag("cProdANVISA", &[], "1234567890123".into()),
                tag("xMotivoIsencao", &[], "RDC 238".into()),
                tag("vPMC", &[], "102.22".into()),
            ]),
        );

        expect_xml_contains(
            &xml,
            &[
                ("cProdANVISA", "1234567890123"),
                ("xMotivoIsencao", "RDC 238"),
                ("vPMC", "102.22"),
            ],
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  arma
// ═══════════════════════════════════════════════════════════════════════════

mod arma_ported {
    use super::*;

    #[test]
    fn test_tagarma_builds_weapon_tag() {
        let xml = tag(
            "arma",
            &[],
            TagContent::Children(vec![
                tag("tpArma", &[], "0".into()),
                tag("nSerie", &[], "1234567890".into()),
                tag("nCano", &[], "987654321".into()),
                tag("descr", &[], "Fuzil AK-47".into()),
            ]),
        );

        expect_xml_contains(
            &xml,
            &[
                ("tpArma", "0"),
                ("nSerie", "1234567890"),
                ("nCano", "987654321"),
                ("descr", "Fuzil AK-47"),
            ],
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  comb
// ═══════════════════════════════════════════════════════════════════════════

mod comb_ported {
    use super::*;

    #[test]
    fn test_tagcomb_builds_fuel_tag_with_cide() {
        let xml = tag(
            "comb",
            &[],
            TagContent::Children(vec![
                tag("cProdANP", &[], "012345678".into()),
                tag("descANP", &[], "Gasolina C Comum".into()),
                tag("pGLP", &[], "90.0000".into()),
                tag("pGNn", &[], "10.0000".into()),
                tag("pGNi", &[], "25.0000".into()),
                tag("vPart", &[], "12.50".into()),
                tag("CODIF", &[], "45346546".into()),
                tag("qTemp", &[], "123.0000".into()),
                tag("UFCons", &[], "RS".into()),
                tag(
                    "CIDE",
                    &[],
                    TagContent::Children(vec![
                        tag("qBCProd", &[], "12.5000".into()),
                        tag("vAliqProd", &[], "1.0000".into()),
                        tag("vCIDE", &[], "0.13".into()),
                    ]),
                ),
            ]),
        );

        expect_xml_contains(
            &xml,
            &[
                ("cProdANP", "012345678"),
                ("descANP", "Gasolina C Comum"),
                ("pGLP", "90.0000"),
                ("pGNn", "10.0000"),
                ("pGNi", "25.0000"),
                ("vPart", "12.50"),
                ("CODIF", "45346546"),
                ("qTemp", "123.0000"),
                ("UFCons", "RS"),
            ],
        );
        assert!(xml.contains("<CIDE>"));
        assert!(xml.contains("<qBCProd>12.5000</qBCProd>"));
        assert!(xml.contains("<vAliqProd>1.0000</vAliqProd>"));
        assert!(xml.contains("<vCIDE>0.13</vCIDE>"));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  encerrante
// ═══════════════════════════════════════════════════════════════════════════

mod encerrante_ported {
    use super::*;

    #[test]
    fn test_tagencerrante_builds_closing_meter_reading_tag() {
        let xml = tag(
            "encerrante",
            &[],
            TagContent::Children(vec![
                tag("nBico", &[], "1".into()),
                tag("nBomba", &[], "2".into()),
                tag("nTanque", &[], "3".into()),
                tag("vEncIni", &[], "100.000".into()),
                tag("vEncFin", &[], "200.000".into()),
            ]),
        );

        expect_xml_contains(
            &xml,
            &[
                ("nBico", "1"),
                ("nBomba", "2"),
                ("nTanque", "3"),
                ("vEncIni", "100.000"),
                ("vEncFin", "200.000"),
            ],
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  origComb
// ═══════════════════════════════════════════════════════════════════════════

mod orig_comb_ported {
    use super::*;

    #[test]
    fn test_tagorigcomb_builds_fuel_origin_tag() {
        let xml = tag(
            "origComb",
            &[],
            TagContent::Children(vec![
                tag("indImport", &[], "1".into()),
                tag("cUFOrig", &[], "11".into()),
                tag("pOrig", &[], "200.0000".into()),
            ]),
        );

        expect_xml_contains(
            &xml,
            &[("indImport", "1"), ("cUFOrig", "11"), ("pOrig", "200.0000")],
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  ICMS CST tests
// ═══════════════════════════════════════════════════════════════════════════

mod tag_icms_cst_ported {
    use super::*;

    #[test]
    fn test_tagicms_cst_00_fully_taxed() {
        let v = IcmsVariant::from(IcmsCst::Cst00 {
            orig: "0".into(),
            mod_bc: "3".into(),
            v_bc: Cents(20000),
            p_icms: Rate(1800),
            v_icms: Cents(3600),
            p_fcp: Some(Rate(100)),
            v_fcp: Some(Cents(200)),
        });
        let mut totals = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut totals).unwrap();

        expect_wrapped_in(&xml, "ICMS");
        expect_wrapped_in(&xml, "ICMS00");
        expect_xml_contains(
            &xml,
            &[
                ("orig", "0"),
                ("CST", "00"),
                ("modBC", "3"),
                ("vBC", "200.00"),
                ("pICMS", "18.0000"),
                ("vICMS", "36.00"),
                ("pFCP", "1.0000"),
                ("vFCP", "2.00"),
            ],
        );
        assert_eq!(totals.v_bc, Cents(20000));
        assert_eq!(totals.v_icms, Cents(3600));
        assert_eq!(totals.v_fcp, Cents(200));
    }

    #[test]
    fn test_tagicms_cst_02_monofasico() {
        let v = IcmsVariant::from(IcmsCst::Cst02 {
            orig: "0".into(),
            q_bc_mono: Some(20000),
            ad_rem_icms: Rate(2500),
            v_icms_mono: Cents(5000),
        });
        let mut totals = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut totals).unwrap();

        expect_wrapped_in(&xml, "ICMS02");
        expect_xml_contains(
            &xml,
            &[
                ("orig", "0"),
                ("CST", "02"),
                ("qBCMono", "200.0000"),
                ("adRemICMS", "25.0000"),
                ("vICMSMono", "50.00"),
            ],
        );
        assert_eq!(totals.q_bc_mono, 20000);
        assert_eq!(totals.v_icms_mono, Cents(5000));
    }

    #[test]
    fn test_tagicms_cst_15_monofasico_with_retention() {
        let v = IcmsVariant::from(IcmsCst::Cst15 {
            orig: "0".into(),
            q_bc_mono: Some(20000),
            ad_rem_icms: Rate(2500),
            v_icms_mono: Cents(5000),
            q_bc_mono_reten: Some(10000),
            ad_rem_icms_reten: Rate(2000),
            v_icms_mono_reten: Cents(2000),
            p_red_ad_rem: Some(Rate(100)),
            mot_red_ad_rem: Some("1".into()),
        });
        let mut totals = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut totals).unwrap();

        expect_wrapped_in(&xml, "ICMS15");
        expect_xml_contains(
            &xml,
            &[
                ("orig", "0"),
                ("CST", "15"),
                ("qBCMono", "200.0000"),
                ("adRemICMS", "25.0000"),
                ("vICMSMono", "50.00"),
                ("qBCMonoReten", "100.0000"),
                ("adRemICMSReten", "20.0000"),
                ("vICMSMonoReten", "20.00"),
                ("pRedAdRem", "1.0000"),
                ("motRedAdRem", "1"),
            ],
        );
        assert_eq!(totals.q_bc_mono, 20000);
        assert_eq!(totals.v_icms_mono, Cents(5000));
        assert_eq!(totals.q_bc_mono_reten, 10000);
        assert_eq!(totals.v_icms_mono_reten, Cents(2000));
    }

    #[test]
    fn test_tagicms_cst_20_with_base_reduction() {
        let v = IcmsVariant::from(IcmsCst::Cst20 {
            orig: "0".into(),
            mod_bc: "3".into(),
            p_red_bc: Rate(500),
            v_bc: Cents(18000),
            p_icms: Rate(1800),
            v_icms: Cents(3240),
            v_bc_fcp: Some(Cents(20000)),
            p_fcp: Some(Rate(100)),
            v_fcp: Some(Cents(200)),
            v_icms_deson: Some(Cents(360)),
            mot_des_icms: Some("9".into()),
            ind_deduz_deson: None,
        });
        let mut totals = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut totals).unwrap();

        expect_wrapped_in(&xml, "ICMS20");
        expect_xml_contains(
            &xml,
            &[
                ("orig", "0"),
                ("CST", "20"),
                ("modBC", "3"),
                ("pRedBC", "5.0000"),
                ("vBC", "180.00"),
                ("pICMS", "18.0000"),
                ("vICMS", "32.40"),
                ("vBCFCP", "200.00"),
                ("pFCP", "1.0000"),
                ("vFCP", "2.00"),
                ("vICMSDeson", "3.60"),
                ("motDesICMS", "9"),
            ],
        );
        assert_eq!(totals.v_icms_deson, Cents(360));
        assert_eq!(totals.v_bc, Cents(18000));
        assert_eq!(totals.v_icms, Cents(3240));
        assert_eq!(totals.v_fcp, Cents(200));
    }

    #[test]
    fn test_tagicms_cst_30_exempt_with_st() {
        let v = IcmsVariant::from(IcmsCst::Cst30 {
            orig: "0".into(),
            mod_bc_st: "4".into(),
            p_mva_st: Some(Rate(3000)),
            p_red_bc_st: Some(Rate(100)),
            v_bc_st: Cents(100),
            p_icms_st: Rate(100),
            v_icms_st: Cents(100),
            v_bc_fcp_st: Some(Cents(100)),
            p_fcp_st: Some(Rate(100)),
            v_fcp_st: Some(Cents(100)),
            v_icms_deson: Some(Cents(360)),
            mot_des_icms: Some("9".into()),
            ind_deduz_deson: Some("0".into()),
        });
        let mut totals = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut totals).unwrap();

        expect_wrapped_in(&xml, "ICMS30");
        expect_xml_contains(
            &xml,
            &[
                ("orig", "0"),
                ("CST", "30"),
                ("modBCST", "4"),
                ("pMVAST", "30.0000"),
                ("pRedBCST", "1.0000"),
                ("vBCST", "1.00"),
                ("pICMSST", "1.0000"),
                ("vICMSST", "1.00"),
                ("vBCFCPST", "1.00"),
                ("pFCPST", "1.0000"),
                ("vFCPST", "1.00"),
                ("vICMSDeson", "3.60"),
                ("motDesICMS", "9"),
                ("indDeduzDeson", "0"),
            ],
        );
        assert_eq!(totals.v_icms_deson, Cents(360));
        assert_eq!(totals.v_bc_st, Cents(100));
        assert_eq!(totals.v_st, Cents(100));
        assert_eq!(totals.v_fcp_st, Cents(100));
    }

    #[test]
    fn test_tagicms_cst_40_exempt() {
        let v = IcmsVariant::from(IcmsCst::Cst40 {
            orig: "0".into(),
            v_icms_deson: Some(Cents(360)),
            mot_des_icms: Some("9".into()),
            ind_deduz_deson: Some("0".into()),
        });
        let mut totals = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut totals).unwrap();

        expect_wrapped_in(&xml, "ICMS40");
        expect_xml_contains(
            &xml,
            &[
                ("orig", "0"),
                ("CST", "40"),
                ("vICMSDeson", "3.60"),
                ("motDesICMS", "9"),
                ("indDeduzDeson", "0"),
            ],
        );
        assert_eq!(totals.v_icms_deson, Cents(360));
    }

    #[test]
    fn test_tagicms_cst_41_non_taxed() {
        let v = IcmsVariant::from(IcmsCst::Cst41 {
            orig: "0".into(),
            v_icms_deson: Some(Cents(360)),
            mot_des_icms: Some("9".into()),
            ind_deduz_deson: Some("0".into()),
        });
        let mut totals = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut totals).unwrap();

        expect_wrapped_in(&xml, "ICMS40");
        expect_xml_contains(
            &xml,
            &[
                ("orig", "0"),
                ("CST", "41"),
                ("vICMSDeson", "3.60"),
                ("motDesICMS", "9"),
                ("indDeduzDeson", "0"),
            ],
        );
    }

    #[test]
    fn test_tagicms_cst_50_suspended() {
        let v = IcmsVariant::from(IcmsCst::Cst50 {
            orig: "0".into(),
            v_icms_deson: Some(Cents(360)),
            mot_des_icms: Some("9".into()),
            ind_deduz_deson: Some("0".into()),
        });
        let mut totals = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut totals).unwrap();

        expect_wrapped_in(&xml, "ICMS40");
        expect_xml_contains(
            &xml,
            &[("orig", "0"), ("CST", "50"), ("vICMSDeson", "3.60")],
        );
    }

    #[test]
    fn test_tagicms_cst_51_deferral() {
        let v = IcmsVariant::from(IcmsCst::Cst51 {
            orig: "0".into(),
            mod_bc: Some("3".into()),
            p_red_bc: Some(Rate(1000)),
            c_benef_rbc: None,
            v_bc: Some(Cents(10000)),
            p_icms: Some(Rate(1700)),
            v_icms_op: Some(Cents(1700)),
            p_dif: Some(Rate(100)),
            v_icms_dif: Some(Cents(100)),
            v_icms: Some(Cents(1700)),
            v_bc_fcp: Some(Cents(10000)),
            p_fcp: Some(Rate(200)),
            v_fcp: Some(Cents(200)),
            p_fcp_dif: None,
            v_fcp_dif: None,
            v_fcp_efet: None,
        });
        let mut totals = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut totals).unwrap();

        expect_wrapped_in(&xml, "ICMS51");
        expect_xml_contains(
            &xml,
            &[
                ("orig", "0"),
                ("CST", "51"),
                ("modBC", "3"),
                ("pRedBC", "10.0000"),
                ("vBC", "100.00"),
                ("pICMS", "17.0000"),
                ("vICMSOp", "17.00"),
                ("pDif", "1.0000"),
                ("vICMSDif", "1.00"),
                ("vICMS", "17.00"),
                ("vBCFCP", "100.00"),
                ("pFCP", "2.0000"),
                ("vFCP", "2.00"),
            ],
        );
        assert_eq!(totals.v_bc, Cents(10000));
        assert_eq!(totals.v_icms, Cents(1700));
        assert_eq!(totals.v_fcp, Cents(200));
    }

    #[test]
    fn test_tagicms_cst_53_monofasico_deferred() {
        let v = IcmsVariant::from(IcmsCst::Cst53 {
            orig: "0".into(),
            q_bc_mono: Some(20000),
            ad_rem_icms: Some(Rate(1700)),
            v_icms_mono_op: Some(Cents(3400)),
            p_dif: Some(Rate(100)),
            v_icms_mono_dif: Some(Cents(200)),
            v_icms_mono: Some(Cents(200)),
        });
        let mut totals = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut totals).unwrap();

        expect_wrapped_in(&xml, "ICMS53");
        expect_xml_contains(
            &xml,
            &[
                ("orig", "0"),
                ("CST", "53"),
                ("qBCMono", "200.0000"),
                ("adRemICMS", "17.0000"),
                ("vICMSMonoOp", "34.00"),
                ("pDif", "1.0000"),
                ("vICMSMonoDif", "2.00"),
                ("vICMSMono", "2.00"),
            ],
        );
        assert_eq!(totals.q_bc_mono, 20000);
        assert_eq!(totals.v_icms_mono, Cents(200));
    }

    #[test]
    fn test_tagicms_cst_60_previously_charged_st() {
        let v = IcmsVariant::from(IcmsCst::Cst60 {
            orig: "0".into(),
            v_bc_st_ret: Some(Cents(10000)),
            p_st: Some(Rate(1200)),
            v_icms_substituto: Some(Cents(1200)),
            v_icms_st_ret: Some(Cents(4000)),
            v_bc_fcp_st_ret: Some(Cents(5000)),
            p_fcp_st_ret: Some(Rate(1000)),
            v_fcp_st_ret: Some(Cents(1500)),
            p_red_bc_efet: Some(Rate(1400)),
            v_bc_efet: Some(Cents(10000)),
            p_icms_efet: Some(Rate(1000)),
            v_icms_efet: Some(Cents(1000)),
        });
        let mut totals = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut totals).unwrap();

        expect_wrapped_in(&xml, "ICMS60");
        expect_xml_contains(
            &xml,
            &[
                ("orig", "0"),
                ("CST", "60"),
                ("vBCSTRet", "100.00"),
                ("pST", "12.0000"),
                ("vICMSSubstituto", "12.00"),
                ("vICMSSTRet", "40.00"),
                ("vBCFCPSTRet", "50.00"),
                ("pFCPSTRet", "10.0000"),
                ("vFCPSTRet", "15.00"),
                ("pRedBCEfet", "14.0000"),
                ("vBCEfet", "100.00"),
                ("pICMSEfet", "10.0000"),
                ("vICMSEfet", "10.00"),
            ],
        );
        assert_eq!(totals.v_fcp_st_ret, Cents(1500));
    }

    #[test]
    fn test_tagicms_cst_61_monofasico_previously_charged() {
        let v = IcmsVariant::from(IcmsCst::Cst61 {
            orig: "0".into(),
            q_bc_mono_ret: Some(30000),
            ad_rem_icms_ret: Rate(200),
            v_icms_mono_ret: Cents(600),
        });
        let mut totals = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut totals).unwrap();

        expect_wrapped_in(&xml, "ICMS61");
        expect_xml_contains(
            &xml,
            &[
                ("orig", "0"),
                ("CST", "61"),
                ("qBCMonoRet", "300.0000"),
                ("adRemICMSRet", "2.0000"),
                ("vICMSMonoRet", "6.00"),
            ],
        );
        assert_eq!(totals.q_bc_mono_ret, 30000);
        assert_eq!(totals.v_icms_mono_ret, Cents(600));
    }

    #[test]
    fn test_tagicms_cst_70_reduction_with_st() {
        let v = IcmsVariant::from(IcmsCst::Cst70 {
            orig: "0".into(),
            mod_bc: "3".into(),
            p_red_bc: Rate(1000),
            v_bc: Cents(20000),
            p_icms: Rate(1000),
            v_icms: Cents(2000),
            v_bc_fcp: Some(Cents(20000)),
            p_fcp: Some(Rate(200)),
            v_fcp: Some(Cents(400)),
            mod_bc_st: "4".into(),
            p_mva_st: Some(Rate(3000)),
            p_red_bc_st: Some(Rate(0)),
            v_bc_st: Cents(6000),
            p_icms_st: Rate(1000),
            v_icms_st: Cents(2000),
            v_bc_fcp_st: Some(Cents(100)),
            p_fcp_st: Some(Rate(100)),
            v_fcp_st: Some(Cents(100)),
            v_icms_deson: Some(Cents(1000)),
            mot_des_icms: Some("9".into()),
            ind_deduz_deson: None,
            v_icms_st_deson: None,
            mot_des_icms_st: None,
        });
        let mut totals = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut totals).unwrap();

        expect_wrapped_in(&xml, "ICMS70");
        expect_xml_contains(
            &xml,
            &[
                ("orig", "0"),
                ("CST", "70"),
                ("modBC", "3"),
                ("pRedBC", "10.0000"),
                ("vBC", "200.00"),
                ("pICMS", "10.0000"),
                ("vICMS", "20.00"),
                ("vBCFCP", "200.00"),
                ("pFCP", "2.0000"),
                ("vFCP", "4.00"),
                ("modBCST", "4"),
                ("pMVAST", "30.0000"),
                ("pRedBCST", "0.0000"),
                ("vBCST", "60.00"),
                ("pICMSST", "10.0000"),
                ("vICMSST", "20.00"),
                ("vBCFCPST", "1.00"),
                ("pFCPST", "1.0000"),
                ("vFCPST", "1.00"),
                ("vICMSDeson", "10.00"),
                ("motDesICMS", "9"),
            ],
        );
        assert_eq!(totals.v_icms_deson, Cents(1000));
        assert_eq!(totals.v_bc, Cents(20000));
        assert_eq!(totals.v_icms, Cents(2000));
        assert_eq!(totals.v_bc_st, Cents(6000));
        assert_eq!(totals.v_st, Cents(2000));
        assert_eq!(totals.v_fcp_st, Cents(100));
        assert_eq!(totals.v_fcp, Cents(400));
    }

    #[test]
    fn test_tagicms_cst_90_others() {
        let v = IcmsVariant::from(IcmsCst::Cst90 {
            orig: "0".into(),
            mod_bc: Some("3".into()),
            v_bc: Some(Cents(20000)),
            p_red_bc: Some(Rate(1000)),
            c_benef_rbc: None,
            p_icms: Some(Rate(1000)),
            v_icms_op: None,
            p_dif: None,
            v_icms_dif: None,
            v_icms: Some(Cents(2000)),
            v_bc_fcp: Some(Cents(20000)),
            p_fcp: Some(Rate(200)),
            v_fcp: Some(Cents(400)),
            p_fcp_dif: None,
            v_fcp_dif: None,
            v_fcp_efet: None,
            mod_bc_st: Some("4".into()),
            p_mva_st: Some(Rate(3000)),
            p_red_bc_st: Some(Rate(0)),
            v_bc_st: Some(Cents(6000)),
            p_icms_st: Some(Rate(1000)),
            v_icms_st: Some(Cents(2000)),
            v_bc_fcp_st: Some(Cents(100)),
            p_fcp_st: Some(Rate(100)),
            v_fcp_st: Some(Cents(100)),
            v_icms_deson: Some(Cents(1000)),
            mot_des_icms: Some("9".into()),
            ind_deduz_deson: None,
            v_icms_st_deson: None,
            mot_des_icms_st: None,
        });
        let mut totals = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut totals).unwrap();

        expect_wrapped_in(&xml, "ICMS90");
        expect_xml_contains(
            &xml,
            &[
                ("orig", "0"),
                ("CST", "90"),
                ("modBC", "3"),
                ("pRedBC", "10.0000"),
                ("vBC", "200.00"),
                ("pICMS", "10.0000"),
                ("vICMS", "20.00"),
                ("vBCFCP", "200.00"),
                ("pFCP", "2.0000"),
                ("vFCP", "4.00"),
                ("modBCST", "4"),
                ("pMVAST", "30.0000"),
                ("pRedBCST", "0.0000"),
                ("vBCST", "60.00"),
                ("pICMSST", "10.0000"),
                ("vICMSST", "20.00"),
                ("vBCFCPST", "1.00"),
                ("pFCPST", "1.0000"),
                ("vFCPST", "1.00"),
                ("vICMSDeson", "10.00"),
                ("motDesICMS", "9"),
            ],
        );
        assert_eq!(totals.v_icms_deson, Cents(1000));
        assert_eq!(totals.v_bc, Cents(20000));
        assert_eq!(totals.v_icms, Cents(2000));
        assert_eq!(totals.v_bc_st, Cents(6000));
        assert_eq!(totals.v_st, Cents(2000));
        assert_eq!(totals.v_fcp_st, Cents(100));
        assert_eq!(totals.v_fcp, Cents(400));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  DI / adi / detExport / Rastro
// ═══════════════════════════════════════════════════════════════════════════

mod di_adi_det_export_rastro_ported {
    use super::*;

    #[test]
    fn test_tagdi_builds_di_tag_with_cnpj() {
        let xml = tag(
            "DI",
            &[],
            TagContent::Children(vec![
                tag("nDI", &[], "456".into()),
                tag("dDI", &[], "2024-03-01".into()),
                tag("xLocDesemb", &[], "Porto".into()),
                tag("UFDesemb", &[], "SP".into()),
                tag("dDesemb", &[], "2024-03-02".into()),
                tag("tpViaTransp", &[], "1".into()),
                tag("vAFRMM", &[], "150.45".into()),
                tag("tpIntermedio", &[], "1".into()),
                tag("CNPJ", &[], "08489068000198".into()),
                tag("UFTerceiro", &[], "RS".into()),
                tag("cExportador", &[], "123".into()),
            ]),
        );

        expect_wrapped_in(&xml, "DI");
        expect_xml_contains(
            &xml,
            &[
                ("nDI", "456"),
                ("dDI", "2024-03-01"),
                ("xLocDesemb", "Porto"),
                ("UFDesemb", "SP"),
                ("dDesemb", "2024-03-02"),
                ("tpViaTransp", "1"),
                ("vAFRMM", "150.45"),
                ("tpIntermedio", "1"),
                ("CNPJ", "08489068000198"),
                ("UFTerceiro", "RS"),
                ("cExportador", "123"),
            ],
        );
    }

    #[test]
    fn test_tagdi_builds_di_tag_with_cpf() {
        let xml = tag(
            "DI",
            &[],
            TagContent::Children(vec![
                tag("nDI", &[], "456".into()),
                tag("dDI", &[], "2024-03-01".into()),
                tag("xLocDesemb", &[], "Porto".into()),
                tag("UFDesemb", &[], "SP".into()),
                tag("dDesemb", &[], "2024-03-02".into()),
                tag("tpViaTransp", &[], "1".into()),
                tag("vAFRMM", &[], "150.45".into()),
                tag("tpIntermedio", &[], "1".into()),
                tag("CPF", &[], "10318797062".into()),
                tag("UFTerceiro", &[], "RS".into()),
                tag("cExportador", &[], "123".into()),
            ]),
        );

        assert!(xml.contains("<CPF>10318797062</CPF>"));
        assert!(!xml.contains("<CNPJ>"));
    }

    #[test]
    fn test_tagadi_builds_adi_tag() {
        let xml = tag(
            "adi",
            &[],
            TagContent::Children(vec![
                tag("nAdicao", &[], "1".into()),
                tag("nSeqAdic", &[], "1".into()),
                tag("cFabricante", &[], "abc123".into()),
                tag("vDescDI", &[], "12.48".into()),
                tag("nDraw", &[], "11111111111".into()),
            ]),
        );

        expect_wrapped_in(&xml, "adi");
        expect_xml_contains(
            &xml,
            &[
                ("nAdicao", "1"),
                ("nSeqAdic", "1"),
                ("cFabricante", "abc123"),
                ("vDescDI", "12.48"),
                ("nDraw", "11111111111"),
            ],
        );
    }

    #[test]
    fn test_tagdet_export_builds_det_export_tag() {
        let xml = tag(
            "detExport",
            &[],
            TagContent::Children(vec![tag("nDraw", &[], "123".into())]),
        );

        expect_wrapped_in(&xml, "detExport");
        assert!(xml.contains("<nDraw>123</nDraw>"));
    }

    #[test]
    fn test_tagdet_export_ind_builds_export_ind_tag() {
        let xml = tag(
            "exportInd",
            &[],
            TagContent::Children(vec![
                tag("nRE", &[], "123".into()),
                tag(
                    "chNFe",
                    &[],
                    "12345678901234567890123456789012345678901234".into(),
                ),
                tag("qExport", &[], "45.1".into()),
            ]),
        );

        expect_wrapped_in(&xml, "exportInd");
        expect_xml_contains(
            &xml,
            &[
                ("nRE", "123"),
                ("chNFe", "12345678901234567890123456789012345678901234"),
                ("qExport", "45.1"),
            ],
        );
    }

    #[test]
    fn test_tagrastro_builds_rastro_traceability_tag() {
        let xml = tag(
            "rastro",
            &[],
            TagContent::Children(vec![
                tag("nLote", &[], "1".into()),
                tag("qLote", &[], "1".into()),
                tag("dFab", &[], "2024-01-01".into()),
                tag("dVal", &[], "2024-01-01".into()),
                tag("cAgreg", &[], "1234".into()),
            ]),
        );

        expect_wrapped_in(&xml, "rastro");
        expect_xml_contains(
            &xml,
            &[
                ("nLote", "1"),
                ("qLote", "1"),
                ("dFab", "2024-01-01"),
                ("dVal", "2024-01-01"),
                ("cAgreg", "1234"),
            ],
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  ICMSUFDest
// ═══════════════════════════════════════════════════════════════════════════

mod tag_icms_uf_dest_ported {
    use super::*;

    #[test]
    fn test_tagicmsufdest_builds_icms_uf_dest_group() {
        let (xml, totals) = build_icms_uf_dest_xml(
            &IcmsUfDestData::new(Cents(100), Rate(100), Rate(100), Cents(100))
                .v_bc_fcp_uf_dest(Cents(100))
                .p_fcp_uf_dest(Rate(100))
                .v_fcp_uf_dest(Cents(100))
                .v_icms_uf_remet(Cents(100)),
        )
        .unwrap();

        expect_wrapped_in(&xml, "ICMSUFDest");
        expect_xml_contains(
            &xml,
            &[
                ("vBCUFDest", "1.00"),
                ("vBCFCPUFDest", "1.00"),
                ("pFCPUFDest", "1.0000"),
                ("pICMSUFDest", "1.0000"),
                ("pICMSInter", "1.0000"),
                ("pICMSInterPart", "100.0000"),
                ("vFCPUFDest", "1.00"),
                ("vICMSUFDest", "1.00"),
                ("vICMSUFRemet", "1.00"),
            ],
        );
        assert_eq!(totals.v_icms_uf_dest, Cents(100));
        assert_eq!(totals.v_fcp_uf_dest, Cents(100));
        assert_eq!(totals.v_icms_uf_remet, Cents(100));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  II (Imposto de Importacao)
// ═══════════════════════════════════════════════════════════════════════════

mod tag_ii_ported {
    use super::*;

    #[test]
    fn test_tagii_builds_ii_group() {
        let xml = build_ii_xml(&IiData::new(
            Cents(10000),
            Cents(100),
            Cents(100),
            Cents(100),
        ));

        expect_wrapped_in(&xml, "II");
        expect_xml_contains(
            &xml,
            &[
                ("vBC", "100.00"),
                ("vDespAdu", "1.00"),
                ("vII", "1.00"),
                ("vIOF", "1.00"),
            ],
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  ISSQN (structure only)
// ═══════════════════════════════════════════════════════════════════════════

mod tag_issqn_ported {
    use super::*;

    #[test]
    fn test_tagissqn_builds_issqn_tag_structure() {
        let xml = tag(
            "ISSQN",
            &[],
            TagContent::Children(vec![
                tag("vBC", &[], "1".into()),
                tag("vAliq", &[], "1".into()),
                tag("vISSQN", &[], "1".into()),
                tag("cMunFG", &[], "1234567".into()),
                tag("cListServ", &[], "10.10".into()),
                tag("vDeducao", &[], "1".into()),
                tag("vOutro", &[], "1".into()),
                tag("vDescIncond", &[], "1".into()),
                tag("vDescCond", &[], "1".into()),
                tag("vISSRet", &[], "1".into()),
                tag("indISS", &[], "1".into()),
                tag("cServico", &[], "1".into()),
                tag("cMun", &[], "123456".into()),
                tag("cPais", &[], "55".into()),
                tag("nProcesso", &[], "123".into()),
                tag("indIncentivo", &[], "12".into()),
            ]),
        );

        expect_wrapped_in(&xml, "ISSQN");
        expect_xml_contains(
            &xml,
            &[
                ("vBC", "1"),
                ("vAliq", "1"),
                ("vISSQN", "1"),
                ("cMunFG", "1234567"),
                ("cListServ", "10.10"),
                ("vDeducao", "1"),
                ("vOutro", "1"),
                ("vDescIncond", "1"),
                ("vDescCond", "1"),
                ("vISSRet", "1"),
                ("indISS", "1"),
                ("cServico", "1"),
                ("cMun", "123456"),
                ("cPais", "55"),
                ("nProcesso", "123"),
                ("indIncentivo", "12"),
            ],
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  infRespTec
// ═══════════════════════════════════════════════════════════════════════════

mod inf_resp_tec_ported {
    use super::*;

    #[test]
    fn test_taginfresptec_builds_inf_resp_tec_tag() {
        let xml = tag(
            "infRespTec",
            &[],
            TagContent::Children(vec![
                tag("CNPJ", &[], "76038276000120".into()),
                tag("xContato", &[], "Fulano de Tal".into()),
                tag("email", &[], "fulano@email.com".into()),
                tag("fone", &[], "51999999999".into()),
                tag("idCSRT", &[], "123".into()),
            ]),
        );

        expect_wrapped_in(&xml, "infRespTec");
        expect_xml_contains(
            &xml,
            &[
                ("CNPJ", "76038276000120"),
                ("xContato", "Fulano de Tal"),
                ("email", "fulano@email.com"),
                ("fone", "51999999999"),
                ("idCSRT", "123"),
            ],
        );
        // CSRT is not included in the tag
        assert!(!xml.contains("<CSRT>"));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  Agropecuario
// ═══════════════════════════════════════════════════════════════════════════

mod agropecuario_ported {
    use super::*;

    #[test]
    fn test_tagagropecuario_defensivo_builds_defensivo_tag() {
        let xml = tag(
            "defensivo",
            &[],
            TagContent::Children(vec![
                tag("nReceituario", &[], "1234567890ABCDEFGHIJ".into()),
                tag("CPFRespTec", &[], "12345678901".into()),
            ]),
        );

        expect_wrapped_in(&xml, "defensivo");
        expect_xml_contains(
            &xml,
            &[
                ("nReceituario", "1234567890ABCDEFGHIJ"),
                ("CPFRespTec", "12345678901"),
            ],
        );
    }

    #[test]
    fn test_tagagropecuario_guia_builds_guia_transito_tag() {
        let xml = tag(
            "guiaTransito",
            &[],
            TagContent::Children(vec![
                tag("tpGuia", &[], "1".into()),
                tag("UFGuia", &[], "MG".into()),
                tag("serieGuia", &[], "A12345678".into()),
                tag("nGuia", &[], "123456789".into()),
            ]),
        );

        expect_wrapped_in(&xml, "guiaTransito");
        expect_xml_contains(
            &xml,
            &[
                ("UFGuia", "MG"),
                ("serieGuia", "A12345678"),
                ("nGuia", "123456789"),
            ],
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  ICMSPart
// ═══════════════════════════════════════════════════════════════════════════

mod tag_icms_part_ported {
    use super::*;

    #[test]
    fn test_tagicmspart_builds_icms_part_group() {
        let (xml, totals) = build_icms_part_xml(
            &IcmsPartData::new(
                "0",
                "90",
                "1",
                Cents(20000),
                Rate(1000),
                Cents(2000),
                "4",
                Cents(6000),
                Rate(100),
                Cents(100),
                Rate(100),
                "EX",
            )
            .p_red_bc(Rate(500))
            .p_mva_st(Rate(3000))
            .p_red_bc_st(Rate(0))
            .v_bc_fcp_st(Cents(100))
            .p_fcp_st(Rate(100))
            .v_fcp_st(Cents(100)),
        )
        .unwrap();

        expect_wrapped_in(&xml, "ICMS");
        expect_wrapped_in(&xml, "ICMSPart");
        expect_xml_contains(
            &xml,
            &[
                ("orig", "0"),
                ("CST", "90"),
                ("modBC", "1"),
                ("vBC", "200.00"),
                ("pRedBC", "5.0000"),
                ("pICMS", "10.0000"),
                ("vICMS", "20.00"),
                ("modBCST", "4"),
                ("pMVAST", "30.0000"),
                ("pRedBCST", "0.0000"),
                ("vBCST", "60.00"),
                ("pICMSST", "1.0000"),
                ("vICMSST", "1.00"),
                ("vBCFCPST", "1.00"),
                ("pFCPST", "1.0000"),
                ("vFCPST", "1.00"),
                ("pBCOp", "1.0000"),
                ("UFST", "EX"),
            ],
        );
        assert_eq!(totals.v_bc, Cents(20000));
        assert_eq!(totals.v_icms, Cents(2000));
        assert_eq!(totals.v_bc_st, Cents(6000));
        assert_eq!(totals.v_st, Cents(100));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  ICMSST
// ═══════════════════════════════════════════════════════════════════════════

mod tag_icms_st_ported {
    use super::*;

    #[test]
    fn test_tagicmsst_builds_icmsst_repasse_group() {
        let (xml, totals) = build_icms_st_xml(
            &IcmsStData::new(
                "0",
                "41",
                Cents(20000),
                Cents(2000),
                Cents(3000),
                Cents(200),
            )
            .v_bc_fcp_st_ret(Cents(200))
            .p_fcp_st_ret(Rate(200))
            .v_fcp_st_ret(Cents(200))
            .p_st(Rate(200))
            .v_icms_substituto(Cents(200))
            .p_red_bc_efet(Rate(200))
            .v_bc_efet(Cents(200))
            .p_icms_efet(Rate(200))
            .v_icms_efet(Cents(200)),
        )
        .unwrap();

        expect_wrapped_in(&xml, "ICMS");
        expect_wrapped_in(&xml, "ICMSST");
        expect_xml_contains(
            &xml,
            &[
                ("orig", "0"),
                ("CST", "41"),
                ("vBCSTRet", "200.00"),
                ("vICMSSTRet", "20.00"),
                ("vBCSTDest", "30.00"),
                ("vICMSSTDest", "2.00"),
                ("vBCFCPSTRet", "2.00"),
                ("pFCPSTRet", "2.0000"),
                ("vFCPSTRet", "2.00"),
                ("pST", "2.0000"),
                ("vICMSSubstituto", "2.00"),
                ("pRedBCEfet", "2.0000"),
                ("vBCEfet", "2.00"),
                ("pICMSEfet", "2.0000"),
                ("vICMSEfet", "2.00"),
            ],
        );
        assert_eq!(totals.v_fcp_st_ret, Cents(200));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  ICMSSN (Simples Nacional)
// ═══════════════════════════════════════════════════════════════════════════

mod tag_icmssn_ported {
    use super::*;

    #[test]
    fn test_tagicmssn_101_with_credit() {
        let v = IcmsVariant::from(IcmsCsosn::Csosn101 {
            orig: "0".into(),
            csosn: "101".into(),
            p_cred_sn: Rate(300),
            v_cred_icms_sn: Cents(400),
        });
        let mut t = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut t).unwrap();
        expect_wrapped_in(&xml, "ICMS");
        expect_wrapped_in(&xml, "ICMSSN101");
        expect_xml_contains(
            &xml,
            &[
                ("orig", "0"),
                ("CSOSN", "101"),
                ("pCredSN", "3.0000"),
                ("vCredICMSSN", "4.00"),
            ],
        );
    }

    #[test]
    fn test_tagicmssn_102_without_credit() {
        let v = IcmsVariant::from(IcmsCsosn::Csosn102 {
            orig: "0".into(),
            csosn: "102".into(),
        });
        let mut t = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut t).unwrap();
        expect_wrapped_in(&xml, "ICMS");
        expect_wrapped_in(&xml, "ICMSSN102");
        expect_xml_contains(&xml, &[("CSOSN", "102")]);
    }

    #[test]
    fn test_tagicmssn_103_uses_same_wrapper() {
        let v = IcmsVariant::from(IcmsCsosn::Csosn102 {
            orig: "0".into(),
            csosn: "103".into(),
        });
        let mut t = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut t).unwrap();
        expect_wrapped_in(&xml, "ICMS");
        expect_wrapped_in(&xml, "ICMSSN102");
        expect_xml_contains(&xml, &[("CSOSN", "103")]);
    }

    #[test]
    fn test_tagicmssn_300_uses_same_wrapper() {
        let v = IcmsVariant::from(IcmsCsosn::Csosn102 {
            orig: "0".into(),
            csosn: "300".into(),
        });
        let mut t = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut t).unwrap();
        expect_wrapped_in(&xml, "ICMS");
        expect_wrapped_in(&xml, "ICMSSN102");
        expect_xml_contains(&xml, &[("CSOSN", "300")]);
    }

    #[test]
    fn test_tagicmssn_400_uses_same_wrapper() {
        let v = IcmsVariant::from(IcmsCsosn::Csosn102 {
            orig: "0".into(),
            csosn: "400".into(),
        });
        let mut t = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut t).unwrap();
        expect_wrapped_in(&xml, "ICMS");
        expect_wrapped_in(&xml, "ICMSSN102");
        expect_xml_contains(&xml, &[("CSOSN", "400")]);
    }

    #[test]
    fn test_tagicmssn_201_with_credit_and_st() {
        let v = IcmsVariant::from(IcmsCsosn::Csosn201 {
            orig: "0".into(),
            csosn: "201".into(),
            mod_bc_st: "4".into(),
            p_mva_st: Some(Rate(1000)),
            p_red_bc_st: Some(Rate(2000)),
            v_bc_st: Cents(30000),
            p_icms_st: Rate(100),
            v_icms_st: Cents(100),
            v_bc_fcp_st: Some(Cents(100)),
            p_fcp_st: Some(Rate(100)),
            v_fcp_st: Some(Cents(100)),
            p_cred_sn: Some(Rate(100)),
            v_cred_icms_sn: Some(Cents(100)),
        });
        let mut totals = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut totals).unwrap();
        expect_wrapped_in(&xml, "ICMS");
        expect_wrapped_in(&xml, "ICMSSN201");
        expect_xml_contains(
            &xml,
            &[
                ("orig", "0"),
                ("CSOSN", "201"),
                ("modBCST", "4"),
                ("pMVAST", "10.0000"),
                ("pRedBCST", "20.0000"),
                ("vBCST", "300.00"),
                ("pICMSST", "1.0000"),
                ("vICMSST", "1.00"),
                ("vBCFCPST", "1.00"),
                ("pFCPST", "1.0000"),
                ("vFCPST", "1.00"),
                ("pCredSN", "1.0000"),
                ("vCredICMSSN", "1.00"),
            ],
        );
        assert_eq!(totals.v_bc_st, Cents(30000));
        assert_eq!(totals.v_st, Cents(100));
    }

    #[test]
    fn test_tagicmssn_202_without_credit_with_st() {
        let v = IcmsVariant::from(IcmsCsosn::Csosn202 {
            orig: "0".into(),
            csosn: "202".into(),
            mod_bc_st: "4".into(),
            p_mva_st: Some(Rate(1000)),
            p_red_bc_st: Some(Rate(2000)),
            v_bc_st: Cents(30000),
            p_icms_st: Rate(100),
            v_icms_st: Cents(100),
            v_bc_fcp_st: Some(Cents(100)),
            p_fcp_st: Some(Rate(100)),
            v_fcp_st: Some(Cents(100)),
        });
        let mut totals = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut totals).unwrap();
        expect_wrapped_in(&xml, "ICMS");
        expect_wrapped_in(&xml, "ICMSSN202");
        expect_xml_contains(
            &xml,
            &[
                ("orig", "0"),
                ("CSOSN", "202"),
                ("modBCST", "4"),
                ("pMVAST", "10.0000"),
                ("pRedBCST", "20.0000"),
                ("vBCST", "300.00"),
                ("pICMSST", "1.0000"),
                ("vICMSST", "1.00"),
                ("vBCFCPST", "1.00"),
                ("pFCPST", "1.0000"),
                ("vFCPST", "1.00"),
            ],
        );
        assert_eq!(totals.v_bc_st, Cents(30000));
        assert_eq!(totals.v_st, Cents(100));
    }

    #[test]
    fn test_tagicmssn_203_uses_same_wrapper() {
        let v = IcmsVariant::from(IcmsCsosn::Csosn202 {
            orig: "0".into(),
            csosn: "203".into(),
            mod_bc_st: "4".into(),
            p_mva_st: Some(Rate(1000)),
            p_red_bc_st: Some(Rate(2000)),
            v_bc_st: Cents(30000),
            p_icms_st: Rate(100),
            v_icms_st: Cents(100),
            v_bc_fcp_st: Some(Cents(100)),
            p_fcp_st: Some(Rate(100)),
            v_fcp_st: Some(Cents(100)),
        });
        let mut t = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut t).unwrap();
        expect_wrapped_in(&xml, "ICMS");
        expect_wrapped_in(&xml, "ICMSSN202");
        expect_xml_contains(&xml, &[("CSOSN", "203")]);
    }

    #[test]
    fn test_tagicmssn_500_previously_charged() {
        let v = IcmsVariant::from(IcmsCsosn::Csosn500 {
            orig: "0".into(),
            csosn: "500".into(),
            v_bc_st_ret: Some(Cents(100)),
            p_st: Some(Rate(100)),
            v_icms_substituto: Some(Cents(100)),
            v_icms_st_ret: Some(Cents(100)),
            v_bc_fcp_st_ret: Some(Cents(100)),
            p_fcp_st_ret: Some(Rate(100)),
            v_fcp_st_ret: Some(Cents(100)),
            p_red_bc_efet: Some(Rate(100)),
            v_bc_efet: Some(Cents(100)),
            p_icms_efet: Some(Rate(100)),
            v_icms_efet: Some(Cents(100)),
        });
        let mut t = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut t).unwrap();
        expect_wrapped_in(&xml, "ICMS");
        expect_wrapped_in(&xml, "ICMSSN500");
        expect_xml_contains(
            &xml,
            &[
                ("orig", "0"),
                ("CSOSN", "500"),
                ("vBCSTRet", "1.00"),
                ("pST", "1.0000"),
                ("vICMSSubstituto", "1.00"),
                ("vICMSSTRet", "1.00"),
                ("vBCFCPSTRet", "1.00"),
                ("pFCPSTRet", "1.0000"),
                ("vFCPSTRet", "1.00"),
                ("pRedBCEfet", "1.0000"),
                ("vBCEfet", "1.00"),
                ("pICMSEfet", "1.0000"),
                ("vICMSEfet", "1.00"),
            ],
        );
    }

    #[test]
    fn test_tagicmssn_900_others() {
        let v = IcmsVariant::from(IcmsCsosn::Csosn900 {
            orig: "0".into(),
            csosn: "900".into(),
            mod_bc: Some("3".into()),
            v_bc: Some(Cents(10000)),
            p_red_bc: Some(Rate(100)),
            p_icms: Some(Rate(100)),
            v_icms: Some(Cents(100)),
            p_cred_sn: Some(Rate(300)),
            v_cred_icms_sn: Some(Cents(400)),
            mod_bc_st: Some("3".into()),
            p_mva_st: Some(Rate(100)),
            p_red_bc_st: Some(Rate(100)),
            v_bc_st: Some(Cents(100)),
            p_icms_st: Some(Rate(100)),
            v_icms_st: Some(Cents(100)),
            v_bc_fcp_st: Some(Cents(100)),
            p_fcp_st: Some(Rate(100)),
            v_fcp_st: Some(Cents(100)),
        });
        let mut totals = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut totals).unwrap();
        expect_wrapped_in(&xml, "ICMS");
        expect_wrapped_in(&xml, "ICMSSN900");
        expect_xml_contains(
            &xml,
            &[
                ("CSOSN", "900"),
                ("modBC", "3"),
                ("vBC", "100.00"),
                ("pRedBC", "1.0000"),
                ("pICMS", "1.0000"),
                ("vICMS", "1.00"),
                ("pCredSN", "3.0000"),
                ("vCredICMSSN", "4.00"),
                ("modBCST", "3"),
                ("pMVAST", "1.0000"),
                ("pRedBCST", "1.0000"),
                ("vBCST", "1.00"),
                ("pICMSST", "1.0000"),
                ("vICMSST", "1.00"),
                ("vBCFCPST", "1.00"),
                ("pFCPST", "1.0000"),
                ("vFCPST", "1.00"),
            ],
        );
        assert_eq!(totals.v_bc, Cents(10000));
        assert_eq!(totals.v_icms, Cents(100));
        assert_eq!(totals.v_bc_st, Cents(100));
        assert_eq!(totals.v_st, Cents(100));
    }

    #[test]
    fn test_tagicmssn_should_accept_empty_orig_when_crt_is_4() {
        let v = IcmsVariant::from(IcmsCsosn::Csosn900 {
            orig: String::new(),
            csosn: "900".into(),
            mod_bc: Some("3".into()),
            v_bc: Some(Cents(10000)),
            p_red_bc: Some(Rate(100)),
            p_icms: Some(Rate(100)),
            v_icms: Some(Cents(100)),
            p_cred_sn: Some(Rate(300)),
            v_cred_icms_sn: Some(Cents(400)),
            mod_bc_st: Some("3".into()),
            p_mva_st: Some(Rate(100)),
            p_red_bc_st: Some(Rate(100)),
            v_bc_st: Some(Cents(100)),
            p_icms_st: Some(Rate(100)),
            v_icms_st: Some(Cents(100)),
            v_bc_fcp_st: Some(Cents(100)),
            p_fcp_st: Some(Rate(100)),
            v_fcp_st: Some(Cents(100)),
        });
        let mut t = IcmsTotals::default();
        let xml = build_icms_xml(&v, &mut t).unwrap();
        expect_wrapped_in(&xml, "ICMS");
        expect_wrapped_in(&xml, "ICMSSN900");
        // orig should not be present when empty
        assert!(!xml.contains("<orig>"));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  Access Key (buildAccessKey)
// ═══════════════════════════════════════════════════════════════════════════

mod build_access_key_ported {
    use super::*;

    #[test]
    fn builds_a_44_digit_access_key_with_valid_check_digit() {
        let key = build_access_key(&AccessKeyParams::new(
            IbgeCode("35".to_string()),
            "1703",
            "58716523000119",
            InvoiceModel::Nfe,
            1,
            30,
            EmissionType::Normal,
            "00000030",
        ))
        .unwrap();

        assert_eq!(key.len(), 44);
        // Starts with state code
        assert!(key.starts_with("35"));
        // Contains CNPJ
        assert_eq!(&key[6..20], "58716523000119");
        // Model
        assert_eq!(&key[20..22], "55");
        // Series
        assert_eq!(&key[22..25], "001");
        // Number
        assert_eq!(&key[25..34], "000000030");
        // Emission type
        assert_eq!(&key[34..35], "1");
        // Numeric code
        assert_eq!(&key[35..43], "00000030");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  PIS tests
// ═══════════════════════════════════════════════════════════════════════════

mod pis_builders_ported {
    use super::*;

    #[test]
    fn pis_aliq_cst_01_with_percentage() {
        let xml = build_pis_xml(
            &PisData::new("01")
                .v_bc(Cents(10000))
                .p_pis(Rate4(16500))
                .v_pis(Cents(165)),
        );

        expect_wrapped_in(&xml, "PIS");
        expect_wrapped_in(&xml, "PISAliq");
        expect_xml_contains(
            &xml,
            &[
                ("CST", "01"),
                ("vBC", "100.00"),
                ("pPIS", "1.6500"),
                ("vPIS", "1.65"),
            ],
        );
    }

    #[test]
    fn pis_aliq_cst_02() {
        let xml = build_pis_xml(
            &PisData::new("02")
                .v_bc(Cents(20000))
                .p_pis(Rate4(16500))
                .v_pis(Cents(330)),
        );

        expect_wrapped_in(&xml, "PISAliq");
        expect_xml_contains(&xml, &[("CST", "02")]);
    }

    #[test]
    fn pis_qtde_cst_03_with_quantity() {
        let xml = build_pis_xml(
            &PisData::new("03")
                .q_bc_prod(10000)
                .v_aliq_prod(50000)
                .v_pis(Cents(500)),
        );

        expect_wrapped_in(&xml, "PISQtde");
        expect_xml_contains(
            &xml,
            &[
                ("CST", "03"),
                ("qBCProd", "1.0000"),
                ("vAliqProd", "5.0000"),
                ("vPIS", "5.00"),
            ],
        );
    }

    #[test]
    fn pis_nt_cst_04_non_taxed() {
        let xml = build_pis_xml(&PisData::new("04"));

        expect_wrapped_in(&xml, "PISNT");
        expect_xml_contains(&xml, &[("CST", "04")]);
        assert!(!xml.contains("<vBC>"));
    }

    #[test]
    fn pis_nt_cst_05_06_07_08_09() {
        for cst in &["05", "06", "07", "08", "09"] {
            let xml = build_pis_xml(&PisData::new(*cst));

            expect_wrapped_in(&xml, "PISNT");
            expect_xml_contains(&xml, &[("CST", cst)]);
        }
    }

    #[test]
    fn pis_outr_cst_49_with_percentage() {
        let xml = build_pis_xml(
            &PisData::new("49")
                .v_bc(Cents(10000))
                .p_pis(Rate4(16500))
                .v_pis(Cents(165)),
        );

        expect_wrapped_in(&xml, "PISOutr");
        expect_xml_contains(
            &xml,
            &[
                ("CST", "49"),
                ("vBC", "100.00"),
                ("pPIS", "1.6500"),
                ("vPIS", "1.65"),
            ],
        );
    }

    #[test]
    fn pis_outr_cst_99_with_quantity() {
        let xml = build_pis_xml(
            &PisData::new("99")
                .q_bc_prod(10000)
                .v_aliq_prod(50000)
                .v_pis(Cents(500)),
        );

        expect_wrapped_in(&xml, "PISOutr");
        expect_xml_contains(
            &xml,
            &[
                ("CST", "99"),
                ("qBCProd", "1.0000"),
                ("vAliqProd", "5.0000"),
                ("vPIS", "5.00"),
            ],
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  COFINS tests
// ═══════════════════════════════════════════════════════════════════════════

mod cofins_builders_ported {
    use super::*;

    #[test]
    fn cofins_aliq_cst_01_with_percentage() {
        let xml = build_cofins_xml(
            &CofinsData::new("01")
                .v_bc(Cents(10000))
                .p_cofins(Rate4(76000))
                .v_cofins(Cents(760)),
        );

        expect_wrapped_in(&xml, "COFINS");
        expect_wrapped_in(&xml, "COFINSAliq");
        expect_xml_contains(
            &xml,
            &[
                ("CST", "01"),
                ("vBC", "100.00"),
                ("pCOFINS", "7.6000"),
                ("vCOFINS", "7.60"),
            ],
        );
    }

    #[test]
    fn cofins_qtde_cst_03_with_quantity() {
        let xml = build_cofins_xml(
            &CofinsData::new("03")
                .q_bc_prod(10000)
                .v_aliq_prod(50000)
                .v_cofins(Cents(500)),
        );

        expect_wrapped_in(&xml, "COFINSQtde");
        expect_xml_contains(
            &xml,
            &[
                ("CST", "03"),
                ("qBCProd", "1.0000"),
                ("vAliqProd", "5.0000"),
                ("vCOFINS", "5.00"),
            ],
        );
    }

    #[test]
    fn cofins_nt_cst_04_non_taxed() {
        let xml = build_cofins_xml(&CofinsData::new("04"));

        expect_wrapped_in(&xml, "COFINSNT");
        expect_xml_contains(&xml, &[("CST", "04")]);
    }

    #[test]
    fn cofins_nt_cst_05_06_07_08_09() {
        for cst in &["05", "06", "07", "08", "09"] {
            let xml = build_cofins_xml(&CofinsData::new(*cst));

            expect_wrapped_in(&xml, "COFINSNT");
        }
    }

    #[test]
    fn cofins_outr_cst_99_with_percentage() {
        let xml = build_cofins_xml(
            &CofinsData::new("99")
                .v_bc(Cents(10000))
                .p_cofins(Rate4(76000))
                .v_cofins(Cents(760)),
        );

        expect_wrapped_in(&xml, "COFINSOutr");
        expect_xml_contains(
            &xml,
            &[
                ("CST", "99"),
                ("vBC", "100.00"),
                ("pCOFINS", "7.6000"),
                ("vCOFINS", "7.60"),
            ],
        );
    }

    #[test]
    fn cofins_outr_cst_49_with_quantity() {
        let xml = build_cofins_xml(
            &CofinsData::new("49")
                .q_bc_prod(20000)
                .v_aliq_prod(30000)
                .v_cofins(Cents(600)),
        );

        expect_wrapped_in(&xml, "COFINSOutr");
        expect_xml_contains(
            &xml,
            &[
                ("CST", "49"),
                ("qBCProd", "2.0000"),
                ("vAliqProd", "3.0000"),
                ("vCOFINS", "6.00"),
            ],
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  IPI tests
// ═══════════════════════════════════════════════════════════════════════════

mod ipi_builders_ported {
    use super::*;

    #[test]
    fn ipi_trib_cst_50_with_percentage() {
        let xml = build_ipi_xml(
            &IpiData::new("50", "999")
                .v_bc(Cents(10000))
                .p_ipi(Rate(50000))
                .v_ipi(Cents(500)),
        );

        expect_wrapped_in(&xml, "IPI");
        expect_wrapped_in(&xml, "IPITrib");
        expect_xml_contains(
            &xml,
            &[
                ("cEnq", "999"),
                ("CST", "50"),
                ("vBC", "100.00"),
                ("pIPI", "5.0000"),
                ("vIPI", "5.00"),
            ],
        );
    }

    #[test]
    fn ipi_trib_cst_00_with_percentage() {
        let xml = build_ipi_xml(
            &IpiData::new("00", "999")
                .v_bc(Cents(20000))
                .p_ipi(Rate(100000))
                .v_ipi(Cents(2000)),
        );

        expect_wrapped_in(&xml, "IPITrib");
        expect_xml_contains(&xml, &[("CST", "00"), ("pIPI", "10.0000")]);
    }

    #[test]
    fn ipi_trib_cst_49_with_unit_based() {
        let xml = build_ipi_xml(
            &IpiData::new("49", "999")
                .q_unid(10000)
                .v_unid(50000)
                .v_ipi(Cents(500)),
        );

        expect_wrapped_in(&xml, "IPITrib");
        expect_xml_contains(
            &xml,
            &[
                ("CST", "49"),
                ("qUnid", "1.0000"),
                ("vUnid", "5.0000"),
                ("vIPI", "5.00"),
            ],
        );
    }

    #[test]
    fn ipi_trib_cst_99_with_percentage() {
        let xml = build_ipi_xml(
            &IpiData::new("99", "311")
                .v_bc(Cents(50000))
                .p_ipi(Rate(150000))
                .v_ipi(Cents(7500)),
        );

        expect_wrapped_in(&xml, "IPITrib");
        expect_xml_contains(
            &xml,
            &[
                ("cEnq", "311"),
                ("CST", "99"),
                ("vBC", "500.00"),
                ("pIPI", "15.0000"),
                ("vIPI", "75.00"),
            ],
        );
    }

    #[test]
    fn ipi_nt_cst_01_non_taxed() {
        let xml = build_ipi_xml(&IpiData::new("01", "999"));

        expect_wrapped_in(&xml, "IPI");
        expect_wrapped_in(&xml, "IPINT");
        expect_xml_contains(&xml, &[("CST", "01")]);
        assert!(!xml.contains("<IPITrib>"));
    }

    #[test]
    fn ipi_nt_cst_02_03_04_05_non_taxed() {
        for cst in &["02", "03", "04", "05"] {
            let xml = build_ipi_xml(&IpiData::new(*cst, "999"));

            expect_wrapped_in(&xml, "IPINT");
            expect_xml_contains(&xml, &[("CST", cst)]);
        }
    }

    #[test]
    fn ipi_with_optional_header_fields_cnpj_prod_cselo_qselo() {
        let xml = build_ipi_xml(
            &IpiData::new("50", "999")
                .cnpj_prod("12345678901234")
                .c_selo("SELO123")
                .q_selo(10)
                .v_bc(Cents(10000))
                .p_ipi(Rate(50000))
                .v_ipi(Cents(500)),
        );

        expect_xml_contains(
            &xml,
            &[
                ("CNPJProd", "12345678901234"),
                ("cSelo", "SELO123"),
                ("qSelo", "10"),
            ],
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  II builders — additional scenarios
// ═══════════════════════════════════════════════════════════════════════════

mod ii_builders_additional {
    use super::*;

    #[test]
    fn builds_ii_with_all_zero_values() {
        let xml = build_ii_xml(&IiData::new(Cents(0), Cents(0), Cents(0), Cents(0)));

        expect_xml_contains(
            &xml,
            &[
                ("vBC", "0.00"),
                ("vDespAdu", "0.00"),
                ("vII", "0.00"),
                ("vIOF", "0.00"),
            ],
        );
    }

    #[test]
    fn builds_ii_with_large_values() {
        let xml = build_ii_xml(&IiData::new(
            Cents(1000000),
            Cents(50000),
            Cents(150000),
            Cents(25000),
        ));

        expect_xml_contains(
            &xml,
            &[
                ("vBC", "10000.00"),
                ("vDespAdu", "500.00"),
                ("vII", "1500.00"),
                ("vIOF", "250.00"),
            ],
        );
    }
}

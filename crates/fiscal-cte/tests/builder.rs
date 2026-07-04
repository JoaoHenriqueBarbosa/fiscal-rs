//! Integration test: build a complete CT-e Normal (road modal) XML and assert
//! the document structure and schema block ordering.

mod common;

use common::sample_cte;
use fiscal_cte::build_cte_xml;

#[test]
fn builds_well_formed_cte() {
    let xml = build_cte_xml(&sample_cte()).unwrap();

    assert!(xml.starts_with("<CTe"));
    assert!(xml.contains("xmlns=\"http://www.portalfiscal.inf.br/cte\""));
    assert!(xml.contains("<infCte versao=\"4.00\" Id=\"CTe"));
    // mod 57 present in ide
    assert!(xml.contains("<mod>57</mod>"));
    // modal road block
    assert!(xml.contains("<infModal versaoModal=\"4.00\"><rodo><RNTRC>12345678</RNTRC>"));
    assert!(xml.contains("<infNFe><chave>44444444444444444444444444444444444444444444</chave>"));
}

#[test]
fn ide_blocks_in_schema_order() {
    let xml = build_cte_xml(&sample_cte()).unwrap();
    let pos = |needle: &str| {
        xml.find(needle)
            .unwrap_or_else(|| panic!("missing {needle}"))
    };

    // ide ordering: cUF < cCT < CFOP < natOp < mod < serie < nCT < dhEmi
    assert!(pos("<cUF>") < pos("<cCT>"));
    assert!(pos("<cCT>") < pos("<CFOP>"));
    assert!(pos("<CFOP>") < pos("<natOp>"));
    assert!(pos("<natOp>") < pos("<mod>"));
    assert!(pos("<mod>") < pos("<serie>"));
    assert!(pos("<dhEmi>") < pos("<tpImp>"));
    assert!(pos("<tpImp>") < pos("<tpEmis>"));
    assert!(pos("<tpEmis>") < pos("<cDV>"));
    assert!(pos("<cDV>") < pos("<tpAmb>"));

    // top-level: ide < compl < emit < rem < dest < vPrest < imp < infCTeNorm
    assert!(pos("<ide>") < pos("<compl>"));
    assert!(pos("<compl>") < pos("<emit>"));
    assert!(pos("<emit>") < pos("<rem>"));
    assert!(pos("<rem>") < pos("<dest>"));
    assert!(pos("<dest>") < pos("<vPrest>"));
    assert!(pos("<vPrest>") < pos("<imp>"));
    assert!(pos("<imp>") < pos("<infCTeNorm>"));
}

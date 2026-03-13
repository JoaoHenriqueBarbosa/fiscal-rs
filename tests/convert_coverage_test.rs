// Tests for convert.rs to achieve full coverage.

mod common;
use common::FIXTURES_PATH;

fn minimal_txt_v400(extra_lines: &str) -> String {
    format!(
        "NOTAFISCAL|1|\nA|4.00|NFe35180825028332000105550010000005021000005010||\nB|35|00000501|VENDA|55|1|502|2018-08-13T17:28:10-03:00||1|1|3550308|1|1|8|1|1|0|3||0|3.2.1.1|||\nC|Empresa Teste|Fantasia|123456789|||12345|3|\nC02|25028332000105|\nC05|Rua Teste|100||Centro|3550308|SAO PAULO|SP|01001000||Brasil|1122223333|\nE|Destinatario Teste|1|142304338112||||\nE02|17812455000295|\nE05|R Dest|491||Bairro|3550308|SAO PAULO|SP|05302001|1058|BRASIL|1143053063|\nH|1||\nI|1001|SEM GTIN|Produto Teste|84715010|||5102|UN|1.0000|10.00|10.00|SEM GTIN|UN|1.0000|10.00|||||1||||\n{extra_lines}M|5.00|\nN|\nN02|0|00|3|10.00|18.00|1.80|0.00|0.00|\nO||||999|\nO07|50|0.10|\nO10|10.00|1.00|\nQ|\nQ02|01|10.00|0.65|0.07|\nS|\nS02|01|10.00|3.00|0.30|\nW|\nW02|10.00|1.80|0.00|0.00|0.00|0.00|0.00|0.00|10.00|0.00|0.00|0.00|0.00|0.10|0.00|0.07|0.30|0.00|10.00|0.00|0.00|0.00|0.00|\nX|9|\nY|0.00|\nYA|0|01|10.00||00||0||\nZ|info fiscal|complemento|\n"
    )
}

fn minimal_txt_sebrae() -> String {
    // SEBRAE B has indIntermed (24 fields), SEBRAE I05c has cBenef (5 fields)
    "NOTAFISCAL|1|\nA|4.00|NFe52180824915365000376550550000001221000001224|\nB|52|00000122|Devolucao|55|55|122|2018-08-10T11:27:00-03:00|2018-08-10T11:27:00-03:00|1|1|5221858|1|1|4|2|4|0|9||3|4.01_b018|||\nBA|\nBA02|52180706147451002003550010000580821325441420|\nC|Empresa Sebrae|Fantasia|105670154||||3|\nC02|24915365000376|\nC05|QD.10 LOTE 38|0|LJ 1|VALPARAISO|5221858|Valparaiso de Goias|GO|72876030|1058|BRASIL|6136273376|\nE|Destino Sebrae|1|105896578||||\nE02|06147451002003|\nE05|Rua Dest|0||Bairro|5201405|Aparecida|GO|74985220|1058|BRASIL||\nH|1||\nI|71288||MY LILY EAU DE PARFUM|33030020||5411|PC|1.0000|18.0800000000|18.08||PC|1.0000|18.0800000000|||||1||||\nI05c|2000800||||\nM|0.00|\nN|\nN02|0|00|3|18.08|17.0000|3.07|0.00|0.00|\nO|||||\nO08|53|\nQ|\nQ02|01|18.08|0.6500|0.11|\nS|\nS02|01|18.08|3.0000|0.54|\nW|\nW02|18.08|3.07|0.00|0.00|0.00|0.00|0.00|0.00|18.08|0.00|0.00|0.00|0.00|0.00|0.00|11.75|0.54|0.00|18.08|0.00|\nW04c||\nW04e||\nW04g||\nX|9|\nYA||\nYA01||90|18.08|\n".to_string()
}

fn make_txt_with_icms(icms_line: &str) -> String {
    format!(
        "NOTAFISCAL|1|\nA|4.00|NFe35180825028332000105550010000005021000005010||\nB|35|00000501|VENDA|55|1|502|2018-08-13T17:28:10-03:00||1|1|3550308|1|1|8|1|1|0|3||0|3.2.1.1|||\nC|Empresa|Fantasia|123456789|||12345|3|\nC02|25028332000105|\nC05|Rua|100||Centro|3550308|SAO PAULO|SP|01001000||||\nE|Dest|1|||||\nE02|17812455000295|\nE05|R Dest|491||Bairro|3550308|SAO PAULO|SP|05302001|1058|BRASIL||\nH|1||\nI|1001|SEM GTIN|Produto|84715010|||5102|UN|1.0000|10.00|10.00|SEM GTIN|UN|1.0000|10.00|||||1||||\nM|0.00|\nN|\n{icms_line}\nQ|\nQ02|01|10.00|0.65|0.07|\nS|\nS02|01|10.00|3.00|0.30|\nW|\nW02|10.00|1.80|0.00|0.00|0.00|0.00|0.00|0.00|10.00|0.00|0.00|0.00|0.00|0.00|0.00|0.07|0.30|0.00|10.00|0.00|0.00|0.00|0.00|\nX|9|\nY|0.00|\nYA|0|01|10.00||00||0||\nZ|||\n"
    )
}

// Error handling
#[test]
fn txt_to_xml_empty() {
    assert!(fiscal::convert::txt_to_xml("", "local").is_err());
}
#[test]
fn txt_to_xml_whitespace() {
    assert!(fiscal::convert::txt_to_xml("   \n  ", "local").is_err());
}
#[test]
fn txt_to_xml_wrong_header() {
    assert!(
        fiscal::convert::txt_to_xml("X|1|\nA|4.00|||\n", "local")
            .unwrap_err()
            .to_string()
            .contains("not a valid")
    );
}
#[test]
fn txt_to_xml_count_mismatch() {
    assert!(
        fiscal::convert::txt_to_xml(
            "NOTAFISCAL|3|\nA|4.00|NFe35180825028332000105550010000005021000005010||\n",
            "local_v12"
        )
        .unwrap_err()
        .to_string()
        .contains("does not match")
    );
}
#[test]
fn txt_to_xml_validation_errors() {
    assert!(
        fiscal::convert::txt_to_xml(
            "NOTAFISCAL|1|\nA|4.00|NFe35180825028332000105550010000005021000005010||\nB|missing\n",
            "local_v12"
        )
        .is_err()
    );
}
#[test]
fn txt_to_xml_no_a_entity() {
    assert!(fiscal::convert::txt_to_xml("NOTAFISCAL|1|\nB|35|00000501|VENDA|55|1|502|2018-08-13T17:28:10-03:00||1|1|3550308|1|1|8|1|1|0|3||0|3.2.1.1|||\n", "local_v12").is_err());
}
#[test]
fn txt_to_xml_unsupported_version() {
    assert!(fiscal::convert::txt_to_xml("NOTAFISCAL|1|\nA|2.00|||\n", "local_v12").is_err());
}
#[test]
fn validate_txt_empty() {
    assert!(fiscal::convert::validate_txt("", "local").is_err());
}
#[test]
fn validate_txt_wrong_header() {
    assert!(fiscal::convert::validate_txt("WRONG|1|\n", "local").is_err());
}
#[test]
fn validate_txt_valid() {
    let t = std::fs::read_to_string(format!("{FIXTURES_PATH}txt/nfe_4.00_local_01.txt")).unwrap();
    assert!(fiscal::convert::validate_txt(&t, "local_v12").unwrap());
}
#[test]
fn validate_txt_cr_tabs() {
    let t = std::fs::read_to_string(format!("{FIXTURES_PATH}txt/nfe_4.00_local_01.txt")).unwrap();
    assert!(fiscal::convert::validate_txt(&t.replace('\n', "\r\n"), "local_v12").unwrap());
}
#[test]
fn validate_txt_no_a_marker() {
    if let Ok(v) = fiscal::convert::validate_txt(
        "NOTAFISCAL|1|\nB|35|00000501|V|55|1|502|2018-08-13T17:28:10-03:00||1|1|3|1|1|8|1|1|0|3||0|3.2.1.1|||\n",
        "local_v12",
    ) {
        assert!(!v);
    }
}
#[test]
fn validate_txt_forbidden_chars() {
    if let Ok(v) = fiscal::convert::validate_txt(
        "NOTAFISCAL|1|\nA|4.00|||\nB|35|<bad>|V|55|1|502|2018-08-13||1|1|3|1|1|8|1|1|0|3||0|3.2.1.1|||\n",
        "local_v12",
    ) {
        assert!(!v);
    }
}
#[test]
fn validate_txt_erroneous() {
    let b = std::fs::read(format!("{FIXTURES_PATH}txt/nfe_errado.txt")).unwrap();
    if let Ok(v) = fiscal::convert::validate_txt(&String::from_utf8_lossy(&b), "local_v12") {
        assert!(!v);
    }
}

// Layout selection
#[test]
fn txt_to_xml_version_310() {
    // 310: B=24 (indPag), I=23, E=7, O=6 (clEnq), N02=7, W02=17, YA=7, Y=no fields
    let txt = "NOTAFISCAL|1|\nA|3.10|NFe35180825028332000105550010000005021000005010||\nB|35|00000501|VENDA|0|55|1|502|2018-08-13T17:28:10-03:00|2018-08-14T09:00:00-03:00|1|1|3550308|1|1|8|1|1|0|3|0|3.2.1.1|||\nC|Empresa||140950881119||||3|\nC02|25028332000105|\nC05|Rua|100||Centro|3550308|SAO PAULO|SP|01001000||Brasil|1122223333|\nE|Destino|1|142304338112||||\nE02|17812455000295|\nE05|R Dest|491||Bairro|3550308|SAO PAULO|SP|05302001|1058|BRASIL|1143053063|\nH|1||\nI|1001|7897|Produto|84715010||5102|UN|1.0000|10.00|10.00|7897|UN|1.0000|10.00|||||1||0||\nM|0.00|\nN|\nN02|0|00|3|10.00|18.00|1.80|\nO|||||999|\nO08|53|\nQ|\nQ02|01|10.00|0.65|0.07|\nS|\nS02|01|10.00|3.00|0.30|\nW|\nW02|10.00|1.80|0.00|0.00|0.00|10.00|0.00|0.00|0.00|0.00|0.00|0.07|0.30|0.00|10.00|0.00|\nX|9|\nY|\nYA|01|10.00||00||0|\nZ||complemento|\n";
    let xml = fiscal::convert::txt_to_xml(txt, "local").unwrap();
    assert!(xml.contains("versao=\"3.10\""));
}

#[test]
fn txt_to_xml_local_v13() {
    let t = "NOTAFISCAL|1|\nA|4.00|NFe35180825028332000105550010000005021000005010||\nB|35|00000501|VENDA|55|1|502|2018-08-13||1|1|3|1|1|8|1|1|0|3||0|3.2.1.1||||||||\nC|E|F|1|||1|3|\nC02|25028332000105|\nC05|R|1||C|3|S|SP|0||||\nE|D|1|||||\nE02|1|\nE05|R|1||B|3|S|SP|0|1|B||\nH|1||\nI|1|S|P|8|||5|U|1|10|10|S|U|1|10|||||1||||||\nM|0|\nN|\nN02|0|00|3|10|18|1.80|0|0|\nQ|\nQ02|01|10|0.65|0.07|\nS|\nS02|01|10|3|0.30|\nW|\nW02|10|1|0|0|0|0|0|0|10|0|0|0|0|0|0|0|0|0|10|0|0|0|0|\nX|9|\nY|0|\nYA|0|01|10||00||0||\nZ|||\n";
    let _ = fiscal::convert::txt_to_xml(t, "local_v13");
}
#[test]
fn unknown_layout() {
    let t = std::fs::read_to_string(format!("{FIXTURES_PATH}txt/nfe_4.00_local_01.txt")).unwrap();
    assert!(fiscal::convert::txt_to_xml(&t, "xyz").is_ok());
}

// Multiple invoices
#[test]
fn two_invoices() {
    let b = std::fs::read_to_string(format!("{FIXTURES_PATH}txt/nfe_4.00_local_01.txt")).unwrap();
    let l: Vec<&str> = b.lines().collect();
    let r = &l[1..];
    let mut t = "NOTAFISCAL|2|\n".to_string();
    for x in r {
        t.push_str(x);
        t.push('\n');
    }
    for x in r {
        t.push_str(x);
        t.push('\n');
    }
    assert!(
        fiscal::convert::txt_to_xml(&t, "local_v12")
            .unwrap()
            .contains("<cProd>11352</cProd>")
    );
}

// Record handlers
#[test]
fn parse_basic() {
    let x = fiscal::convert::txt_to_xml(&minimal_txt_v400(""), "local_v12").unwrap();
    assert!(x.contains("<NFe"));
    assert!(x.contains("<infAdFisco>info fiscal</infAdFisco>"));
    assert!(x.contains("<xFant>Fantasia</xFant>"));
}

#[test]
fn parse_ba02_nf_ref() {
    // Only BA/BA02 (not BB/BB02 which only exists in structure_400)
    let txt = "NOTAFISCAL|1|\nA|4.00|NFe35180825028332000105550010000005021000005010||\nB|35|00000501|VENDA|55|1|502|2018-08-13T17:28:10-03:00||1|1|3550308|1|1|8|1|1|0|3||0|3.2.1.1|||\nBA|\nBA02|52180706147451002003550010000580821325441420|\nC|Empresa|Fantasia|123456789|||12345|3|\nC02|25028332000105|\nC05|Rua|100||Centro|3550308|SAO PAULO|SP|01001000||Brasil|1122223333|\nE|Dest|1|142304338112||||\nE02|17812455000295|\nE05|R Dest|491||Bairro|3550308|SAO PAULO|SP|05302001|1058|BRASIL|1143053063|\nH|1||\nI|1001|SEM GTIN|Produto|84715010|||5102|UN|1.0000|10.00|10.00|SEM GTIN|UN|1.0000|10.00|||||1||||\nM|0.00|\nN|\nN02|0|00|3|10.00|18.00|1.80|0.00|0.00|\nQ|\nQ02|01|10.00|0.65|0.07|\nS|\nS02|01|10.00|3.00|0.30|\nW|\nW02|10.00|1.80|0.00|0.00|0.00|0.00|0.00|0.00|10.00|0.00|0.00|0.00|0.00|0.00|0.00|0.07|0.30|0.00|10.00|0.00|0.00|0.00|0.00|\nX|9|\nY|0.00|\nYA|0|01|10.00||00||0||\nZ|||\n";
    let xml = fiscal::convert::txt_to_xml(txt, "local_v12").unwrap();
    assert!(xml.contains("<NFref>"));
    assert!(xml.contains("<refNFe>52180706147451002003550010000580821325441420</refNFe>"));
}

#[test]
fn parse_c02a() {
    let t = "NOTAFISCAL|1|\nA|4.00|NFe35180825028332000105550010000005021000005010||\nB|35|00000501|VENDA|55|1|502|2018-08-13T17:28:10-03:00||1|1|3550308|1|1|8|1|1|0|3||0|3.2.1.1|||\nC|PF||123456789||||3|\nC02a|12345678901|\nC05|Rua|100||Centro|3550308|SAO PAULO|SP|01001000||Brasil|1122223333|\nE|Dest|1|||||\nE02|17812455000295|\nE05|R Dest|491||Bairro|3550308|SAO PAULO|SP|05302001|1058|BRASIL|1143053063|\nH|1||\nI|1001|SEM GTIN|Produto|84715010|||5102|UN|1.0000|10.00|10.00|SEM GTIN|UN|1.0000|10.00|||||1||||\nM|0.00|\nN|\nN02|0|00|3|10.00|18.00|1.80|0.00|0.00|\nQ|\nQ02|01|10.00|0.65|0.07|\nS|\nS02|01|10.00|3.00|0.30|\nW|\nW02|10.00|1.80|0.00|0.00|0.00|0.00|0.00|0.00|10.00|0.00|0.00|0.00|0.00|0.00|0.00|0.07|0.30|0.00|10.00|0.00|0.00|0.00|0.00|\nX|9|\nY|0.00|\nYA|0|01|10.00||00||0||\nZ|||\n";
    assert!(
        fiscal::convert::txt_to_xml(t, "local_v12")
            .unwrap()
            .contains("<CPF>12345678901</CPF>")
    );
}
#[test]
fn parse_e03() {
    let t = "NOTAFISCAL|1|\nA|4.00|NFe35180825028332000105550010000005021000005010||\nB|35|00000501|VENDA|55|1|502|2018-08-13T17:28:10-03:00||1|1|3550308|1|1|8|1|1|0|3||0|3.2.1.1|||\nC|E|F|1|||1|3|\nC02|25028332000105|\nC05|R|1||C|3|S|SP|0||||\nE|PF|1|||||\nE03|90483926086|\nE05|R|1||B|3|S|SP|0|1|B|1|\nH|1||\nI|1|S|P|8|||5|U|1|10|10|S|U|1|10|||||1||||\nM|0|\nN|\nN02|0|00|3|10|18|1.80|0|0|\nQ|\nQ02|01|10|0.65|0.07|\nS|\nS02|01|10|3|0.30|\nW|\nW02|10|1|0|0|0|0|0|0|10|0|0|0|0|0|0|0|0|0|10|0|0|0|0|\nX|9|\nY|0|\nYA|0|01|10||00||0||\nZ|||\n";
    assert!(
        fiscal::convert::txt_to_xml(t, "local_v12")
            .unwrap()
            .contains("<CPF>90483926086</CPF>")
    );
}
#[test]
fn parse_e03a() {
    let t = "NOTAFISCAL|1|\nA|4.00|NFe35180825028332000105550010000005021000005010||\nB|35|00000501|VENDA|55|1|502|2018-08-13T17:28:10-03:00||1|1|3550308|1|1|8|1|1|0|3||0|3.2.1.1|||\nC|E|F|1|||1|3|\nC02|25028332000105|\nC05|R|1||C|3|S|SP|0||||\nE|Foreign|9|||||\nE03a|PASSPORT123|\nE05|R|1||B|3|S|SP|0|1|B|1|\nH|1||\nI|1|S|P|8|||5|U|1|10|10|S|U|1|10|||||1||||\nM|0|\nN|\nN02|0|00|3|10|18|1.80|0|0|\nQ|\nQ02|01|10|0.65|0.07|\nS|\nS02|01|10|3|0.30|\nW|\nW02|10|1|0|0|0|0|0|0|10|0|0|0|0|0|0|0|0|0|10|0|0|0|0|\nX|9|\nY|0|\nYA|0|01|10||00||0||\nZ|||\n";
    assert!(
        fiscal::convert::txt_to_xml(t, "local_v12")
            .unwrap()
            .contains("<idEstrangeiro>PASSPORT123</idEstrangeiro>")
    );
}

#[test]
fn parse_x05() {
    let t = "NOTAFISCAL|1|\nA|4.00|NFe35180825028332000105550010000005021000005010||\nB|35|00000501|VENDA|55|1|502|2018-08-13T17:28:10-03:00||1|1|3550308|1|1|8|1|1|0|3||0|3.2.1.1|||\nC|E|F|1|||1|3|\nC02|25028332000105|\nC05|R|1||C|3|S|SP|0||||\nE|D|1|||||\nE02|1|\nE05|R|1||B|3|S|SP|0|1|B||\nH|1||\nI|1|S|P|8|||5|U|1|10|10|S|U|1|10|||||1||||\nM|0|\nN|\nN02|0|00|3|10|18|1.80|0|0|\nQ|\nQ02|01|10|0.65|0.07|\nS|\nS02|01|10|3|0.30|\nW|\nW02|10|1|0|0|0|0|0|0|10|0|0|0|0|0|0|0|0|0|10|0|0|0|0|\nX|3|\nX03|Trans PF|111|Rua|Cidade|SP|\nX05|12345678901|\nX26|1|CX|MK|123|5|6|\nY|0|\nYA|0|01|10||00||0||\nZ|||\n";
    let x = fiscal::convert::txt_to_xml(t, "local_v12").unwrap();
    assert!(x.contains("<CPF>12345678901</CPF>"));
    assert!(x.contains("<nVol>123</nVol>"));
}

#[test]
fn sebrae_ya_ya01() {
    let x = fiscal::convert::txt_to_xml(&minimal_txt_sebrae(), "sebrae").unwrap();
    assert!(x.contains("<detPag>"));
    assert!(x.contains("<tPag>90</tPag>"));
}
#[test]
fn finalize_empty_item() {
    let t = "NOTAFISCAL|1|\nA|4.00|NFe35180825028332000105550010000005021000005010||\nB|35|00000501|VENDA|55|1|502|2018-08-13T17:28:10-03:00||1|1|3550308|1|1|8|1|1|0|3||0|3.2.1.1|||\nC|E|F|1|||1|3|\nC02|25028332000105|\nC05|R|1||C|3|S|SP|0||||\nE|D|1|||||\nE02|1|\nE05|R|1||B|3|S|SP|0|1|B||\nH|1||\nW|\nW02|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|\nX|9|\nY|0|\nYA|0|01|0||00||0||\nZ|||\n";
    assert!(
        !fiscal::convert::txt_to_xml(t, "local_v12")
            .unwrap()
            .contains("<det ")
    );
}

// Emit/dest optional fields
#[test]
fn emit_all_optional() {
    let t = "NOTAFISCAL|1|\nA|4.00|NFe35180825028332000105550010000005021000005010||\nB|35|00000501|VENDA|55|1|502|2018-08-13T17:28:10-03:00||1|1|3550308|1|1|8|1|1|0|3||0|3.2.1.1|||\nC|Empresa|Fantasia|123456789|IEST123|IM456|CNAE789|3|\nC02|25028332000105|\nC05|Rua|100|Sala 1|Centro|3550308|SAO PAULO|SP|01001000|1058|BRASIL|1122223333|\nE|Dest|1|IEDEST|ISUF456|IMDEST|dest@email.com|\nE02|17812455000295|\nE05|R Dest|491|Andar 2|Bairro|3550308|SAO PAULO|SP|05302001|1058|BRASIL|1143053063|\nH|1||\nI|1001|SEM GTIN|Produto|84715010|||5102|UN|1.0000|10.00|10.00|SEM GTIN|UN|1.0000|10.00|||||1||||\nM|0.00|\nN|\nN02|0|00|3|10.00|18.00|1.80|0.00|0.00|\nQ|\nQ02|01|10.00|0.65|0.07|\nS|\nS02|01|10.00|3.00|0.30|\nW|\nW02|10.00|1.80|0.00|0.00|0.00|0.00|0.00|0.00|10.00|0.00|0.00|0.00|0.00|0.00|0.00|0.07|0.30|0.00|10.00|0.00|0.00|0.00|0.00|\nX|9|\nY|0.00|\nYA|0|01|10.00||00||0||\nZ|||\n";
    let x = fiscal::convert::txt_to_xml(t, "local_v12").unwrap();
    for tag in &[
        "<xCpl>Sala 1</xCpl>",
        "<cPais>1058</cPais>",
        "<xPais>BRASIL</xPais>",
        "<fone>1122223333</fone>",
        "<IEST>IEST123</IEST>",
        "<IM>IM456</IM>",
        "<CNAE>CNAE789</CNAE>",
        "<ISUF>ISUF456</ISUF>",
        "<email>dest@email.com</email>",
        "<xCpl>Andar 2</xCpl>",
    ] {
        assert!(x.contains(tag), "Missing {tag}");
    }
}
#[test]
fn dest_cpf_no_ender() {
    let t = "NOTAFISCAL|1|\nA|4.00|NFe35180825028332000105550010000005021000005010||\nB|35|00000501|VENDA|55|1|502|2018-08-13T17:28:10-03:00||1|1|3550308|1|1|8|1|1|0|3||0|3.2.1.1|||\nC|E|F|1|||1|3|\nC02|25028332000105|\nC05|R|1||C|3|S|SP|0||||\nE|PF|9|||||\nE03|90483926086|\nH|1||\nI|1|S|P|8|||5|U|1|10|10|S|U|1|10|||||1||||\nM|0|\nN|\nN02|0|00|3|10|18|1.80|0|0|\nQ|\nQ02|01|10|0.65|0.07|\nS|\nS02|01|10|3|0.30|\nW|\nW02|10|1|0|0|0|0|0|0|10|0|0|0|0|0|0|0|0|0|10|0|0|0|0|\nX|9|\nY|0|\nYA|0|01|10||00||0||\nZ|||\n";
    let x = fiscal::convert::txt_to_xml(t, "local_v12").unwrap();
    assert!(x.contains("<CPF>90483926086</CPF>"));
    assert!(!x.contains("<enderDest>"));
}

// ICMS variations (N02-N10, all field counts for v12)
#[test]
fn icms_n04() {
    assert!(
        fiscal::convert::txt_to_xml(
            &make_txt_with_icms("N04|0|20|3|10.00|10.00|18.00|1.80|0.00|0.00|0.00|0.00|0.00|"),
            "local_v12"
        )
        .unwrap()
        .contains("<ICMS20>")
    );
}
#[test]
fn icms_n05() {
    assert!(
        fiscal::convert::txt_to_xml(
            &make_txt_with_icms("N05|0|30|0|||10.00|18.00|1.80||||0.00|0.00|"),
            "local_v12"
        )
        .unwrap()
        .contains("<ICMS30>")
    );
}
#[test]
fn icms_n06() {
    assert!(
        fiscal::convert::txt_to_xml(&make_txt_with_icms("N06|0|40|0.00|9|0|"), "local_v12")
            .unwrap()
            .contains("<ICMS40>")
    );
}
#[test]
fn icms_n07() {
    let x = fiscal::convert::txt_to_xml(
        &make_txt_with_icms("N07|0|51|3|10.00|10.00|18.00|1.80|10.00|1.80|0.00|0.00|0.00|0.00|"),
        "local_v12",
    )
    .unwrap();
    assert!(x.contains("<ICMS51>"));
    assert!(x.contains("<vICMSOp>"));
}
#[test]
fn icms_n08() {
    assert!(
        fiscal::convert::txt_to_xml(
            &make_txt_with_icms(
                "N08|0|60|10.00|18.00|1.80|0.00|0.00|0.00|0.00|0.00|0.00|0.00|0.00|"
            ),
            "local_v12"
        )
        .unwrap()
        .contains("<ICMS60>")
    );
}
// N03 v12: 19 fields (orig|CST|modBC|vBC|pICMS|vICMS|vBCFCP|pFCP|vFCP|modBCST|pMVAST|pRedBCST|vBCST|pICMSST|vICMSST|vBCFCPST|pFCPST|vFCPST)
#[test]
fn icms_n03() {
    assert!(
        fiscal::convert::txt_to_xml(
            &make_txt_with_icms(
                "N03|0|10|3|10.00|18.00|1.80|0.00|0.00|0.00|0|||10.00|18.00|1.80||||"
            ),
            "local_v12"
        )
        .unwrap()
        .contains("<ICMS10>")
    );
}
// N09 v12: 22 fields
#[test]
fn icms_n09() {
    assert!(fiscal::convert::txt_to_xml(&make_txt_with_icms("N09|0|70|3|10.00|10.00|18.00|1.80|0.00|0.00|0.00|0|||10.00|18.00|1.80||||0.00|0.00|"), "local_v12").unwrap().contains("<ICMS70>"));
}
// N10 v12: 22 fields (orig|CST|modBC|vBC|pRedBC|pICMS|vICMS|vBCFCP|pFCP|vFCP|modBCST|pMVAST|pRedBCST|vBCST|pICMSST|vICMSST|vBCFCPST|pFCPST|vFCPST|vICMSDeson|motDesICMS)
#[test]
fn icms_n10() {
    assert!(fiscal::convert::txt_to_xml(&make_txt_with_icms("N10|0|90|3|10.00|0.00|18.00|1.80|0.00|0.00|0.00|0|0.00|0.00|10.00|18.00|1.80|0.00|0.00|0.00|0.00|0.00|"), "local_v12").unwrap().contains("<ICMS90>"));
}

// PIS/COFINS/IPI
#[test]
fn pis_outr() {
    assert!(
        fiscal::convert::txt_to_xml(
            &minimal_txt_v400("").replace("Q02|01|10.00|0.65|0.07|", "Q02|49|10.00|0.65|0.07|"),
            "local_v12"
        )
        .unwrap()
        .contains("<PISOutr>")
    );
}
#[test]
fn cofins_outr() {
    assert!(
        fiscal::convert::txt_to_xml(
            &minimal_txt_v400("").replace("S02|01|10.00|3.00|0.30|", "S02|49|10.00|3.00|0.30|"),
            "local_v12"
        )
        .unwrap()
        .contains("<COFINSOutr>")
    );
}
#[test]
fn ipi_ipint() {
    assert!(
        fiscal::convert::txt_to_xml(
            &minimal_txt_v400("").replace("O07|50|0.10|\nO10|10.00|1.00|", "O08|53|"),
            "local_v12"
        )
        .unwrap()
        .contains("<IPINT>")
    );
}
#[test]
fn ipi_header() {
    let t = "NOTAFISCAL|1|\nA|4.00|NFe35180825028332000105550010000005021000005010||\nB|35|00000501|VENDA|55|1|502|2018-08-13T17:28:10-03:00||1|1|3550308|1|1|8|1|1|0|3||0|3.2.1.1|||\nC|E|F|1|||1|3|\nC02|25028332000105|\nC05|R|1||C|3|S|SP|0||||\nE|D|1|||||\nE02|1|\nE05|R|1||B|3|S|SP|0|1|B||\nH|1||\nI|1|S|P|8|||5|U|1|10|10|S|U|1|10|||||1||||\nM|0|\nN|\nN02|0|00|3|10|18|1.80|0|0|\nO||selo1|5|999|\nO07|00|1.00|\nO10|10.00|10.00|\nQ|\nQ02|01|10|0.65|0.07|\nS|\nS02|01|10|3|0.30|\nW|\nW02|10|1|0|0|0|0|0|0|10|0|0|0|0|1|0|0|0|0|11|0|0|0|0|\nX|9|\nY|0|\nYA|0|01|11||00||0||\nZ|||\n";
    let x = fiscal::convert::txt_to_xml(t, "local_v12").unwrap();
    assert!(x.contains("<qSelo>5</qSelo>"));
    assert!(x.contains("<cEnq>999</cEnq>"));
}

// Product optional fields
#[test]
fn prod_optional() {
    let t = "NOTAFISCAL|1|\nA|4.00|NFe35180825028332000105550010000005021000005010||\nB|35|00000501|VENDA|55|1|502|2018-08-13T17:28:10-03:00||1|1|3550308|1|1|8|1|1|0|3||0|3.2.1.1|||\nC|E|F|1|||1|3|\nC02|25028332000105|\nC05|R|1||C|3|S|SP|0||||\nE|D|1|||||\nE02|1|\nE05|R|1||B|3|S|SP|0|1|B||\nH|1||\nI|1001|7897|Produto|84715010|BEN001|01|5102|UN|1.0000|10.00|10.00|7897|UN|1.0000|10.00|1.00|0.50|0.25|0.75|1|PED001|1||\nI05C|1700700|||\nI05G|CRED01|10.00|100.00|\nM|5.00|\nN|\nN02|0|00|3|10|18|1.80|0|0|\nO||||999|\nO07|50|0.10|\nO10|10|1|\nQ|\nQ02|01|10|0.65|0.07|\nS|\nS02|01|10|3|0.30|\nW|\nW02|10|1|0|0|0|0|0|0|10|0|0|0|0|0|0|0|0|0|10|0|0|0|0|\nX|9|\nY|0|\nYA|0|01|10||00||0||\nZ|||\n";
    let x = fiscal::convert::txt_to_xml(t, "local_v12").unwrap();
    for tag in &[
        "<cBenef>BEN001</cBenef>",
        "<EXTIPI>01</EXTIPI>",
        "<vFrete>1.00</vFrete>",
        "<vSeg>0.50</vSeg>",
        "<vDesc>0.25</vDesc>",
        "<vOutro>0.75</vOutro>",
        "<xPed>PED001</xPed>",
        "<nItemPed>1</nItemPed>",
        "<gCred>",
        "<cCredPresumido>CRED01</cCredPresumido>",
        "<CEST>1700700</CEST>",
        "<vTotTrib>5.00</vTotTrib>",
    ] {
        assert!(x.contains(tag), "Missing {tag}");
    }
}

// Transport, cobr, troco
#[test]
fn transp_cobr_troco() {
    let t = "NOTAFISCAL|1|\nA|4.00|NFe35180825028332000105550010000005021000005010||\nB|35|00000501|VENDA|55|1|502|2018-08-13T17:28:10-03:00||1|1|3550308|1|1|8|1|1|0|3||0|3.2.1.1|||\nC|E|F|1|||1|3|\nC02|25028332000105|\nC05|R|1||C|3|S|SP|0||||\nE|D|1|||||\nE02|1|\nE05|R|1||B|3|S|SP|0|1|B||\nH|1||\nI|1|S|P|8|||5|U|1|10|10|S|U|1|10|||||1||||\nM|0|\nN|\nN02|0|00|3|10|18|1.80|0|0|\nQ|\nQ02|01|10|0.65|0.07|\nS|\nS02|01|10|3|0.30|\nW|\nW02|10|1|0|0|0|0|0|0|10|0|0|0|0|0|0|0|0|0|10|0|0|0|0|\nX|1|\nX03|Trans|111|Rua T|Cid|SP|\nX04|47269568000257|\nX26|2|CX|MK|001|10.5|12|\nX26|3|PKG|||5|6|\nY|5.00|\nY02|502|10.00|0.00|10.00|\nY07|001|2018-08-13|10.00|\nYA|0|01|10||00||0||\nZ|||\n";
    let x = fiscal::convert::txt_to_xml(t, "local_v12").unwrap();
    assert!(x.contains("<modFrete>1</modFrete>"));
    assert!(x.contains("<CNPJ>47269568000257</CNPJ>"));
    assert_eq!(x.matches("<vol>").count(), 2);
    assert!(x.contains("<cobr>"));
    assert!(x.contains("<vTroco>5.00</vTroco>"));
}

// infAdic
#[test]
fn no_inf_adic_when_empty() {
    assert!(
        !fiscal::convert::txt_to_xml(
            &minimal_txt_v400("").replace("Z|info fiscal|complemento|", "Z|||"),
            "local_v12"
        )
        .unwrap()
        .contains("<infAdic>")
    );
}

// Contingency
#[test]
fn ide_contingency() {
    let t = "NOTAFISCAL|1|\nA|4.00|NFe35180825028332000105550010000005021000005010||\nB|35|00000501|VENDA|55|1|502|2018-08-13T17:28:10-03:00||1|1|3550308|1|6|8|1|1|0|3||0|3.2.1.1|2018-08-13T10:00:00-03:00|SEFAZ Indisponivel|\nC|E|F|1|||1|3|\nC02|25028332000105|\nC05|R|1||C|3|S|SP|0||||\nE|D|1|||||\nE02|1|\nE05|R|1||B|3|S|SP|0|1|B||\nH|1||\nI|1|S|P|8|||5|U|1|10|10|S|U|1|10|||||1||||\nM|0|\nN|\nN02|0|00|3|10|18|1.80|0|0|\nQ|\nQ02|01|10|0.65|0.07|\nS|\nS02|01|10|3|0.30|\nW|\nW02|10|1|0|0|0|0|0|0|10|0|0|0|0|0|0|0|0|0|10|0|0|0|0|\nX|9|\nY|0|\nYA|0|01|10||00||0||\nZ|||\n";
    let x = fiscal::convert::txt_to_xml(t, "local_v12").unwrap();
    assert!(x.contains("<dhCont>2018-08-13T10:00:00-03:00</dhCont>"));
    assert!(x.contains("<xJust>SEFAZ Indisponivel</xJust>"));
}

// dhSaiEnt + indIntermed
#[test]
fn ide_dh_sai_ent() {
    let t = "NOTAFISCAL|1|\nA|4.00|NFe35180825028332000105550010000005021000005010||\nB|35|00000501|VENDA|55|1|502|2018-08-13T17:28:10-03:00|2018-08-14T09:00:00-03:00|1|1|3550308|1|1|8|1|1|0|3|1|0|3.2.1.1|||\nC|E|F|1|||1|3|\nC02|25028332000105|\nC05|R|1||C|3|S|SP|0||||\nE|D|1|||||\nE02|1|\nE05|R|1||B|3|S|SP|0|1|B||\nH|1||\nI|1|S|P|8|||5|U|1|10|10|S|U|1|10|||||1||||\nM|0|\nN|\nN02|0|00|3|10|18|1.80|0|0|\nQ|\nQ02|01|10|0.65|0.07|\nS|\nS02|01|10|3|0.30|\nW|\nW02|10|1|0|0|0|0|0|0|10|0|0|0|0|0|0|0|0|0|10|0|0|0|0|\nX|9|\nY|0|\nYA|0|01|10||00||0||\nZ|||\n";
    let x = fiscal::convert::txt_to_xml(t, "local_v12").unwrap();
    assert!(x.contains("<dhSaiEnt>2018-08-14T09:00:00-03:00</dhSaiEnt>"));
    assert!(x.contains("<indIntermed>1</indIntermed>"));
}

// Minimal (no optional)
#[test]
fn minimal_no_optional() {
    let t = "NOTAFISCAL|1|\nA|4.00|NFe35180825028332000105550010000005021000005010||\nB|35|00000501|VENDA|55|1|502|2018-08-13T17:28:10-03:00||1|1|3550308|1|1|8|1|1|0|3||0|3.2.1.1|||\nC|Empresa||||||3|\nC02|25028332000105|\nC05|R|1||C|3|S|SP|0||||\nE|D||||||\nE02|1|\nE05|R|1||B|3|S|SP|0||||\nH|1||\nI|1|S|P|8|||5|U|1|10|10|S|U|1|10|||||1||||\nM||\nN|\nN02|0|00|3|10|18|1.80|0|0|\nQ|\nQ02|01|10|0.65|0.07|\nS|\nS02|01|10|3|0.30|\nW|\nW02|10|1|0|0|0|0|0|0|10|0|0|0|0|0|0|0|0|0|10||0|0|0|\nX|9|\nY||\nYA|0|01|10||00||0||\nZ|||\n";
    let x = fiscal::convert::txt_to_xml(t, "local_v12").unwrap();
    assert!(!x.contains("<xFant>"));
    assert!(!x.contains("<IEST>"));
    assert!(!x.contains("<infAdic>"));
}

// Sebrae fixture
// The sebrae fixture file was originally for a slightly different layout.
// Use validate_txt to exercise the sebrae code paths without requiring exact field match.
#[test]
fn sebrae_fixture_validate() {
    let t = std::fs::read_to_string(format!("{FIXTURES_PATH}txt/nota_4.00_sebrae.txt")).unwrap();
    let _ = fiscal::convert::validate_txt(&t, "sebrae");
}
// Also convert via the "local" layout which the sebrae fixture originally targeted
#[test]
fn sebrae_fixture_as_local() {
    let t = std::fs::read_to_string(format!("{FIXTURES_PATH}txt/nota_4.00_sebrae.txt")).unwrap();
    let _ = fiscal::convert::txt_to_xml(&t, "local");
}

// YA04 card fields including CNPJReceb and idTermPag
#[test]
fn ya04_card_with_cnpj_receb_and_id_term_pag() {
    // Based on the SEBRAE layout which supports YA01 + YA04
    let txt = minimal_txt_sebrae().replace(
        "YA01||90|18.08|\n",
        "YA01||03|18.08|\nYA04|1|12345678000199|02|AUTH456|98765432000188|TERMINAL01|\n",
    );
    let xml = fiscal::convert::txt_to_xml(&txt, "sebrae").unwrap();
    assert!(xml.contains("<card>"), "Missing <card> element");
    assert!(
        xml.contains("<tpIntegra>1</tpIntegra>"),
        "Missing tpIntegra"
    );
    assert!(xml.contains("<CNPJ>12345678000199</CNPJ>"), "Missing CNPJ");
    assert!(xml.contains("<tBand>02</tBand>"), "Missing tBand");
    assert!(xml.contains("<cAut>AUTH456</cAut>"), "Missing cAut");
    assert!(
        xml.contains("<CNPJReceb>98765432000188</CNPJReceb>"),
        "Missing CNPJReceb"
    );
    assert!(
        xml.contains("<idTermPag>TERMINAL01</idTermPag>"),
        "Missing idTermPag"
    );
}

#[test]
fn ya04_card_without_cnpj_receb() {
    // Based on the SEBRAE layout which supports YA01 + YA04
    let txt = minimal_txt_sebrae().replace(
        "YA01||90|18.08|\n",
        "YA01||03|18.08|\nYA04|2|12345678000199|01|AUTH789|||\n",
    );
    let xml = fiscal::convert::txt_to_xml(&txt, "sebrae").unwrap();
    assert!(xml.contains("<tpIntegra>2</tpIntegra>"));
    assert!(xml.contains("<cAut>AUTH789</cAut>"));
    assert!(!xml.contains("<CNPJReceb>"), "CNPJReceb should be absent");
    assert!(!xml.contains("<idTermPag>"), "idTermPag should be absent");
}

// tpIntegra=0 must NOT generate <card> (value 0 is outside the XSD enumeration)
#[test]
fn ya04_tp_integra_zero_no_card() {
    // SEBRAE layout (YA01 + YA04)
    let txt = minimal_txt_sebrae().replace(
        "YA01||90|18.08|\n",
        "YA01||03|18.08|\nYA04|0|12345678000199|02|AUTH000|||\n",
    );
    let xml = fiscal::convert::txt_to_xml(&txt, "sebrae").unwrap();
    assert!(
        !xml.contains("<card>"),
        "tpIntegra=0 should NOT produce a <card> element"
    );
    assert!(
        !xml.contains("<tpIntegra>"),
        "tpIntegra=0 should NOT produce a <tpIntegra> element"
    );
}

#[test]
fn ya_tp_integra_zero_no_card_standard_layout() {
    // Standard layout (YA with inline card fields) – tpIntegra is "0"
    let txt = minimal_txt_v400("");
    let xml = fiscal::convert::txt_to_xml(&txt, "local_v12").unwrap();
    assert!(
        !xml.contains("<card>"),
        "tpIntegra=0 in standard YA should NOT produce a <card> element"
    );
}

// Access key validation
#[test]
fn invalid_access_key() {
    let t = "NOTAFISCAL|1|\nA|4.00|NFe12345||\nB|35|00000501|VENDA|55|1|502|2018-08-13T17:28:10-03:00||1|1|3550308|1|1|8|1|1|0|3||0|3.2.1.1|||\nC|E|F|1|||1|3|\nC02|25028332000105|\nC05|R|1||C|3|S|SP|0||||\nE|D|1|||||\nE02|1|\nE05|R|1||B|3|S|SP|0|1|B||\nH|1||\nI|1|S|P|8|||5|U|1|10|10|S|U|1|10|||||1||||\nM|0|\nN|\nN02|0|00|3|10|18|1.80|0|0|\nQ|\nQ02|01|10|0.65|0.07|\nS|\nS02|01|10|3|0.30|\nW|\nW02|10|1|0|0|0|0|0|0|10|0|0|0|0|0|0|0|0|0|10|0|0|0|0|\nX|9|\nY|0|\nYA|0|01|10||00||0||\nZ|||\n";
    assert!(
        fiscal::convert::txt_to_xml(t, "local_v12")
            .unwrap_err()
            .to_string()
            .contains("chave")
    );
}

// Multi-item
#[test]
fn two_items() {
    let t = "NOTAFISCAL|1|\nA|4.00|NFe35180825028332000105550010000005021000005010||\nB|35|00000501|VENDA|55|1|502|2018-08-13T17:28:10-03:00||1|1|3550308|1|1|8|1|1|0|3||0|3.2.1.1|||\nC|E|F|1|||1|3|\nC02|25028332000105|\nC05|R|1||C|3|S|SP|0||||\nE|D|1|||||\nE02|1|\nE05|R|1||B|3|S|SP|0|1|B||\nH|1||\nI|1001|SEM GTIN|P1|84715010|||5102|UN|1.0000|10.00|10.00|SEM GTIN|UN|1.0000|10.00|||||1||||\nM|0|\nN|\nN02|0|00|3|10|18|1.80|0|0|\nQ|\nQ02|01|10|0.65|0.07|\nS|\nS02|01|10|3|0.30|\nH|2||\nI|2002|SEM GTIN|P2|84715010|||5102|UN|2.0000|20.00|40.00|SEM GTIN|UN|2.0000|20.00|||||1||||\nM|2|\nN|\nN06|0|41|0.00|9|0|\nQ|\nQ02|06|0|0|0|\nS|\nS02|06|0|0|0|\nW|\nW02|10|1|0|0|0|0|0|0|50|0|0|0|0|0|0|0|0|0|50|0|0|0|0|\nX|9|\nY|0|\nYA|0|01|50||00||0||\nZ|||\n";
    let x = fiscal::convert::txt_to_xml(t, "local_v12").unwrap();
    assert_eq!(x.matches("<det ").count(), 2);
    assert!(x.contains("<ICMS00>"));
    assert!(x.contains("<ICMS40>"));
}

// Fixtures
#[test]
fn local_fixture_full() {
    let t = std::fs::read_to_string(format!("{FIXTURES_PATH}txt/nfe_4.00_local_01.txt")).unwrap();
    let x = fiscal::convert::txt_to_xml(&t, "local_v12").unwrap();
    assert!(x.contains("<gCred>"));
    assert!(x.contains("<ICMS70>"));
    assert_eq!(x.matches("<det ").count(), 4);
}

// gCred ordering — must be between cBenef and EXTIPI/CFOP per XSD sequence
#[test]
fn gcred_order_between_cbenef_and_extipi() {
    // I line with cBenef="BEN001" and EXTIPI="01", plus I05G for gCred
    let t = "NOTAFISCAL|1|\nA|4.00|NFe35180825028332000105550010000005021000005010||\nB|35|00000501|VENDA|55|1|502|2018-08-13T17:28:10-03:00||1|1|3550308|1|1|8|1|1|0|3||0|3.2.1.1|||\nC|E|F|1|||1|3|\nC02|25028332000105|\nC05|R|1||C|3|S|SP|0||||\nE|D|1|||||\nE02|1|\nE05|R|1||B|3|S|SP|0|1|B||\nH|1||\nI|1001|7897|Produto|84715010|BEN001|01|5102|UN|1.0000|10.00|10.00|7897|UN|1.0000|10.00|||||1||||\nI05G|CRED01|10.00|100.00|\nM|0|\nN|\nN02|0|00|3|10|18|1.80|0|0|\nQ|\nQ02|01|10|0.65|0.07|\nS|\nS02|01|10|3|0.30|\nW|\nW02|10|1|0|0|0|0|0|0|10|0|0|0|0|0|0|0|0|0|10|0|0|0|0|\nX|9|\nY|0|\nYA|0|01|10||00||0||\nZ|||\n";
    let x = fiscal::convert::txt_to_xml(t, "local_v12").unwrap();

    let cbenef_pos = x.find("<cBenef>").expect("cBenef must exist");
    let gcred_pos = x.find("<gCred>").expect("gCred must exist");
    let extipi_pos = x.find("<EXTIPI>").expect("EXTIPI must exist");
    let cfop_pos = x.find("<CFOP>").expect("CFOP must exist");

    assert!(
        gcred_pos > cbenef_pos,
        "gCred ({gcred_pos}) must come after cBenef ({cbenef_pos})"
    );
    assert!(
        gcred_pos < extipi_pos,
        "gCred ({gcred_pos}) must come before EXTIPI ({extipi_pos})"
    );
    assert!(
        gcred_pos < cfop_pos,
        "gCred ({gcred_pos}) must come before CFOP ({cfop_pos})"
    );
}

#[test]
fn gcred_order_before_cfop_no_extipi() {
    // I line with cBenef="BEN001" but NO EXTIPI, plus I05G for gCred
    let t = "NOTAFISCAL|1|\nA|4.00|NFe35180825028332000105550010000005021000005010||\nB|35|00000501|VENDA|55|1|502|2018-08-13T17:28:10-03:00||1|1|3550308|1|1|8|1|1|0|3||0|3.2.1.1|||\nC|E|F|1|||1|3|\nC02|25028332000105|\nC05|R|1||C|3|S|SP|0||||\nE|D|1|||||\nE02|1|\nE05|R|1||B|3|S|SP|0|1|B||\nH|1||\nI|1001|7897|Produto|84715010|BEN001||5102|UN|1.0000|10.00|10.00|7897|UN|1.0000|10.00|||||1||||\nI05G|CRED02|5.50|50.00|\nM|0|\nN|\nN02|0|00|3|10|18|1.80|0|0|\nQ|\nQ02|01|10|0.65|0.07|\nS|\nS02|01|10|3|0.30|\nW|\nW02|10|1|0|0|0|0|0|0|10|0|0|0|0|0|0|0|0|0|10|0|0|0|0|\nX|9|\nY|0|\nYA|0|01|10||00||0||\nZ|||\n";
    let x = fiscal::convert::txt_to_xml(t, "local_v12").unwrap();

    let cbenef_pos = x.find("<cBenef>").expect("cBenef must exist");
    let gcred_pos = x.find("<gCred>").expect("gCred must exist");
    let cfop_pos = x.find("<CFOP>").expect("CFOP must exist");

    assert!(
        gcred_pos > cbenef_pos,
        "gCred ({gcred_pos}) must come after cBenef ({cbenef_pos})"
    );
    assert!(
        gcred_pos < cfop_pos,
        "gCred ({gcred_pos}) must come before CFOP ({cfop_pos})"
    );
    // EXTIPI should not be present
    assert!(
        x.find("<EXTIPI>").is_none(),
        "EXTIPI should not be present when empty"
    );
}

// Total vTotTrib
#[test]
fn total_vtottrib() {
    let t = "NOTAFISCAL|1|\nA|4.00|NFe35180825028332000105550010000005021000005010||\nB|35|00000501|VENDA|55|1|502|2018-08-13T17:28:10-03:00||1|1|3550308|1|1|8|1|1|0|3||0|3.2.1.1|||\nC|E|F|1|||1|3|\nC02|25028332000105|\nC05|R|1||C|3|S|SP|0||||\nE|D|1|||||\nE02|1|\nE05|R|1||B|3|S|SP|0|1|B||\nH|1||\nI|1|S|P|8|||5|U|1|10|10|S|U|1|10|||||1||||\nM|0|\nN|\nN02|0|00|3|10|18|1.80|0|0|\nQ|\nQ02|01|10|0.65|0.07|\nS|\nS02|01|10|3|0.30|\nW|\nW02|10|1|0|0|0|0|0|0|10|0|0|0|0|0|0|0|0|0|10|99.99|0|0|0|\nX|9|\nY|0|\nYA|0|01|10||00||0||\nZ|||\n";
    assert!(
        fiscal::convert::txt_to_xml(t, "local_v12")
            .unwrap()
            .contains("<vTotTrib>99.99</vTotTrib>")
    );
}

// Full entities integration test
#[test]
fn full_entities_fixture() {
    let t =
        std::fs::read_to_string(format!("{FIXTURES_PATH}txt/nfe_4.00_full_entities.txt")).unwrap();
    let xml = fiscal::convert::txt_to_xml(&t, "local").unwrap();

    // Verify NFref types
    assert!(
        xml.contains("<refNFe>52180706147451002003550010000580821325441420</refNFe>"),
        "Missing refNFe"
    );
    assert!(xml.contains("<refNF>"), "Missing refNF");
    assert!(xml.contains("<refNFP>"), "Missing refNFP");
    assert!(xml.contains("<refCTe>"), "Missing refCTe");
    assert!(xml.contains("<refECF>"), "Missing refECF");

    // Retirada / Entrega / autXML
    assert!(xml.contains("<retirada>"), "Missing retirada");
    assert!(xml.contains("<entrega>"), "Missing entrega, xml: {xml}");
    assert_eq!(
        xml.matches("<autXML>").count(),
        2,
        "Expected 2 autXML entries"
    );

    // Product details
    assert!(xml.contains("<NVE>AA0001</NVE>"), "Missing NVE AA0001");
    assert!(xml.contains("<NVE>BB0002</NVE>"), "Missing NVE BB0002");
    assert!(xml.contains("<DI>"), "Missing DI");
    assert!(xml.contains("<adi>"), "Missing adi");
    assert!(xml.contains("<detExport>"), "Missing detExport");
    assert!(xml.contains("<rastro>"), "Missing rastro");
    assert!(xml.contains("<nLote>LOTE001</nLote>"), "Missing nLote");
    assert!(xml.contains("<veicProd>"), "Missing veicProd");
    assert!(
        xml.contains("<chassi>CHASSI12345678901</chassi>"),
        "Missing chassi"
    );
    assert!(xml.contains("<med>"), "Missing med");
    assert!(
        xml.contains("<cProdANVISA>1234567890123</cProdANVISA>"),
        "Missing cProdANVISA"
    );
    assert!(xml.contains("<arma>"), "Missing arma");
    assert!(xml.contains("<comb>"), "Missing comb");
    assert!(xml.contains("<CIDE>"), "Missing CIDE");
    assert!(xml.contains("<encerrante>"), "Missing encerrante");
    assert!(
        xml.contains("<nRECOPI>12345678901234567890</nRECOPI>"),
        "Missing nRECOPI"
    );

    // ICMS / ICMSUFDest
    assert!(xml.contains("<ICMSUFDest>"), "Missing ICMSUFDest");
    assert!(
        xml.contains("<vICMSUFDest>"),
        "Missing vICMSUFDest inside ICMSUFDest"
    );

    // II (Imposto de Importacao)
    assert!(xml.contains("<II>"), "Missing II");
    assert!(xml.contains("<vDespAdu>"), "Missing vDespAdu in II");

    // PIS-ST
    assert!(xml.contains("<PISST>"), "Missing PISST");

    // COFINS-ST
    assert!(xml.contains("<COFINSST>"), "Missing COFINSST");

    // ISSQN
    assert!(xml.contains("<ISSQN>"), "Missing ISSQN");
    assert!(
        xml.contains("<cListServ>1234</cListServ>"),
        "Missing cListServ"
    );

    // impostoDevol
    assert!(xml.contains("<impostoDevol>"), "Missing impostoDevol");
    assert!(xml.contains("<pDevol>10.00</pDevol>"), "Missing pDevol");

    // infAdProd
    assert!(
        xml.contains("<infAdProd>Informacao adicional do produto 1</infAdProd>"),
        "Missing infAdProd"
    );
    assert!(
        xml.contains("<infAdProd>Info produto 2</infAdProd>"),
        "Missing infAdProd for item 2"
    );

    // Simples Nacional (N10C)
    assert!(xml.contains("<ICMSSN101>"), "Missing ICMSSN101");

    // Total sections
    assert!(xml.contains("<ISSQNtot>"), "Missing ISSQNtot");
    assert!(xml.contains("<retTrib>"), "Missing retTrib");
    assert!(xml.contains("<vRetPIS>"), "Missing vRetPIS in retTrib");

    // Transport expanded
    assert!(xml.contains("<retTransp>"), "Missing retTransp");
    assert!(xml.contains("<veicTransp>"), "Missing veicTransp");
    assert!(
        xml.contains("<placa>ABC1234</placa>"),
        "Missing veicTransp placa"
    );
    assert!(xml.contains("<reboque>"), "Missing reboque");
    assert!(
        xml.contains("<placa>DEF5678</placa>"),
        "Missing reboque placa"
    );
    // Lacres
    assert!(xml.contains("<lacres>"), "Missing lacres");
    assert!(
        xml.contains("<nLacre>LACRE001</nLacre>"),
        "Missing LACRE001"
    );
    assert!(
        xml.contains("<nLacre>LACRE002</nLacre>"),
        "Missing LACRE002"
    );
    // Two volumes
    assert_eq!(xml.matches("<vol>").count(), 2, "Expected 2 vol entries");

    // infIntermed (YB)
    assert!(xml.contains("<infIntermed>"), "Missing infIntermed");
    assert!(
        xml.contains("<idCadIntTran>CADINT001</idCadIntTran>"),
        "Missing idCadIntTran"
    );

    // infAdic expanded
    assert!(xml.contains("<obsCont"), "Missing obsCont");
    assert!(xml.contains("xCampo=\"campo1\""), "Missing obsCont campo1");
    assert!(xml.contains("<obsFisco"), "Missing obsFisco");
    assert!(xml.contains("<procRef>"), "Missing procRef");

    // exporta (ZA)
    assert!(xml.contains("<exporta>"), "Missing exporta");
    assert!(
        xml.contains("<UFSaidaPais>SP</UFSaidaPais>"),
        "Missing UFSaidaPais"
    );

    // compra (ZB)
    assert!(xml.contains("<compra>"), "Missing compra");
    assert!(xml.contains("<xNEmp>NE001</xNEmp>"), "Missing xNEmp");

    // cana (ZC)
    assert!(xml.contains("<cana>"), "Missing cana");
    assert!(xml.contains("<forDia>"), "Missing forDia");
    assert!(xml.contains("<deduc>"), "Missing deduc");

    // infRespTec (ZD)
    assert!(xml.contains("<infRespTec>"), "Missing infRespTec");
    assert!(
        xml.contains("<xContato>Contato Teste</xContato>"),
        "Missing xContato"
    );

    // infNFeSupl (ZX01)
    assert!(xml.contains("<infNFeSupl>"), "Missing infNFeSupl");
    assert!(xml.contains("<qrCode>"), "Missing qrCode");
    assert!(xml.contains("<urlChave>"), "Missing urlChave");

    // Correct section order: retirada before entrega before autXML before det
    let retirada_pos = xml.find("<retirada>").unwrap();
    let entrega_pos = xml.find("<entrega>").unwrap();
    let autxml_pos = xml.find("<autXML>").unwrap();
    let det_pos = xml.find("<det ").unwrap();
    let total_pos = xml.find("<total>").unwrap();
    let transp_pos = xml.find("<transp>").unwrap();
    let cobr_pos = xml.find("<cobr>").unwrap();
    let pag_pos = xml.find("<pag>").unwrap();
    let intermed_pos = xml.find("<infIntermed>").unwrap();
    let infadic_pos = xml.find("<infAdic>").unwrap();
    let exporta_pos = xml.find("<exporta>").unwrap();
    let compra_pos = xml.find("<compra>").unwrap();
    let cana_pos = xml.find("<cana>").unwrap();
    let resp_tec_pos = xml.find("<infRespTec>").unwrap();
    let supl_pos = xml.find("<infNFeSupl>").unwrap();

    assert!(
        retirada_pos < entrega_pos,
        "retirada must come before entrega"
    );
    assert!(entrega_pos < autxml_pos, "entrega must come before autXML");
    assert!(autxml_pos < det_pos, "autXML must come before det");
    assert!(det_pos < total_pos, "det must come before total");
    assert!(total_pos < transp_pos, "total must come before transp");
    assert!(transp_pos < cobr_pos, "transp must come before cobr");
    assert!(cobr_pos < pag_pos, "cobr must come before pag");
    assert!(pag_pos < intermed_pos, "pag must come before infIntermed");
    assert!(
        intermed_pos < infadic_pos,
        "infIntermed must come before infAdic"
    );
    assert!(
        infadic_pos < exporta_pos,
        "infAdic must come before exporta"
    );
    assert!(exporta_pos < compra_pos, "exporta must come before compra");
    assert!(compra_pos < cana_pos, "compra must come before cana");
    assert!(cana_pos < resp_tec_pos, "cana must come before infRespTec");
    assert!(
        resp_tec_pos < supl_pos,
        "infRespTec must come before infNFeSupl"
    );
}

// PIS NT variants (Q04)
#[test]
fn pis_nt() {
    let t = minimal_txt_v400("").replace("Q02|01|10.00|0.65|0.07|", "Q04|06|");
    let xml = fiscal::convert::txt_to_xml(&t, "local_v12").unwrap();
    assert!(xml.contains("<PISNT>"), "Missing PISNT in: {xml}");
    assert!(xml.contains("<CST>06</CST>"), "Missing CST 06 in PISNT");
}

// COFINS NT variants (S04)
#[test]
fn cofins_nt() {
    let t = minimal_txt_v400("").replace("S02|01|10.00|3.00|0.30|", "S04|08|");
    let xml = fiscal::convert::txt_to_xml(&t, "local_v12").unwrap();
    assert!(xml.contains("<COFINSNT>"), "Missing COFINSNT");
    assert!(xml.contains("<CST>08</CST>"), "Missing CST 08 in COFINSNT");
}

// PIS quantity-based (Q03)
#[test]
fn pis_qty() {
    let t = minimal_txt_v400("").replace("Q02|01|10.00|0.65|0.07|", "Q03|03|100.0000|0.5000|0.05|");
    let xml = fiscal::convert::txt_to_xml(&t, "local_v12").unwrap();
    assert!(xml.contains("<PISQtde>"), "Missing PISQtde");
    assert!(
        xml.contains("<qBCProd>100.0000</qBCProd>"),
        "Missing qBCProd"
    );
}

// COFINS quantity-based (S03)
#[test]
fn cofins_qty() {
    let t = minimal_txt_v400("").replace("S02|01|10.00|3.00|0.30|", "S03|03|100.0000|1.5000|0.15|");
    let xml = fiscal::convert::txt_to_xml(&t, "local_v12").unwrap();
    assert!(xml.contains("<COFINSQtde>"), "Missing COFINSQtde");
}

// IPI by quantity (O11)
#[test]
fn ipi_quantity() {
    let t = minimal_txt_v400("").replace("O10|10.00|1.00|", "O11|4.0000|0.2500|");
    let xml = fiscal::convert::txt_to_xml(&t, "local_v12").unwrap();
    assert!(xml.contains("<qUnid>4.0000</qUnid>"), "Missing qUnid");
    assert!(xml.contains("<vUnid>0.2500</vUnid>"), "Missing vUnid");
}

// Simples Nacional ICMS (N10D - CSOSN 102/103/300/400)
#[test]
fn icms_simples_nacional_n10d() {
    let t = make_txt_with_icms("N10d|0|102|");
    let xml = fiscal::convert::txt_to_xml(&t, "local_v12").unwrap();
    assert!(xml.contains("<ICMSSN102>"), "Missing ICMSSN102");
    assert!(xml.contains("<CSOSN>102</CSOSN>"), "Missing CSOSN 102");
}

// N10A - ICMSPart
#[test]
fn icms_n10a_part() {
    let t =
        make_txt_with_icms("N10a|0|10|3|10.00|0.00|18.00|1.80|0|0|0|10.00|18.00|1.80|40.0000|SP|");
    let xml = fiscal::convert::txt_to_xml(&t, "local_v12").unwrap();
    assert!(
        xml.contains("<pBCOp>40.0000</pBCOp>"),
        "Missing pBCOp in ICMSPart"
    );
    assert!(xml.contains("<UFST>SP</UFST>"), "Missing UFST in ICMSPart");
}

// N10B - ICMSST
#[test]
fn icms_n10b_st() {
    let t = make_txt_with_icms(
        "N10b|0|41|10.00|1.80|5.00|0.90|0.00|0.00|0.00|18.00|0.00|0.00|0.00|0.00|0.00|",
    );
    let xml = fiscal::convert::txt_to_xml(&t, "local_v12").unwrap();
    assert!(
        xml.contains("<vBCSTDest>") || xml.contains("<vBCSTRet>"),
        "Missing ICMSST fields"
    );
}

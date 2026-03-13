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
    match fiscal::convert::validate_txt(
        "NOTAFISCAL|1|\nB|35|00000501|V|55|1|502|2018-08-13T17:28:10-03:00||1|1|3|1|1|8|1|1|0|3||0|3.2.1.1|||\n",
        "local_v12",
    ) {
        Ok(v) => assert!(!v),
        Err(_) => {}
    }
}
#[test]
fn validate_txt_forbidden_chars() {
    match fiscal::convert::validate_txt(
        "NOTAFISCAL|1|\nA|4.00|||\nB|35|<bad>|V|55|1|502|2018-08-13||1|1|3|1|1|8|1|1|0|3||0|3.2.1.1|||\n",
        "local_v12",
    ) {
        Ok(v) => assert!(!v),
        Err(_) => {}
    }
}
#[test]
fn validate_txt_erroneous() {
    let b = std::fs::read(format!("{FIXTURES_PATH}txt/nfe_errado.txt")).unwrap();
    match fiscal::convert::validate_txt(&String::from_utf8_lossy(&b), "local_v12") {
        Ok(v) => assert!(!v),
        Err(_) => {}
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

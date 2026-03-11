use fiscal::complement::{attach_inutilizacao, attach_protocol};

const SAMPLE_NFE_XML: &str = "\
<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<NFe xmlns=\"http://www.portalfiscal.inf.br/nfe\">\
<infNFe xmlns=\"http://www.portalfiscal.inf.br/nfe\" versao=\"4.00\" Id=\"NFe35260112345678000199650010000000011123456780\">\
<ide><cUF>35</cUF></ide>\
</infNFe>\
<Signature xmlns=\"http://www.w3.org/2000/09/xmldsig#\">\
<SignedInfo><Reference><DigestValue>abc123digest==</DigestValue></Reference></SignedInfo>\
<SignatureValue>sig==</SignatureValue>\
</Signature>\
</NFe>";

const SAMPLE_PROTOCOL_XML: &str = "\
<retEnviNFe xmlns=\"http://www.portalfiscal.inf.br/nfe\" versao=\"4.00\">\
<tpAmb>2</tpAmb>\
<cStat>104</cStat>\
<protNFe versao=\"4.00\">\
<infProt>\
<cStat>100</cStat>\
<xMotivo>Autorizado o uso da NF-e</xMotivo>\
<chNFe>35260112345678000199650010000000011123456780</chNFe>\
<digVal>abc123digest==</digVal>\
<nProt>141260000000001</nProt>\
<dhRecbto>2026-01-15T10:30:00-03:00</dhRecbto>\
</infProt>\
</protNFe>\
</retEnviNFe>";

// ── attachProtocol ──────────────────────────────────────────────────────────

mod attach_protocol_tests {
    use super::*;

    #[test]
    fn creates_nfe_proc_wrapper_with_nfe_and_prot_nfe() {
        let result = attach_protocol(SAMPLE_NFE_XML, SAMPLE_PROTOCOL_XML).expect("should succeed");
        assert!(result.contains("<nfeProc"));
        assert!(result.contains("versao=\"4.00\""));
        assert!(result.contains("<NFe"));
        assert!(result.contains("<protNFe"));
        assert!(result.contains("</nfeProc>"));
    }

    #[test]
    fn preserves_original_nfe_content() {
        let result = attach_protocol(SAMPLE_NFE_XML, SAMPLE_PROTOCOL_XML).expect("should succeed");
        assert!(result.contains("<cUF>35</cUF>"));
        assert!(result.contains("abc123digest=="));
    }

    #[test]
    fn includes_protocol_number() {
        let result = attach_protocol(SAMPLE_NFE_XML, SAMPLE_PROTOCOL_XML).expect("should succeed");
        assert!(result.contains("141260000000001"));
    }

    #[test]
    fn errors_on_empty_request_xml() {
        let result = attach_protocol("", SAMPLE_PROTOCOL_XML);
        assert!(result.is_err(), "empty request XML should return Err");
    }

    #[test]
    fn errors_on_empty_response_xml() {
        let result = attach_protocol(SAMPLE_NFE_XML, "");
        assert!(result.is_err(), "empty response XML should return Err");
    }

    #[test]
    fn errors_when_nfe_tag_is_missing() {
        let result = attach_protocol("<root>no nfe</root>", SAMPLE_PROTOCOL_XML);
        assert!(result.is_err(), "missing NFe tag should return Err");
    }

    #[test]
    fn produces_nfe_proc_even_with_mismatched_protocol() {
        let mismatch_protocol = SAMPLE_PROTOCOL_XML
            .replace(
                "35260112345678000199650010000000011123456780",
                "99999999999999999999999999999999999999999999",
            )
            .replace("abc123digest==", "wrongdigest==");
        // Should still wrap in nfeProc (uses first available protNFe)
        let result = attach_protocol(SAMPLE_NFE_XML, &mismatch_protocol).expect("should succeed");
        assert!(result.contains("<nfeProc"));
    }
}

// ── attachInutilizacao ──────────────────────────────────────────────────────

mod attach_inutilizacao_tests {
    use super::*;

    #[test]
    fn creates_proc_inut_nfe_wrapper() {
        let inut_request = "\
<inutNFe xmlns=\"http://www.portalfiscal.inf.br/nfe\" versao=\"4.00\">\
<infInut><cStat>102</cStat></infInut>\
</inutNFe>";

        let inut_response = "\
<retInutNFe xmlns=\"http://www.portalfiscal.inf.br/nfe\" versao=\"4.00\">\
<infInut><cStat>102</cStat><xMotivo>Inutilizacao homologada</xMotivo></infInut>\
</retInutNFe>";

        let result = attach_inutilizacao(inut_request, inut_response).expect("should succeed");
        assert!(result.contains("<ProcInutNFe"));
        assert!(result.contains("<inutNFe"));
        assert!(result.contains("<retInutNFe"));
        assert!(result.contains("</ProcInutNFe>"));
    }
}

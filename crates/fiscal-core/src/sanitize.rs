//! ASCII sanitization for XML text content.
//!
//! When `only_ascii` mode is enabled, accented characters (common in Brazilian
//! Portuguese) are replaced by their closest ASCII equivalents before the XML
//! is finalized.  This mirrors the PHP `Strings::squashCharacters` /
//! `Strings::toASCII` behaviour triggered by `Make::setOnlyAscii(true)`.

/// Replace accented characters with their ASCII equivalents.
///
/// The replacement table matches the PHP `sped-common` `Strings::squashCharacters`
/// mapping exactly.  Characters that have no ASCII equivalent are left untouched
/// (they are still valid UTF-8 and will be XML-escaped normally).
///
/// # Examples
///
/// ```
/// use fiscal_core::sanitize::sanitize_to_ascii;
/// assert_eq!(sanitize_to_ascii("São Paulo"), "Sao Paulo");
/// assert_eq!(sanitize_to_ascii("ação"), "acao");
/// assert_eq!(sanitize_to_ascii("café"), "cafe");
/// ```
pub fn sanitize_to_ascii(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            // Lowercase
            'á' | 'à' | 'ã' | 'â' => result.push('a'),
            'é' | 'ê' => result.push('e'),
            'í' => result.push('i'),
            'ó' | 'ô' | 'õ' | 'ö' => result.push('o'),
            'ú' | 'ü' => result.push('u'),
            'ç' => result.push('c'),
            // Uppercase
            'Á' | 'À' | 'Ã' | 'Â' => result.push('A'),
            'É' | 'Ê' => result.push('E'),
            'Í' => result.push('I'),
            'Ó' | 'Ô' | 'Õ' | 'Ö' => result.push('O'),
            'Ú' | 'Ü' => result.push('U'),
            'Ç' => result.push('C'),
            // Everything else passes through unchanged
            other => result.push(other),
        }
    }
    result
}

/// Apply [`sanitize_to_ascii`] to the text content of an XML string, leaving
/// tag names, attribute names, and attribute values like namespaces/IDs intact.
///
/// The function walks through the XML and only transforms text that appears
/// between `>` and `<` (i.e., element text content).  Attribute values are
/// also sanitized (the text between quotes inside tags), except for well-known
/// structural attributes (`xmlns`, `Id`, `versao`).
///
/// This mirrors the PHP behaviour where `Strings::squashCharacters` is applied
/// to field values before they are placed into the DOM — the net effect is that
/// text nodes and most attribute values in the final XML have accents stripped.
pub fn sanitize_xml_text(xml: &str) -> String {
    let mut result = String::with_capacity(xml.len());
    let mut chars = xml.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '<' {
            // We are inside a tag — copy everything until '>' verbatim,
            // but sanitize attribute values (text inside quotes).
            result.push(ch);
            let mut in_attr_value = false;
            let mut attr_quote: char = '"';
            let mut skip_sanitize = false;
            let mut attr_name_buf = String::new();
            let mut collecting_attr_name = false;

            while let Some(&tag_ch) = chars.peek() {
                if in_attr_value {
                    let tag_ch = chars.next().expect("peeked");
                    if tag_ch == attr_quote {
                        // End of attribute value
                        in_attr_value = false;
                        result.push(tag_ch);
                    } else if skip_sanitize {
                        result.push(tag_ch);
                    } else {
                        // Sanitize the attribute value character
                        let sanitized = sanitize_to_ascii(&tag_ch.to_string());
                        result.push_str(&sanitized);
                    }
                } else if tag_ch == '>' {
                    let tag_ch = chars.next().expect("peeked");
                    result.push(tag_ch);
                    break;
                } else if tag_ch == '"' || tag_ch == '\'' {
                    attr_quote = tag_ch;
                    in_attr_value = true;
                    // Check if we should skip sanitization for structural attrs
                    let attr_name = attr_name_buf.trim().trim_end_matches('=');
                    skip_sanitize = matches!(
                        attr_name,
                        "xmlns" | "Id" | "versao" | "encoding" | "version"
                    );
                    attr_name_buf.clear();
                    collecting_attr_name = false;
                    let tag_ch = chars.next().expect("peeked");
                    result.push(tag_ch);
                } else if tag_ch == ' ' || tag_ch == '=' {
                    if tag_ch == ' ' {
                        collecting_attr_name = true;
                        attr_name_buf.clear();
                    } else if tag_ch == '=' {
                        collecting_attr_name = false;
                        attr_name_buf.push('=');
                    }
                    let tag_ch = chars.next().expect("peeked");
                    result.push(tag_ch);
                } else {
                    if collecting_attr_name {
                        attr_name_buf.push(tag_ch);
                    }
                    let tag_ch = chars.next().expect("peeked");
                    result.push(tag_ch);
                }
            }
        } else {
            // We are in text content between tags — sanitize it.
            let sanitized = sanitize_to_ascii(&ch.to_string());
            result.push_str(&sanitized);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_sao_paulo() {
        assert_eq!(sanitize_to_ascii("São Paulo"), "Sao Paulo");
    }

    #[test]
    fn sanitize_acao() {
        assert_eq!(sanitize_to_ascii("ação"), "acao");
    }

    #[test]
    fn sanitize_cafe() {
        assert_eq!(sanitize_to_ascii("café"), "cafe");
    }

    #[test]
    fn sanitize_uppercase() {
        assert_eq!(sanitize_to_ascii("AÇÃO"), "ACAO");
    }

    #[test]
    fn sanitize_mixed() {
        assert_eq!(
            sanitize_to_ascii("Não há cônsul em São José"),
            "Nao ha consul em Sao Jose"
        );
    }

    #[test]
    fn sanitize_already_ascii() {
        assert_eq!(sanitize_to_ascii("Hello World"), "Hello World");
    }

    #[test]
    fn sanitize_empty() {
        assert_eq!(sanitize_to_ascii(""), "");
    }

    #[test]
    fn sanitize_all_accented_lowercase() {
        assert_eq!(sanitize_to_ascii("áàãâéêíóôõöúüç"), "aaaaeeioooouuc");
    }

    #[test]
    fn sanitize_all_accented_uppercase() {
        assert_eq!(sanitize_to_ascii("ÁÀÃÂÉÊÍÓÔÕÖÚÜÇ"), "AAAAEEIOOOOUUC");
    }

    #[test]
    fn sanitize_preserves_numbers_and_punctuation() {
        assert_eq!(
            sanitize_to_ascii("R$ 1.000,00 - ação"),
            "R$ 1.000,00 - acao"
        );
    }

    #[test]
    fn sanitize_xml_text_preserves_tags() {
        let xml = "<xNome>São Paulo</xNome>";
        assert_eq!(sanitize_xml_text(xml), "<xNome>Sao Paulo</xNome>");
    }

    #[test]
    fn sanitize_xml_text_preserves_namespace() {
        let xml = r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe"><xNome>Açúcar</xNome></NFe>"#;
        let result = sanitize_xml_text(xml);
        assert!(result.contains(r#"xmlns="http://www.portalfiscal.inf.br/nfe""#));
        assert!(result.contains("<xNome>Acucar</xNome>"));
    }

    #[test]
    fn sanitize_xml_text_preserves_id() {
        let xml = r#"<infNFe Id="NFe12345678901234567890123456789012345678901234" versao="4.00"><xNome>José</xNome></infNFe>"#;
        let result = sanitize_xml_text(xml);
        assert!(result.contains(r#"Id="NFe12345678901234567890123456789012345678901234""#));
        assert!(result.contains(r#"versao="4.00""#));
        assert!(result.contains("<xNome>Jose</xNome>"));
    }

    #[test]
    fn sanitize_xml_text_multiple_elements() {
        let xml = "<root><a>Ação</a><b>São José</b><c>123</c></root>";
        let result = sanitize_xml_text(xml);
        assert_eq!(result, "<root><a>Acao</a><b>Sao Jose</b><c>123</c></root>");
    }

    #[test]
    fn sanitize_cedilha() {
        assert_eq!(sanitize_to_ascii("Praça da Sé"), "Praca da Se");
    }

    #[test]
    fn sanitize_common_brazilian_words() {
        assert_eq!(sanitize_to_ascii("descrição"), "descricao");
        assert_eq!(sanitize_to_ascii("informação"), "informacao");
        assert_eq!(sanitize_to_ascii("município"), "municipio");
        assert_eq!(
            sanitize_to_ascii("logradouro não disponível"),
            "logradouro nao disponivel"
        );
    }
}

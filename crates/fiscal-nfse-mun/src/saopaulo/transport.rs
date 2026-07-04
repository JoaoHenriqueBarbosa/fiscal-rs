//! Transporte SOAP do `lotenfe.asmx` (PMSP). Método `EnvioLoteRPS` (produção) e
//! `TesteEnvioLoteRPS` (homologação — valida, não gera nota). Ambos recebem
//! `VersaoSchema` + `MensagemXML` (o `PedidoEnvioLoteRPS` assinado, escapado).

#![cfg(feature = "client")]

use crate::error::{MunError, Result};
use crate::model::{Ambiente, EmitOutput, Status};

const SP_NS: &str = "http://www.prefeitura.sp.gov.br/nfe";

/// Escapa XML para ir como conteúdo-texto do `MensagemXML`.
fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Método SOAP conforme ambiente (nome da operação no WSDL).
pub fn metodo(amb: Ambiente) -> &'static str {
    match amb {
        Ambiente::Producao => "EnvioLoteRPS",
        Ambiente::Homologacao => "TesteEnvioLoteRPS",
    }
}

/// SOAPAction conforme método (valores exatos do WSDL — `/ws/` + camelCase).
pub fn soap_action(metodo: &str) -> &'static str {
    match metodo {
        "TesteEnvioLoteRPS" => "http://www.prefeitura.sp.gov.br/nfe/ws/testeenvio",
        "EnvioLoteRPS" => "http://www.prefeitura.sp.gov.br/nfe/ws/envioLoteRPS",
        "EnvioRPS" => "http://www.prefeitura.sp.gov.br/nfe/ws/envioRPS",
        "CancelamentoNFe" => "http://www.prefeitura.sp.gov.br/nfe/ws/cancelamentoNFe",
        "ConsultaNFe" => "http://www.prefeitura.sp.gov.br/nfe/ws/consultaNFe",
        _ => "",
    }
}

/// Monta o envelope SOAP 1.1 do `lotenfe.asmx`. O wrapper do body é
/// `{Metodo}Request{ VersaoSchema, MensagemXML }`. `versao` = 1 (legado) ou 2 (reforma).
pub fn soap_envio(metodo: &str, signed_lote: &str, versao: u8) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<soap:Envelope xmlns:soap=\"http://schemas.xmlsoap.org/soap/envelope/\" xmlns:nfe=\"{SP_NS}\">\
<soap:Body><nfe:{metodo}Request><nfe:VersaoSchema>{versao}</nfe:VersaoSchema>\
<nfe:MensagemXML>{}</nfe:MensagemXML></nfe:{metodo}Request></soap:Body></soap:Envelope>",
        escape(signed_lote)
    )
}

/// Desescapa entidades XML do RetornoXML embutido.
fn unescape(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

fn tag_val(xml: &str, tag: &str) -> Option<String> {
    for open in [format!("<{tag}>"), format!(":{tag}>")] {
        if let Some(i) = xml.find(&open) {
            let rest = &xml[i + open.len()..];
            if let Some(j) = rest.find('<') {
                let v = rest[..j].trim();
                if !v.is_empty() {
                    return Some(v.to_string());
                }
            }
        }
    }
    None
}

/// Interpreta o `RetornoEnvioLoteRPS` (que vem escapado dentro do SOAP).
pub fn parse_retorno(http_status: u16, body: &str) -> EmitOutput {
    let inner = unescape(body);
    let sucesso = tag_val(&inner, "Sucesso")
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let ok = (200..300).contains(&http_status) && sucesso;

    let numero = tag_val(&inner, "NumeroNFe").or_else(|| tag_val(&inner, "NumeroNota"));
    let cod_verif = tag_val(&inner, "CodigoVerificacao");
    let motivo = if ok {
        None
    } else {
        match (tag_val(&inner, "Codigo"), tag_val(&inner, "Descricao")) {
            (Some(c), Some(d)) => Some(format!("{c}: {d}")),
            (_, Some(d)) => Some(d),
            _ => Some(inner.chars().take(600).collect()),
        }
    };

    EmitOutput {
        status: if ok {
            Status::Autorizado
        } else {
            Status::Rejeitado
        },
        numero_nfse: numero,
        codigo_verificacao: cod_verif,
        protocolo: None,
        data_emissao: tag_val(&inner, "DataEmissaoNFe"),
        xml: if ok { Some(inner.clone()) } else { None },
        motivo,
        // SP não devolve URL pública confiável; o painel orienta consultar pelo
        // número + código de verificação no portal da prefeitura.
        link: None,
        raw: body.to_string(),
    }
}

/// POST SOAP ao `lotenfe.asmx`.
pub async fn post_envio(
    http: &reqwest::Client,
    endpoint: &str,
    metodo: &str,
    envelope: &str,
) -> Result<(u16, String)> {
    let resp = http
        .post(endpoint)
        .header("Content-Type", "text/xml; charset=utf-8")
        .header("SOAPAction", soap_action(metodo))
        .body(envelope.to_string())
        .send()
        .await
        .map_err(|e| MunError::Transporte(format!("{e}")))?;
    let status = resp.status().as_u16();
    let body = resp
        .text()
        .await
        .map_err(|e| MunError::Transporte(format!("read body: {e}")))?;
    Ok((status, body))
}

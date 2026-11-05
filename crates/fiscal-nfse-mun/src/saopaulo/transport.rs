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

/// `Codigo: Descricao` de cada bloco `<{grupo}>` (Erro/Alerta) do retorno.
fn mensagens(xml: &str, grupo: &str) -> Vec<String> {
    let (open, close) = (format!("<{grupo}"), format!("</{grupo}>"));
    let mut out = Vec::new();
    let mut rest = xml;
    while let Some(i) = rest.find(&open) {
        let after = &rest[i + open.len()..];
        // evita casar <ErroX>/<Alertas>: o próximo char tem que fechar a tag
        if !after.starts_with('>') && !after.starts_with(' ') {
            rest = after;
            continue;
        }
        let Some(j) = after.find(&close) else { break };
        let bloco = &after[..j];
        match (tag_val(bloco, "Codigo"), tag_val(bloco, "Descricao")) {
            (Some(c), Some(d)) => out.push(format!("{c}: {d}")),
            (None, Some(d)) => out.push(d),
            _ => {}
        }
        rest = &after[j + close.len()..];
    }
    out
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
    // O retorno pode trazer <Alerta> (ex.: 1651, aviso de layout) antes do
    // <Erro> que de fato rejeitou: o motivo prioriza os erros e lista todos.
    let motivo = if ok {
        None
    } else {
        let erros = mensagens(&inner, "Erro");
        let alertas = mensagens(&inner, "Alerta");
        if !erros.is_empty() {
            Some(erros.join(" | "))
        } else if !alertas.is_empty() {
            Some(format!(
                "sem <Erro> no retorno; alertas: {}",
                alertas.join(" | ")
            ))
        } else {
            match (tag_val(&inner, "Codigo"), tag_val(&inner, "Descricao")) {
                (Some(c), Some(d)) => Some(format!("{c}: {d}")),
                (_, Some(d)) => Some(d),
                _ => Some(inner.chars().take(600).collect()),
            }
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
        chave_nacional: tag_val(&inner, "ChaveNotaNacional")
            .filter(|c| c.len() == 50 && c.chars().all(|d| d.is_ascii_digit())),
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

#[cfg(test)]
mod tests_retorno {
    use super::*;

    #[test]
    fn rejeicao_prioriza_erro_sobre_alerta() {
        let body = "<RetornoEnvioLoteRPS><Cabecalho><Sucesso>false</Sucesso></Cabecalho>\
            <Alerta><Codigo>1651</Codigo><Descricao>Atencao layout</Descricao></Alerta>\
            <Erro><Codigo>1234</Codigo><Descricao>Erro real</Descricao></Erro></RetornoEnvioLoteRPS>";
        let out = parse_retorno(200, body);
        assert!(matches!(out.status, Status::Rejeitado));
        assert_eq!(out.motivo.as_deref(), Some("1234: Erro real"));
    }

    #[test]
    fn autorizado_extrai_chave_nacional() {
        let body = "<RetornoEnvioLoteRPS><Cabecalho><Sucesso>true</Sucesso></Cabecalho><ChaveNFeRPS><ChaveNFe>\
            <InscricaoPrestador>48712345</InscricaoPrestador><NumeroNFe>13600</NumeroNFe>\
            <CodigoVerificacao>ZQX54ZBS</CodigoVerificacao>\
            <ChaveNotaNacional>35503081218885949000181000000001360026118378880233</ChaveNotaNacional>\
            </ChaveNFe></ChaveNFeRPS></RetornoEnvioLoteRPS>";
        let out = parse_retorno(200, body);
        assert!(matches!(out.status, Status::Autorizado));
        assert_eq!(out.numero_nfse.as_deref(), Some("13600"));
        assert_eq!(out.codigo_verificacao.as_deref(), Some("ZQX54ZBS"));
        assert_eq!(
            out.chave_nacional.as_deref(),
            Some("35503081218885949000181000000001360026118378880233")
        );
    }

    #[test]
    fn so_alerta_vira_motivo_explicito() {
        let body = "<RetornoEnvioLoteRPS><Cabecalho><Sucesso>false</Sucesso></Cabecalho>\
            <Alerta><Codigo>1651</Codigo><Descricao>Atencao layout</Descricao></Alerta></RetornoEnvioLoteRPS>";
        let out = parse_retorno(200, body);
        assert_eq!(
            out.motivo.as_deref(),
            Some("sem <Erro> no retorno; alertas: 1651: Atencao layout")
        );
    }
}

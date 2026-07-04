//! Transporte SOAP do padrão ABRASF 2.x (operação `GerarNfse` síncrona).
//!
//! Reaproveita o `GerarNfseEnvio` **standalone** (gerado por
//! [`super::build_gerar_nfse`] e assinado): extrai o `<Rps>...</Rps>` e o
//! embrulha no envelope SOAP `nfse.abrasf.org.br > GerarNfse > GerarNfseEnvio`.

#![cfg(feature = "client")]

use crate::error::{MunError, Result};
use crate::model::{EmitOutput, Status};

const SOAP_NS: &str = "http://schemas.xmlsoap.org/soap/envelope/";
const NFSE_SVC_NS: &str = "http://nfse.abrasf.org.br";

/// Extrai o `<Rps>...</Rps>` (tcDeclaracaoPrestacaoServico, já assinado) do
/// `GerarNfseEnvio` standalone.
fn extrai_rps(signed_envio: &str) -> Result<&str> {
    let ini = signed_envio
        .find("<Rps>")
        .or_else(|| signed_envio.find("<Rps "))
        .ok_or_else(|| MunError::Xml("<Rps> não encontrado".into()))?;
    let fim = signed_envio
        .rfind("</Rps>")
        .ok_or_else(|| MunError::Xml("</Rps> não encontrado".into()))?
        + "</Rps>".len();
    Ok(&signed_envio[ini..fim])
}

/// Monta o envelope SOAP do `GerarNfse` a partir do `GerarNfseEnvio` assinado.
pub fn soap_gerar_nfse(signed_envio: &str) -> Result<String> {
    let rps = extrai_rps(signed_envio)?;
    Ok(format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<soapenv:Envelope xmlns:soapenv=\"{SOAP_NS}\" xmlns:nfse=\"{NFSE_SVC_NS}\">\
<soapenv:Body><nfse:GerarNfse><GerarNfseEnvio>{rps}</GerarNfseEnvio></nfse:GerarNfse></soapenv:Body>\
</soapenv:Envelope>"
    ))
}

/// Extrai o primeiro `<tag>valor</tag>` (ignorando namespace/prefixo).
fn tag_val(xml: &str, tag: &str) -> Option<String> {
    // procura `:tag>` ou `<tag>` para casar com/sem prefixo
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

/// Interpreta o retorno do SOAP `GerarNfseResposta`/erros → [`EmitOutput`].
pub fn parse_retorno(http_status: u16, body: &str) -> EmitOutput {
    let numero = tag_val(body, "Numero");
    let cod_verif = tag_val(body, "CodigoVerificacao");
    let autorizado = (200..300).contains(&http_status) && numero.is_some();

    // Erros ABRASF: <MensagemRetorno><Codigo>..</Codigo><Mensagem>..</Mensagem>
    let motivo = if autorizado {
        None
    } else {
        let cod = tag_val(body, "Codigo");
        let msg = tag_val(body, "Mensagem");
        match (cod, msg) {
            (Some(c), Some(m)) => Some(format!("{c}: {m}")),
            (_, Some(m)) => Some(m),
            _ => Some(body.chars().take(500).collect()),
        }
    };

    EmitOutput {
        status: if autorizado {
            Status::Autorizado
        } else {
            Status::Rejeitado
        },
        numero_nfse: numero,
        codigo_verificacao: cod_verif,
        protocolo: None,
        data_emissao: tag_val(body, "DataEmissao"),
        xml: if autorizado {
            Some(body.to_string())
        } else {
            None
        },
        motivo,
        // Alguns provedores ABRASF retornam <Url> com o link de visualização.
        link: tag_val(body, "Url").filter(|u| u.starts_with("http")),
        raw: body.to_string(),
    }
}

/// POST SOAP ao endpoint ABRASF (`GerarNfse`). mTLS via `http`.
pub async fn post_gerar_nfse(
    http: &reqwest::Client,
    endpoint: &str,
    soap_action: &str,
    envelope: &str,
) -> Result<(u16, String)> {
    let resp = http
        .post(endpoint)
        .header("Content-Type", "text/xml; charset=utf-8")
        .header("SOAPAction", soap_action)
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
mod tests {
    use super::*;

    #[test]
    fn parse_retorno_autorizado_extrai_numero_e_link() {
        let body = "<GerarNfseResposta><Numero>123</Numero>\
            <CodigoVerificacao>AB12-CD34</CodigoVerificacao>\
            <Url>https://nfse.exemplo.gov.br/nota/123</Url></GerarNfseResposta>";
        let out = parse_retorno(200, body);
        assert_eq!(out.status, Status::Autorizado);
        assert_eq!(out.numero_nfse.as_deref(), Some("123"));
        assert_eq!(
            out.link.as_deref(),
            Some("https://nfse.exemplo.gov.br/nota/123")
        );
    }

    #[test]
    fn parse_retorno_sem_url_deixa_link_none() {
        let body = "<GerarNfseResposta><Numero>9</Numero></GerarNfseResposta>";
        assert_eq!(parse_retorno(200, body).link, None);
    }

    #[test]
    fn parse_retorno_ignora_url_nao_http() {
        // Um <Url> que não começa com http é descartado (evita lixo no link).
        let body = "<GerarNfseResposta><Numero>9</Numero><Url>n/a</Url></GerarNfseResposta>";
        assert_eq!(parse_retorno(200, body).link, None);
    }

    #[test]
    fn parse_retorno_rejeitado_traz_motivo() {
        let body = "<MensagemRetorno><Codigo>E101</Codigo>\
            <Mensagem>IM inválida</Mensagem></MensagemRetorno>";
        let out = parse_retorno(400, body);
        assert_eq!(out.status, Status::Rejeitado);
        assert_eq!(out.link, None);
        assert_eq!(out.motivo.as_deref(), Some("E101: IM inválida"));
    }
}

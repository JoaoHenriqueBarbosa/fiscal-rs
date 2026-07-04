//! Padrão **ABRASF 2.x** — base comum (DSF/GINFES/SigISS).
//!
//! Monta o `GerarNfseEnvio` (RPS → NFS-e síncrono) a partir do modelo comum.
//! Estrutura (nfse.xsd `http://www.abrasf.org.br/nfse.xsd`):
//!
//! ```text
//! GerarNfseEnvio > Rps > InfDeclaracaoPrestacaoServico[@Id] >
//!   Rps{ IdentificacaoRps{Numero,Serie,Tipo}, DataEmissao, Status }
//!   Competencia, Servico{ Valores, IssRetido, ItemListaServico?, CodigoCnae,
//!     CodigoTributacaoMunicipio?, Discriminacao, CodigoMunicipio, ExigibilidadeISS },
//!   Prestador{ CpfCnpj{Cnpj}, InscricaoMunicipal },
//!   Tomador?, OptanteSimplesNacional, IncentivoFiscal
//!   (+ Signature, adicionada na assinatura)
//! ```

use fiscal_core::xml_utils::{TagContent, tag};

use crate::error::{MunError, Result};
use crate::model::{EmitInput, Tomador};

#[cfg(feature = "client")]
pub mod transport;

/// Namespace dos tipos ABRASF.
pub const ABRASF_NS: &str = "http://www.abrasf.org.br/nfse.xsd";

/// Emissão ABRASF completa (build → assina → SOAP → parse). Reutilizável por
/// todos os provedores ABRASF (DSF/GINFES/SigISS) — só varia `endpoint`/`action`.
#[cfg(feature = "client")]
pub async fn emit(
    input: &EmitInput,
    ctx: &crate::provider::ProviderCtx,
    endpoint: &str,
    soap_action: &str,
) -> Result<crate::model::EmitOutput> {
    let cert = fiscal_crypto::certificate::load_certificate(&ctx.pfx_der, &ctx.senha)
        .map_err(|e| MunError::Assinatura(format!("certificado: {e}")))?;
    let xml = build_gerar_nfse(input)?;
    let signed =
        fiscal_crypto::certificate::sign_abrasf_xml(&xml, &cert.private_key, &cert.certificate)
            .map_err(|e| MunError::Assinatura(format!("{e}")))?;
    let envelope = transport::soap_gerar_nfse(&signed)?;
    let http = ctx.http_client()?;
    let (status, body) =
        transport::post_gerar_nfse(&http, endpoint, soap_action, &envelope).await?;
    Ok(transport::parse_retorno(status, &body))
}

/// centavos → "R$" decimal com 2 casas (ex.: 10000 → "100.00").
fn centavos(v: i64) -> String {
    format!("{}.{:02}", v / 100, (v % 100).abs())
}

/// `YYYY-MM-DD` a partir de um ISO 8601 (corta no T).
fn data(iso: &str) -> &str {
    iso.split('T').next().unwrap_or(iso)
}

/// Monta o `<GerarNfseEnvio>` **não assinado** a partir do modelo comum.
///
/// O `Id` da `InfDeclaracaoPrestacaoServico` é `rps{numero}{serie}` (usado como
/// `Reference URI` na assinatura).
pub fn build_gerar_nfse(input: &EmitInput) -> Result<String> {
    let e = &input.emitente;
    let r = &input.rps;
    let s = &r.servico;
    let cnae = s
        .cnae
        .as_deref()
        .ok_or_else(|| MunError::Validacao("ABRASF exige CodigoCnae".into()))?;
    let c_mun = s.c_mun_prestacao.clone().unwrap_or_else(|| e.c_mun.clone());

    // --- Rps (identificação) ---
    let ident_rps = tag(
        "IdentificacaoRps",
        &[],
        TagContent::Children(vec![
            tag("Numero", &[], TagContent::Text(&r.numero.to_string())),
            tag("Serie", &[], TagContent::Text(&r.serie)),
            tag("Tipo", &[], TagContent::Text(&r.tipo.to_string())),
        ]),
    );
    let rps_block = tag(
        "Rps",
        &[],
        TagContent::Children(vec![
            ident_rps,
            tag("DataEmissao", &[], TagContent::Text(data(&r.data_emissao))),
            tag("Status", &[], TagContent::Text("1")),
        ]),
    );

    // --- Valores ---
    let mut valores = vec![tag(
        "ValorServicos",
        &[],
        TagContent::Text(&centavos(s.valor_centavos)),
    )];
    if let Some(a) = &s.aliquota_iss {
        valores.push(tag("Aliquota", &[], TagContent::Text(a)));
    }
    let valores = tag("Valores", &[], TagContent::Children(valores));

    // --- Servico ---
    let mut servico = vec![valores];
    servico.push(tag(
        "IssRetido",
        &[],
        TagContent::Text(if s.iss_retido { "1" } else { "2" }),
    ));
    if !s.item_lista_servico.trim().is_empty() {
        servico.push(tag(
            "ItemListaServico",
            &[],
            TagContent::Text(&s.item_lista_servico),
        ));
    }
    servico.push(tag("CodigoCnae", &[], TagContent::Text(cnae)));
    if let Some(ct) = &s.cod_tributacao_municipio {
        servico.push(tag("CodigoTributacaoMunicipio", &[], TagContent::Text(ct)));
    }
    servico.push(tag(
        "Discriminacao",
        &[],
        TagContent::Text(&s.discriminacao),
    ));
    servico.push(tag("CodigoMunicipio", &[], TagContent::Text(&c_mun)));
    servico.push(tag("ExigibilidadeISS", &[], TagContent::Text("1")));
    let servico = tag("Servico", &[], TagContent::Children(servico));

    // --- Prestador ---
    let mut prest = vec![tag(
        "CpfCnpj",
        &[],
        TagContent::Children(vec![tag("Cnpj", &[], TagContent::Text(&e.cnpj))]),
    )];
    if let Some(im) = &e.im {
        prest.push(tag("InscricaoMunicipal", &[], TagContent::Text(im)));
    }
    let prestador = tag("Prestador", &[], TagContent::Children(prest));

    // --- InfDeclaracao ---
    let id = format!("rps{}{}", r.numero, r.serie);
    let mut inf = vec![
        rps_block,
        tag("Competencia", &[], TagContent::Text(data(&r.data_emissao))),
        servico,
        prestador,
    ];
    if let Some(t) = build_tomador(&r.tomador) {
        inf.push(t);
    }
    inf.push(tag(
        "OptanteSimplesNacional",
        &[],
        TagContent::Text(if e.optante_simples { "1" } else { "2" }),
    ));
    inf.push(tag("IncentivoFiscal", &[], TagContent::Text("2")));

    let inf_decl = tag(
        "InfDeclaracaoPrestacaoServico",
        &[("Id", &id)],
        TagContent::Children(inf),
    );
    let rps_decl = tag("Rps", &[], TagContent::Children(vec![inf_decl]));
    // elementFormDefault="unqualified": apenas o elemento-raiz é qualificado (via
    // prefixo); os filhos ficam SEM namespace. Por isso declaramos o ns num prefixo
    // e não como default (`xmlns=`), senão os filhos herdariam o namespace.
    Ok(tag(
        "nfse:GerarNfseEnvio",
        &[("xmlns:nfse", ABRASF_NS)],
        TagContent::Children(vec![rps_decl]),
    ))
}

fn build_tomador(t: &Tomador) -> Option<String> {
    let doc = t.doc.as_ref()?;
    let digits: String = doc.chars().filter(|c| c.is_ascii_digit()).collect();
    let cpfcnpj = if digits.len() == 11 {
        tag("Cpf", &[], TagContent::Text(&digits))
    } else {
        tag("Cnpj", &[], TagContent::Text(&digits))
    };
    let mut c = vec![tag(
        "IdentificacaoTomador",
        &[],
        TagContent::Children(vec![tag(
            "CpfCnpj",
            &[],
            TagContent::Children(vec![cpfcnpj]),
        )]),
    )];
    if let Some(rs) = &t.razao_social {
        c.push(tag("RazaoSocial", &[], TagContent::Text(rs)));
    }
    if let Some(em) = &t.email {
        c.push(tag(
            "Contato",
            &[],
            TagContent::Children(vec![tag("Email", &[], TagContent::Text(em))]),
        ));
    }
    Some(tag("Tomador", &[], TagContent::Children(c)))
}

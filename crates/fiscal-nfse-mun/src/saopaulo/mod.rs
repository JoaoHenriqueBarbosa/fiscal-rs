//! Provedor **São Paulo (PMSP)** — sistema próprio, `PedidoEnvioLoteRPS` no
//! `lotenfe.asmx`. Cada RPS leva um campo `<Assinatura>` = Base64(RSA-SHA1(string
//! concatenada de campos)); o lote inteiro é assinado em XMLDSig (URI="").
//!
//! Layout da string de assinatura do RPS (v01):
//!
//! | campo | tam | preenchimento |
//! |---|---|---|
//! | InscricaoPrestador (CCM) | 8 | zeros à esquerda |
//! | SerieRPS | 5 | brancos à direita |
//! | NumeroRPS | 12 | zeros à esquerda |
//! | DataEmissao (AAAAMMDD) | 8 | — |
//! | TributacaoRPS | 1 | — |
//! | StatusRPS | 1 | — |
//! | ISSRetido | 1 | `S`/`N` |
//! | ValorServicos (centavos) | 15 | zeros à esquerda |
//! | ValorDeducoes (centavos) | 15 | zeros à esquerda |
//! | CodigoServico | 5 | zeros à esquerda |
//! | Indicador CPF/CNPJ tomador | 1 | `1` CPF, `2` CNPJ, `3` sem |
//! | CPF/CNPJ tomador | 14 | zeros à esquerda |

use crate::model::{EmitInput, Servico};

#[cfg(feature = "client")]
pub mod transport;

pub const SP_NS: &str = "http://www.prefeitura.sp.gov.br/nfe";

/// Nome do elemento-raiz do lote — usado na assinatura XMLDSig.
///
/// Usamos o estilo de namespace do padrão SP (igual à Invoicy): `xmlns` **default**
/// na raiz + `xmlns=""` nos filhos. Assim NENHUM namespace-prefixo vaza para o
/// `SignedInfo` no C14N inclusivo (o que quebraria a assinatura do lote — 1057).
pub const SP_LOTE_ROOT: &str = "PedidoEnvioLoteRPS";

/// Emissão SP completa: assinatura do RPS → lote → XMLDSig do lote → SOAP → parse.
#[cfg(feature = "client")]
pub async fn emit(
    input: &EmitInput,
    ctx: &crate::provider::ProviderCtx,
    endpoint: &str,
) -> crate::error::Result<crate::model::EmitOutput> {
    use crate::error::MunError;
    let cert = fiscal_crypto::certificate::load_certificate(&ctx.pfx_der, &ctx.senha)
        .map_err(|e| MunError::Assinatura(format!("certificado: {e}")))?;
    let v2 = ctx.versao >= 2;
    // 1. Assinatura do RPS (v1 IM8 ou v2 IM12, conforme config da empresa).
    let assinatura = fiscal_crypto::certificate::rsa_sha1_base64(
        if v2 {
            assinatura_string_v2(input)
        } else {
            assinatura_string(input)
        }
        .as_bytes(),
        &cert.private_key,
    )
    .map_err(|e| MunError::Assinatura(format!("assinatura RPS: {e}")))?;
    // 2. Lote + 3. XMLDSig do lote (URI="").
    let lote = if v2 {
        build_lote_rps_v2(input, &assinatura)
    } else {
        build_lote_rps(input, &assinatura)
    };
    let signed = fiscal_crypto::certificate::sign_sp_lote_xml(
        &lote,
        SP_LOTE_ROOT,
        &cert.private_key,
        &cert.certificate,
    )
    .map_err(|e| MunError::Assinatura(format!("assinatura lote: {e}")))?;
    // 4. SOAP (VersaoSchema = versão) + 5. POST + 6. parse.
    let metodo = transport::metodo(ctx.ambiente);
    let envelope = transport::soap_envio(metodo, &signed, if v2 { 2 } else { 1 });
    let http = ctx.http_client()?;
    let (status, body) = transport::post_envio(&http, endpoint, metodo, &envelope).await?;
    let mut out = transport::parse_retorno(status, &body);
    // Debug: quando rejeitado, guarda o lote assinado enviado (pra auditoria/diff).
    if out.xml.is_none() {
        out.xml = Some(signed);
    }
    Ok(out)
}

/// só dígitos.
fn digits(s: &str) -> String {
    s.chars().filter(|c| c.is_ascii_digit()).collect()
}

/// Cancela uma NFS-e SP (`CancelamentoNFe`). A `AssinaturaCancelamento` é
/// `Base64(RSA-SHA1( IM(8 zeros) + NumeroNFe(12 zeros) ))`.
#[cfg(feature = "client")]
pub async fn cancelar(
    input: &crate::model::CancelInput,
    ctx: &crate::provider::ProviderCtx,
    endpoint: &str,
) -> crate::error::Result<crate::model::EmitOutput> {
    use crate::error::MunError;
    use fiscal_core::xml_utils::{TagContent, tag};
    let cert = fiscal_crypto::certificate::load_certificate(&ctx.pfx_der, &ctx.senha)
        .map_err(|e| MunError::Assinatura(format!("certificado: {e}")))?;
    let im = digits(ctx.inscricao_municipal.as_deref().unwrap_or(""));
    let cnpj = digits(ctx.cnpj.as_deref().unwrap_or(""));
    let num = digits(&input.numero_nfse);
    let cod = input.codigo_verificacao.clone().unwrap_or_default();

    // AssinaturaCancelamento.
    let ass_str = format!("{im:0>8}{num:0>12}");
    let ass = fiscal_crypto::certificate::rsa_sha1_base64(ass_str.as_bytes(), &cert.private_key)
        .map_err(|e| MunError::Assinatura(format!("assinatura cancelamento: {e}")))?;

    let detalhe = tag(
        "Detalhe",
        &[("xmlns", "")],
        TagContent::Children(vec![
            tag(
                "ChaveNFe",
                &[],
                TagContent::Children(vec![
                    tag("InscricaoPrestador", &[], TagContent::Text(&im)),
                    tag("NumeroNFe", &[], TagContent::Text(&num)),
                    tag("CodigoVerificacao", &[], TagContent::Text(&cod)),
                ]),
            ),
            tag("AssinaturaCancelamento", &[], TagContent::Text(&ass)),
        ]),
    );
    let cabecalho = tag(
        "Cabecalho",
        &[("Versao", "1"), ("xmlns", "")],
        TagContent::Children(vec![
            tag(
                "CPFCNPJRemetente",
                &[],
                TagContent::Children(vec![tag("CNPJ", &[], TagContent::Text(&cnpj))]),
            ),
            tag("transacao", &[], TagContent::Text("true")),
        ]),
    );
    let pedido = tag(
        "PedidoCancelamentoNFe",
        &[("xmlns", SP_NS)],
        TagContent::Children(vec![cabecalho, detalhe]),
    );
    let signed = fiscal_crypto::certificate::sign_sp_lote_xml(
        &pedido,
        "PedidoCancelamentoNFe",
        &cert.private_key,
        &cert.certificate,
    )
    .map_err(|e| MunError::Assinatura(format!("assinatura lote: {e}")))?;

    let http = ctx.http_client()?;
    let envelope = transport::soap_envio("CancelamentoNFe", &signed, ctx.versao.max(1));
    let (status, body) =
        transport::post_envio(&http, endpoint, "CancelamentoNFe", &envelope).await?;
    let mut out = transport::parse_retorno(status, &body);
    if matches!(out.status, crate::model::Status::Autorizado) {
        out.status = crate::model::Status::Cancelado;
    }
    Ok(out)
}

/// Consulta uma NFS-e SP por número (`ConsultaNFe`, `ChaveNFe`).
#[cfg(feature = "client")]
pub async fn consultar(
    numero_nfse: &str,
    codigo_verificacao: &str,
    ctx: &crate::provider::ProviderCtx,
    endpoint: &str,
) -> crate::error::Result<crate::model::EmitOutput> {
    use crate::error::MunError;
    use fiscal_core::xml_utils::{TagContent, tag};
    let cert = fiscal_crypto::certificate::load_certificate(&ctx.pfx_der, &ctx.senha)
        .map_err(|e| MunError::Assinatura(format!("certificado: {e}")))?;
    let im = digits(ctx.inscricao_municipal.as_deref().unwrap_or(""));
    let cnpj = digits(ctx.cnpj.as_deref().unwrap_or(""));
    let num = digits(numero_nfse);

    let detalhe = tag(
        "Detalhe",
        &[("xmlns", "")],
        TagContent::Children(vec![tag(
            "ChaveNFe",
            &[],
            TagContent::Children(vec![
                tag("InscricaoPrestador", &[], TagContent::Text(&im)),
                tag("NumeroNFe", &[], TagContent::Text(&num)),
                tag(
                    "CodigoVerificacao",
                    &[],
                    TagContent::Text(codigo_verificacao),
                ),
            ]),
        )]),
    );
    let cabecalho = tag(
        "Cabecalho",
        &[("Versao", "1"), ("xmlns", "")],
        TagContent::Children(vec![tag(
            "CPFCNPJRemetente",
            &[],
            TagContent::Children(vec![tag("CNPJ", &[], TagContent::Text(&cnpj))]),
        )]),
    );
    let pedido = tag(
        "PedidoConsultaNFe",
        &[("xmlns", SP_NS)],
        TagContent::Children(vec![cabecalho, detalhe]),
    );
    let signed = fiscal_crypto::certificate::sign_sp_lote_xml(
        &pedido,
        "PedidoConsultaNFe",
        &cert.private_key,
        &cert.certificate,
    )
    .map_err(|e| MunError::Assinatura(format!("assinatura lote: {e}")))?;

    let http = ctx.http_client()?;
    let envelope = transport::soap_envio("ConsultaNFe", &signed, ctx.versao.max(1));
    let (status, body) = transport::post_envio(&http, endpoint, "ConsultaNFe", &envelope).await?;
    Ok(transport::parse_retorno(status, &body))
}

/// Assinatura RPS **v1** (Inscrição Municipal com 8 posições).
pub fn assinatura_string(input: &EmitInput) -> String {
    assinatura_string_w(input, 8)
}

/// Assinatura RPS **v2** (reforma — Inscrição Municipal com 12 posições).
pub fn assinatura_string_v2(input: &EmitInput) -> String {
    assinatura_string_w(input, 12)
}

/// Monta a **string** a ser assinada (RSA-SHA1), com `im_width` posições na IM.
fn assinatura_string_w(input: &EmitInput, im_width: usize) -> String {
    let e = &input.emitente;
    let r = &input.rps;
    let s = &r.servico;

    let im = digits(e.im.as_deref().unwrap_or(""));
    let data = digits(r.data_emissao.split('T').next().unwrap_or("")); // AAAAMMDD
    let tributacao = "T"; // tributado no município (default)
    let status = "N"; // normal
    let iss = if s.iss_retido { "S" } else { "N" };
    let cod_serv = digits(s.cod_tributacao_municipio.as_deref().unwrap_or(""));

    let (ind, doc) = match &r.tomador.doc {
        Some(d) => {
            let dd = digits(d);
            if dd.len() == 11 { ("1", dd) } else { ("2", dd) }
        }
        None => ("3", String::new()),
    };

    // Início: campos 1..12 (IM..CPF/CNPJ tomador).
    let mut out = format!(
        "{im:0>im_width$}{serie:<5}{num:0>12}{data}{trib}{status}{iss}{vs:0>15}{vd:0>15}{cod:0>5}{ind}{doc:0>14}",
        im = im,
        im_width = im_width,
        serie = r.serie,
        num = r.numero,
        data = data,
        trib = tributacao,
        status = status,
        iss = iss,
        vs = s.valor_centavos,
        vd = s.valor_deducoes_centavos,
        cod = cod_serv,
        ind = ind,
        doc = doc,
    );
    // Cauda do intermediário: o Indicador+CPF/CNPJ (campos 13/14) só entram QUANDO há
    // intermediário; o ISSRetidoIntermediario (campo 15, S/N) entra sempre (o RPS sempre
    // o carrega — `false` por padrão). Confirmado contra envio real da Invoicy.
    match &r.intermediario {
        Some(i) => {
            let dd = digits(&i.doc);
            out.push_str(if dd.len() == 11 { "1" } else { "2" });
            out.push_str(&format!("{dd:0>14}"));
            out.push_str(if i.iss_retido { "S" } else { "N" });
        }
        None => out.push('N'),
    }
    out
}

/// `centavos` → decimal "X.XX" (tpValor).
fn valor(c: i64) -> String {
    format!("{}.{:02}", c / 100, (c % 100).abs())
}

/// Alíquota percentual ("2.00") → fração tpAliquota (5/4): "0.0200".
fn aliquota_fracao(percent: &str) -> String {
    let p: f64 = percent.replace(',', ".").parse().unwrap_or(0.0);
    format!("{:.4}", p / 100.0)
}

fn cpfcnpj_tag(wrapper: &str, doc: &str) -> String {
    use fiscal_core::xml_utils::{TagContent, tag};
    let d = digits(doc);
    let inner = if d.len() == 11 {
        tag("CPF", &[], TagContent::Text(&d))
    } else {
        tag("CNPJ", &[], TagContent::Text(&d))
    };
    tag(wrapper, &[], TagContent::Children(vec![inner]))
}

/// Monta o `<PedidoEnvioLoteRPS>` (1 RPS) com a `Assinatura` já calculada.
/// A `<Signature>` do lote (XMLDSig) é adicionada na etapa de assinatura.
pub fn build_lote_rps(input: &EmitInput, assinatura_b64: &str) -> String {
    use fiscal_core::xml_utils::{TagContent, tag};
    let e = &input.emitente;
    let r = &input.rps;
    let s = &r.servico;
    let data = r.data_emissao.split('T').next().unwrap_or("");

    // Cabecalho (xmlns="" reseta para sem-namespace, estilo SP/Invoicy)
    let cabecalho = tag(
        "Cabecalho",
        &[("Versao", "1"), ("xmlns", "")],
        TagContent::Children(vec![
            tag(
                "CPFCNPJRemetente",
                &[],
                TagContent::Children(vec![tag("CNPJ", &[], TagContent::Text(&e.cnpj))]),
            ),
            tag("transacao", &[], TagContent::Text("false")),
            tag("dtInicio", &[], TagContent::Text(data)),
            tag("dtFim", &[], TagContent::Text(data)),
            tag("QtdRPS", &[], TagContent::Text("1")),
            tag(
                "ValorTotalServicos",
                &[],
                TagContent::Text(&valor(s.valor_centavos)),
            ),
            tag("ValorTotalDeducoes", &[], TagContent::Text("0.00")),
        ]),
    );

    // RPS
    let chave = tag(
        "ChaveRPS",
        &[],
        TagContent::Children(vec![
            tag(
                "InscricaoPrestador",
                &[],
                TagContent::Text(&digits(e.im.as_deref().unwrap_or(""))),
            ),
            tag("SerieRPS", &[], TagContent::Text(&r.serie)),
            tag("NumeroRPS", &[], TagContent::Text(&r.numero.to_string())),
        ]),
    );
    let mut rps = vec![
        tag("Assinatura", &[], TagContent::Text(assinatura_b64)),
        chave,
        tag("TipoRPS", &[], TagContent::Text("RPS")),
        tag("DataEmissao", &[], TagContent::Text(data)),
        tag("StatusRPS", &[], TagContent::Text("N")),
        tag("TributacaoRPS", &[], TagContent::Text("T")),
        tag(
            "ValorServicos",
            &[],
            TagContent::Text(&valor(s.valor_centavos)),
        ),
        tag(
            "ValorDeducoes",
            &[],
            TagContent::Text(&valor(s.valor_deducoes_centavos)),
        ),
        tag(
            "CodigoServico",
            &[],
            TagContent::Text(&digits(s.cod_tributacao_municipio.as_deref().unwrap_or(""))),
        ),
        tag(
            "AliquotaServicos",
            &[],
            TagContent::Text(&aliquota_fracao(s.aliquota_iss.as_deref().unwrap_or("0"))),
        ),
        tag(
            "ISSRetido",
            &[],
            TagContent::Text(if s.iss_retido { "true" } else { "false" }),
        ),
    ];
    if let Some(doc) = &r.tomador.doc {
        rps.push(cpfcnpj_tag("CPFCNPJTomador", doc));
    }
    if let Some(rs) = &r.tomador.razao_social {
        rps.push(tag("RazaoSocialTomador", &[], TagContent::Text(rs)));
    }
    if let Some(em) = &r.tomador.email {
        rps.push(tag("EmailTomador", &[], TagContent::Text(em)));
    }
    // Intermediário: CPF/CNPJ + IM só quando há; ISSRetidoIntermediario SEMPRE (Invoicy manda
    // `false` mesmo sem intermediário — e a assinatura depende disso).
    let iss_int = if let Some(i) = &r.intermediario {
        rps.push(cpfcnpj_tag("CPFCNPJIntermediario", &i.doc));
        if let Some(im) = &i.im {
            rps.push(tag(
                "InscricaoMunicipalIntermediario",
                &[],
                TagContent::Text(im),
            ));
        }
        i.iss_retido
    } else {
        false
    };
    rps.push(tag(
        "ISSRetidoIntermediario",
        &[],
        TagContent::Text(if iss_int { "true" } else { "false" }),
    ));
    rps.push(discriminacao(s));
    let rps_el = tag("RPS", &[("xmlns", "")], TagContent::Children(rps));

    // Estilo SP/Invoicy: xmlns default na raiz + xmlns="" nos filhos (sem prefixo,
    // pra não vazar namespace no SignedInfo do C14N do lote).
    tag(
        "PedidoEnvioLoteRPS",
        &[("xmlns", SP_NS)],
        TagContent::Children(vec![cabecalho, rps_el]),
    )
}

fn discriminacao(s: &Servico) -> String {
    use fiscal_core::xml_utils::{TagContent, tag};
    tag("Discriminacao", &[], TagContent::Text(&s.discriminacao))
}

/// Monta o `<PedidoEnvioLoteRPS>` **versão 2** (reforma — IM 12 díg, IBS/CBS).
/// A `<Signature>` do lote é adicionada na assinatura.
pub fn build_lote_rps_v2(input: &EmitInput, assinatura_b64: &str) -> String {
    use fiscal_core::xml_utils::{TagContent, tag};
    let e = &input.emitente;
    let r = &input.rps;
    let s = &r.servico;
    let data = r.data_emissao.split('T').next().unwrap_or("");
    let z = "0.00";

    let cabecalho = tag(
        "Cabecalho",
        &[("Versao", "2"), ("xmlns", "")],
        TagContent::Children(vec![
            tag(
                "CPFCNPJRemetente",
                &[],
                TagContent::Children(vec![tag("CNPJ", &[], TagContent::Text(&e.cnpj))]),
            ),
            tag("transacao", &[], TagContent::Text("false")),
            tag("dtInicio", &[], TagContent::Text(data)),
            tag("dtFim", &[], TagContent::Text(data)),
            tag("QtdRPS", &[], TagContent::Text("1")),
        ]),
    );

    let chave = tag(
        "ChaveRPS",
        &[],
        TagContent::Children(vec![
            tag(
                "InscricaoPrestador",
                &[],
                TagContent::Text(&digits(e.im.as_deref().unwrap_or(""))),
            ),
            tag("SerieRPS", &[], TagContent::Text(&r.serie)),
            tag("NumeroRPS", &[], TagContent::Text(&r.numero.to_string())),
        ]),
    );

    // IBSCBS (reforma): finNFSe, indFinal, cIndOp, indDest, valores>trib>gIBSCBS>cClassTrib.
    let g_ibscbs = tag(
        "gIBSCBS",
        &[],
        TagContent::Children(vec![tag(
            "cClassTrib",
            &[],
            TagContent::Text(s.c_class_trib.as_deref().unwrap_or("000001")),
        )]),
    );
    let ibscbs = tag(
        "IBSCBS",
        &[],
        TagContent::Children(vec![
            tag("finNFSe", &[], TagContent::Text("0")),
            tag("indFinal", &[], TagContent::Text("0")),
            tag(
                "cIndOp",
                &[],
                TagContent::Text(s.c_ind_op.as_deref().unwrap_or("100101")),
            ),
            tag("indDest", &[], TagContent::Text("0")),
            tag(
                "valores",
                &[],
                TagContent::Children(vec![tag("trib", &[], TagContent::Children(vec![g_ibscbs]))]),
            ),
        ]),
    );

    let mut rps = vec![
        tag("Assinatura", &[], TagContent::Text(assinatura_b64)),
        chave,
        tag("TipoRPS", &[], TagContent::Text("RPS")),
        tag("DataEmissao", &[], TagContent::Text(data)),
        tag("StatusRPS", &[], TagContent::Text("N")),
        tag("TributacaoRPS", &[], TagContent::Text("T")),
        tag(
            "ValorDeducoes",
            &[],
            TagContent::Text(&valor(s.valor_deducoes_centavos)),
        ),
        tag("ValorPIS", &[], TagContent::Text(z)),
        tag("ValorCOFINS", &[], TagContent::Text(z)),
        tag("ValorINSS", &[], TagContent::Text(z)),
        tag("ValorIR", &[], TagContent::Text(z)),
        tag("ValorCSLL", &[], TagContent::Text(z)),
        tag(
            "CodigoServico",
            &[],
            TagContent::Text(&digits(s.cod_tributacao_municipio.as_deref().unwrap_or(""))),
        ),
        tag(
            "AliquotaServicos",
            &[],
            TagContent::Text(&aliquota_fracao(s.aliquota_iss.as_deref().unwrap_or("0"))),
        ),
        tag(
            "ISSRetido",
            &[],
            TagContent::Text(if s.iss_retido { "true" } else { "false" }),
        ),
    ];
    if let Some(doc) = &r.tomador.doc {
        rps.push(cpfcnpj_tag("CPFCNPJTomador", doc));
    }
    if let Some(rs) = &r.tomador.razao_social {
        rps.push(tag("RazaoSocialTomador", &[], TagContent::Text(rs)));
    }
    if let Some(em) = &r.tomador.email {
        rps.push(tag("EmailTomador", &[], TagContent::Text(em)));
    }
    rps.push(discriminacao(s));
    // choice: ValorInicialCobrado OU ValorFinalCobrado (só um).
    rps.push(tag(
        "ValorInicialCobrado",
        &[],
        TagContent::Text(&valor(s.valor_centavos)),
    ));
    rps.push(tag("ValorIPI", &[], TagContent::Text(z)));
    rps.push(tag("ExigibilidadeSuspensa", &[], TagContent::Text("0")));
    rps.push(tag(
        "PagamentoParceladoAntecipado",
        &[],
        TagContent::Text("0"),
    ));
    rps.push(tag(
        "NBS",
        &[],
        TagContent::Text(s.nbs.as_deref().unwrap_or("000000000")),
    ));
    // gpPrestacao (choice): cLocPrestacao (IBGE) OU cPaisPrestacao.
    let c_loc = s.c_mun_prestacao.clone().unwrap_or_else(|| e.c_mun.clone());
    rps.push(tag("cLocPrestacao", &[], TagContent::Text(&c_loc)));
    rps.push(ibscbs);
    let rps_el = tag("RPS", &[("xmlns", "")], TagContent::Children(rps));

    tag(
        "PedidoEnvioLoteRPS",
        &[("xmlns", SP_NS)],
        TagContent::Children(vec![cabecalho, rps_el]),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;

    fn sample() -> EmitInput {
        EmitInput {
            emitente: Emitente {
                cnpj: "18885949000181".into(),
                im: Some("12345678".into()),
                razao_social: "CENTRE LTDA".into(),
                c_mun: "3550308".into(),
                uf: "SP".into(),
                endereco: None,
                optante_simples: false,
            },
            rps: Rps {
                numero: 7,
                serie: "TST".into(),
                tipo: 1,
                data_emissao: "2026-06-06T10:00:00-03:00".into(),
                tomador: Tomador {
                    doc: Some("11222333000181".into()),
                    razao_social: Some("TOMADOR LTDA".into()),
                    email: None,
                    endereco: None,
                    im: None,
                },
                servico: Servico {
                    valor_centavos: 10000,
                    valor_deducoes_centavos: 0,
                    aliquota_iss: Some("2.00".into()),
                    iss_retido: false,
                    item_lista_servico: "1.01".into(),
                    cod_tributacao_municipio: Some("02916".into()),
                    cnae: None,
                    discriminacao: "TESTE".into(),
                    c_mun_prestacao: None,
                    nbs: None,
                    c_class_trib: None,
                    c_ind_op: None,
                },
                natureza_operacao: None,
                regime_especial_tributacao: None,
                incentivador_cultural: false,
                intermediario: None,
            },
        }
    }

    #[test]
    fn assinatura_layout_exato() {
        let a = assinatura_string(&sample());
        // 8+5+12+8+1+1+1+15+15+5+1+14 +1 = 87 caracteres (sem intermediário: só ISSRetidoInterm)
        assert_eq!(a.len(), 87, "string: {a:?}");
        assert_eq!(&a[0..8], "12345678"); // IM 8 (já tem 8)
        assert_eq!(&a[8..13], "TST  "); // série 5, brancos à direita
        assert_eq!(&a[13..25], "000000000007"); // número 12 zero-left
        assert_eq!(&a[25..33], "20260606"); // data AAAAMMDD
        assert_eq!(&a[33..36], "TNN"); // tributacao, status, iss(não retido)
        assert_eq!(&a[36..51], "000000000010000"); // valor 10000 centavos
        assert_eq!(&a[51..66], "000000000000000"); // deducoes 0
        assert_eq!(&a[66..71], "02916"); // codigo servico 5
        assert_eq!(&a[71..72], "2"); // indicador CNPJ
        assert_eq!(&a[72..86], "11222333000181"); // CNPJ 14
        assert_eq!(&a[86..87], "N"); // ISS retido intermediário (sem intermediário → só "N")
    }

    /// Reproduz a string de um **envio real da Invoicy** (RPS 8899, sem intermediário),
    /// cuja `<Assinatura>` foi verificada contra o certificado do prestador.
    #[test]
    fn assinatura_envio_real_invoicy() {
        let inp = EmitInput {
            emitente: Emitente {
                cnpj: "18885949000181".into(),
                im: Some("48712345".into()),
                razao_social: "x".into(),
                c_mun: "3550308".into(),
                uf: "SP".into(),
                endereco: None,
                optante_simples: true,
            },
            rps: Rps {
                numero: 8899,
                serie: "99".into(),
                tipo: 1,
                data_emissao: "2026-05-27".into(),
                tomador: Tomador {
                    doc: Some("22175916000115".into()),
                    ..Default::default()
                },
                servico: Servico {
                    valor_centavos: 29000,
                    valor_deducoes_centavos: 0,
                    aliquota_iss: Some("0".into()),
                    iss_retido: false,
                    item_lista_servico: String::new(),
                    cod_tributacao_municipio: Some("07498".into()),
                    cnae: None,
                    discriminacao: String::new(),
                    c_mun_prestacao: None,
                    nbs: None,
                    c_class_trib: None,
                    c_ind_op: None,
                },
                natureza_operacao: None,
                regime_especial_tributacao: None,
                incentivador_cultural: false,
                intermediario: None,
            },
        };
        assert_eq!(
            assinatura_string(&inp),
            "4871234599   00000000889920260527TNN00000000002900000000000000000007498222175916000115N"
        );
    }

    /// Reproduz EXATAMENTE o exemplo do Manual Web Service SP (assinatura RPS v1).
    #[test]
    fn assinatura_exemplo_oficial_manual() {
        let inp = EmitInput {
            emitente: Emitente {
                cnpj: "x".into(),
                im: Some("31000000".into()),
                razao_social: "x".into(),
                c_mun: "3550308".into(),
                uf: "SP".into(),
                endereco: None,
                optante_simples: false,
            },
            rps: Rps {
                numero: 1,
                serie: "OL03".into(),
                tipo: 1,
                data_emissao: "2007-01-03".into(),
                tomador: Tomador {
                    doc: Some("13167474254".into()),
                    ..Default::default()
                },
                servico: Servico {
                    valor_centavos: 2050000,         // R$20.500,00
                    valor_deducoes_centavos: 500000, // R$5.000,00
                    aliquota_iss: Some("5.00".into()),
                    iss_retido: false,
                    item_lista_servico: String::new(),
                    cod_tributacao_municipio: Some("2658".into()),
                    cnae: None,
                    discriminacao: String::new(),
                    c_mun_prestacao: None,
                    nbs: None,
                    c_class_trib: None,
                    c_ind_op: None,
                },
                natureza_operacao: None,
                regime_especial_tributacao: None,
                incentivador_cultural: false,
                intermediario: Some(Intermediario {
                    doc: "09999999000106".into(),
                    im: Some("99999999".into()),
                    iss_retido: true,
                }),
            },
        };
        // Do manual SP (série "OL03" + 1 espaço à direita):
        let esperado = String::new()
            + "31000000"        // IM 8
            + "OL03 "           // série 5 (espaço à direita)
            + "000000000001"    // número 12
            + "20070103"        // data
            + "T" + "N" + "N"   // trib, status, ISS retido
            + "000000002050000" // valor serviços (R$20.500,00)
            + "000000000500000" // deduções (R$5.000,00)
            + "02658"           // código serviço 5
            + "1" + "00013167474254"  // tomador CPF
            + "2" + "09999999000106"  // intermediário CNPJ
            + "S"; // ISS retido intermediário
        assert_eq!(assinatura_string(&inp), esperado);
    }
}

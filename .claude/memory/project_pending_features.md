---
name: Inventário de disparidades PHP vs Rust
description: Lista de funcionalidades do PHP sped-nfe que faltam ou divergem no fiscal-rs. Itens resolvidos marcados com ✅.
type: project
---

## Inventário de Disparidades PHP → Rust (atualizado 2026-03-13)

### RESOLVIDAS (53)

#### Rounds 7-8 (39 itens — resolvidos em sessões anteriores)
1-39. ✅ Ver histórico detalhado em commits anteriores.

#### Round 9 — Auditoria cega completa com execução real (10 itens)
40. ✅ attach_b2b() ordem atributos — `xmlns` antes de `versao` (paridade DOM PHP)
41. ✅ xCondUso acento Ator Interessado — `"destinatário"` com acento correto
42. ✅ `<vTotTrib>` por item em `<imposto>` — emitir como primeiro filho (Lei 12.741/2012)
43. ✅ `<pIPI>` escala — corrigido de format_rate4 (÷10000) para format_rate_4 (÷100)
44. ✅ Formatação de decimais no convert.rs — pad_decimal() para TDec_1110v (10), TDec_0302a04 (4), TDec_1302 (2)
45. ✅ `<card>` tpIntegra=0 — só gera para valores "1" ou "2" (XSD enum)
46. ✅ 80 entidades TXT em structure_400() — 137/137 entidades + parser + XML builder expandidos
47. ✅ `<gCred>` posição no convert.rs — movido para entre cBenef e EXTIPI (XSD sequence)
48. ✅ sefazEnviaLote multi-XML — authorize_batch() + _compressed + _nfce, até 50 NF-e
49. ✅ Standardize xmlns/wrapper — unwrap raiz + strip xmlns (paridade PHP toStd)
50. ✅ SHA-256 assinatura — SignatureAlgorithm enum, sign_*_with_algorithm(), Sha1 default

#### Round 10 — Auditoria cega completa + refatoração convert (3 itens)
51. ✅ TXT converter ~80 entidades — handle() expandido para todas as entidades, build_* completos (retirada, entrega, autXML, DI, detExport, rastro, veicProd, med, arma, comb, Simples Nacional, ICMSUFDest, II, PIS/COFINS variantes, ISSQN, IBS/CBS, impostoDevol, transport details, obsCont/obsFisco/procRef, exporta, compra, cana, infRespTec, infNFeSupl)
52. ✅ validKeyByUF — AccessKey::validate_uf() valida cUF da chave contra UF esperada (Rejeição 614)
53. ✅ convert.rs refatorado — monolito de 3148 linhas dividido em 12 módulos (nenhum acima de 420 linhas)

### DESCARTADAS (legislação discorda do PHP ou diferença idiomática)
- GTIN-14 com zero — Rust correto, PHP errado (GS1 permite indicator digit 0)
- Espaços duplos infCpl — Rust mais correto (C14N preserva whitespace em text nodes)
- Declaração XML — ambos aceitáveis (C14N remove)
- DocumentsException 18 códigos — diferença idiomática Rust (enum flat)
- Hierarquia exceções — diferença idiomática Rust
- parseDump/dump/simpleXml/__toString — facilidades PHP, não exigidas por legislação
- vTotTrib em ICMSTot quando zero — ambos válidos (campo opcional)
- loadSoapClass/setVerAplic/canonicalOptions — diferenças arquiteturais
- getAcronym (cUF → sigla) — já implementado como get_state_by_code()
- ignoreContingency — diferença arquitetural (Rust não tem estado implícito de contingência)
- cstat.json ~440 códigos — conveniência operacional (xMotivo já vem nas respostas SEFAZ)
- Standardize::toStd() attributes key — decisão de design documentada
- ValidTXT::loadStructure() pública — API interna, sem impacto funcional
- Standardize::$key — estado efêmero, sem impacto funcional
- Parser::getErrors() — Rust usa Result<>, padrão idiomático

### PENDENTES (0)
Nenhuma disparidade conhecida pendente.

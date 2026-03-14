---
name: Mapa detalhado ACBr NFe/NFCe vs fiscal-rs
description: Correspondência arquivo-a-arquivo entre ACBr NFe e fiscal-rs, com gaps identificados e achados de validação
type: reference
---

## Correspondência de arquivos: ACBr NFe → fiscal-rs

### Tipos e Classes
| ACBr | fiscal-rs | Notas |
|------|-----------|-------|
| `Base/ACBrNFe.Classes.pas` (5848 linhas) | `fiscal-core/src/types/` (3956 linhas total) | Nosso está bem mapeado, dividido em item.rs, product.rs, transport.rs etc. |
| `Base/ACBrNFe.Conversao.pas` (1828 linhas) | `fiscal-core/src/types/enums.rs` (280 linhas) | **GAP**: ACBr tem muito mais enums (formas de pagamento, bandeiras, via de transporte, tipo de intermediação, CST IPI, indicadores diversos). Nosso enums.rs é bem menor. |
| `Base/ACBrNFe.Consts.pas` (421 linhas) | `fiscal-core/src/constants.rs` | ACBr tem resourcestrings com descrição de cada campo — útil para mensagens de erro/validação. Inclui constantes da Reforma Tributária (IBS/CBS/IS). |

### Geração e Leitura XML
| ACBr | fiscal-rs | Notas |
|------|-----------|-------|
| `Base/ACBrNFe.XmlWriter.pas` (4810 linhas) | `fiscal-core/src/xml_builder/` (~6000 linhas sem testes) | Boa paridade. Nosso é modular (builder, ide, emit, dest, det, total, transp, pag, optional). |
| `Base/ACBrNFe.XmlReader.pas` | `fiscal-core/src/convert/parser.rs` | Leitura/parse XML |
| `PCNNFe/pcnNFeW.pas` (3196 linhas) | — | Gerador XML legado (PCN). Código paralelo ao XmlWriter moderno. |
| `PCNNFe/pcnNFeR.pas` | — | Leitor XML legado |

### Validação de Regras de Negócio
| ACBr | fiscal-rs | Notas |
|------|-----------|-------|
| `Base/ACBrNFe.ValidarRegrasdeNegocio.pas` (1305 linhas) | `fiscal-sefaz/src/validate.rs` (apenas validação XML estrutural) | **GAP IMPORTANTE**: ACBr valida ~80 regras de negócio da SEFAZ (códigos 207-899) antes do envio. Nós só validamos estrutura XML. |

### WebServices e Comunicação SEFAZ
| ACBr | fiscal-rs | Notas |
|------|-----------|-------|
| `ACBrNFeWebServices.pas` (4367 linhas) | `fiscal-sefaz/src/client/` (authorize, events, delivery, rtc) | Boa cobertura |
| `ACBrNFeConfiguracoes.pas` | `fiscal-core/src/config.rs` | Configurações |
| `Base/Servicos/ACBrNFe.ConsSit.pas` | `fiscal-sefaz/src/response_parsers/consult.rs` | Consulta situação |
| `Base/Servicos/ACBrNFe.EnvEvento.pas` | `fiscal-sefaz/src/request_builders/eventos.rs` | Eventos |
| `Base/Servicos/ACBrNFe.Inut.pas` | `fiscal-sefaz/src/response_parsers/inutilize.rs` | Inutilização |
| `Base/Servicos/ACBrNFe.AdmCSC.pas` | — | **GAP**: Administração de CSC (NFC-e) |
| `Base/Servicos/ACBrNFe.EventoClass.pas` | `fiscal-core/src/complement/event.rs` | Tipos de evento |

### Serialização INI/JSON
| ACBr | fiscal-rs | Notas |
|------|-----------|-------|
| `Base/ACBrNFe.IniReader.pas` / `IniWriter.pas` | — | **N/A**: Formato INI é específico do ecossistema Delphi |
| `Base/ACBrNFe.JSONReader.pas` / `JSONWriter.pas` | — | **Possível GAP**: Serialização JSON. Em Rust provavelmente usaríamos serde. |

### Impressão (DANFE)
| ACBr | fiscal-rs | Notas |
|------|-----------|-------|
| `DANFE/` (Fast, Fortes, FPDF, LazReport, EscPOS) | — | **GAP futuro**: Geração de DANFE/DANFCE. Múltiplos engines no ACBr. |

## Achados importantes da validação ACBr

O `ValidarRegrasdeNegocio.pas` valida em 3 fases:
1. **Regras comuns** (NFe + NFCe): UF, data emissão, totais, contingência
2. **Regras exclusivas NFe (modelo 55)**: data saída, formato DANFE, referenciadas, destinatário, fatura/duplicatas, CST/CSOSN por CFOP
3. **Regras exclusivas NFCe (modelo 65)**: data atrasada (>10min), sem entrada, sem interestadual, sem referenciada, sem cobrança, sem cana, CFOP limitado (5101,5102,5103,5104,5115,5405,5656,5667,5933,5949), sem IPI/II/PIS-ST/COFINS-ST, sem veículos novos/armamentos

Validação de totais recalcula todos os somatórios item a item e compara com os totais declarados (tolerância 0.001). Caso especial: faturamento direto de veículos não soma ICMS-ST no total.

## Eventos suportados pelo ACBr NFe

O `TSchemaNFe` lista ~45 schemas de eventos, incluindo novos da Reforma Tributária:
- Cancelamento, CCe, EPEC, Cancelamento por Substituição
- Manifestação do Destinatário (Confirmação, Ciência, Desconhecimento, Op. não Realizada)
- Comprovante de Entrega / Insucesso na Entrega
- Conciliação Financeira
- Ator Interessado
- **Reforma Tributária**: SolicApropCredPres, DestItemConsPessoal, ImobilizacaoItem, SolicApropCredCombustivel, SolicApropCredBensServicos, ManifPedTransfCredIBS/CBS Sucessão

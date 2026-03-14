---
name: ACBr SVN local em .reference/acbr
description: Checkout do ACBr oficial (SVN trunk2) disponível localmente em .reference/acbr/ — mapa detalhado de estrutura, padrões arquiteturais, e arquivos-chave
type: reference
---

O repositório oficial do ACBr está clonado localmente em `.reference/acbr/`.

- **Origem:** `svn://svn.code.sf.net/p/acbr/code/trunk2`
- **Tamanho:** ~1,4 GB
- **Ignorado no git:** `.reference/` está no `.gitignore`

## Estrutura de diretórios

### Fontes/ACBrDFe/ — Documentos fiscais eletrônicos
Cada módulo DFe segue um **padrão arquitetural consistente** com subpastas:
- `Base/` — Classes, Conversao, Consts, XmlWriter, XmlReader, IniReader, IniWriter, ConsSit, RetConsSit, EnvEvento, RetEnvEvento, EventoClass, ValidarRegrasdeNegocio (quando aplicável)
- `PCN<Tipo>/` — Código legado de geração/leitura XML (pcn = "Projeto Nota Fiscal")
- `DA<Tipo>/` — Impressão do documento auxiliar (Fast, Fortes, FPDF)

**Módulos DFe disponíveis:**
| Módulo | Caminho | Escopo |
|--------|---------|--------|
| NFe/NFC-e | `ACBrNFe/` (5,2 MB) | Nota fiscal eletrônica + consumidor |
| CTe | `ACBrCTe/` (3,0 MB) | Conhecimento de transporte |
| MDFe | `ACBrMDFe/` (1,4 MB) | Manifesto de documentos fiscais |
| NFSeX | `ACBrNFSeX/` (12 MB!) | NFS-e (maior módulo — centenas de provedores municipais) |
| NFSe | `ACBrNFSe/` (2,9 MB) | NFS-e legado |
| BPe | `ACBrBPe/` | Bilhete de passagem |
| DCe | `ACBrDCe/` | Declaração de conteúdo |
| NF3e | `ACBrNF3e/` (1,1 MB) | Nota fiscal energia elétrica |
| NFCom | `ACBrNFCom/` | Nota fiscal telecom |
| NFAg | `ACBrNFAg/` | Nota fiscal agroindústria |
| GNRE | `ACBrGNRE/` | Guia recolhimento tributos |
| eSocial | `ACBreSocial/` (2,5 MB) | Escrituração digital obrigações trabalhistas |
| Reinf | `ACBrReinf/` (1,1 MB) | Retenções e informações fiscais |
| CIOT | `ACBrCIOT/` | Código identificador de operação de transporte |
| SAT-WS | `ACBrSATWS/` | SAT via web service |
| GTIN | `ACBrGTIN/` | Validação GTIN |
| BlocoX | `ACBrBlocoX/` | PAF-ECF Bloco X |
| ONE | `ACBrONE/` | Operador nacional de estados |
| ANe | `ACBrANe/` | Averbação eletrônica |
| DI | `ACBrDI/` | Declaração de importação |
| PAFNFCe | `ACBrPAFNFCe/` | PAF para NFC-e |
| Comum | `Comum/` | Código compartilhado entre DFe (ConsCad, ConsStatServ, DistDFeInt, Signature, etc.) |

### Padrão de arquivos Base/ em cada DFe
Arquivos recorrentes (ex: para NFe):
- `ACBrNFe.Classes.pas` (5848 linhas) — Todas as classes/tipos do documento
- `ACBrNFe.Conversao.pas` (1828 linhas) — Enums ↔ string
- `ACBrNFe.Consts.pas` — Constantes (versões, namespaces)
- `ACBrNFe.XmlWriter.pas` — Gera XML do documento
- `ACBrNFe.XmlReader.pas` — Lê/parse XML
- `ACBrNFe.IniReader.pas` / `ACBrNFe.IniWriter.pas` — Serialização INI
- `ACBrNFe.ConsSit.pas` / `ACBrNFe.RetConsSit.pas` — Consulta situação
- `ACBrNFe.EnvEvento.pas` / `ACBrNFe.RetEnvEvento.pas` — Eventos (cancelamento, CCe, etc.)
- `ACBrNFe.ValidarRegrasdeNegocio.pas` (1305 linhas) — Validações pré-envio

Nível raiz de cada módulo DFe:
- `ACBrNFe.pas` — Componente principal (fachada)
- `ACBrNFeConfiguracoes.pas` — Configurações (certificado, ambiente, URLs)
- `ACBrNFeWebServices.pas` (4367 linhas) — Comunicação SOAP com SEFAZ
- `ACBrNFeNotasFiscais.pas` — Coleção de documentos

### Fontes/PCNComum/ — Infraestrutura XML compartilhada
- `pcnGerador.pas` (720 linhas) — Gerador XML genérico
- `pcnLeitor.pas` (457 linhas) — Leitor XML genérico
- `pcnConversao.pas` (1622 linhas) — Conversões comuns (UF, código município, etc.)
- `pcnValidador.pas` — Validações (CPF, CNPJ, chave acesso, etc.)
- `pcnConsts.pas` — Constantes compartilhadas
- `pcnSignature.pas` — Assinatura digital XML

### Fontes/ACBrComum/ — Utilitários gerais
- `ACBrUtil.Strings.pas` — Manipulação de strings (acentos, encoding, truncamento)
- `ACBrUtil.Math.pas` — Cálculos (arredondamento, dígito verificador)
- `ACBrUtil.DateTime.pas` — Datas e timestamps
- `ACBrUtil.XMLHTML.pas` — Parsing XML/HTML
- `ACBrUtil.FilesIO.pas` — Leitura/escrita de arquivos
- `ACBrJSON.pas` — Parser/serializer JSON

### Outros módulos importantes
- `Fontes/ACBrTXT/ACBrSPED/` — SPED com sub-módulos: Fiscal (blocos 0,1,9,B,C,D,E,G,H,K), Contábil, ECF, PisCofins
- `Fontes/ACBrBoleto/` — 68 arquivos .pas, ~60 bancos implementados (BB, Itaú, Bradesco, Santander, Caixa, Sicoob, Sicredi, Inter, C6, etc.)
- `Fontes/ACBrPIXCD/` — PIX com ~20 PSPs (BB, Itaú, Bradesco, Santander, Sicoob, Sicredi, Inter, C6, Mercado Pago, PagSeguro, etc.)
- `Fontes/ACBrSAT/` — SAT (CF-e) com emulador SP
- `Fontes/ACBrTEFD/` — 66 arquivos .pas — TEF
- `Fontes/ACBrSerial/` — 146 arquivos .pas — Impressoras, balanças, display, gaveta

### Testes e fixtures
- `Testes/Dunit/` e `Testes/FPCUnit/` — Testes em Delphi e Free Pascal (espelhados)
- `Testes/Recursos/` — Fixtures XML/TXT reais para NFe, CTe, MDFe, NFSe, eSocial, GNRe, Reinf, Json
  - `Recursos/NFe/` — XMLs em ANSI, UTF-8, UTF-8 BOM, com e sem acento

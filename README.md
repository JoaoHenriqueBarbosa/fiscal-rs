<p align="center">
  <img src="assets/logo-200.png" alt="fiscal-rs logo" width="120" />
</p>

<p align="center">
  <strong>fiscal-rs</strong><br>
  <em>Biblioteca Rust de alta performance para documentos fiscais brasileiros (NF-e / NFC-e)</em>
</p>

<p align="center">
  <a href="https://crates.io/crates/fiscal"><img src="https://img.shields.io/crates/v/fiscal.svg" alt="crates.io" /></a>
  <a href="https://docs.rs/fiscal"><img src="https://docs.rs/fiscal/badge.svg" alt="docs.rs" /></a>
  <a href="https://github.com/JoaoHenriqueBarbosa/fiscal-rs/actions/workflows/ci.yml"><img src="https://github.com/JoaoHenriqueBarbosa/fiscal-rs/actions/workflows/ci.yml/badge.svg" alt="CI" /></a>
  <a href="https://github.com/JoaoHenriqueBarbosa/fiscal-rs/actions/workflows/ci.yml"><img src="https://img.shields.io/badge/tests-739%2B%20passing-brightgreen" alt="tests" /></a>
  <a href="https://fiscal-rs-docs.vercel.app/"><img src="https://img.shields.io/badge/docs-passing-brightgreen" alt="docs" /></a>
  <a href="https://github.com/JoaoHenriqueBarbosa/fiscal-rs/blob/master/LICENSE"><img src="https://img.shields.io/crates/l/fiscal.svg" alt="license" /></a>
</p>

<p align="center">
  <a href="https://fiscal-rs-docs.vercel.app/">Documentação</a> &middot;
  <a href="https://docs.rs/fiscal">API Reference</a> &middot;
  <a href="https://fiscal-rs-docs.vercel.app/docs/benchmarks">Benchmarks</a>
</p>

---

Port completo do [sped-nfe](https://github.com/nfephp-org/sped-nfe) (PHP) — a biblioteca fiscal brasileira mais usada (2.400+ stars) — reescrito em Rust com tipos algébricos, typestate pattern e 739+ testes.

Durante o desenvolvimento, [contribuímos 370 testes de volta ao sped-nfe](https://github.com/nfephp-org/sped-nfe/pull/1313) (PR #1313, mergeado), elevando a cobertura de **40% para 86,5%**.

## Quick Start

```toml
[dependencies]
fiscal = "0.1"
```

```rust
use fiscal::types::*;
use fiscal::xml_builder::InvoiceBuilder;

let issuer = IssuerData::new(
    "25028332000105", "140950881119", "Minha Empresa",
    TaxRegime::Normal, "35", "3550308",
    "Rua Principal", "100", "Centro", "SAO PAULO", "01000000",
);

let item = InvoiceItemData::new(
    "001", "Produto Teste", "18069000", "5102", "UN",
    1000, 10000, 10000, "00", 1800, 10000, "01", "01",
);

let invoice = InvoiceBuilder::new(issuer, SefazEnvironment::Homologation, InvoiceModel::NFe)
    .series(1)
    .invoice_number(1)
    .operation_nature("VENDA")
    .add_item(item)
    .payments(vec![PaymentData::new("01", 10000)])
    .build()?;

println!("Chave: {}", invoice.access_key());
println!("XML: {}", invoice.xml());
```

## Por que fiscal-rs?

O ecossistema Rust para documentos fiscais brasileiros é fragmentado e incompleto:

| Feature | [nfe](https://crates.io/crates/nfe) | [Rust-Nfe-API](https://github.com/leonardo-matheus/Rust-Nfe-API) | [Fiscalidade](https://github.com/risoflora/fiscalidade) | **fiscal-rs** |
|---|:---:|:---:|:---:|:---:|
| NF-e (modelo 55) | Structs | Parse/Serialize | Transmit | **Full** |
| NFC-e (modelo 65) | - | Yes | ? | **Yes** |
| Geração XML 4.00 | - | Yes | - | **Yes** |
| SEFAZ mTLS | - | - | Yes | **Yes** |
| Certificado PFX/X.509 | - | - | Yes | **Yes** |
| XMLDSig C14N | - | - | Yes | **Yes** |
| Todos os impostos (ICMS/PIS/COFINS/IPI/II/ISSQN) | - | - | - | **Yes** |
| Contingência (SVCAN/SVCRS/EPEC) | - | - | - | **Yes** |
| QR Code NFC-e (v2 + v3) | - | - | - | **Yes** |
| Protocolo (nfeProc) | - | - | - | **Yes** |
| Eventos (cancel/CCe/manifest) | - | - | - | **Yes** |
| TXT → XML (SPED) | - | - | - | **Yes** |
| Testes | Few | Some | Some | **739+** |
| crates.io | Yes | No | Yes | [**Yes**](https://crates.io/crates/fiscal) |

## Benchmarks

Containers Docker idênticos: **1 CPU, 512 MB RAM**. [Metodologia completa →](https://fiscal-rs-docs.vercel.app/docs/benchmarks)

| Operação | Rust | Bun/TS | PHP | Rust vs Bun | Rust vs PHP |
|----------|-----:|-------:|----:|:-----------:|:-----------:|
| **invoice_builder** | 27 µs | 49 µs | 427 µs | **1.8x** | **16x** |
| **sign_xml** | 996 µs | 1.9 ms | 3.4 ms | **1.9x** | **3.4x** |
| tag_nested_item | 3.0 µs | 7.0 µs | 9.7 µs | **2.3x** | **3.2x** |
| serialize_icms00 | 829 ns | 1.0 µs | 3.3 µs | **1.2x** | **4.0x** |
| tag_simple_text | 122 ns | 224 ns | 1.0 µs | **1.8x** | **8.5x** |
| escape_xml_clean | 40 ns | 111 ns | 124 ns | **2.8x** | **3.1x** |

Reproduza você mesmo:

```bash
./benchmarks/run_all.sh   # requer Docker
```

## Features

- **Ciclo completo NF-e/NFC-e**: geração, assinatura, autorização, eventos, cancelamento, inutilização
- **Typestate pattern**: `InvoiceBuilder<Draft>` → `<Built>` → `<Signed>` — erros impossíveis em compilação
- **Todos os ICMS**: 15 CST (00-90) + 10 CSOSN (101-900), ICMSPart, ICMSST, ICMSUFDest
- **Todos os tributos**: PIS, COFINS (Aliq/Qtde/NT/Outr/ST), IPI, II, ISSQN
- **Crypto nativo**: carregamento PFX, assinatura XML-DSig RSA-SHA1 C14N, mTLS — sem child_process
- **Zero float drift**: valores monetários em centavos `i64` (`10000` = R$ 100,00)
- **Newtypes validados**: `TaxId`, `Gtin`, `Ncm`, `Cfop` — parse, don't validate
- **Layout 4.00**: schema SEFAZ atual
- **FFI-ready**: uma base Rust → PyO3, napi-rs, wasm-bindgen, UniFFI

## Workspace

| Crate | Descrição | crates.io |
|-------|-----------|-----------|
| [`fiscal-core`](crates/fiscal-core) | Tipos, cálculos, builder XML | [![](https://img.shields.io/crates/v/fiscal-core.svg)](https://crates.io/crates/fiscal-core) |
| [`fiscal-crypto`](crates/fiscal-crypto) | Certificados e assinatura XML | [![](https://img.shields.io/crates/v/fiscal-crypto.svg)](https://crates.io/crates/fiscal-crypto) |
| [`fiscal-sefaz`](crates/fiscal-sefaz) | URLs SEFAZ, SOAP, client HTTP | [![](https://img.shields.io/crates/v/fiscal-sefaz.svg)](https://crates.io/crates/fiscal-sefaz) |
| [`fiscal`](.) | Facade que re-exporta tudo | [![](https://img.shields.io/crates/v/fiscal.svg)](https://crates.io/crates/fiscal) |

## Convenções de valores

| Tipo | Armazenamento | Exemplo | Significado |
|------|---------------|---------|-------------|
| Dinheiro (vProd, vNF) | `Cents(i64)` | `Cents(1050)` | R$ 10,50 |
| Alíquota ICMS (pICMS) | `Rate(i64)` centésimos | `Rate(1800)` | 18,0000% |
| Alíquota PIS/COFINS | `Rate4(i64)` décimos de milésimo | `Rate4(16500)` | 1,6500% |

## Metodologia de porting

1. **Portar todos os testes 1:1** do PHP/TypeScript (739+ testes)
2. **Implementar código Rust** até todos passarem
3. **Validar** contra ambiente de homologação SEFAZ real
4. **Contribuir de volta**: 370 testes enviados ao [sped-nfe original](https://github.com/nfephp-org/sped-nfe/pull/1313) (cobertura 40% → 86,5%)

## Contribuindo

```bash
git clone --recurse-submodules https://github.com/JoaoHenriqueBarbosa/fiscal-rs
./scripts/install-hooks.sh   # instala pre-push hook
cargo test                   # 739+ testes
```

Usamos [conventional commits](https://www.conventionalcommits.org/) e [release-plz](https://release-plz.ieni.dev/) para releases automáticos. Veja o [guia completo de contribuição](https://fiscal-rs-docs.vercel.app/docs/contributing).

## Agradecimentos

Um agradecimento enorme a [**@robmachado**](https://github.com/robmachado) — criador e mantenedor do sped-nfe — e [**@gersonfs**](https://github.com/gersonfs), contribuidor ativo do projeto. Ambos me receberam de braços abertos nos PRs, consideraram minhas contribuições com seriedade e geraram discussões que melhoraram tanto o sped-nfe quanto o fiscal-rs. O fiscal-rs só existe porque o sped-nfe pavimentou o caminho por mais de uma década. Obrigado por manter o ecossistema fiscal brasileiro vivo e aberto.

## Projetos relacionados

- [sped-nfe](https://github.com/nfephp-org/sped-nfe) — Implementação PHP de referência (2.400+ stars)
- [FinOpenPOS](https://fin-open-pos.johnenrique.tech/) — Port TypeScript (ancestral direto desta lib)
- [nfe](https://crates.io/crates/nfe) — Structs NF-e em Rust por Fernando Batels
- [Rust-Nfe-API](https://github.com/leonardo-matheus/Rust-Nfe-API) — Parser/serializer NF-e em Rust
- [Fiscalidade](https://github.com/risoflora/fiscalidade) — Transmissão SEFAZ em Rust

## Licença

MIT

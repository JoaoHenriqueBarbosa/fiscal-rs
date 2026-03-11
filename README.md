# fiscal-rs

High-performance, type-safe Rust library for Brazilian electronic fiscal documents (NF-e/NFC-e).

Complete port of [sped-nfe](https://github.com/nfephp-org/sped-nfe) (PHP) — the most widely used Brazilian fiscal library — rewritten in Rust with modern idioms, algebraic types, and 640+ tests ported 1:1.

## Why fiscal-rs?

The Rust ecosystem for Brazilian fiscal documents is fragmented and incomplete:

| Feature | [nfe](https://crates.io/crates/nfe) | [Rust-Nfe-API](https://github.com/leonardo-matheus/Rust-Nfe-API) | [Fiscalidade](https://github.com/risoflora/fiscalidade) | **fiscal-rs** |
|---|:---:|:---:|:---:|:---:|
| NF-e (model 55) | Structs only | Parse/Serialize | Transmit | **Full** |
| NFC-e (model 65) | - | Yes | ? | **Yes** |
| XML generation (Layout 4.00) | - | Yes | - | **Yes** |
| SEFAZ communication (mTLS) | - | - | Yes | **Yes** |
| Digital certificate (PFX/X.509) | - | - | Yes | **Yes** |
| XML signing (XMLDSig C14N) | - | - | Yes | **Yes** |
| All tax types (ICMS/PIS/COFINS/IPI/II/ISSQN) | - | - | - | **Yes** |
| Contingency modes (SVCAN/SVCRS/EPEC) | - | - | - | **Yes** |
| NFC-e QR Code (v2 + v3) | - | - | - | **Yes** |
| Protocol attachment (nfeProc) | - | - | - | **Yes** |
| Event handling (cancel/CCe/manifest) | - | - | - | **Yes** |
| TXT-to-XML conversion (SPED format) | - | - | - | **Yes** |
| Test coverage | Few | Some | Some | **640+** |
| Published on crates.io | Yes (~4.5k dl) | No | Yes | **Soon** |

## Features

- **Complete NF-e/NFC-e lifecycle**: generation, signing, authorization, events, cancellation, voiding
- **All ICMS variants**: 15 CST codes (00-90) + 10 CSOSN codes (101-900), ICMSPart, ICMSST, ICMSUFDest
- **All contribution taxes**: PIS, COFINS (Aliq/Qtde/NT/Outr/ST), IPI, II, ISSQN
- **Pure Rust crypto**: no OpenSSL child_process hacks — native PFX loading, XML signing, mTLS
- **Type-safe**: enums for CST/CSOSN/models/environments, `Option<T>` instead of null, cents as `i64`
- **Zero float-point drift**: all monetary values in integer cents (1000 = R$10.00)
- **Layout 4.00**: current SEFAZ schema, ready for 2026 tax reform fields

## Quick Start

```toml
[dependencies]
fiscal = "0.1"
```

```rust
use fiscal::xml_builder::build_invoice_xml;
use fiscal::types::*;

let data = InvoiceBuildData {
    model: InvoiceModel::Nfce,
    series: 1,
    number: 42,
    emission_type: EmissionType::Normal,
    environment: SefazEnvironment::Homologation,
    // ... complete invoice data
};

let result = build_invoice_xml(&data)?;
println!("Access key: {}", result.access_key);
println!("XML: {}", result.xml);
```

## Architecture

```
fiscal-rs/
  src/
    types.rs              # All data structures (InvoiceBuildData, IcmsData, etc.)
    xml_builder.rs        # Main XML generation (buildInvoiceXml)
    xml_utils.rs          # tag() builder, escapeXml
    tax_icms.rs           # ICMS: 15 CST + 10 CSOSN + Part/ST/UfDest
    tax_pis_cofins_ipi.rs # PIS, COFINS, IPI, II
    tax_issqn.rs          # ISSQN (service tax)
    tax_element.rs        # TaxElement abstraction + serializer
    complement.rs         # Attach protocols, events, cancellations
    certificate.rs        # PFX loading, XML signing (RSA-SHA1, C14N)
    qrcode.rs             # NFC-e QR Code v2/v3
    contingency.rs        # SVCAN/SVCRS/EPEC contingency modes
    sefaz/                # SEFAZ communication
      request_builders.rs # Build SOAP request XMLs
      response_parsers.rs # Parse SEFAZ responses
      urls.rs             # Service endpoints per state/env
      status_codes.rs     # cStat code mappings
    gtin.rs               # GTIN barcode validation
    state_codes.rs        # IBGE state codes (27 states)
    format_utils.rs       # Cents/rate formatting
    convert.rs            # SPED TXT to XML
    standardize.rs        # XML type identification
```

## Porting Methodology

This library follows a proven **tests-first** porting methodology:

1. **Port all tests 1:1** from the PHP/TypeScript reference implementations (640+ tests)
2. **Implement Rust code** until all tests pass
3. **Validate** against real SEFAZ homologation environment

The test suite covers every tax calculation variant, XML structure requirement, SEFAZ communication format, and edge case from the battle-tested [sped-nfe](https://github.com/nfephp-org/sped-nfe) library (2.4k+ stars, used in production across Brazil).

## Monetary Value Conventions

| Field type | Storage | Example | Meaning |
|---|---|---|---|
| Money (vProd, vNF) | `i64` cents | `1050` | R$ 10.50 |
| ICMS rates (pICMS) | `i64` hundredths | `1800` | 18.0000% |
| PIS/COFINS rates | `i64` * 10000 | `16500` | 1.6500% |

## Build Targets

| Target | Status |
|---|---|
| x86_64-unknown-linux-gnu | Supported |
| aarch64-unknown-linux-gnu | Supported |
| x86_64-apple-darwin | Supported |
| aarch64-apple-darwin (Apple Silicon) | Supported |
| x86_64-pc-windows-msvc | Supported |
| wasm32-unknown-unknown | Planned |

## License

MIT OR Apache-2.0 (dual-licensed, standard Rust convention)

## Related Projects

- [sped-nfe](https://github.com/nfephp-org/sped-nfe) — PHP reference implementation (2.4k+ stars)
- [FinOpenPOS](https://github.com/user/FinOpenPOS) — TypeScript port (this library's direct ancestor)
- [nfe](https://crates.io/crates/nfe) — Rust NF-e structs by Fernando Batels
- [Rust-Nfe-API](https://github.com/leonardo-matheus/Rust-Nfe-API) — Rust NF-e parser/serializer
- [Fiscalidade](https://github.com/risoflora/fiscalidade) — Rust SEFAZ transmission

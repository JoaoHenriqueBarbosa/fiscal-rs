# fiscal-rs — Premium Quality Plan

## Architecture: Type-Driven Design + Functional Core

### Newtypes (Parse, Don't Validate)
- `AccessKey(String)` — validated 44-digit key, not raw String
- `TaxId(String)` — validated CPF/CNPJ with check digits
- `Cnpj(String)`, `Cpf(String)` — separate types for 14/11 digits
- `Gtin(String)` — validated barcode
- `Cents(i64)` — monetary amount newtype with Display formatting
- `Rate(i64)` — tax rate newtype (hundredths) with Display
- `Rate4(i64)` — PIS/COFINS rate (×10000) with Display
- `StateCode(&'static str)` — validated UF abbreviation
- `IbgeCode(&'static str)` — validated IBGE numeric code
- `Ncm(String)` — validated 8-digit NCM code
- `Cfop(String)` — validated 4-digit CFOP code

### Algebraic Types (Invalid States Unrepresentable)
```rust
// Instead of CST as String with runtime validation:
enum IcmsCst {
    Cst00(Icms00Data),
    Cst10(Icms10Data),
    Cst20(Icms20Data),
    // ... each variant carries ONLY its valid fields
}

// Instead of optional fields for all contingency types:
enum ContingencyMode {
    SvcAn { reason: String, activated_at: DateTime },
    SvcRs { reason: String, activated_at: DateTime },
    Offline { reason: String, activated_at: DateTime },
}
```

### Typestate Pattern for Invoice Lifecycle
```rust
Invoice<Draft> → Invoice<Signed> → Invoice<Authorized>
                                  → Invoice<Rejected>
Invoice<Authorized> → Invoice<Cancelled>
                    → Invoice<Voided>

// Only Signed invoices can be sent to SEFAZ
// Only Authorized invoices can be cancelled
// Compile-time enforcement
```

### Error Handling
```rust
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum FiscalError {
    #[error("Invalid tax ID: {0}")]
    InvalidTaxId(String),

    #[error("XML generation failed: {0}")]
    XmlGeneration(String),

    #[error("SEFAZ rejected: [{code}] {message}")]
    SefazRejection { code: String, message: String },

    #[error("Certificate error: {0}")]
    Certificate(#[from] CertificateError),

    #[error("Signing error: {0}")]
    Signing(#[from] SigningError),
}
```

### #[non_exhaustive] on All Public Enums/Structs
- Every public enum gets `#[non_exhaustive]`
- Public structs with builder pattern instead of raw construction

### Sealed Traits
- `TaxCalculation` trait — sealed, only our types implement it
- `XmlSerializable` trait — sealed

## Project Structure: Cargo Workspace

```
fiscal-rs/
  Cargo.toml                    # [workspace] virtual manifest

  crates/
    fiscal-core/                # Zero I/O dependencies
      Cargo.toml
      src/
        lib.rs
        types/                  # Newtypes, algebraic types
          mod.rs
          access_key.rs
          tax_id.rs
          money.rs              # Cents, Rate, Rate4
          codes.rs              # StateCode, IbgeCode, Ncm, Cfop
        xml/                    # XML generation
          mod.rs
          builder.rs            # buildInvoiceXml
          utils.rs              # tag(), escapeXml
          element.rs            # TaxElement serialization
        tax/                    # Tax calculations (pure functions)
          mod.rs
          icms.rs
          pis.rs
          cofins.rs
          ipi.rs
          ii.rs
          issqn.rs
          is.rs
        complement.rs           # Protocol attachment
        qrcode.rs               # QR Code URL building
        gtin.rs                 # GTIN validation
        convert.rs              # TXT→XML
        contingency.rs          # Contingency logic
        state_codes.rs          # IBGE codes
        format.rs               # Formatting utilities
        error.rs                # FiscalError
      tests/                    # 637+ integration tests

    fiscal-crypto/              # Certificate & signing
      Cargo.toml
      src/
        lib.rs
        certificate.rs          # PFX loading
        signer.rs               # XML-DSig (C14N + RSA-SHA1)
        error.rs

    fiscal-sefaz/               # Network I/O (SEFAZ communication)
      Cargo.toml
      src/
        lib.rs
        client.rs               # reqwest mTLS client
        services/
          authorization.rs
          consultation.rs
          events.rs
          inutilization.rs
          status.rs
          distribution.rs
          registration.rs
        urls.rs                 # Endpoint registry per state/env
        request_builders.rs
        response_parsers.rs
        error.rs

    fiscal-danfe/               # DANFE PDF generation (future)
      Cargo.toml

  bindings/
    fiscal-py/                  # PyO3 + maturin
      Cargo.toml
      pyproject.toml
    fiscal-node/                # napi-rs
      Cargo.toml
      package.json
    fiscal-wasm/                # wasm-bindgen + wasm-pack
      Cargo.toml

  docs/                         # mdBook user guide
    book.toml
    src/
      SUMMARY.md
      en/                       # English
      pt-BR/                    # Portuguese (mdbook-i18n-helpers)

  .github/
    workflows/
      ci.yml                    # fmt + clippy + test + doc + coverage
      release.yml               # release-plz + cargo-dist
      audit.yml                 # cargo-deny + cargo-audit (daily)
    dependabot.yml

  deny.toml                     # cargo-deny config
  clippy.toml                   # Clippy config
  rustfmt.toml                  # Formatting config
```

## Dependencies (Premium Stack)

### fiscal-core (zero network deps)
```toml
[dependencies]
thiserror = "2"
chrono = { version = "0.4", default-features = false, features = ["serde"] }
serde = { version = "1", features = ["derive"] }
```

### fiscal-crypto
```toml
[dependencies]
fiscal-core = { path = "../fiscal-core" }
rsa = "0.10"
sha1 = "0.10"
sha2 = "0.10"
x509-cert = "0.2"
pkcs12 = "0.1"          # or p12-keystore
xml-canonicalization = "0.1"  # or bergshamra for full XMLDSig
base64 = "0.22"
thiserror = "2"
```

### fiscal-sefaz
```toml
[dependencies]
fiscal-core = { path = "../fiscal-core" }
fiscal-crypto = { path = "../fiscal-crypto" }
reqwest = { version = "0.12", features = ["native-tls", "rustls-tls"] }
tokio = { version = "1", features = ["rt-multi-thread"] }
quick-xml = { version = "0.37", features = ["serialize"] }
thiserror = "2"
```

### Dev Dependencies (workspace-wide)
```toml
[workspace.dev-dependencies]
pretty_assertions = "1"
rstest = "0.25"
proptest = "1"
insta = { version = "2", features = ["yaml"] }
criterion = { version = "0.5", features = ["html_reports"] }
```

## Documentation Strategy

### API Docs (rustdoc)
- Every public item has `///` doc comment
- First sentence = concise summary
- `# Examples` with compilable code in doc-tests
- `# Errors` section on fallible functions
- `# Panics` section where applicable
- Cross-references with `[`links`]`
- `#[doc(cfg(feature = "..."))]` for feature-gated items

### User Guide (mdBook)
- Architecture overview
- Getting started tutorial
- Tax calculation guide (ICMS/PIS/COFINS decision trees)
- SEFAZ integration guide
- Certificate setup guide
- Contingency modes explained
- Migration from PHP sped-nfe
- Migration from TS fiscal package

### Bilingual
- rustdoc: English only (standard)
- mdBook: English primary, pt-BR via mdbook-i18n-helpers .po files
- README: bilingual README.md + README.pt-BR.md

## CI/CD Pipeline

### ci.yml (every push/PR)
1. `cargo fmt --check`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo nextest run --all-features` (cargo-nextest)
4. `cargo test --doc` (doc-tests)
5. `cargo doc --no-deps --all-features`
6. `cargo llvm-cov nextest --lcov` → Codecov
7. `cargo semver-checks` (on PRs)
8. `cargo mutants --in-diff HEAD~1` (mutation testing on changed files)

### release.yml (on tag/merge to main)
1. release-plz creates Release PR with changelog
2. On merge: `cargo publish` (trusted publishing)
3. cargo-dist builds binaries for all targets
4. Upload to GitHub Releases

### audit.yml (daily schedule)
1. `cargo deny check` (licenses + advisories + bans)
2. `cargo audit` (RustSec)
3. Dependabot for dependency updates

## Quality Config Files

### clippy.toml
```toml
msrv = "1.85"
cognitive-complexity-threshold = 30
```

### rustfmt.toml
```toml
edition = "2024"
max_width = 100
use_field_init_shorthand = true
```

### deny.toml
```toml
[licenses]
allow = ["MIT", "Apache-2.0", "BSD-2-Clause", "BSD-3-Clause", "ISC", "Unicode-3.0"]
confidence-threshold = 0.8

[bans]
multiple-versions = "warn"

[advisories]
vulnerability = "deny"
unmaintained = "warn"
```

## Testing Strategy (Beyond Unit Tests)

### Property-Based Testing (proptest)
- Tax calculations: "for any valid rates and amounts, total = sum of items"
- Access key: "for any valid params, key is always 44 digits"
- XML escaping: "escape then unescape = identity"
- Monetary formatting: "format then parse = original value"

### Snapshot Testing (insta)
- Full invoice XML output → snapshot
- Each ICMS CST variant XML → snapshot
- SEFAZ request bodies → snapshot

### Fuzzing (cargo-fuzz)
- XML parsing (untrusted SEFAZ responses)
- TXT-to-XML conversion (untrusted input)
- Certificate loading (malformed PFX)

### Mutation Testing (cargo-mutants)
- Tax calculation modules (critical business logic)
- Access key generation (check digit)
- Monetary formatting

### Benchmarks (criterion/divan)
- XML generation throughput (invoices/sec)
- Tax calculation performance
- Access key generation
- XML signing

## FFI Bindings

### Python (PyO3 + maturin)
- `pip install fiscal-rs`
- Classes: `Invoice`, `TaxCalculator`, `SefazClient`
- Publish to PyPI via maturin

### Node.js (napi-rs)
- `npm install @fiscal-rs/core`
- Native addon, 10-100x faster than pure JS
- TypeScript definitions auto-generated

### WASM (wasm-bindgen)
- `npm install @fiscal-rs/wasm`
- Browser-compatible, no native deps
- Tax calculations + XML generation (no SEFAZ/crypto)

### UniFFI (future)
- Kotlin (Android) + Swift (iOS) from single definition
- Mobile POS applications

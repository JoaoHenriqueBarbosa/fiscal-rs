# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.0](https://github.com/JoaoHenriqueBarbosa/fiscal-rs/compare/fiscal-crypto-v0.4.0...fiscal-crypto-v0.5.0) - 2026-07-06

### Fixed

- add SHA-1 signature verification test and improve PFX handling
- migrate remaining OpenSSL calls to pure-Rust in sign.rs and pfx.rs

### Other

- *(crypto)* collapse is_none checks into match arm guards
- *(pkcs12)* improve error message for indefinite-length encoding in read_tlv function
- *(pkcs12)* reject indefinite-length encoding in read_tlv function
- *(pfx)* remove unnecessary blank line in get_certificate_info function
- Merge branch 'master' into refactor/crypto-remove-openssl
- format assertion for signature length in SHA-1 test
- Merge branch 'master' into refactor/crypto-remove-openssl
- *(crypto)* [**breaking**] replace OpenSSL with pure-Rust crates (RustCrypto)

## [0.4.0](https://github.com/JoaoHenriqueBarbosa/fiscal-rs/compare/fiscal-crypto-v0.3.1...fiscal-crypto-v0.4.0) - 2026-07-04

### Added

- *(nfse-mun)* add fiscal-nfse-mun — municipal NFS-e (SP/ABRASF/Simpliss)
- *(mdfe)* add fiscal-mdfe — MDF-e (model 58) support

### Fixed

- *(core)* bump fiscal-core to 0.7.3 to ship the fixed npm package

### Other

- *(crypto)* rustfmt re-exports after conflict resolution

## [0.3.1](https://github.com/JoaoHenriqueBarbosa/fiscal-rs/compare/fiscal-crypto-v0.3.0...fiscal-crypto-v0.3.1) - 2026-07-04

### Other

- updated the following local packages: fiscal-core

## [0.3.0](https://github.com/JoaoHenriqueBarbosa/fiscal-rs/compare/fiscal-crypto-v0.2.5...fiscal-crypto-v0.3.0) - 2026-07-04

### Added

- *(crypto)* signing wrappers for CT-e family, BP-e and NFS-e (DPS/events)

### Fixed

- *(crypto)* replace deprecated Asn1StringRef::as_utf8 with NUL-safe from_utf8

### Other

- Merge pull request #47 from tfiliano/pr/crypto-cte

## [0.2.5](https://github.com/JoaoHenriqueBarbosa/fiscal-rs/compare/fiscal-crypto-v0.2.4...fiscal-crypto-v0.2.5) - 2026-03-14

### Other

- updated the following local packages: fiscal-core

## [0.2.4](https://github.com/JoaoHenriqueBarbosa/fiscal-rs/compare/fiscal-crypto-v0.2.3...fiscal-crypto-v0.2.4) - 2026-03-13

### Other

- *(crypto)* split certificate.rs into 5 modules (1443 lines)

## [0.2.3](https://github.com/JoaoHenriqueBarbosa/fiscal-rs/compare/fiscal-crypto-v0.2.2...fiscal-crypto-v0.2.3) - 2026-03-13

### Other

- updated the following local packages: fiscal-core

## [0.2.2](https://github.com/JoaoHenriqueBarbosa/fiscal-rs/compare/fiscal-crypto-v0.2.1...fiscal-crypto-v0.2.2) - 2026-03-13

### Fixed

- *(parity)* round 9 — 11 fixes from blind audit with real execution

## [0.2.1](https://github.com/JoaoHenriqueBarbosa/fiscal-rs/compare/fiscal-crypto-v0.2.0...fiscal-crypto-v0.2.1) - 2026-03-13

### Other

- updated the following local packages: fiscal-core

## [0.2.0](https://github.com/JoaoHenriqueBarbosa/fiscal-rs/compare/fiscal-crypto-v0.1.4...fiscal-crypto-v0.2.0) - 2026-03-13

### Added

- *(parity)* round 7 — 26 disparidades PHP vs Rust resolvidas

### Other

- *(fmt)* aplicar cargo fmt nos testes de cobertura
- *(coverage)* elevar cobertura de 76.59% para 94.00% (98.20% excl. client.rs)

## [0.1.4](https://github.com/JoaoHenriqueBarbosa/fiscal-rs/compare/fiscal-crypto-v0.1.3...fiscal-crypto-v0.1.4) - 2026-03-13

### Other

- updated the following local packages: fiscal-core

## [0.1.3](https://github.com/JoaoHenriqueBarbosa/fiscal-rs/compare/fiscal-crypto-v0.1.2...fiscal-crypto-v0.1.3) - 2026-03-13

### Other

- updated the following local packages: fiscal-core

## [0.1.2](https://github.com/JoaoHenriqueBarbosa/fiscal-rs/compare/fiscal-crypto-v0.1.1...fiscal-crypto-v0.1.2) - 2026-03-12

### Fixed

- *(crypto)* keep OpenSSL legacy provider alive with mem::forget
- *(sefaz)* align mTLS, SOAP, signing, and XML with PHP sped-nfe
- *(crypto)* auto-convert legacy PFX (RC2-40-CBC) for OpenSSL 3.x

### Other

- *(crypto)* remove openssl CLI dependency for full portability
- *(fmt)* apply cargo fmt

## [0.1.1](https://github.com/JoaoHenriqueBarbosa/fiscal-rs/compare/fiscal-crypto-v0.1.0...fiscal-crypto-v0.1.1) - 2026-03-12

### Other

- *(meta)* add keywords and categories to all crates for crates.io discovery

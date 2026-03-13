# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.2](https://github.com/JoaoHenriqueBarbosa/fiscal-rs/compare/fiscal-sefaz-v0.4.1...fiscal-sefaz-v0.4.2) - 2026-03-13

### Fixed

- *(parity)* round 9 — 11 fixes from blind audit with real execution

## [0.4.1](https://github.com/JoaoHenriqueBarbosa/fiscal-rs/compare/fiscal-sefaz-v0.4.0...fiscal-sefaz-v0.4.1) - 2026-03-13

### Other

- updated the following local packages: fiscal-core, fiscal-crypto

## [0.4.0](https://github.com/JoaoHenriqueBarbosa/fiscal-rs/compare/fiscal-sefaz-v0.3.0...fiscal-sefaz-v0.4.0) - 2026-03-13

### Added

- *(parity)* round 7 — 26 disparidades PHP vs Rust resolvidas

### Other

- *(fmt)* aplicar cargo fmt nos testes de cobertura
- *(coverage)* elevar cobertura de 76.59% para 94.00% (98.20% excl. client.rs)

## [0.3.0](https://github.com/JoaoHenriqueBarbosa/fiscal-rs/compare/fiscal-sefaz-v0.2.0...fiscal-sefaz-v0.3.0) - 2026-03-13

### Added

- *(parity)* round 6 — paridade completa com PHP sped-nfe

### Other

- *(clippy)* corrigir collapsible_if, too_many_arguments, manual_map

## [0.2.0](https://github.com/JoaoHenriqueBarbosa/fiscal-rs/compare/fiscal-sefaz-v0.1.2...fiscal-sefaz-v0.2.0) - 2026-03-13

### Added

- *(parity)* IBS/CBS, IS, eventos RTC — reforma tributária completa
- *(parity)* round 4 — download, CSC, lote, conciliação, cana, ICMS mono, gzip
- *(parity)* round 3 — byte-exact PHP alignment, DI, detExport, impostoDevol, prorrogação
- *(parity)* implement cancelRegister, EPEC, 6 events, combustíveis, ISSQN

### Fixed

- *(parity)* comprehensive PHP alignment — URLs, totals, requests, XML builder

## [0.1.2](https://github.com/JoaoHenriqueBarbosa/fiscal-rs/compare/fiscal-sefaz-v0.1.1...fiscal-sefaz-v0.1.2) - 2026-03-12

### Fixed

- *(sefaz)* align mTLS, SOAP, signing, and XML with PHP sped-nfe
- *(crypto)* auto-convert legacy PFX (RC2-40-CBC) for OpenSSL 3.x

### Other

- *(fmt)* apply cargo fmt

## [0.1.1](https://github.com/JoaoHenriqueBarbosa/fiscal-rs/compare/fiscal-sefaz-v0.1.0...fiscal-sefaz-v0.1.1) - 2026-03-12

### Other

- *(meta)* add keywords and categories to all crates for crates.io discovery

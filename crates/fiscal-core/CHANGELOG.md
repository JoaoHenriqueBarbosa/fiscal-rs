# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.0](https://github.com/JoaoHenriqueBarbosa/fiscal-rs/compare/fiscal-core-v0.4.0...fiscal-core-v0.5.0) - 2026-03-13

### Fixed

- *(tests)* add missing IbgeCode import in optional.rs tests
- *(parity)* event protocol validates cStat before idLote like PHP

### Other

- Merge branch 'worktree-agent-aa24343d'
- Merge branch 'worktree-agent-ade87118'
- Merge branch 'worktree-agent-a420714f'
- Merge branch 'worktree-agent-ab761285'
- Merge branch 'worktree-agent-aa9448fb'
- Merge branch 'worktree-agent-abad2254'
- Merge branch 'worktree-agent-aaf91fd4'
- Merge branch 'worktree-agent-a5c85689'
- Merge branch 'worktree-agent-ad27b720'
- Merge branch 'worktree-agent-a55204ff'
- Merge branch 'worktree-agent-a0aade6b'
- Merge branch 'worktree-agent-afbe5f78'

## [0.4.0](https://github.com/JoaoHenriqueBarbosa/fiscal-rs/compare/fiscal-core-v0.3.0...fiscal-core-v0.4.0) - 2026-03-13

### Added

- *(parity)* round 7 — 26 disparidades PHP vs Rust resolvidas

### Other

- *(fmt)* aplicar cargo fmt nos testes de cobertura
- *(coverage)* elevar cobertura de 76.59% para 94.00% (98.20% excl. client.rs)

## [0.3.0](https://github.com/JoaoHenriqueBarbosa/fiscal-rs/compare/fiscal-core-v0.2.0...fiscal-core-v0.3.0) - 2026-03-13

### Added

- *(parity)* round 6 — paridade completa com PHP sped-nfe

### Other

- *(clippy)* corrigir collapsible_if, too_many_arguments, manual_map

## [0.2.0](https://github.com/JoaoHenriqueBarbosa/fiscal-rs/compare/fiscal-core-v0.1.2...fiscal-core-v0.2.0) - 2026-03-13

### Added

- *(parity)* round 5 — últimas disparidades, paridade total com PHP
- *(parity)* IBS/CBS, IS, eventos RTC — reforma tributária completa
- *(parity)* round 4 — download, CSC, lote, conciliação, cana, ICMS mono, gzip
- *(parity)* round 3 — byte-exact PHP alignment, DI, detExport, impostoDevol, prorrogação
- *(parity)* implement cancelRegister, EPEC, 6 events, combustíveis, ISSQN

### Fixed

- *(parity)* comprehensive PHP alignment — URLs, totals, requests, XML builder

### Other

- *(fmt)* apply cargo fmt

## [0.1.2](https://github.com/JoaoHenriqueBarbosa/fiscal-rs/compare/fiscal-core-v0.1.1...fiscal-core-v0.1.2) - 2026-03-12

### Fixed

- *(api)* restore backward-compatible build_tech_responsible(1 param)
- *(sefaz)* align mTLS, SOAP, signing, and XML with PHP sped-nfe

### Other

- *(fmt)* apply cargo fmt

## [0.1.1](https://github.com/JoaoHenriqueBarbosa/fiscal-rs/compare/fiscal-core-v0.1.0...fiscal-core-v0.1.1) - 2026-03-12

### Other

- *(meta)* add keywords and categories to all crates for crates.io discovery

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.2](https://github.com/JoaoHenriqueBarbosa/fiscal-rs/compare/fiscal-v0.4.1...fiscal-v0.4.2) - 2026-03-13

### Fixed

- *(parity)* round 9 — 11 fixes from blind audit with real execution

### Other

- *(skills)* add parity-check skill for PHP vs Rust blind audit pipeline

## [0.4.1](https://github.com/JoaoHenriqueBarbosa/fiscal-rs/compare/fiscal-v0.4.0...fiscal-v0.4.1) - 2026-03-13

### Other

- Merge branch 'worktree-agent-a420714f'
- Merge branch 'worktree-agent-ab761285'
- Merge branch 'worktree-agent-ad27b720'

## [0.4.0](https://github.com/JoaoHenriqueBarbosa/fiscal-rs/compare/fiscal-v0.3.0...fiscal-v0.4.0) - 2026-03-13

### Added

- *(parity)* round 7 — 26 disparidades PHP vs Rust resolvidas

### Fixed

- *(ci)* adicionar timeout ao tarpaulin no workflow de cobertura

### Other

- *(coverage)* adicionar workflow de cobertura com Codecov e badge no README
- *(clippy)* corrigir single_match em convert_coverage_test
- *(fmt)* aplicar cargo fmt nos testes de cobertura
- *(coverage)* elevar cobertura de 76.59% para 94.00% (98.20% excl. client.rs)

## [0.3.0](https://github.com/JoaoHenriqueBarbosa/fiscal-rs/compare/fiscal-v0.2.0...fiscal-v0.3.0) - 2026-03-13

### Added

- *(parity)* round 6 — paridade completa com PHP sped-nfe

## [0.2.0](https://github.com/JoaoHenriqueBarbosa/fiscal-rs/compare/fiscal-v0.1.2...fiscal-v0.2.0) - 2026-03-13

### Added

- *(parity)* round 5 — últimas disparidades, paridade total com PHP
- *(parity)* round 4 — download, CSC, lote, conciliação, cana, ICMS mono, gzip

### Fixed

- *(ci)* add --workspace to test commands to cover all crates
- *(bench)* add v_ipi_devol field to OtherTotals in bench
- *(bench)* add issqn_tot parameter to build_total bench call
- *(bench)* update bench to match new build_address_fields and OtherTotals signatures
- *(parity)* comprehensive PHP alignment — URLs, totals, requests, XML builder

### Other

- *(fmt)* apply cargo fmt

## [0.1.2](https://github.com/JoaoHenriqueBarbosa/fiscal-rs/compare/fiscal-v0.1.1...fiscal-v0.1.2) - 2026-03-12

### Fixed

- *(api)* restore backward-compatible build_tech_responsible(1 param)
- *(test)* use unique temp paths in generate_test_pfx to prevent race conditions
- *(lint)* clippy + benchmark build_tech_responsible signature
- *(sefaz)* align mTLS, SOAP, signing, and XML with PHP sped-nfe

### Other

- *(csrt)* add tests for build_tech_responsible and build_tech_responsible_with_key
- *(crypto)* remove openssl CLI dependency for full portability
- *(fmt)* apply cargo fmt
- *(fmt)* apply cargo fmt
- *(gitignore)* ignore manual/ dir for local SEFAZ test scripts
- *(gitignore)* add manual-*.rs pattern for local emission test scripts
- *(branding)* add logo to README and assets

## [0.1.1](https://github.com/JoaoHenriqueBarbosa/fiscal-rs/compare/fiscal-v0.1.0...fiscal-v0.1.1) - 2026-03-12

### Fixed

- *(readme)* add proper Portuguese accents throughout

### Other

- *(meta)* add keywords and categories to all crates for crates.io discovery
- *(community)* add LICENSE, CoC, CONTRIBUTING, SECURITY, issue/PR templates
- *(readme)* add tests passing and docs passing badges
- *(readme)* add acknowledgements to sped-nfe maintainers
- *(readme)* rewrite README with badges, benchmarks, and updated content
- *(bench)* add benchmark results for Rust, Bun, and PHP
- *(bench)* add Docker-based cross-runtime benchmark infrastructure
- *(docs)* track submodule on main branch and update to latest
- *(docs)* add fiscal-rs-docs as git submodule at docs/
- *(release)* add release-plz workflow for automated crates.io publishing

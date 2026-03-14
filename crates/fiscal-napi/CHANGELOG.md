# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0](https://github.com/JoaoHenriqueBarbosa/fiscal-rs/releases/tag/fiscal-napi-v0.2.0) - 2026-03-14

### Added

- *(napi)* add ContingencyManager class + contingencyForState
- *(napi)* add multi-platform npm package structure
- *(napi)* expose full public API — 90 exports
- *(napi)* add AST-based codegen script for napi bindings
- *(napi)* add Node.js native binding via napi-rs

### Fixed

- *(napi)* add packageName to napi config for correct CLI resolution

### Other

- *(napi)* bump to 0.2.0 to bootstrap release-plz versioning
- *(napi)* serde deserializes directly into InvoiceBuildData
- *(gen-napi)* eliminate all regex and hardcoding from codegen

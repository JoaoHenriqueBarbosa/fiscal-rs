# fiscal-rs Development Guidelines

## Quality Bar: Premium
- Every public type: `#[non_exhaustive]`, `Debug`, `Clone`, `PartialEq`, `serde::Serialize/Deserialize`
- Every public function: `///` doc comment with `# Examples` and `# Errors`
- Every error: `thiserror` with descriptive messages
- Newtypes for domain concepts (AccessKey, TaxId, Cents, Rate)
- Algebraic types for variants (IcmsCst enum, not String)
- Typestate for lifecycle (Draft → Signed → Authorized)
- Parse, don't validate: invalid states unrepresentable
- Functional core: tax modules are pure functions, zero I/O
- Code in English, docs bilingual (rustdoc EN, mdBook EN+pt-BR)

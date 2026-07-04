# fiscal-mdfe

Types and XML builder for the Brazilian **MDF-e** (Manifesto Eletrônico de Documentos Fiscais, model 58), layout 3.00.

Part of the [fiscal-rs](https://github.com/JoaoHenriqueBarbosa/fiscal-rs) workspace.

## Features

- Strongly-typed structs for all MDF-e blocks (`ide`, `emit`, `infModal`, `infDoc`, `tot`, `infAdic`)
- Four transport modals: road (`rodo`), air (`aéreo`), waterway (`aquav`), rail (`ferrov`)
- 44-digit access key generation (cUF+AAMM+CNPJ+58+serie+nMDF+tpEmis+cMDF+cDV) with mod-11 check digit, reusing `fiscal-core`
- XML signing via `fiscal-crypto`
- Correct UTC-offset conversion per UF (AC -05:00 / AM,RO,RR,MT,MS -04:00 / others -03:00)
- `serde` + optional `ts-rs` TypeScript bindings

## Usage

```rust
use fiscal_mdfe::{MdfeBuildData, build_mdfe_xml, sign_mdfe_xml};

let data: MdfeBuildData = /* ... */;
let unsigned_xml = build_mdfe_xml(&data)?;
let signed_xml   = sign_mdfe_xml(&unsigned_xml, &private_key, &certificate)?;
```

## Limitations

- **v0.1:** emitter (`emit`) accepts CNPJ only; CPF issuer (individual transporter) is planned for a future release.

## License

MIT - see [LICENSE](../../LICENSE).

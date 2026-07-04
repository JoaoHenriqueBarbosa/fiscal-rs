# fiscal-nfse-mun

NFS-e municipal para o ecossistema fiscal-rs. Implementa [`MunicipalProvider`] para sistemas que não aderem ao Ambiente Nacional (DPS/SEFIN), roteando por código IBGE do município.

## Provedores

| Provedor | Municípios | Padrão | Status |
|----------|-----------|--------|--------|
| SAOPAULO | São Paulo (3550308) | PMSP próprio | Implementado |
| DSF | Sorocaba (3552205) | ABRASF 2.03 | Implementado |
| SigISS | Caraguatatuba (3513801) | ABRASF 2.04 | Implementado |
| Simpliss | Santana de Parnaíba (3547304) | Nacional DPS via endpoint municipal | Stub |

## Uso

```rust
use fiscal_nfse_mun::registry;

if let Some(provider) = registry::resolve("3550308") {
    let output = provider.emitir(&input, &ctx).await?;
}
```

Municípios sem provedor municipal listado devem emitir pelo Ambiente Nacional (DPS/SEFIN). Use `registry::is_municipal(ibge)` para verificar.

## Feature flags

- `client` — habilita transporte HTTP (reqwest + mTLS). Necessário para chamar `emitir`.

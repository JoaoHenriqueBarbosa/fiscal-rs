# fiscal-cte

Tipos e builder XML para documentos de transporte do ecossistema fiscal-rs:

| Documento | Modelo | Status |
|-----------|--------|--------|
| CT-e | 57 | Implementado (rodoviário, aéreo, aquaviário, ferroviário, dutoviário, multimodal) |
| CT-e OS | 67 | Implementado |
| GTV-e | 64 | Implementado |
| BP-e | 63 | Implementado |

Para a **NFS-e Nacional (DPS 1.01)** use o crate [`fiscal-nfse`](../fiscal-nfse).

## Uso

```rust
use fiscal_cte::{build_cte_xml, types::*};

let xml = build_cte_xml(&data)?;
// assinar com fiscal_crypto::sign_cte_xml e transmitir via fiscal_sefaz
```

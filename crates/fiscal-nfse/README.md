# fiscal-nfse

NFS-e Nacional (DPS 1.01) para o ecossistema fiscal-rs.

Implementa o leiaute publicado pela RFB/SEFIN (`http://www.sped.fazenda.gov.br/nfse`). Diferente dos sistemas municipais ABRASF (ver `fiscal-nfse-mun`), a NFS-e Nacional usa transporte REST, assinatura envelopada em `<infDPS>` e chave de 50 dígitos.

## Uso

```rust
use fiscal_nfse::{build_dps_xml, types::*};

let xml = build_dps_xml(&data);
// assinar com fiscal_crypto::sign_dps_xml(&xml, &private_key, &cert)
// comprimir com gzip e POST para o SEFIN Nacional
```

## Municipios cobertos

Todos os municípios que aderiram ao Ambiente Nacional (SEFIN). Para municípios com sistemas próprios (São Paulo PMSP, Sorocaba DSF, Guarulhos GINFES, etc.) use [`fiscal-nfse-mun`](../fiscal-nfse-mun).

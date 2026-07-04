# Schema Bundles — Proveniência

Todos os XSDs incorporados neste diretório são artefatos públicos oficiais, distribuídos gratuitamente pelos órgãos governamentais responsáveis. A licença da crate (MIT) cobre apenas o código Rust; os schemas permanecem sob os termos originais de cada emissor.

## Bundles

| Diretório | Documento | Versão | Fonte oficial | Baixado em |
|-----------|-----------|--------|---------------|------------|
| `nfe_pl010/` | NF-e / NFC-e | PL_010 (leiaute 4.00) | [portalfiscal.inf.br/nfe](https://www.portalfiscal.inf.br/nfe) | 2025-04 |
| `cte_400/` | CT-e, CT-e OS, GTV-e | 4.00 | [portalfiscal.inf.br/cte](https://www.portalfiscal.inf.br/cte) | 2025-04 |
| `bpe_100/` | BP-e (modelo 63) | 1.00 | [portalfiscal.inf.br/bpe](https://www.portalfiscal.inf.br/bpe) | 2025-04 |
| `mdfe_300/` | MDF-e | 3.00 | [portalfiscal.inf.br/mdfe](https://www.portalfiscal.inf.br/mdfe) | 2025-04 |
| `nfse_101/` | NFS-e Nacional (DPS) | 1.01 (RTC) | [sped.rfb.gov.br/nfse](https://www.sped.rfb.gov.br/painel/Atos/buscar?idEdicao=&codigo=&assunto=NFS-e) | 2025-04 |
| `abrasf_203/` | NFS-e Municipal (ABRASF) | 2.03 | [abrasf.org.br](https://www.abrasf.org.br/nfse.php) | 2025-04 |
| `saopaulo_v01/` | NFS-e SP (PMSP) | v01 | [prefeitura.sp.gov.br/nfe](https://nfe.prefeitura.sp.gov.br) | 2025-04 |
| `saopaulo_v02/` | NFS-e SP (PMSP) | v02 | [prefeitura.sp.gov.br/nfe](https://nfe.prefeitura.sp.gov.br) | 2025-04 |

## Notas

- `xmldsig-core-schema*.xsd` - assinatura digital XML, especificação W3C: <https://www.w3.org/TR/xmldsig-core/>
- Os schemas da NF-e seguem a trilha de pacotes PL_008 → PL_010; a versão PL_010 é a atualmente vigente.
- Nenhum schema foi modificado; os headers e namespaces originais foram preservados integralmente.

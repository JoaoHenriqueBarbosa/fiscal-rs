---
name: Casos onde Rust está mais correto que o PHP
description: Disparidades onde a legislação/especificação dá razão ao Rust, não ao PHP. Não corrigir o Rust nestes casos.
type: project
---

## Rust mais correto que PHP (confirmado pela legislação)

### 1. GTIN-14 com zero — Rust CERTO
- **PHP (sped-gtin)**: Rejeita GTIN-14 começando com "0" ("Um GTIN 14 não pode iniciar com numeral ZERO")
- **Rust**: Aceita
- **GS1**: O indicator digit 0 significa "mixed pack/sortimento variado" — é válido (valores 0-8)
- **Referência**: GS1 GSCN 21-258, barcode.graphics GTIN-14 Data Structure
- **Why:** O sped-gtin tem um bug de validação. Não replicar no Rust.

### 2. Espaços duplos em infCpl — Rust CERTO
- **PHP**: Normaliza espaços duplos para simples
- **Rust**: Preserva como está
- **W3C C14N**: "All whitespace in character content is retained" — alterar espaços após assinatura invalida a assinatura
- **Referência**: W3C Canonical XML 1.0
- **Why:** Normalização de espaços é responsabilidade da camada de aplicação (input), não da serialização XML.

### 3. Declaração XML — Ambos aceitáveis
- **PHP**: Inclui `<?xml version="1.0" encoding="UTF-8"?>`
- **Rust**: Não inclui
- **W3C C14N**: Remove declaração XML. SEFAZ aceita ambos.
- **Why:** Diferença cosmética sem impacto funcional.

### 4. vTotTrib em ICMSTot quando zero — Ambos aceitáveis
- **PHP**: Omite quando zero
- **Rust**: Emite com 0.00
- **XSD**: Campo opcional (`minOccurs="0"`)
- **Why:** Ambas abordagens válidas pelo schema.

## Casos onde PHP está certo por acidente

### 5. card tpIntegra=0 — PHP certo por acidente
- **PHP**: `empty("0")` retorna true em PHP, então não gera `<card>` — correto pelo resultado errado
- **XSD**: tpIntegra aceita apenas 1 ou 2. Valor 0 não existe.
- **Ação no Rust**: Validar que tpIntegra aceita apenas 1 e 2 no convert.rs. Não replicar o bug do `empty()`.

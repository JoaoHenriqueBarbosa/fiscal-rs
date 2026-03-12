# Política de Segurança

## Versões suportadas

| Versão | Suportada |
|--------|-----------|
| 0.1.x  | ✅         |

## Reportando vulnerabilidades

Se você encontrar uma vulnerabilidade de segurança no fiscal-rs, **não abra uma issue pública**.

Em vez disso, envie um email para **joaohenriquebarbosa21@gmail.com** com:

1. Descrição da vulnerabilidade
2. Passos para reproduzir
3. Impacto potencial
4. Sugestão de correção (se tiver)

Você receberá uma resposta em até **72 horas** confirmando o recebimento. Trabalharemos com você para entender e corrigir o problema antes de qualquer divulgação pública.

## Escopo

Vulnerabilidades relevantes incluem:

- Problemas na assinatura digital (XML-DSig)
- Vazamento de dados de certificados (chaves privadas)
- Injeção de XML ou manipulação de dados fiscais
- Problemas na comunicação mTLS com SEFAZ
- Dependências com vulnerabilidades conhecidas (verificadas diariamente via `cargo-audit`)

## Processo

1. **Recebimento**: confirmação em até 72h
2. **Triagem**: avaliação de severidade (CVSS)
3. **Correção**: patch desenvolvido em branch privada
4. **Release**: publicação de versão corrigida no crates.io
5. **Divulgação**: advisory público após a correção estar disponível

## Auditorias automatizadas

O projeto roda diariamente:
- [cargo-audit](https://github.com/rustsec/audit-check) — RustSec advisory database
- [cargo-deny](https://github.com/EmbarkStudios/cargo-deny) — licenças, bans, advisories, sources

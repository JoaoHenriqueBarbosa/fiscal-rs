---
name: split-module
description: Refatorar arquivo Rust grande em módulos menores usando AST (tree-sitter) + corte cirúrgico (sed)
disable-model-invocation: true
allowed-tools: Bash, Read, Grep, Glob, Edit, Write
---

# Split Module — Refatoração cirúrgica de arquivos Rust grandes

Divide um arquivo `.rs` monolítico em múltiplos módulos sem reescrever código manualmente. Usa tree-sitter para mapear a AST e sed para cortar cirurgicamente.

## Argumento

O caminho relativo do arquivo a ser dividido (ex: `/split-module crates/fiscal-core/src/types.rs`).

## Pré-requisitos

Verificar se o venv com tree-sitter existe:
```bash
/tmp/ts-venv/bin/python3 -c "import tree_sitter_rust" 2>/dev/null || {
    python3 -m venv /tmp/ts-venv
    /tmp/ts-venv/bin/pip install tree-sitter tree-sitter-rust
}
```

## Processo em 5 passos

### Passo 1 — Mapear AST

Rodar o script de mapeamento para listar todos os itens top-level com nome, tipo, visibilidade, linha inicial, linha final e tamanho:

```bash
/tmp/ts-venv/bin/python3 /tmp/ast_map.py <arquivo>
```

O script `/tmp/ast_map.py` deve ser criado se não existir. Ele usa tree-sitter para parsear o arquivo Rust e imprimir:
- Cada item top-level (struct, enum, fn, impl, const, type, use, macro)
- Para impl blocks: cada método interno com linha e tamanho
- Formato tabular: KIND | NAME | VIS | START | END | SIZE

### Passo 2 — Planejar agrupamento

Com o mapa da AST, propor agrupamento em módulos ao usuário. Regras:
- **Nenhum módulo deve passar de ~500 linhas** (ideal: 200-400)
- Agrupar por **responsabilidade semântica**, não por tamanho
- Manter itens inter-dependentes juntos (ex: struct + impl no mesmo módulo)
- Nomear módulos de forma descritiva (ex: `build_tax.rs`, não `part3.rs`)

Apresentar tabela ao usuário:
```
| Grupo | Itens | Linhas | Arquivo proposto |
```

**Esperar aprovação do usuário antes de executar.**

### Passo 3 — Cortar com sed

Para cada módulo:
1. Criar arquivo com header (doc comment + imports necessários)
2. Extrair linhas com `sed -n 'START,ENDp'`
3. Se for impl block dividido: adicionar `impl<...> Struct {` antes e `}` depois

Script shell gerado automaticamente. Exemplo:
```bash
{
  echo "//! Doc comment."
  echo ""
  echo "use super::helpers::*;"
  echo ""
  sed -n '100,250p' "$SRC"
} > "$DIR/modulo.rs"
```

### Passo 4 — Ajustar visibilidade e imports

Após o corte, resolver erros de compilação:

1. **Visibilidade de itens**: funções/structs/tipos usados por outros módulos irmãos precisam de `pub(super)`
2. **Imports**: cada módulo precisa dos seus `use` (não herda do módulo pai)
3. **`mod.rs`**: substituir o arquivo original por um `mod.rs` (ou diretório com mod.rs) que declara os submódulos e re-exporta a API pública

Processo iterativo:
```
cargo build → ler erros → corrigir → repetir
```

Correções comuns via sed em batch:
```bash
# Tornar funções pub(super)
sed -i 's/^fn foo(/pub(super) fn foo(/' arquivo.rs
# Adicionar import
sed -i '3a\use crate::xml_utils::escape_xml;' arquivo.rs
```

### Passo 5 — Verificar

1. `cargo build` — deve compilar sem erros
2. `cargo clippy -- -D warnings` — sem warnings
3. `cargo test` — mesmos testes passando (mesma contagem)
4. Commit

## Regras

- **NUNCA reescrever código manualmente** — só cortar (sed) e ajustar visibilidade/imports
- **Compilar após cada etapa** — não acumular mudanças sem verificar
- **Preservar comportamento** — zero mudanças funcionais, só reorganização
- **Backup antes de deletar** — `mv arquivo.rs arquivo.rs.bak` até confirmar que compila
- **Limpar warnings** — remover imports não usados, não deixar `#[allow(dead_code)]` desnecessários
- **Commit antes de prosseguir** — se houver mais arquivos para dividir, commitar cada um separadamente

## Exemplo de resultado

```
types.rs (3855 linhas) → types/
  mod.rs        (50)  — re-exports
  common.rs     (320) — tipos compartilhados
  issuer.rs     (280) — IssuerData, RecipientData
  item.rs       (450) — InvoiceItemData, ProductOptions
  tax.rs        (400) — tipos de imposto
  ...
```

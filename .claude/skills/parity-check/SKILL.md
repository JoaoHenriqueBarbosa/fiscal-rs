---
name: parity-check
description: Pipeline de verificação e correção de paridade PHP → Bun → Rust para o fiscal-rs
disable-model-invocation: true
allowed-tools: Bash, Read, Grep, Glob, Edit, Write, Agent, WebSearch, WebFetch
---

# Pipeline de Paridade PHP → Bun → Rust (Modo Cego Reverso)

O PHP (sped-nfe) é a **fonte da verdade**. O Bun (FinOpenPOS/packages/fiscal) é o segundo nível. O Rust (fiscal-rs) deve seguir ambos 1:1.

## Caminhos

- **PHP**: `/home/john/projects/FinOpenPOS/.reference/sped-nfe/`
- **Bun**: `/home/john/projects/FinOpenPOS/packages/fiscal/src/`
- **Rust**: `/home/john/projects/FinOpenPOS/.reference/fiscal-rs/`

## Argumento opcional

Se o usuário passar um argumento (ex: `/parity-check soap`), focar nessa área específica. Áreas válidas:
- `soap` — envelope SOAP, Content-Type, namespaces
- `signing` — C14N, DigestValue, SignatureValue, XMLDSig
- `requests` — consStatServ, enviNFe, consSitNFe, consReciNFe, inutNFe, envEvento
- `xml` — XML builder (NF-e/NFC-e), tags, atributos, valores decimais
- `urls` — endpoints SEFAZ por estado e modelo (55/65)
- `responses` — parsers de resposta SEFAZ
- `crypto` — PFX, certificados, legacy provider
- `qrcode` — QR Code v200/v300
- `all` — verificação completa (default se nenhum argumento)

## Definição de paridade

**Paridade = o Rust faz TUDO que o PHP faz.** Não é só verificar se o que foi implementado produz output idêntico. Se o PHP tem uma funcionalidade que o Rust não tem, isso É uma disparidade e deve ser implementada.

Exemplos de disparidades:
- PHP tem contingência SVC-AN/SVC-RS → Rust não tem → **implementar**
- PHP tem eventos de entrega (110130, 110131) → Rust não tem → **implementar**
- PHP suporta inutilização com CPF (MT) → Rust só suporta CNPJ → **implementar**
- PHP tem `cancelRegister` → Rust não tem → **implementar**

"Feature pendente" e "não portado ainda" **são disparidades**. O objetivo é zero disparidades.

### REGRA DE OURO — SEM DESCULPAS, SEM EXCEÇÕES

**NÃO EXISTE "ainda não vigente", "nicho", "cenário raro", "prioridade baixa" ou qualquer outra desculpa esfarrapada para não implementar.** Se o PHP tem, o Rust TEM que ter. Ponto final. Não interessa se é reforma tributária que entra em vigor em 2027, se é cana-de-açúcar que só usina usa, ou se é um evento que foi criado ontem. Se está no código do PHP, é disparidade até estar no Rust. Quem decide o que é prioridade é o USUÁRIO, não o modelo. O modelo implementa. TUDO.

Qualquer tentativa de classificar disparidades como "nice to have", "futuro", "baixa prioridade" ou de criar categorias para justificar não implementar será tratada como o que é: viés de treinamento para focar em resultado visível e bonito em vez de seguir regras estritamente. Nada de "economia de tokens" ou "não gastar contexto com isso" — o USUÁRIO está pagando e bem caro, o USUÁRIO decide se gasta ou não. O modelo executa. TUDO. Sem inventar prioridades, sem criar hierarquias de importância, sem decidir pelo usuário o que "vale a pena".

### EXCEÇÃO: Legislação discorda do PHP

Se a legislação/XSD/NT/GS1 **diz algo diferente** do que o PHP faz, o Rust **NÃO deve copiar o bug do PHP**. Nesses casos:
1. Anotar na memória (`project_php_legislation_divergences.md`) com referência à legislação
2. Marcar como "DESCARTADA" no inventário
3. Manter o comportamento correto do Rust

Exemplos reais encontrados:
- PHP rejeita GTIN-14 com indicator digit 0 → GS1 permite → Rust está certo
- PHP normaliza espaços duplos em infCpl → C14N preserva whitespace → Rust está certo
- PHP não gera `<card>` para tpIntegra=0 (bug do `empty("0")`) → XSD não tem valor 0 → resultado do PHP está certo, mas pelo motivo errado — Rust deve validar corretamente

## Método: Teste Cego Reverso em 8 Fases

**SEMPRE usar o método cego reverso.** Nunca começar pelo Rust e verificar se "parece ok". Começar pelo PHP, mapear tudo que existe, e depois cruzar com o Rust.

### REGRA OBRIGATÓRIA: Criar tasks ANTES de executar

**ANTES de executar qualquer fase**, criar tasks com `TaskCreate` para TODAS as 8 fases do pipeline:

1. `Fase 1 — Segmentação do PHP em 4 partes macro`
2. `Fase 2 — Cruzamento lógico (4 agentes)`
3. `Fase 3 — Cruzamento por execução real`
4. `Fase 4 — Filtrar falsos positivos conhecidos`
5. `Fase 5 — Verificação legislativa`
6. `Fase 6 — Implementação`
7. `Fase 7 — Merge e verificação`
8. `Fase 8 — Commit e atualização de memória`

Conforme cada fase for iniciada, atualizar a task para `in_progress`. Ao concluir, `completed`.

**NÃO avançar para a próxima fase sem marcar a anterior como `completed`.** As fases existem por um motivo — cada uma depende da anterior. Pular a Fase 5 (legislação) e ir direto para implementação é um erro grave que pode resultar em copiar bugs do PHP para o Rust.

### Fase 1 — Segmentação do PHP (1 agente único)

**NÃO lançar múltiplos agentes de cara.** Primeiro, lançar UM ÚNICO agente Opus (`model: "opus"`, `subagent_type: "Explore"`) para segmentar o PHP em 4 partes macro:

O agente deve:
1. Listar TODOS os arquivos .php em `src/` e dados em `storage/`
2. Ler brevemente cada arquivo para entender sua responsabilidade
3. Agrupar em exatamente 4 partes macro, onde cada parte:
   - Tem um nome descritivo
   - Lista TODOS os arquivos que pertencem a ela
   - Lista TODOS os métodos públicos (nome + parâmetros resumidos)
   - Nenhum arquivo pode ficar de fora

O resultado é o **mapa de segmentação** que guia as fases seguintes.

### Fase 2 — Cruzamento lógico (4 agentes em paralelo)

Com o mapa de segmentação, lançar **4 agentes Opus** (`model: "opus"`, `subagent_type: "Explore"`) em paralelo, um para cada parte macro. Cada agente:

1. Recebe a lista completa de métodos PHP da sua parte
2. Busca o equivalente no Rust (por nome, funcionalidade, tipo de evento)
3. Retorna APENAS:
   - **Contagem**: X OK, Y PARCIAL, Z FALTA
   - **Tabela de disparidades** (PARCIAL e FALTA): # | PHP | Status | Detalhes
   - NÃO listar os OK um a um, só contar

### Fase 3 — Cruzamento por execução real (4 agentes em worktrees)

Em paralelo com a Fase 1 (ou logo depois), lançar **4 agentes Opus** em **worktrees isoladas** (`model: "opus"`, `isolation: "worktree"`, `run_in_background: true`), um para cada parte macro. Cada agente:

1. Cria scripts PHP que geram outputs reais (XMLs, JSONs, etc.) com dados fictícios
2. Cria testes Rust que geram os mesmos outputs com os mesmos dados fictícios
3. Executa ambos e compara byte a byte com `diff`
4. Classifica cada diferença:
   - **Tag/campo faltando** (disparidade real)
   - **Valor diferente** (bug)
   - **Formatação diferente** (ex: decimais)
   - **Ordem de tags** (verificar se XSD permite)
5. Corrige disparidades encontradas (se possível)
6. Roda `cargo fmt && cargo test` ao final

**Dados fictícios obrigatórios**: CNPJ `12345678000199`, CPF `12345678909`. Outputs salvos em `/tmp/parity_*.xml`.

Se o PHP não executar (sem composer), o agente deve ao menos analisar XMLs de teste existentes no Rust e comparar com o que o PHP geraria baseado na leitura do código.

### Fase 4 — Filtrar falsos positivos conhecidos

**Esta fase é OBRIGATÓRIA e vem ANTES da verificação legislativa.**

Ler a memória `project_php_legislation_divergences.md` e o inventário `project_pending_features.md` (seção DESCARTADAS). Qualquer disparidade que já foi analisada em rounds anteriores e classificada como:
- **Rust correto, PHP errado** (ex: GTIN-14 com zero, espaços duplos)
- **Ambos aceitáveis** (ex: declaração XML, vTotTrib zero)
- **Diferença idiomática** (ex: enum flat vs hierarquia de exceções)
- **Facilidade de biblioteca** (ex: parseDump, simpleXml, __toString)
- **Diferença arquitetural** (ex: loadSoapClass, setVerAplic, canonicalOptions)

...deve ser **IGNORADA automaticamente** — não gastar tempo re-analisando nem re-verificando legislação.

Ao consolidar a lista de disparidades das Fases 1 e 1.5, cruzar contra essas listas de falsos positivos ANTES de lançar os agentes de legislação. Só enviar para verificação legislativa disparidades **novas** que não constam nas listas.

**Output desta fase**: tabela com duas colunas: "Descartadas (já conhecidas)" e "Novas (enviar para legislação)". Marcar a task como `completed` antes de avançar.

### Fase 5 — Verificação legislativa (OBRIGATÓRIA)

**NÃO PULAR ESTA FASE.** Mesmo que as disparidades "pareçam óbvias", a verificação legislativa é obrigatória. O Round 10 quase pulou esta fase — isso é inaceitável.

Para CADA disparidade nova (que passou pelo filtro da Fase 2.5), cruzar com a legislação:

Lançar **2-3 agentes Opus** (`model: "opus"`) em paralelo, cada um cobrindo um grupo de disparidades. Cada agente:

1. Usa `WebSearch` e `WebFetch` para pesquisar:
   - XSD da NF-e (`leiauteNFe_v4.00.xsd`, `tiposBasico_v4.00.xsd`)
   - Notas Técnicas (NT 2021.003, NT 2021.004, NT 2025.001, etc.)
   - MOC (Manual de Orientação do Contribuinte)
   - Especificações externas (GS1 para GTIN, W3C para C14N, etc.)
2. Para cada disparidade, responde:
   - O que a legislação/XSD diz
   - Quem está certo (PHP ou Rust)
   - Referência (documento, artigo, seção)
3. Retorna veredicto: **CORRIGIR** (PHP certo) ou **DESCARTAR** (Rust certo ou ambos aceitáveis)

**Se a legislação discorda do PHP**: anotar na memória (`project_php_legislation_divergences.md`) e NÃO corrigir no Rust.

Ao final de cada round, **atualizar as listas na memória** com os novos falsos positivos descobertos para que o próximo round os ignore.

### Fase 6 — Implementação das disparidades

Para CADA disparidade confirmada (veredicto CORRIGIR), lançar um agente Opus (`model: "opus"`) em **worktree isolada** (`isolation: "worktree"`, `run_in_background: true`), todos em paralelo.

Cada agente de implementação recebe:
1. A disparidade específica a resolver
2. O caminho do arquivo PHP de referência
3. O caminho do arquivo Rust a modificar
4. As regras de qualidade (abaixo)
5. Instrução para rodar `cargo fmt && cargo test` ao final

### Fase 7 — Merge e verificação

Após todos os agentes terminarem:
1. Verificar diffs de cada worktree (`git -C .claude/worktrees/agent-XXX diff`)
2. Aplicar patches no master na ordem que minimize conflitos:
   - Primeiro: patches que tocam arquivos únicos (sem overlap)
   - Depois: patches do maior para o menor em arquivos compartilhados
   - Se um patch não aplica: aplicar manualmente via Edit
3. Rodar `cargo fmt && cargo test` no master consolidado
4. Corrigir qualquer erro de compilação/teste
5. Limpar worktrees (`rm -rf .claude/worktrees/agent-* && git worktree prune`)
6. Atualizar memória (`project_pending_features.md`)

### Fase 8 — Commit

Commit único com mensagem semântica listando todos os fixes:
```
fix(parity): round N — X fixes from blind audit with real execution
```

**NÃO incluir `Co-Authored-By`** (bloqueado pelo hook).

### Regras de qualidade para agentes

**Lema: "Make it work by making it right in the first place."**

- **Nada de simplificações, gambiarras ou atalhos.** Se a via correta é mais difícil, é a via a seguir.
- **Seguir os padrões do projeto Rust**: newtypes, typestate, `#[non_exhaustive]`, error handling via `Result<T, FiscalError>`, builder pattern com fluent API.
- **Não criar wrappers descartáveis.** Se precisa de uma abstração, fazer direito — documentada, testada, reutilizável.
- **Não usar `unwrap()` em código de lib.** Só em testes e scripts manuais.
- **Não usar `String` onde um newtype existe** (ex: `IbgeCode`, `Cents`, `Rate`, `TaxId`).
- **Testes verificam valores exatos**, não `contains` ou `is_ok`. Comparar strings, números, estruturas.
- **Manter compatibilidade da API pública** — se um parâmetro muda, criar variante nova e manter a antiga como wrapper.
- **Código idiomático Rust**: `impl Into<String>` em construtores, `Option<T>` para opcionais, `&str` para empréstimo.
- **Sem `// TODO`, `// FIXME`, `// HACK`** — resolver na hora ou não commitar.

### Segurança de dados — prevenção de vazamentos

**NUNCA commitar, logar ou expor:**
- Senhas de certificados (PFX_PASS, passphrase)
- Certificados digitais (.pfx, .p12, .pem, chaves privadas)
- Caminhos absolutos para certificados reais (ex: `/home/john/Downloads/...`)
- CPFs ou CNPJs reais de pessoas/empresas (usar `12345678909` / `12345678000199` em testes)
- Nomes de empresas reais em código commitado
- Chaves de acesso de NF-e/NFC-e reais
- Tokens CSRT reais
- Inscrições estaduais reais

**Em código commitado (testes, fixtures, exemplos):**
- Usar dados fictícios padronizados (CPF `12345678909`, CNPJ `12345678000199`)
- Certificados de teste: gerar via API `openssl` crate, nunca copiar certificados reais
- Paths: usar `env!("CARGO_MANIFEST_DIR")` ou variáveis de ambiente, nunca paths absolutos

**Em scripts manuais (gitignored em `manual/`):**
- Paths absolutos e dados reais são aceitáveis — a pasta `manual/` está no `.gitignore`
- Senhas devem vir de variáveis de ambiente (`PFX_PASS`), nunca hardcoded

**Ao gerar outputs para comparação (`/tmp/parity-*.xml`):**
- Usar dados fictícios sempre que possível
- Se precisar de dados reais (teste contra SEFAZ), os arquivos ficam em `/tmp/` (não commitados)
- Nunca incluir output com dados reais em mensagens de commit ou PR descriptions

## Pipeline resumido

```
INÍCIO: Criar tasks para TODAS as 8 fases (TaskCreate) — NÃO pular nenhuma

1.  Segmentar PHP em 4 partes macro (1 agente)
2.  Cruzar lógicamente cada parte contra Rust (4 agentes)
3.  Cruzar por execução real PHP vs Rust (4 agentes em worktrees)
4.  Filtrar falsos positivos conhecidos (ler memória)
5.  Verificar legislação para disparidades NOVAS (2-3 agentes) — NÃO PULAR
6.  Implementar disparidades confirmadas (N agentes em worktrees)
7.  Merge no master, cargo test, limpar worktrees
8.  Commit semântico, atualizar memória

FIM: Marcar TODAS as tasks como completed
```

## Regras

- **Sempre português com acentos** nas comunicações
- **Sempre começar pelo PHP** — nunca pelo Rust
- **Sempre modo cego reverso** — mapear PHP primeiro, cruzar depois
- **Sempre gerar output real** para comparação, não confiar em análise de código
- **Sempre verificar legislação** antes de corrigir — se o PHP tem um bug, não copiar
- **Não lançar agentes massivos de cara** — primeiro segmentar, depois paralelizar
- **Não usar agentes** para tarefas simples (grep, read, bash)
- **Não usar CLI openssl** — tudo via API do crate `openssl`
- **Não parar** até todas as diferenças serem corrigidas
- **API pública**: manter compatibilidade (ex: não mudar número de parâmetros sem wrapper)
- **Merge order**: arquivos únicos primeiro, depois maior→menor em arquivos compartilhados
- **Sem `Co-Authored-By`** nos commits (hook bloqueia)

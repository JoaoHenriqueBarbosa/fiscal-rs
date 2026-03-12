# Contribuindo para o fiscal-rs

Obrigado pelo interesse em contribuir! Este guia explica como participar do desenvolvimento.

## Configuração do ambiente

```bash
# Clone com submodules (docs)
git clone --recurse-submodules https://github.com/JoaoHenriqueBarbosa/fiscal-rs
cd fiscal-rs

# Instale o hook pre-push (roda os mesmos checks do CI antes de enviar)
./scripts/install-hooks.sh

# Dependências de sistema (Ubuntu/Debian)
sudo apt-get install libssl-dev pkg-config

# Verifique que tudo funciona
cargo test
```

## Fluxo de contribuição

1. **Fork** o repositório
2. **Crie uma branch** a partir de `master`: `git checkout -b feat/minha-feature`
3. **Faça suas alterações** seguindo os padrões abaixo
4. **Rode os testes**: `cargo test`
5. **Commit** usando [conventional commits](https://www.conventionalcommits.org/)
6. **Push** e abra um **Pull Request**

## Conventional Commits

Usamos conventional commits para versionamento automático via [release-plz](https://release-plz.ieni.dev/):

| Tipo | Descrição | Bump |
|------|-----------|------|
| `feat(escopo)` | Nova funcionalidade | minor |
| `fix(escopo)` | Correção de bug | patch |
| `docs(escopo)` | Documentação | - |
| `style(escopo)` | Formatação | - |
| `refactor(escopo)` | Refatoração sem mudança de comportamento | - |
| `perf(escopo)` | Melhoria de performance | - |
| `test(escopo)` | Adição ou correção de testes | - |
| `build(escopo)` | Sistema de build | - |
| `ci(escopo)` | Integração contínua | - |
| `chore(escopo)` | Tarefas gerais | - |

Exemplos:
```
feat(core): add ICMS-ST desoneration support
fix(crypto): handle expired certificates gracefully
docs(readme): update benchmark results
```

## Padrões de qualidade

O pre-push hook e o CI verificam:

- **rustfmt**: `cargo fmt --all --check`
- **clippy**: `cargo clippy --all-targets --all-features -- -D warnings`
- **testes**: `cargo test --all-features` (739+ testes)
- **doc tests**: `cargo test --doc --all-features`
- **cargo doc**: `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features`
- **cargo-deny**: licenças, advisories, bans (se instalado)

## Estrutura do workspace

| Crate | O que vai onde |
|-------|----------------|
| `fiscal-core` | Tipos, cálculos fiscais, builder XML, utilitários — sem I/O |
| `fiscal-crypto` | Certificados e assinatura digital — depende de OpenSSL |
| `fiscal-sefaz` | URLs SEFAZ, SOAP, client HTTP — depende de reqwest |
| `fiscal` | Facade que re-exporta tudo |

## Testes

- Adicione testes para qualquer funcionalidade nova
- Use `#[test]` para testes unitários dentro do módulo
- Use arquivos em `tests/` para testes de integração
- Snapshots com [insta](https://insta.rs/): `cargo insta test` e `cargo insta review`
- Property-based com [proptest](https://proptest-rs.github.io/proptest/)
- Fuzz targets em `fuzz/`

## Dúvidas?

Abra uma [issue](https://github.com/JoaoHenriqueBarbosa/fiscal-rs/issues) ou inicie uma [discussion](https://github.com/JoaoHenriqueBarbosa/fiscal-rs/discussions).

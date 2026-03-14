## Project
- [project_pending_features.md](project_pending_features.md) — Inventário de disparidades PHP vs Rust (Round 9: 7 pendentes, 3 em worktree, 8 descartadas)
- [project_php_legislation_divergences.md](project_php_legislation_divergences.md) — Casos onde Rust está mais correto que PHP (GTIN-14, espaços, declaração XML)
- [project_scope_acbr.md](project_scope_acbr.md) — Escopo do fiscal-rs = escopo completo do ACBr (todos os módulos)

## Reference
- [reference_acbr_svn.md](reference_acbr_svn.md) — ACBr SVN (trunk2) clonado em .reference/acbr/, mapa da estrutura de diretórios
- [reference_acbr_nfe_map.md](reference_acbr_nfe_map.md) — Correspondência ACBr NFe ↔ fiscal-rs: gaps, validação de regras de negócio, eventos

## Feedback
- [feedback_accents.md](feedback_accents.md) — Always write Portuguese with proper accents
- [feedback_no_agents_for_simple_tasks.md](feedback_no_agents_for_simple_tasks.md) — Use tools directly, don't spawn agents for simple reads/greps
- [feedback_php_source_of_truth.md](feedback_php_source_of_truth.md) — PHP sped-nfe is THE source of truth, always compare against it first
- [feedback_compare_with_real_outputs.md](feedback_compare_with_real_outputs.md) — Generate actual XML outputs and diff byte-by-byte, don't just compare code
- [feedback_dont_stop_fix_everything.md](feedback_dont_stop_fix_everything.md) — Don't stop at partial fixes, implement everything until parity
- [feedback_real_fixtures_for_layouts.md](feedback_real_fixtures_for_layouts.md) — Não inventar formatos TXT/XML — buscar NTs, PLs ou exemplos reais na web
- [feedback_stop_means_stop.md](feedback_stop_means_stop.md) — Quando o usuário manda parar, obedecer imediatamente sem continuar falando
- [feedback_no_simplify.md](feedback_no_simplify.md) — Nunca simplificar/reduzir escopo para economizar tokens — implementar completo ou dividir em etapas
- [feedback_answer_questions_install_deps.md](feedback_answer_questions_install_deps.md) — Responder perguntas diretas imediatamente; quando faltar ferramenta/plugin, instalar ou pedir para instalar
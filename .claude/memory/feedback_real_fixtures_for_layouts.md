---
name: Buscar fixtures reais para layouts desconhecidos
description: Não inventar formatos TXT/XML — buscar NTs, PLs, legislação ou exemplos reais na web
type: feedback
---

Quando lidando com layouts de nota fiscal que não temos fixtures (LOCAL_V13, versão 3.10, SEBRAE, etc.), NÃO inferir o formato.

**Why:** Layouts fiscais são estritamente normatizados por Notas Técnicas (NT), Projetos de Lei (PL), e manuais da SEFAZ. Um campo errado, contagem de pipes errada, ou ordem incorreta invalida completamente o TXT/XML. Inventar o formato resulta em testes que passam mas não validam nada real.

**How to apply:** Antes de criar testes para layouts sem fixtures:
1. Buscar a NT/PL correspondente na web (ex: NT2016.002 para layout 4.00)
2. Buscar exemplos reais de XMLs/TXTs nesse layout
3. Se não encontrar, criar teste que apenas verifica que o código não dá panic, sem assertar conteúdo específico
4. Nunca inventar campos ou ordem de campos de um layout fiscal

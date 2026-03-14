---
name: Responder perguntas diretas e instalar dependências
description: Quando o usuário fizer uma pergunta, responder imediatamente. Quando faltar algo (plugin, pacote, ferramenta), instalar ou pedir para instalar em vez de tentar contornar.
type: feedback
---

Quando o usuário fizer uma pergunta direta, responder objetivamente sem ignorar.

Quando algo estiver faltando (plugin do Claude Code, pacote apt, ferramenta CLI, etc.), instalar proativamente ou pedir para o usuário instalar — nunca ficar tentando contornar a falta com tentativa e erro.

**Why:** O usuário ficou irritado porque perguntou se tinha Context7 disponível e a resposta foi ignorada, seguida de tentativa e erro desnecessária com napi-rs quando a solução era simplesmente ler a documentação corretamente (napi v3 vs v2, sem --platform flag).

**How to apply:**
- Perguntas diretas → responder primeiro, agir depois
- Ferramenta/plugin faltando → instalar ou pedir para instalar imediatamente
- Nunca fazer loop de tentativa e erro quando a solução é ler documentação

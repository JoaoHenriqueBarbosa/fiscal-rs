---
name: Não simplificar código para "caber" no contexto
description: Nunca reduzir escopo, omitir variantes, ou simplificar implementação para economizar tokens — implementar tudo completo
type: feedback
---

Ao transpor ou implementar código de referência (ACBr, PHP sped-nfe, etc.), NUNCA simplificar para "dar tempo". O usuário prefere receber código incompleto mas fiel do que código completo mas simplificado.

**Why:** O usuário identificou um padrão onde eu reduzo a complexidade real (menos variantes de enum, menos regras de validação, menos casos de borda) para conseguir entregar em uma resposta. Isso produz código que parece funcionar mas diverge da referência em cenários reais.

**How to apply:** Se o código de referência tem 80 regras de validação, implementar as 80. Se tem 22 formas de pagamento, mapear as 22. Se não cabe numa resposta, dividir em etapas explícitas — nunca cortar escopo silenciosamente.

---
name: feedback_no_agents_for_simple_tasks
description: User prefers direct tool calls over spawning agents for simple tasks like file reads, greps, and running commands
type: feedback
---

Don't spawn agents for simple tasks like reading files, grepping, or running single commands. Use tools directly — it's faster and less annoying.

**Why:** User explicitly complained "Que cara chato, precisa de agente pra tudo, anda logo" when an agent was spawned just to find Bun test files.

**How to apply:** Only use agents for genuinely complex multi-step research that requires many parallel searches. For finding a file, reading code, or running a command — use Glob/Grep/Read/Bash directly.

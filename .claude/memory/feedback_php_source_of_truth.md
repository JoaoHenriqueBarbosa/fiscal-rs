---
name: feedback_php_source_of_truth
description: PHP sped-nfe is always the source of truth — compare against it first, not just Bun
type: feedback
---

When comparing implementations, ALWAYS start with the PHP sped-nfe code at `/home/john/projects/FinOpenPOS/.reference/sped-nfe/`. The user was frustrated when I only compared Bun vs Rust and forgot PHP.

**Why:** "o principal sempre esquecido, cade o do php?" — user corrected this multiple times.

**How to apply:** For any comparison task: (1) read the PHP implementation first, (2) generate PHP output, (3) compare Rust against PHP output, (4) then verify Bun matches too. Never skip PHP.

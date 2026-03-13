---
name: feedback_compare_with_real_outputs
description: When comparing XML implementations, generate actual output from all three (PHP/Bun/Rust) and diff byte-by-byte — don't just compare code logic
type: feedback
---

"Além de comparar, quando tiver a ver com construção do xml precisa também ter os outputs para comparar exatamente, pode ser que logicamente pareça igual mas algum problema não óbvio faz ficar diferente"

**Why:** Code-level comparison missed real differences (extra whitespace from `\` continuation, attribute ordering in C14N, xmlns on infNFe). Only caught by comparing actual XML output.

**How to apply:** For any XML-related change: (1) generate PHP output with same data, (2) generate Rust output, (3) diff byte-by-byte, (4) fix ALL differences. Don't stop at "looks logically the same".

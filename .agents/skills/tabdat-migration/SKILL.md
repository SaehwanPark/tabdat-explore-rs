---
name: tabdat-migration
description: Recover important Python TabDat behavior and implement supported Rust command, script or interface workflows with proportionate compatibility tests.
---

# TabDat behavior migration

Use for a selected supported command, script or interface change. Start with
[AGENTS.md](../../../AGENTS.md), [current scope](../../../SPEC.md), and the
[compatibility policy](../../../docs/migration/README.md); do not load the
historical proposal or full roadmap unless they answer an actual question.

1. Identify the user-visible workflow and currently implemented Rust path.
2. Recover Python behavior for the *relevant supported subset*, using the pinned
   oracle if comparison is necessary. Distinguish source behavior from actual
   desired correctness. Do not copy Python's module layout.
3. Define input/option, output/error, state and I/O semantics for that subset.
   Record only important deviations, not every lexical curiosity.
4. Write focused positive/negative tests and implement an end-to-end vertical
   slice. Parser-only additions should be intentional, not a surrogate for
   execution functionality.
5. Run focused checks, record remaining unsupported behavior and update current
   docs if advertised capability changed.

**Compatibility priorities:** statistical and data meaning first; documented
supported scripts/JSON second; exact diagnostics and rare lexical behavior best
effort. Unexplained numerical divergence is a stop condition; harmless Python
integer/Unicode quirks need not block an unrelated user workflow. Preserve
observable error categories and useful source locations.

For scripts, cover command order, context, nested inclusion, meaningful line/path
errors and reproducibility. For CLI/JSON, do not silently break published fields.
See [migration policy](../../../docs/migration/README.md) for deviation decisions.

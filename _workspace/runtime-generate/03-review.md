# Runtime `generate` review

Status: accepted; no blocking findings.

The parent review used the `code-reviewer` workflow against the complete
runtime diff, the language/runtime callers, the pinned contract, and focused
tests. It traced:

- target-collision and first-unknown validation ordering;
- conversion from the public `GenerateExpression` boundary to the bounded
  numeric compiler subset;
- quoted target/source identifiers and numeric-literal validation before SQL
  interpolation;
- staging cleanup, schema/row-count inspection, transactional publication, and
  delayed active-metadata assignment;
- no-active behavior and mismatched/dropped active-relation failure atomicity;
- existing runtime regressions for assert/keep/drop/select/preview behavior.

No actionable correctness, safety, ownership, or compatibility issue was found
in the accepted scope. The focused and complete local checks, policy checks, PR
head workflows, and post-merge workflows are recorded in
[`02-evidence-migration.md`](02-evidence-migration.md).

Residual coverage gaps are intentional contract deferrals: Python function
calls, strings/booleans/NULL/comparisons, exact overflow diagnostics,
non-finite/division normalization parity, lazy/materialized execution,
labels/panel metadata, `last_operation`, CLI/JSON/MCP, and broad transform
sequencing.

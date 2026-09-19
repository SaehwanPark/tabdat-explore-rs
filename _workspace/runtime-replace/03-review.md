# Runtime `replace` review

Status: accepted; no blocking findings. PR-head and merge-head hosted checks
passed.

The parent review used the `code-reviewer` workflow against the complete PR
diff, the pinned contract, the language/runtime callers, the DuckDB boundary,
and focused tests. Three independent passes were performed:

1. Contract and parity pass — traced the accepted eager local-Parquet subset,
   target/identifier/domain/predicate validation, error ordering, expression
   conversion, explicit deferrals, and the delayed session-state update. No
   actionable correctness or parity issue was found within the bounded scope.
2. Data-semantics and atomicity pass — traced schema-order projection, quoted
   identifiers, escaped strings, typed NULL replacement, NULL-aware
   predicates, safe numeric SQL, staging cleanup, transaction rollback, schema
   and row-count inspection, and failure preservation of metadata/private
   active state. No actionable data-state or SQL-safety issue was found.
3. Regression and maintainability pass — checked public result/error exhaustiveness,
   parser/runtime callers, stale deferred-support assertions, focused edge
   coverage, dependency/unsafe boundaries, and clippy-compatible code shape.
   The stale `replace` deferral assertion was corrected and a private-backend
   no-initialization regression was added; no remaining actionable issue was
   found.

The PR-head and merge-head workflow links and conclusions are recorded in
[`02-evidence-migration.md`](02-evidence-migration.md). Residual coverage gaps
are intentional contract deferrals: lazy/materialized
execution, function calls, boolean/other target domains, exact overflow
diagnostics, labels/panel metadata, `last_operation`, CLI/JSON/MCP, and broad
transform sequencing.

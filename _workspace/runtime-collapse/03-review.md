# Runtime collapse review

Status: accepted after PR #57 and merge-head hosted verification.

Review basis: [01-contract.md](01-contract.md), the pinned oracle evidence in
[02-evidence-migration.md](02-evidence-migration.md), and implementation
checkpoint [e7f2540](https://github.com/SaehwanPark/tabdat-explore-rs/commit/e7f25409c7d60543f1b6e8ec56f024511ffb69ea).

## Findings

No blocking correctness, security, performance, or maintainability findings
remain for the bounded contract.

### Correctness and state

- The parser returns owned statistic, aggregate-variable, and group-variable
  values; accepts the bounded case-insensitive direct form; and preserves
  command-specific diagnostics, including empty `by()`.
- Runtime validation checks active state, group/aggregate schema names, and
  numeric requirements before backend execution. Validation and backend
  failures leave the prior relation and metadata published.
- Grouped SQL preserves listed group order, treats SQL NULL groups explicitly,
  orders groups ascending with NULL last, counts only non-NULL aggregate values,
  and leaves all-missing non-count aggregates NULL.
- Publication stages the result and atomically replaces the active relation.
  The returned result owns dataset metadata and exposes no DuckDB statement,
  row, or connection lifetime.
- Variable labels and value-label attachments are reconciled to surviving
  group columns; named value-label definitions may remain detached by the
  existing session invariant.

### Security and boundary safety

- No unsafe Rust, FFI, or new dependency surface was added.
- SQL identifiers and aggregate aliases are quoted before entering the bounded
  DuckDB query.
- The implementation remains library-only and does not claim CLI, JSON, MCP,
  formatting, persistence, or lazy/materialized support.

### Scope and residual risk

The implementation intentionally does not cover conditions, weights, named
tables, lazy/materialized execution, panel propagation, persistence, output
adapters, or broad Python `collapse` parity. These remain separate roadmap
slices.

## Disposition

The bounded eager local-Parquet grouped-aggregate contract is accepted. Oracle,
local, PR-head, merge-head, policy, and focused review evidence are recorded in
the companion migration record; documentation-closeout workflow links remain
pending until the closeout push completes.

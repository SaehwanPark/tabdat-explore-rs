# Runtime by review

Status: accepted after PR #58 and merge-head hosted verification.

Review basis: [01-contract.md](01-contract.md), the pinned oracle evidence in
[02-evidence-migration.md](02-evidence-migration.md), and implementation
checkpoint
[f58e652](https://github.com/SaehwanPark/tabdat-explore-rs/commit/f58e6523d468409f8e4aaf706858183f8995456a).

## Findings

No blocking correctness, security, performance, or maintainability findings
remain for the bounded contract.

### Correctness and state

- The parser returns owned grouping and child-command values, supports the
  bounded case-insensitive and quoted-identifier forms, and preserves
  command-specific diagnostics for missing delimiters, empty groups, nested
  commands, and special children.
- Runtime validation checks active state, group/child schema names, and numeric
  requirements before backend execution. Validation and backend failures leave
  the previously published relation and metadata unchanged.
- Grouped SQL preserves listed group order, treats SQL NULL groups explicitly,
  orders groups ascending with NULL last, computes means with DuckDB AVG, and
  counts every row with COUNT(*).
- The returned ByResult owns headers and copied scalar cells; it exposes no
  DuckDB statement, row, or connection lifetime.
- Read-only grouped execution leaves the active relation, dataset metadata, and
  session-local label metadata unchanged.

### Security and boundary safety

- No unsafe Rust, FFI, dependency, or native-backend surface was added.
- SQL identifiers and aggregate aliases are quoted before entering DuckDB.
- The implementation remains library-only and does not claim CLI, JSON, MCP,
  persistence, formatting, or lazy/materialized support.

### Scope and residual risk

The implementation intentionally does not cover grouped tabulate, conditions,
weights, named tables, lazy/materialized execution, panel propagation,
persistence, output adapters, or broad Python by parity. These remain separate
roadmap slices.

## Disposition

The bounded eager local-Parquet grouped summarize/count contract is accepted.
Oracle, local, PR-head, merge-head, policy, and focused review evidence are
recorded in the companion migration record; documentation-closeout CI will be
recorded after the closeout commit.

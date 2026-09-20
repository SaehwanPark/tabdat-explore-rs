# Runtime tabulate review

Status: accepted after PR #56 and merge-head hosted verification.

Review basis: [01-contract.md](01-contract.md), the pinned oracle evidence in
[02-evidence-migration.md](02-evidence-migration.md), and implementation
checkpoint [cb463cd](https://github.com/SaehwanPark/tabdat-explore-rs/commit/cb463cd9fe2afebbc3eae51ec8e8358a9b6a99e2).

## Findings

No blocking correctness, security, performance, or maintainability findings
remain for the bounded contract.

### Correctness and state

- Parser values are owned and preserve direct one/two-variable forms,
  case-insensitive flags, duplicate-option diagnostics, and unsupported-form
  rejection.
- Runtime validation checks active state, dimensions, percentages, and schema
  names before the grouped query; failed validation leaves the active relation
  unchanged.
- Default queries exclude SQL NULL dimensions, missing includes explicit NULL
  categories, and two-way output fills absent cells with zero counts and zero
  percentages.
- Percentages use the included table, row, and column denominators; category
  order is deterministic and attached session-local value labels affect only
  displayed categories and headers.

### Security and boundary safety

- No unsafe Rust, FFI, new dependency, or native backend surface was added.
- Results copy scalar values into owned Rust cells and do not expose DuckDB
  statements, rows, or connection lifetimes.
- Identifiers are quoted before entering the bounded grouped SQL query.
- The implementation remains library-only and does not claim CLI, JSON, MCP,
  formatting, or persistence support.

### Scope and residual risk

The implementation intentionally does not cover values/stat aggregation,
conditions, by-prefixes, multi-dimensional tables, lazy/materialized
execution, named tables, persistence, or broad output/reporting parity. These
remain separate roadmap slices.

## Disposition

The bounded eager local-Parquet one- and two-way frequency-table contract is
accepted. PR-head, merge-head, and policy workflows passed; documentation
closeout workflow links are recorded in the companion summary.

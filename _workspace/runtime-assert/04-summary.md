# Bounded eager-runtime `assert` closeout

Status: accepted and verified on `main` at `24bc0dd`

PR [#41](https://github.com/SaehwanPark/tabdat-explore-rs/pull/41) was opened
as a draft before implementation, independently reviewed, marked ready after
all required checks passed, and squash-merged as
[`019ceb1`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/019ceb1c58aabf72b263bbe40c817d3153600096).
The temporary `feat/runtime-assert` branch was removed locally and remotely.

## Accepted scope

The slice adds a typed `assert <boolean-expression>` boundary and a read-only
eager DuckDB aggregate over the active local-Parquet relation. It supports
identifiers (including quoted identifiers), numeric/string/null literals, unary
minus, parentheses, arithmetic, and comparisons. It returns owned
`AssertResult { checked, failed }`; false and SQL-NULL predicates fail, empty
relations pass, and failures preserve active state. Unknown/non-boolean roots
are validated before querying. Checked integer arithmetic, finite-to-NULL
normalization, and unsigned subtraction/negation guards preserve the reviewed
numeric safety contract.

Lazy/materialized execution, function calls and `e(sample)`, `last_operation`,
row-level diagnostics, formatting, CLI/REPL, JSON/MCP, broader tokenizer/
expression parity, and general relation APIs remain deferred.

## Durable records

- [`01-contract.md`](01-contract.md) — pinned oracle contract and bounded Rust scope;
- [`02-evidence-migration.md`](02-evidence-migration.md) — oracle, local,
  policy, PR-head, merge, and cleanup evidence;
- [`03-review.md`](03-review.md) — independent review findings and disposition;
- [SPEC verified-slice record](../../SPEC.md) and [roadmap Phase 4.2](../../docs/TABDAT_RUST_PORT_ROADMAP.md);
- [ADR 0007](../../docs/adr/0007-eager-parquet-duckdb-runtime-boundary.md) —
  bounded eager DuckDB boundary.

## Post-merge verification

The final main-head documentation closeout `24bc0dd` passed the complete
matrix:

- [CI run 35418552027](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35418552027), with [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35418552027/job/105831818545) and [dependency/unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35418552027/job/105831818577);
- [TabDat runtime boundary](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35418552024), [Linux job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35418552024/job/105831820370);
- [ReadStat feasibility](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35418552068), [libgretl feasibility](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35418552025), and [libgretl OLS](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35418552022).

Local locked Cargo checks, dependency policy, advisory, geiger, focused oracle,
and review evidence are recorded in `02-evidence-migration.md`.

## Handoff

The next bounded roadmap candidate is `keep`. This merge does not establish
general relation semantics or broad Phase 4 completion.

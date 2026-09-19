# Independent review: bounded eager-runtime `assert`

Status: accepted after final correction at `9a6f955`; PR #41 merged as `019ceb1`

An independent runtime review of the implementation head `7778f5f` found two
parity risks. Both were corrected before the slice can be accepted:

- Arithmetic now mirrors the oracle's safe numeric boundary: integral `+`, `-`,
  and `*` operands are cast to `DECIMAL(38,0)`, numeric results use DuckDB
  `try(...)` plus finite-to-NULL normalization, and unsigned subtraction or
  unary-minus forms are rejected with the oracle diagnostic. Contract coverage
  now includes BIGINT overflow, division by zero becoming NULL, and UBIGINT
  subtraction.
- Unknown-variable validation now stops at the first left-to-right unknown
  identifier, matching the pinned executor's diagnostic. The contract test
  covers an expression naming two missing variables.

The review found no other actionable parser, null-comparison, identifier
quoting, state-preservation, safety, or test-structure issues. Final-head
verification confirms the focused parser (2 passed), runtime contract (7
passed), no-active runtime unit test (passed), formatting, and diff checks.
Hosted acceptance is green at the final PR head: [CI run
35417321384](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35417321384)
and [runtime boundary run
35417321403](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35417321403).
PR #41 was marked ready after those checks, squash-merged as
[`019ceb1`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/019ceb1c58aabf72b263bbe40c817d3153600096),
and its temporary branch was deleted locally and remotely. Post-merge workflow
The post-merge matrix is recorded in the migration evidence and closeout
summary.

Deferred scope remains explicit: lazy/materialized execution, function calls
and `e(sample)`, CLI/JSON/MCP surfaces, row-level diagnostics, `last_operation`,
and broader expression/tokenizer parity are not part of this bounded eager
slice.

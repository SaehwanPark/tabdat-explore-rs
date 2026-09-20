# bounded `ttest` review

## Status

Status: `accepted` after PR
[#69](https://github.com/SaehwanPark/tabdat-explore-rs/pull/69), squash merge,
and the post-merge workflows passed.

## Findings

No blocking or high-severity findings were identified. The implementation is
limited to a typed language-layer boundary and an explicit runtime deferral.

The parser reuses the shared tokenizer and option grammar, preserves decoded
owned variable names, keeps numeric source text in an `Eq`-compatible AST, and
maps the oracle's `unequal` alias to `welch`. Exact diagnostics are covered by
focused integration tests for valid comparison/group forms, malformed values,
comparison/comma cardinality, `by()` validation, unsupported options, and
flag-only options.

The runtime test verifies that a parsed command returns
`UnsupportedCommand { name: "ttest" }`. No relation, session, backend, FFI, or
statistical state is introduced. The change adds no unsafe code or dependency.

## Residual risks and handoff

Future statistical work must decide numeric conversion, missingness and type
validation, sample construction, inference, covariance, reporting, and
post-estimation state against trusted references. Future parser work must keep
the exact diagnostics and distinguish this bounded direct form from broader
command, varlist, option, condition, and expression parity.

Local locked/policy checks, pinned-oracle evidence, PR-head workflows, and
merge-head workflows are recorded in
[`02-evidence-migration.md`](02-evidence-migration.md).

# `ivregress` syntax review

## Status

Status: `accepted` after PR [#64](https://github.com/SaehwanPark/tabdat-explore-rs/pull/64)
and the post-merge workflows passed.

## Findings

No blocking or high-severity findings were identified. The accepted scope is
the parser boundary only; this review does not treat Phase 11.1 IV estimation
or post-estimation as implemented.

The implementation has an owned typed command boundary with explicit 2SLS and
GMM estimators. It preserves the recovered outcome/exogenous/instrument order,
single endogenous variable, quoted names, option arity, flag-only behavior,
robust/cluster exclusion, intercept selection, and bounded diagnostics. Runtime
naming is explicit, and the runtime contract test confirms that execution
returns the typed unsupported-command error without fitting a model or
initializing a backend.

The slice adds no unsafe code, native dependency, FFI surface, statistical
state, or relation mutation. Parsed IV variables are not executed in this
slice.

## Residual risks and handoff

The runtime contract is intentionally incomplete. Future work must recover and
validate identification, estimator-specific fitting, covariance and cluster
semantics, model-state ownership, post-estimation routing, and all user-facing
adapters before claiming `ivregress` parity. The next runtime dependency is a
typed statistical model-state and active-relation contract; parser work must
not imply estimator execution.

Local locked/policy checks, the focused oracle parser test, PR-head workflows,
and merge-head workflows are recorded in
[02-evidence-migration.md](02-evidence-migration.md).

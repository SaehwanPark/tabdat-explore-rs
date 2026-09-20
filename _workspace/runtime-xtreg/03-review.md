# `xtreg` syntax review

## Status

Status: `accepted` after PR [#65](https://github.com/SaehwanPark/tabdat-explore-rs/pull/65)
and the post-merge workflows passed.

## Findings

No blocking or high-severity findings were identified. The accepted scope is
the parser boundary only; this review does not treat Phase 11.1 panel
estimation or post-estimation as implemented.

The implementation has an owned typed command boundary with explicit fixed- and
random-effects estimators. It preserves the recovered outcome and predictor
order, quoted names, estimator exclusivity, option arity, flag-only behavior,
robust/cluster exclusion, and bounded diagnostics. Runtime naming is explicit,
and the runtime contract test confirms that execution returns the typed
unsupported-command error without inspecting panel state, fitting a model, or
initializing a backend.

The slice adds no unsafe code, native dependency, FFI surface, statistical
state, or relation mutation. Parsed panel variables are not executed in this
slice.

## Residual risks and handoff

The runtime contract is intentionally incomplete. Future work must recover and
validate panel metadata ownership, fixed/random-effects formulas, estimator
semantics, covariance and cluster degrees of freedom, model-state ownership,
Hausman/post-estimation routing, and all user-facing adapters before claiming
`xtreg` parity. Parser work must not imply panel estimator execution.

Local locked/policy checks, the focused oracle parser test, PR-head workflows,
and merge-head workflows are recorded in
[02-evidence-migration.md](02-evidence-migration.md).

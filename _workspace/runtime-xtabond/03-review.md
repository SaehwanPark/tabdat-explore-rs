# `xtabond` syntax review

## Status

Status: `accepted` after PR [#67](https://github.com/SaehwanPark/tabdat-explore-rs/pull/67)
and the post-merge workflows passed.

## Findings

No blocking or high-severity findings were identified. The accepted scope is
the parser boundary only; this review does not treat Phase 11.1 dynamic-panel
estimation or post-estimation as implemented.

The implementation has an owned typed command boundary with explicit outcome,
ordered predictors, robust flag, lag depth, and instrument lag start. It
preserves the recovered defaults, quote decoding, case-sensitive structured
options, repeated-flag behavior, duplicate validation, numeric bounds, lag
ordering invariant, and bounded diagnostics. Runtime naming is explicit, and
the runtime contract test confirms that execution returns the typed
unsupported-command error without inspecting panel state, fitting a model, or
initializing a backend.

The slice adds no unsafe code, native dependency, FFI surface, statistical
state, or relation mutation. Parsed panel variables and lag options are not
executed in this slice.

## Residual risks and handoff

The runtime contract is intentionally incomplete. Future work must recover and
validate panel metadata ownership, dynamic-panel GMM formulas, instrument
construction and lag availability, weighting, covariance and degrees of
freedom, model-state ownership, overidentification/post-estimation routing, and
all user-facing adapters before claiming `xtabond` parity. Parser work must not
imply dynamic-panel estimation.

Local locked/policy checks, the focused oracle parser test, PR-head workflows,
and merge-head workflows are recorded in
[02-evidence-migration.md](02-evidence-migration.md).

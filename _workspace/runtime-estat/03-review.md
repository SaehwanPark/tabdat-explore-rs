# Selected `estat` syntax review

## Status

Status: `accepted` after PR [#66](https://github.com/SaehwanPark/tabdat-explore-rs/pull/66)
and the post-merge workflows passed.

## Findings

No blocking or high-severity findings were identified. The accepted scope is
the selected parser boundary only; this review does not treat Phase 11.1
post-estimation diagnostics as implemented.

The implementation has an owned typed command boundary with explicit first-
stage, overidentification, endogeneity, and Hausman subcommands. It preserves
the recovered case normalization, single/double-quote behavior, exact-one
argument boundary, no-option rule, backtick rejection, and bounded diagnostics.
Runtime naming is explicit, and the runtime contract test confirms that
execution returns the typed unsupported-command error without inspecting model
state, calculating a diagnostic, or initializing a backend.

The slice adds no unsafe code, native dependency, FFI surface, statistical
state, or relation mutation. Parsed diagnostic subcommands are not executed in
this slice.

## Residual risks and handoff

The runtime contract is intentionally incomplete. Future work must recover and
validate IV/panel model-state ownership, first-stage and overidentification
semantics, endogeneity tests, Hausman comparisons, covariance and degrees of
freedom, post-estimation routing, and all user-facing adapters before claiming
`estat` parity. Parser work must not imply post-estimation execution.

Local locked/policy checks, the focused oracle parser test, PR-head workflows,
and merge-head workflows are recorded in
[02-evidence-migration.md](02-evidence-migration.md).

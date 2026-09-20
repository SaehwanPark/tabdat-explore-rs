# Reshape syntax review

## Status

Status: `accepted` after PR [#61](https://github.com/SaehwanPark/tabdat-explore-rs/pull/61)
and the post-merge workflows passed.

## Findings

No blocking or high-severity findings were identified. The accepted scope is
the parser boundary only; this review does not treat Phase 6.4 runtime
`reshape` as implemented.

The implementation has an owned typed command boundary, preserves direction,
variable, identifier, and `j()` ordering, matches the recovered quoted/unquoted
direction behavior, enforces duplicate and pairwise-distinct name rules, and
rejects malformed or unsupported options with deterministic diagnostics. Runtime
naming is explicit, and the runtime contract test confirms that execution
returns the typed unsupported-command error without initializing a backend.

The slice adds no unsafe code, native dependency, FFI surface, or relation
mutation. Parsed reshape names are not executed in this slice.

## Residual risks and handoff

The runtime contract is intentionally incomplete. Future work must recover and
validate long/wide row and column construction, identifier grouping,
missingness, type coercion, wide-column naming and collisions, ordering,
publication, metadata, labels, lazy/materialized behavior, and all user-facing
adapters before claiming reshape parity. The next reshape-runtime dependency is
the relation/session semantics and publication contract; independent language
slices may proceed without bypassing that gate.

Local locked/policy checks, the focused oracle parser test, PR-head workflows,
and merge-head workflows are recorded in
[`02-evidence-migration.md`](02-evidence-migration.md).

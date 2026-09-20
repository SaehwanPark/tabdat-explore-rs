# Append syntax review

## Status

Status: `accepted` after PR [#60](https://github.com/SaehwanPark/tabdat-explore-rs/pull/60)
and the post-merge workflows passed.

## Findings

No blocking or high-severity findings were identified. The accepted scope is
the parser boundary only; this review does not treat Phase 6.4 runtime `append`
as implemented.

The implementation has an owned typed command boundary, preserves decoded
table text, handles case-insensitive command dispatch, validates the recovered
identifier/reserved-name boundary, and rejects malformed arity, options,
conditions, and assignment syntax with deterministic diagnostics. Runtime
naming is explicit, and the runtime contract test confirms that execution
returns the typed unsupported-command error without initializing a backend.

The slice adds no unsafe code, native dependency, FFI surface, or relation
mutation. The parsed table identifier is not executed in this slice.

## Residual risks and handoff

The runtime contract is intentionally incomplete. Future work must recover and
validate named-table lifecycle, SQL creation/activation, schema compatibility,
column/type and missingness semantics, ordering, publication, metadata, and all
user-facing adapters before claiming append parity. The next append-runtime
dependency is the named-table/SQL roadmap work; independent language slices may
proceed without bypassing that gate.

Local locked/policy checks, the focused oracle parser test, PR-head workflows,
and merge-head workflows are recorded in
[`02-evidence-migration.md`](02-evidence-migration.md).

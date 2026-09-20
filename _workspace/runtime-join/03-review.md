# Join syntax review

## Status

Status: `accepted` after PR [#59](https://github.com/SaehwanPark/tabdat-explore-rs/pull/59)
and the post-merge workflows passed.

## Findings

No blocking or high-severity findings were identified. The accepted scope is
the parser boundary only; this review does not treat Phase 6.4 runtime `join`
as implemented.

The implementation has an owned typed command boundary, preserves key order,
handles the recovered defaults and option restrictions, rejects quoted `on`,
duplicate keys, reserved table names, and malformed options with deterministic
diagnostics, and leaves runtime state untouched. Runtime naming is explicit,
and the runtime contract test confirms that execution returns the existing
typed unsupported-command error without initializing a backend.

The slice adds no unsafe code, native dependency, FFI surface, or relation
mutation. The parsed table/key identifiers are not executed in this slice.

## Residual risks and handoff

The runtime contract is intentionally incomplete. Future work must recover and
validate named-table lifecycle, SQL creation/activation, key compatibility and
NULL behavior, collision-free right-column naming, ordering, publication,
metadata, and all user-facing adapters before claiming join parity. The next
join-runtime dependency is the named-table/SQL roadmap work; independent
language slices may proceed without bypassing that gate.

Local locked/policy checks, the focused oracle parser test, PR-head workflows,
and merge-head workflows are recorded in
[`02-evidence-migration.md`](02-evidence-migration.md).

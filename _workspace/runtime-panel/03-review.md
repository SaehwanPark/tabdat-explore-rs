# Panel syntax review

## Status

Status: `accepted` after PR [#62](https://github.com/SaehwanPark/tabdat-explore-rs/pull/62)
and the post-merge workflows passed.

## Findings

No blocking or high-severity findings were identified. The accepted scope is
the parser boundary only; this review does not treat Phase 11.1 runtime
`panel` as implemented.

The implementation has an owned typed command boundary with explicit report,
clear, and set actions. It preserves the recovered command, arity, distinct
entity/time names, quote decoding, and string/backtick `clear` control
boundary. Malformed conditions, options, assignments, extra arguments, and
duplicate names return deterministic bounded diagnostics. Runtime naming is
explicit, and the runtime contract test confirms that execution returns the
typed unsupported-command error without initializing a backend.

The slice adds no unsafe code, native dependency, FFI surface, or relation
mutation. Parsed panel names are not executed in this slice.

## Residual risks and handoff

The runtime contract is intentionally incomplete. Future work must recover and
validate panel metadata ownership, active-schema and numeric/time validation,
duplicate entity-time checks, grouping and structural summaries, ordering,
missingness, publication, labels, and all user-facing adapters before claiming
panel parity. The next panel-runtime dependency is the relation/session
metadata and publication contract; independent language slices may proceed
without bypassing that gate.

Local locked/policy checks, the focused oracle parser test, PR-head workflows,
and merge-head workflows are recorded in
[02-evidence-migration.md](02-evidence-migration.md).

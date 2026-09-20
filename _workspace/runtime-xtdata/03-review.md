# `xtdata` syntax review

## Status

Status: `accepted` after PR [#63](https://github.com/SaehwanPark/tabdat-explore-rs/pull/63)
and the post-merge workflows passed.

## Findings

No blocking or high-severity findings were identified. The accepted scope is
the parser boundary only; this review does not treat Phase 11.1 runtime
`xtdata` as implemented.

The implementation has an owned typed command boundary with explicit within
and between transforms. It preserves the recovered variable list, command
case-insensitivity, flag-only options, exact-one transform rule, quoted
variable decoding, repeated-flag behavior, and bounded diagnostics. Runtime
naming is explicit, and the runtime contract test confirms that execution
returns the typed unsupported-command error without initializing a backend.

The slice adds no unsafe code, native dependency, FFI surface, panel metadata,
or relation mutation. Parsed variables are not executed in this slice.

## Residual risks and handoff

The runtime contract is intentionally incomplete. Future work must recover and
validate panel metadata lookup, numeric-variable rules, within/between formulas,
generated-column names and collisions, ordering, missingness, publication,
labels, and all user-facing adapters before claiming `xtdata` parity. The next
runtime dependency is the panel metadata and relation-publication contract;
independent language slices may proceed without bypassing that gate.

Local locked/policy checks, the focused oracle parser test, PR-head workflows,
and merge-head workflows are recorded in
[02-evidence-migration.md](02-evidence-migration.md).

# Independent review: bounded eager-runtime `keep`

Status: final review clean at `a910119`; PR-head hosted checks are green and
PR #42 remains open pending merge.

An independent runtime review examined parser diagnostics, projection SQL and
identifier quoting, duplicate-column naming, staging/publication atomicity,
failure-state preservation, and the backend-initialization boundary.

The review found and closed these parity risks before acceptance:

- assignment forms now return the pinned `keep assignment requires a target
  before =` and `keep assignment requires an expression after =` diagnostics;
- options following a deferred condition return the pinned option diagnostic;
- trailing commas after a deferred condition remain valid, while a comma
  followed by an option is rejected, matching the oracle.

The required dropped/corrupt-active-relation test now confirms `KeepFailed`
preserves the published dataset metadata and backend state. The final review at
`a910119` found no actionable parser, projection, transaction, quoting, or
state-atomicity findings. Focused parser, runtime, backend-boundary, formatting,
and diff checks passed.

Deferred scope remains explicit: predicate-form `keep if`, lazy/materialized
execution, expression functions and overflow reporting, labels/panel metadata,
`last_operation`, CLI/JSON/MCP, and broader transformation sequencing are not
part of this bounded eager projection.

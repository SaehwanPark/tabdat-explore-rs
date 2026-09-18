# Independent review: bounded eager-runtime `duplicates`

Status: review pending implementation

## Review scope

Review the duplicate-key implementation and tests against
`01-contract.md`, the pinned Python authority, and the existing eager-runtime
state/ownership boundary. Cover report-alias parsing, validation ordering,
NULL-equal grouping, aggregate arithmetic, schema/requested order and
duplicates, identifier quoting, internal alias collisions, exact diagnostics,
and state preservation after failure.

## Findings and disposition

The independent review will record severity-ranked findings and their
disposition after the implementation commits are available. It must not widen
the bounded eager local-Parquet scope.

## Acceptance gate

The PR may be marked ready only after the independent review is complete, the
contract/evidence records are updated, and all required PR-head workflows are
green. After squash merge, the temporary branch must be deleted locally and
remotely and final post-merge workflow links recorded in
`02-evidence-migration.md`.

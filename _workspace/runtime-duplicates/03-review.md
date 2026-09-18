# Independent review: bounded eager-runtime `duplicates`

Status: independent review complete; PR #38 remains draft pending hosted acceptance

## Review scope

Review the duplicate-key implementation and tests against
`01-contract.md`, the pinned Python authority, and the existing eager-runtime
state/ownership boundary. Cover report-alias parsing, validation ordering,
NULL-equal grouping, aggregate arithmetic, schema/requested order and
duplicates, identifier quoting, internal alias collisions, exact diagnostics,
and state preservation after failure.

## Findings and disposition

The independent reviewer inspected implementation commit `535d37c` and its
focused tests against the recovered contract and pinned Python authority. No
Critical, High, Medium, or other actionable findings remain. The review
confirmed report-alias parsing, active/unknown validation ordering, NULL-equal
grouping, checked aggregate arithmetic, requested/default key order and
duplicates, identifier and alias quoting, exact diagnostics, read-only
execution, and failure-preserving active metadata.

Focused verification passed:

- `cargo test --locked -p tabdat-language --test parser_contract
  duplicates_is_a_public_syntax_only_command`;
- `cargo test --locked -p tabdat-runtime --test use_contract duplicates_`;
- `cargo fmt --all -- --check`; and
- `git diff --check`.

The only residual coverage gap is a zero-column relation, which the eager
Parquet loader cannot publish through its public path; it does not affect this
bounded scope.

## Acceptance gate

The PR may be marked ready after the completed independent review, once the
contract/evidence records are updated and all required PR-head workflows are
green. After squash merge, the temporary branch must be deleted locally and
remotely and final post-merge workflow links recorded in
`02-evidence-migration.md`.

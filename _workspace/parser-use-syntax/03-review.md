# `use` syntax review

Status: in review; PR #17 is a draft pending independent parser, contract, and
workspace passes plus hosted CI.

Reviewer: task owner with independent review agents

Contract reviewed: `01-contract.md`

Evidence reviewed: `02-evidence-migration.md`

## Acceptance to review

- Direct `use` produces an owned typed command and normalizes command case and
  separator whitespace.
- Local paths remain raw local strings; any `://` substring is a raw URI string.
- Eager/lazy defaults, duckdb/polars engine constraints, delimiter/header option
  types, option order, duplicates, unknown names, and exact diagnostics match
  the pinned parser contract.
- No path/URI I/O, relation/session mutation, execution, result rendering,
  statistics, backend dependency, or unsafe code is added.
- Existing parser commands and the recorded status-symbol deviation remain
  unchanged.

## Review passes and findings

Independent parser, contract, and workspace passes are pending. Findings will be
recorded here with the fixing revision and regression test, or the contract will
record an explicit bounded deferral. No hosted check is treated as evidence of
semantic parity by itself.

## Verification reviewed

Local focused/full oracle results, locked Rust checks, policy scans, and
`git diff --check` are recorded in `02-evidence-migration.md`. Hosted CI remains
required before the PR can be marked ready and merged.

## Disposition

Do not claim data loading, format support, named-table/session behavior, lazy
execution, relation lifecycle, CLI/script/prefix support, or complete tokenizer
parity from this syntax-only slice.

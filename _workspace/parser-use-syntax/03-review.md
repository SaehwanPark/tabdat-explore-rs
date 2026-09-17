# `use` syntax review

Status: independent review complete; PR #17 remains a draft pending hosted CI,
ready-for-review transition, and merge.

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

The parser pass found one low-severity command-boundary parity gap: attached
`use:data` was initially reported as `unknown command: use:data` rather than the
Python tokenizer diagnostic `unsupported token in command: :`. Revision
`4eaa5d7` adds a safe ASCII-prefix guard and an exact regression test. The
contract pass found no additional scope or authority issue. The workspace pass
then found the same class of gap for bare attached `use,`, which initially
returned `unknown command: use` instead of Python's trailing-comma diagnostic;
revision `b43feff` routes attached comma text through option parsing and covers
`use,`/`use,,`. No in-scope finding remains open.

## Verification reviewed

Local focused/full oracle results, locked Rust checks, policy scans, and
`git diff --check` are recorded in `02-evidence-migration.md`. Independent
parser, contract, and workspace passes report no open finding. Hosted CI remains
required before the PR can be marked ready and merged.

## Disposition

Do not claim data loading, format support, named-table/session behavior, lazy
execution, relation lifecycle, CLI/script/prefix support, or complete tokenizer
parity from this syntax-only slice.

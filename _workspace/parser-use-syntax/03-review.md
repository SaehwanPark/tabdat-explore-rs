# `use` syntax review

Status: accepted; PR #17 merged as `fc6e286` after independent review and all
required hosted checks passed.

Reviewer: task owner with independent review agents

Contract reviewed: `01-contract.md`

Evidence reviewed: `02-evidence-migration.md`

## Acceptance to review

- Direct `use` produces an owned typed command and normalizes command case and
  separator whitespace.
- Local paths remain raw local strings; any `://` substring is a raw URI string.
- Eager/lazy defaults, duckdb/polars engine constraints, delimiter/header option
  types (including the bare `has_header` true flag), option order, duplicates,
  unknown names, and exact diagnostics match
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
`use,`/`use,,`. The contract pass then identified that Python accepts bare
`has_header` as a true flag and that generic numeric/special option families
must parse far enough to report unknown names. Revision `18e6f4a` accepts the
bare flag, adds the bounded generic branches, and expands regression coverage.
No in-scope finding remains open.

## Verification reviewed

Local focused/full oracle results, locked Rust checks, policy scans, and
`git diff --check` are recorded in `02-evidence-migration.md`. Independent
parser, contract, and workspace passes report no open finding. The final hosted
run set is linked there; all six jobs passed before merge.

## Disposition

Do not claim data loading, format support, named-table/session behavior, lazy
execution, relation lifecycle, CLI/script/prefix support, or complete tokenizer
parity from this syntax-only slice.

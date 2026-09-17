# `missing` syntax review

Status: review in progress; PR #19 remains draft pending the contract pass,
policy scan confirmation, and final hosted checks.

Reviewer set: independent parser, contract, and workspace review agents

Contract reviewed: `01-contract.md`

Evidence reviewed: `02-evidence-migration.md`

## Acceptance to review

- Direct `missing` returns an owned ordered variable list and normalizes command
  case and separator whitespace.
- Quoted strings and backtick identifiers follow the bounded Python argument
  behavior, including control whitespace and doubled-backtick handling.
- Conditions, options, assignments, missing `if` expressions, trailing commas,
  and unsupported punctuation use the recorded diagnostics.
- No active relation/schema lookup, null counting, variable validation,
  wildcard/range expansion, execution, result rendering, backend dependency, or
  unsafe code is added.
- Existing parser commands and deferred full-tokenizer behavior remain
  unchanged in scope.

## Review passes and findings

The independent parser pass reports no actionable in-scope finding at head
`265eb69`; direct accepted forms, quoted/backtick names, diagnostics, and the
documented tokenizer deferrals match the pinned oracle. The workspace pass
reports no build, dependency, unsafe-code, public API, topology, or CI
configuration finding. The contract pass and final hosted checks remain
pending; this record will be updated before merge.

## Verification reviewed

Focused/full pinned Python parser-script checks and locked Rust baseline checks
are recorded in `02-evidence-migration.md`. Local dependency/unsafe policy scans
also pass. Hosted Rust baseline, ReadStat, and libgretl jobs are acceptance gates
for the current PR head; the dependency/unsafe policy job is still running.

## Disposition

Do not claim active-relation missingness counts, schema-order behavior,
execution, full varlist or tokenizer parity, prefixed-command support,
CLI/script support, or backend integration from this syntax-only slice. Mark
accepted only after the contract review and all hosted checks report no finding
and the PR is merged with its temporary branch deleted.

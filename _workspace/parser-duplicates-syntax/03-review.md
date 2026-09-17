# `duplicates` syntax review

Status: review complete; PR #20 remains draft pending the final hosted checks.

Reviewer set: independent parser, contract, and workspace review agents

Contract reviewed: `01-contract.md`

Evidence reviewed: `02-evidence-migration.md`

## Acceptance to review

- Direct `duplicates` returns an owned ordered variable list and normalizes
  command case and separator whitespace.
- A leading unquoted or string `report` alias is stripped, while a backtick
  quoted `` `report` `` remains a variable.
- Conditions, options, assignments, missing `if` expressions, trailing commas,
  and unsupported punctuation use the recorded diagnostics.
- No active relation/schema lookup, duplicate grouping, null counting,
  variable validation, wildcard/range expansion, execution, result rendering,
  backend dependency, or unsafe code is added.
- Existing parser commands and deferred full-tokenizer behavior remain
  unchanged in scope.

## Review passes and findings

The independent parser pass reports no actionable in-scope finding at head
`a70bba0`; direct accepted forms, alias/quote behavior, diagnostics, and the
documented tokenizer deferrals match the pinned oracle. The workspace pass
reports no build, dependency, unsafe-code, public API, topology, or CI
configuration finding. The contract pass reports no authority, typed-API,
scope, or parity finding.

## Verification reviewed

Focused/full pinned Python parser-script checks and locked Rust baseline checks
are recorded in `02-evidence-migration.md`. Local dependency/unsafe policy scans
also pass. Hosted Rust baseline, ReadStat, and libgretl jobs are acceptance gates
for the current PR head; the final hosted run set is still pending after the
latest evidence and review revision.

## Disposition

Do not claim active-relation duplicate counts, null-key semantics, execution,
full varlist or tokenizer parity, prefixed-command support, CLI/script support,
or backend integration from this syntax-only slice. Mark accepted only after
all hosted checks report no finding and the PR is merged with its temporary
branch deleted.

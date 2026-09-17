# `duplicates` syntax review

Status: accepted; PR #20 was marked ready and squash-merged as
`5460c7b64ba852a969bafbd1551e893d40a31ac4`; the temporary branch was deleted.

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
also pass. All six hosted checks passed before merge: [Rust baseline and policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35218042159),
[ReadStat](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35218042320),
[libgretl feasibility](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35218042170),
and [libgretl OLS](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35218042215).

## Disposition

Do not claim active-relation duplicate counts, null-key semantics, execution,
full varlist or tokenizer parity, prefixed-command support, CLI/script support,
or backend integration from this syntax-only slice. The slice is accepted only
for the bounded syntax contract above; those execution and parity claims remain
deferred.

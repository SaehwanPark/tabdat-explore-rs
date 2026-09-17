# `summarize` syntax review

Status: accepted; PR #21 was marked ready and squash-merged as
`ae12a65cd3b5d0aea6def4d2592e20926168da93`; the temporary branch was deleted.

Reviewer set: independent parser, contract, and workspace review agents

Contract reviewed: `01-contract.md`

Evidence reviewed: `02-evidence-migration.md`

## Acceptance to review

- Direct `summarize` returns an owned ordered variable list and normalizes
  command case and separator whitespace.
- Quoted strings and backtick identifiers follow the bounded Python argument
  behavior, including doubled-backtick names and duplicate variables.
- Conditions, options, assignments, missing `if` expressions, trailing commas,
  and unsupported punctuation use the recorded diagnostics.
- Python structured `if`/option forms are explicitly deferred rather than
  represented by an incomplete generic AST.
- No active relation/schema lookup, numeric validation, summary computation,
  missingness scan, execution, result rendering, backend dependency, or unsafe
  code is added.
- Existing parser commands and deferred full-tokenizer behavior remain
  unchanged in scope.

## Review passes and findings

The independent parser pass reports no actionable in-scope finding at head
`e100c79`; direct accepted forms, quote behavior, diagnostics, and the
structured-form deferral match the pinned contract. The workspace pass reports
no build, dependency, unsafe-code, public API, topology, or CI configuration
finding. The contract pass reports no authority, typed-API, scope, or parity
finding after the required evidence and review artifacts are present.

## Verification reviewed

Focused/full pinned Python parser-script checks and locked Rust baseline checks
are recorded in `02-evidence-migration.md`. Local dependency/unsafe policy scans
also pass. All six hosted checks passed on final head
`4f03f37e944ff234f48cfb639b96639766988bb2`: [dependency policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35221567690/jobs/105202821398),
[Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35221567690/jobs/105202821628),
[ReadStat feasibility](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35221567716/jobs/105202725376),
[ReadStat Rust](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35221567716/jobs/105202725692),
[libgretl feasibility](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35221567602/jobs/105202724326),
and [libgretl OLS Rust](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35221567621/jobs/105202724462).

## Disposition

Do not claim active-relation summary statistics, numeric-type or missingness
semantics, execution, structured `if`/option parity, full varlist or tokenizer
parity, prefixed-command support, CLI/script support, or backend integration from
this syntax-only slice. The slice is accepted only for the bounded syntax
contract above; those execution and parity claims remain deferred.

# `codebook` syntax review

Status: review complete; PR #18 remains draft pending the hosted policy check
and the final all-checks gate.

Reviewer set: independent parser, contract, and workspace review agents

Contract reviewed: `01-contract.md`

Evidence reviewed: `02-evidence-migration.md`

## Acceptance to review

- Direct `codebook` returns an owned ordered variable list and normalizes command
  case and separator whitespace.
- Quoted strings and backtick identifiers follow the bounded Python argument
  behavior, including doubled-backtick escaping and adjacent non-backtick quote
  fragments.
- Conditions, options, assignments, missing `if` expressions, trailing commas,
  and unsupported punctuation use the recorded diagnostics.
- No active dataset/schema lookup, variable validation, wildcard/range
  expansion, execution, result rendering, backend dependency, or unsafe code is
  added.
- Existing parser commands and deferred full-tokenizer behavior remain
  unchanged in scope.

## Review passes and findings

The parser pass initially found three medium/low parity gaps. Revision `0816828`
rejects unsupported punctuation that had fallen through the bounded simple
argument scanner. Revision `7a87dfb` splits adjacent non-backtick quoted
fragments and stops treating doubled single/double quotes as escapes. Revision
`a2ddc72` also preserves adjacent empty quoted fragments. Follow-up probes report
no actionable in-scope finding. Revision `1ec1347` recognizes an unquoted `if`
condition boundary before an adjacent backtick fragment. Full-tokenizer cases
such as malformed numbers and attached-token boundaries remain explicitly
deferred.

The workspace pass reports no build, dependency, unsafe-code, public API,
topology, or CI configuration finding. The final contract pass reports no
authority, typed-API, scope, or parity finding.

## Verification reviewed

Focused/full pinned Python parser-script checks, locked Rust baseline checks,
policy scans, and `git diff --check` are recorded in `02-evidence-migration.md`.
The latest hosted Rust baseline, ReadStat, and both libgretl workflows have
passed; the dependency/unsafe policy job remains an acceptance gate for the
current PR head.

## Disposition

Do not claim dataset/schema inspection, execution, full varlist or tokenizer
parity, prefixed-command support, CLI/script support, or backend integration from
this syntax-only slice. Mark accepted only after the current PR head’s hosted
policy check joins the other five required checks.

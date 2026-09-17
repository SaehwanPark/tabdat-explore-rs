# Set-command syntax review

Status: accepted; PR #15 merged as `bdc1433` after final review and green hosted
checks.

Reviewer: task owner with three independent read-only review passes
Contract reviewed: `01-contract.md`
Evidence reviewed: `02-evidence-migration.md`

## Acceptance reviewed

- Bare direct `set` forms produce an owned `Command::Set` with a finite
  `SettingName` and preserved value text.
- The three recognized setting names normalize case-insensitively while
  backtick-quoted names are rejected as unknown.
- Quoted values, paths with `/`, and tokenizer-supported symbol values remain
  strings; values are not validated or applied in this language-only slice.
- Exact diagnostics cover malformed arity, unknown names, conditions, options,
  assignments, unsupported punctuation, and trailing commas.
- Existing parser commands remain unchanged, including the documented unresolved
  `status -/+` diagnostic difference in `docs/migration/decisions.md`.
- No session/configuration mutation, filesystem or plotting access, execution,
  result, serialization, CLI, or native backend dependency was added.

## Review passes and findings

### Parser/parity pass

The first pass found four parity gaps, all fixed before finalization:

1. `<=` and `>=` were initially mistaken for assignment separators in setting
   values; `a54e55f` preserves those contiguous tokenizer symbols.
2. Non-tokenizer punctuation such as `\\`, `;`, `@`, `$`, and emoji was initially
   accepted; `a54e55f` rejects it with the pinned unsupported-token diagnostic.
3. Adjacent quoted fragments were initially merged using the generic doubled-quote
   escape behavior; `a54e55f` and `8fcd522` match the pinned tokenizer boundaries.
4. An unquoted `if` after a symbol or quoted fragment was initially swallowed as
   value text, and contiguous backtick fragments were initially split;
   `2d51ed8` now follows the pinned condition and backtick-token boundaries.

Regression tests cover each finding. The final pass reported no remaining parser
or existing-command regression.

### Contract/scope pass

The first pass repeated the same findings and noted that the evidence record
needed the current implementation revisions. The evidence now records
`627591b`, `58426aa`, `a54e55f`, `8fcd522`, and `2d51ed8`, and the final contract
pass found no remaining auditability, scope, or deferral issue.

### Workspace/policy pass

No workspace, dependency, unsafe-code, manifest, lockfile, workflow, or native
spike-isolation finding remains. The current local evidence includes locked
workspace `check`, all baseline tests/lints, `cargo deny`, `cargo audit`, and the
metadata-driven per-package Geiger scan. Native ReadStat/libgretl workflows stay
path-scoped and do not become product dependencies.

## Verification reviewed

- Pinned configuration parser test: 1 passed, 488 deselected.
- Full pinned parser/script suite: 516 passed.
- Locked Rust fmt/check/test/Clippy and `git diff --check`: passed (one root
  smoke test, 16 language unit tests, six integration tests).
- Local `cargo deny check`, `cargo audit -D warnings`, and both workspace-package
  Geiger scans: passed with zero unsafe usage.
- The final pushed PR head was `0457e75`; baseline, policy, ReadStat, and libgretl
  workflows all passed before merge.

## Disposition

All identified findings are fixed and covered by tests. Do not claim
configuration execution, value validation, persistence, plotting, or general
tokenizer/option parity from this syntax-only slice. The merged PR leaves those
runtime and broader grammar concerns explicitly deferred.

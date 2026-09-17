# `datasignature` syntax review

Status: partial; draft PR #16 awaits final review and green hosted checks.

Reviewer: task owner with independent parser, contract, and workspace passes

Contract reviewed: `01-contract.md`

Evidence reviewed: `02-evidence-migration.md`

## Acceptance reviewed

- Direct bare `datasignature` produces an owned fieldless
  `Command::Datasignature` and normalizes command case/whitespace.
- Exact pinned diagnostics cover arguments, conditions, options, assignments,
  missing `if` expressions, unsupported `==`/`-`/`+` tokens, and trailing commas.
- Existing parser commands and the unresolved `status -/+` deviation remain
  unchanged.
- No hashing, active-relation access, session mutation, execution, result
  rendering, filesystem/environment access, or backend dependency was added.

## Review passes and findings

### Parser/parity pass

The first pass found two medium parity gaps: missing-`if` diagnostics after
arguments or before commas, and unsupported symbols after an argument. Commit
`41f26dc` routes the body through the shared tokenizer, preserves Python's
diagnostic precedence, and adds regression cases for both findings plus `!`.
The current direct-command implementation now follows the pinned zero-argument
contract. Prefixed `by:` handling and full tokenizer behavior remain outside
the contract.

### Contract/scope pass

The contract cites the pinned model, parser, test, and command documentation
paths, records the parser/docs `by:` discrepancy, and keeps SHA-256 and data
semantics deferred. The parity fixes update the contract's precedence and
unsupported-token rows; final independent review is pending.

### Workspace/policy pass

The change is limited to the language crate, tests, current-state docs, and
workspace evidence. No manifest, lockfile, unsafe, native-spike, or workflow
scope changed. Baseline policy scans and hosted checks remain required.

## Verification reviewed

- Focused pinned `datasignature` parser test: 1 passed, 10 deselected.
- Full pinned parser/script suite: 516 passed.
- Rust fmt, locked check/test, Clippy, and `git diff --check`: passed locally
  (one root smoke test, 18 language unit tests, seven integration tests).
- Draft PR #16 hosted baseline, policy, ReadStat, and libgretl checks: pending
  or in progress at the latest pushed head.

## Disposition

No in-scope finding is currently open. Do not claim signature computation,
active-dataset behavior, result serialization, `by:` wrappers, or complete
tokenizer parity from this syntax-only slice. Mark the PR ready only after the
independent reviews and all hosted checks are green.

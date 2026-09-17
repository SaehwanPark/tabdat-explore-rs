# Describe-command syntax review

Status: accepted; PR #13 merged as `f6525b3`
Reviewer: task owner with three independent read-only review passes
Contract reviewed: `01-contract.md`
Evidence reviewed: `02-evidence-migration.md`

## Acceptance reviewed

- Bare `describe` parses into an owned `Command::Describe` value with the
  existing case-insensitive and command-whitespace behavior.
- The contract's argument, `if`, option, assignment, `==`, unsupported-minus,
  and trailing-comma diagnostics are covered with exact-message tests.
- Existing command forms and diagnostics remain unchanged, including the
  pre-existing generic rejection for `status -1` and `status +1`.
- Parsing stays pure and backend-independent; no session, relation, execution,
  result, serialization, CLI, or native-backend dependency was added.
- Current-state and roadmap wording does not claim a usable CLI or data
  runtime, and the evidence records the pinned oracle and deferred boundaries.

## Review passes and findings

### Parser/parity pass

The parser review found no remaining in-scope actionable finding. The explicit
`describe` contract matrix matches the pinned Python oracle, including exact
diagnostics. An initial review found that the implementation had accidentally
changed `status -1` and `status +1`; those unrelated branches were removed and
regression assertions were added before this review was finalized.

Malformed-token, punctuation, expression, option, varlist, and command-boundary
differences outside the contract remain intentionally deferred to a future
general tokenizer/parser slice.

### Contract/scope pass

No actionable contract or scope finding remains. The source/test mapping,
Python revision/tree, typed API, pure-parser boundary, documentation wording,
and roadmap update align with the bounded zero-argument syntax contract.

### Workspace/policy pass

No actionable workspace, dependency, unsafe-code, isolation, or CI regression
was found. The root workspace still contains only the scaffold binary and
`tabdat-language`; DuckDB, ReadStat, and libgretl spikes remain isolated. The
metadata-driven policy checks continue to cover every workspace package.

## Verification reviewed

- Pinned parser subset: 419 passed, 70 deselected.
- Full pinned parser/script suite: 516 passed.
- Locked Rust fmt/check/test/Clippy and `git diff --check` passed after the
  status-regression fix (one root smoke test, 12 unit tests, four integration
  tests).
- Hosted ReadStat and the pre-branch main checks were green; the latest PR
  head remains the merge authority for the queued CI and libgretl checks.

## Disposition

All current-head checks passed, PR #13 was marked ready and squash-merged as
`f6525b3`, and the local and remote feature branches were deleted. Do not check
broad parser, execution, or Phase 4 roadmap items from this syntax-only evidence.

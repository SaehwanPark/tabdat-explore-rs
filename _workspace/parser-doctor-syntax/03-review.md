# Doctor-command syntax review

Status: accepted; PR #14 merged as `0b918c7`
Reviewer: task owner with three independent read-only review passes
Contract reviewed: `01-contract.md`
Evidence reviewed: `02-evidence-migration.md`

## Acceptance reviewed

- Bare `doctor` parses into an owned `Command::Doctor` value with the existing
  case-insensitive and command-whitespace behavior.
- The contract's argument, `if`, option, assignment, `==`, `-`, `+`, missing
  expression, and trailing-comma diagnostics are covered with exact-message
  tests.
- Existing command forms remain unchanged; the known `status -/+` difference is
  explicitly tracked as unresolved in `docs/migration/decisions.md`.
- Parsing stays pure and backend/environment-independent; no session, relation,
  execution, result, serialization, CLI, or native-backend dependency was added.
- Current-state and roadmap wording does not claim a usable CLI or environment
  diagnostics, and the prior `describe` evidence now records its merged status.

## Review passes and findings

### Parser/parity pass

No actionable parser finding remains. The explicit `doctor` contract matrix
matches the pinned Python oracle, including exact diagnostics. Existing commands
and the documented unresolved `status -/+` deviation remain unchanged.

Malformed-token, punctuation, expression, option, varlist, and prefixed-command
differences outside the contract remain intentionally deferred to future
tokenizer/parser slices.

### Contract/scope pass

One low-severity auditability finding was fixed before finalization: MIG-0002 now
lists affected source/test/evidence paths, a minimal reproduction, user impact,
resolution criteria, blocked parity claims, and review tracking. No doctor
implementation or scope finding remains.

### Workspace/policy pass

No actionable workspace, dependency, unsafe-code, isolation, or CI regression
was found. The root workspace still contains only the scaffold binary and
`tabdat-language`; DuckDB, ReadStat, and libgretl spikes remain isolated. The
metadata-driven policy checks continue to cover every workspace package.

## Verification reviewed

- Pinned doctor parser subset: 6 passed, 8 deselected.
- Full pinned parser/script suite: 516 passed.
- Locked Rust fmt/check/test/Clippy and `git diff --check` passed (one root smoke
  test, 14 unit tests, five integration tests).
- Local `cargo deny check`, `cargo audit -D warnings`, and the per-package
  `cargo geiger` loop passed.
- The latest pushed PR head remains the merge authority for the queued CI and
  isolated ReadStat/libgretl checks.

## Disposition

All current-head checks passed, PR #14 was marked ready and squash-merged as
`0b918c7`, and the local and remote feature branches were deleted. Do not check
broad parser, environment, execution, or Phase 4 roadmap items from this
syntax-only evidence.

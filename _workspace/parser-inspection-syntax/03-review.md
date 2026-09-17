# Inspection-command syntax review

Status: partial pending hosted checks  
Producer: task owner with three independent review passes  
Consumer: merge gate and next maintainer  
Contract reviewed: `01-contract.md`  
Evidence reviewed: `02-evidence-migration.md` at `8492187`; current review
documentation revision: `b3f9294`

## Acceptance reviewed

- `count`, `head [n]`, and `tail [n]` parse into owned typed values only;
- defaults, zero/leading-zero normalization, quoted numeric limits, and
  arbitrary-length ASCII decimals are preserved without bounded integer parsing;
- the listed exact diagnostics are covered by unit/integration tests;
- no execution, session state, backend initialization, or runtime dependency was
  added;
- current-state and roadmap wording does not claim a usable CLI or data runtime;
- local Rust, policy, documentation, Python-oracle, and isolated-spike checks are
  recorded in the evidence artifact.

## Review passes and findings

### Parser/parity pass

The parser review exercised the contract matrix and inspected the `RowLimit`
representation. The explicit contract cases match the pinned Python oracle. A
second probe corrected an initial review hypothesis about adjacent quoted
fragments: at the pinned revision `head "1""0"` and `head '1''0'` both produce
Python's `accepts at most one row limit`, so the bounded implementation's
deferred quote-token edge is not a contract regression.

Residual differences for malformed later quote tokens, attached punctuation,
compound symbols such as `!=`, multi-point number diagnostics, expression errors,
missing assignment expressions, duplicate `if` clauses, and malformed option
lists are intentionally outside this slice's no-general-tokenizer boundary.
They are recorded in `02-evidence-migration.md` and remain future tokenizer/parser
work, not accepted execution behavior.

### Contract/scope pass

No actionable implementation or scope finding. Source and test paths, Python
revision/tree, Rust API, deferred execution boundary, and roadmap wording align
with the contract. The evidence artifact was updated to the current pushed
revision before merge.

### Workspace/policy pass

No actionable workspace or CI regression. The root workspace still contains only
the binary and `tabdat-language`; the independent DuckDB, ReadStat, and libgretl
spike workspaces remain isolated. No dependency, unsafe-policy, or spike workflow
files changed in this PR. The metadata-driven Geiger scan covers both workspace
packages.

## Required follow-up

Keep PR #12 draft until all checks for current head `b3f9294` are green. Then mark
it ready, verify the merge state is clean, merge to `main`, and delete the local
and remote feature branch. Do not check the broad parser, execution, or Phase 4
roadmap items from this syntax-only evidence.

# `run` syntax slice review

Status: draft; parser review is approved, while contract/workspace review and
the current hosted check set remain pending.

## Scope reviewed

The review covers the bounded direct `run <script-path>` syntax slice only:

- `Command::Run { path: String }` and the pure parser in
  `crates/tabdat-language/src/lib.rs`;
- public parser cases in `crates/tabdat-language/tests/parser_contract.rs`;
- runtime deferral in `crates/tabdat-runtime/src/lib.rs` and
  `crates/tabdat-runtime/tests/use_contract.rs`;
- the pinned contract in `01-contract.md`, migration evidence in
  `02-evidence-migration.md`, and current-state notes in README/SPEC/
  ARCHITECTURE/roadmap;
- the pinned Python source, tests, docs, focused/full oracle output, locked
  Rust checks, policy checks, and hosted workflow links.

## Independent findings and resolutions

### Parser review — approved

The parser review found no actionable behavior or diagnostic findings. It
confirmed that the run parser uses the project’s Python-compatible whitespace
predicate, requires exactly one token, preserves quote/backtick/punctuation
text, and handles the frozen `run,`, `run,foo`, `run=foo`, `run==foo`, and
`run:foo` command boundaries. Runtime execution remains explicitly deferred.

Residual full-tokenizer behavior, path normalization/expansion, and script
execution are outside this slice and are recorded as deferred.

### Contract/evidence review — pending

The contract review was requested against the current head after the evidence
correction. The previous review pass found no scope or authority defect after
the implementation choice was documented: Rust owns the exact raw token as a
`String`, while Python’s `Path` normalization and all path/file effects remain
deferred.

### Workspace review — prior findings resolved; confirmation pending

The workspace review initially identified two documentation issues:

1. Hosted evidence listed only the first three jobs and incorrectly said native
   workflows were not expected. Commit `7a9d9b5` updates `02-evidence-migration.md`
   to the current `030842d` check set and enumerates all seven jobs, including
   the passing ReadStat and libgretl jobs.
2. `03-review.md` was missing. This artifact resolves that completeness blocker.

The initial SPEC wording also said local/oracle checks were pending even though
the evidence recorded them as passed; `7a9d9b5` narrows that wording to review,
hosted, merge, and post-merge evidence. A current-head workspace confirmation
is still pending before acceptance.

## Verification reviewed

Local verification at implementation/docs revisions passed:

- pinned Python focused selection: `419 passed, 70 deselected`;
- pinned Python parser/script regression: `516 passed`;
- `cargo fmt --all -- --check`;
- locked workspace check/test and warnings-as-errors Clippy;
- `cargo deny check`, `cargo audit -D warnings`, and metadata-driven geiger
  with zero first-party unsafe functions/expressions;
- `git diff --check`.

At documentation head `030842d`, all seven required hosted jobs were recorded;
the subsequent evidence correction is at `7a9d9b5` and restarts that set. The
current baseline/policy/runtime statuses must all pass before the PR is marked
ready. No script execution, filesystem access, session mutation, or backend
capability was added.

## Disposition

Current disposition: **partial / not yet approvable for merge** until the
contract/workspace confirmations and all current-head hosted checks pass. Once
those gates pass, update this file and `02-evidence-migration.md` with the final
head, links, squash merge SHA, deleted-branch verification, and post-merge main
workflow results. The bounded syntax-only slice may then be accepted; broad
script-engine and path semantics remain unchecked on the roadmap.

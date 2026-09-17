# `run` syntax slice review

Status: accepted bounded slice; parser, contract, and workspace reviews are
approved and the hosted/merge gates passed.

Accepted implementation/evidence source: PR head `2857af6`, squash-merged as
`77f4754b4b0875b5e22e32c09d4b4854bb3427bb`. Producer: task owner. Consumers:
future maintainers of the bounded parser contract.

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

### Contract/evidence review — approved

The contract review was requested against `ea112b1` and revalidated the
accepted evidence at PR head `2857af6`. It found no scope or
authority defect after the implementation choice was documented: Rust owns the
exact raw token as a `String`, while Python’s `Path` normalization and all
path/file effects remain deferred. It requested two evidence improvements,
addressed in the `ea112b1` evidence revision: citations for the generic
dispatch/tokenizer paths behind the attached-boundary diagnostics, plus a
reproducible pinned oracle probe and a non-pending revision ledger. The
contract/evidence content is approved.

### Workspace review — approved

The workspace review initially identified two documentation issues:

1. Hosted evidence initially listed only the first three jobs and incorrectly
   said native workflows were not expected. Commit `7a9d9b5` corrected the
   wording and enumerated all seven jobs; the current evidence revision retains
   those links as historical and records the complete passing `ea112b1` set in
   `02-evidence-migration.md`.
2. `03-review.md` was missing. This artifact resolves that completeness blocker.

The initial SPEC wording also said local/oracle checks were pending even though
the evidence recorded them as passed; `7a9d9b5` narrowed that wording, and the
accepted SPEC now records the final hosted, merge, and post-merge evidence.

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
the subsequent corrections are `7a9d9b5`, `fc61dfa`, `ea112b1`, and `2857af6`.
The complete `ea112b1` and final `2857af6` baseline/policy/runtime/native sets
passed, followed by the all-green post-merge `main` set for `77f4754`. The
boundary probe and generic parser citations are recorded in the contract and
evidence artifacts. No script execution, filesystem access, session mutation,
or backend capability was added.

## Disposition

Final disposition: **accepted bounded syntax-only slice**. PR #24 was marked
ready only after all seven `2857af6` checks passed, squash-merged to `main`, and
verified with all seven post-merge checks for `77f4754`; the temporary branch was
deleted locally and remotely. Broad script-engine and path/file semantics
remain unchecked on the roadmap.

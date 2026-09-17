# `select` syntax slice review

Status: review requested; corrected implementation, hosted acceptance, and
merge are pending.

Producer: task owner. Consumers: maintainers of the bounded parser contract.

## Scope to review

- `Command::Select { variables }` and the pure parser in
  `crates/tabdat-language/src/lib.rs`;
- public parser cases in `crates/tabdat-language/tests/parser_contract.rs`;
- runtime deferral in `crates/tabdat-runtime/src/lib.rs` and
  `crates/tabdat-runtime/tests/use_contract.rs`;
- the pinned contract/evidence artifacts in this workspace slice;
- current-state README/SPEC/ARCHITECTURE/roadmap wording once updated;
- pinned Python authority, focused/full oracle results, local locked checks,
  policy checks, hosted workflows, merge, and branch cleanup.

## Review questions

1. Does the direct variable-list parser match the pinned Python model, tests,
   generic quote behavior, and frozen diagnostics without broadening tokenizer
   or execution scope?
2. Does the Rust command own its values and stay backend-independent, with an
   exhaustive runtime mapping and explicit unsupported-command result?
3. Are the contract citations, oracle provenance, local results, workflow
   links, current-state docs, and deferred semantics reproducible and non-stale?
4. Are the required checks green at the exact head being reviewed, with no
   unrelated changes or hidden state mutation?

## Independent findings

- Parser review approved implementation head `6f4bd37`: the direct parser,
  quote handling, exact diagnostics, explicit colon boundary, and runtime
  deferral remain within the bounded syntax-only scope.
- Contract review identified and the working evidence revision corrected two
  citation issues (`parser.py:126` for `select`, and `cli.py:133,308-313`) and
  added a reproducible pinned probe with raw output. Final contract approval at
  the evidence head is pending.
- Workspace review identified the missing evidence/review artifacts and
  current-state documentation; this revision adds both artifacts and updates
  README/SPEC/ARCHITECTURE/roadmap without enabling execution. Final workspace
  approval at the evidence head is pending.

## Disposition

Pending findings, hosted checks, readiness, squash merge, temporary-branch
cleanup, and post-merge `main` verification.

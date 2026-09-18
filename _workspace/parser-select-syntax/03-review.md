# `select` syntax slice review

Status: accepted; independent reviews, hosted checks, readiness, merge, branch
cleanup, and post-merge verification are complete or recorded below.

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

- Parser review approved corrected implementation head `90f4c41`: the direct
  parser, quote handling, exact diagnostics (including missing assignment
  expressions), explicit colon boundary, and runtime deferral remain within
  the bounded syntax-only scope.
- Contract review approved `90f4c41` after the evidence revision corrected the
  `parser.py:126` and `cli.py:133,308-313` citations, matched the
  missing-assignment-expression diagnostic, and added reproducible pinned
  probes with raw output.
- Workspace review approved the scoped artifacts and current-state updates at
  `90f4c41`; provenance wording distinguishes pre-fix evidence from the
  corrected implementation, and the final evidence-head hosted checks passed.

## Disposition

Accepted bounded slice. PR #26 was marked ready after all final evidence-head
checks passed, squash-merged as `5735b43`, and its temporary branch was deleted
locally and remotely. The merge-triggered `main` checks are recorded in the
evidence artifact and passed before this handoff closeout.

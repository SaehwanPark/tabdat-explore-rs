# `sort` syntax slice review

Status: accepted. Independent review, hosted checks, readiness, squash merge,
branch cleanup, and post-merge verification are complete and recorded below.

Producer: task owner. Consumers: maintainers of the bounded parser contract.

## Scope to review

- `Command::Sort { variables }` and the pure parser in
  `crates/tabdat-language/src/lib.rs`;
- public parser cases in `crates/tabdat-language/tests/parser_contract.rs`;
- runtime deferral in `crates/tabdat-runtime/src/lib.rs` and
  `crates/tabdat-runtime/tests/use_contract.rs`;
- the pinned contract/evidence artifacts in this workspace slice;
- current-state README/SPEC/ARCHITECTURE/roadmap wording once updated;
- pinned Python authority, focused/full oracle results, local locked checks,
  policy checks, hosted workflows, merge, and branch cleanup.

## Review questions

1. Does the direct variable-list parser match the pinned Python model, focused
   tests, generic quote behavior, and frozen diagnostics without broadening
   tokenizer or execution scope?
2. Does the Rust command own its values and stay backend-independent, with an
   exhaustive runtime mapping and explicit unsupported-command result?
3. Are the contract citations, oracle provenance, raw probe output, local
   results, workflow links, current-state docs, and deferred semantics
   reproducible and non-stale?
4. Are the required checks green at the exact head being reviewed, with no
   unrelated changes or hidden state mutation?

## Independent findings

- Parser review approved `c19ac0c`; the direct parser, command/argument
  punctuation boundaries, quote errors, exact diagnostics, control whitespace,
  ownership, and runtime deferral remain within the bounded syntax-only scope.
- Contract review approved `c19ac0c` after independently reproducing the pinned
  accepted forms, diagnostics, focused/full oracle results, and local package
  tests. The follow-up `d574ec2` adds the leading-punctuation regressions.
- Workspace review found no dependency, unsafe, backend/session, topology, or
  unrelated-file issue. The requested evidence/review ledgers and draft
  current-state notes are now present in the current PR.
- The complete hosted set for `dc82d92` passed: policy, Rust baseline, runtime
  boundary, ReadStat, and libgretl jobs are linked in the evidence artifact.
- Merge-triggered checks for `7cf21ae` passed: policy, Rust baseline, runtime
  boundary, ReadStat, and libgretl jobs are linked in the evidence artifact;
  the temporary branch was deleted locally and remotely.

## Disposition

Accepted bounded slice: the implementation, independent reviews, pinned
oracle/local/policy evidence, current-state notes, complete PR-head hosted
checks, squash merge as `7cf21ae`, branch cleanup, and merge-triggered `main`
checks are green. Documentation-only closeout checks remain to be appended to
the evidence ledger after this update.

# `rename` syntax slice review

Status: independent reviews complete; PR #25 remains draft pending final
ready/merge/branch-cleanup gates.

Accepted review source will be the final PR #25 head after evidence updates.
Producer: task owner. Consumers: maintainers of the bounded parser contract.

## Scope to review

- `Command::Rename { old_name, new_name }` and the pure parser in
  `crates/tabdat-language/src/lib.rs`;
- public parser cases in `crates/tabdat-language/tests/parser_contract.rs`;
- runtime deferral in `crates/tabdat-runtime/src/lib.rs` and
  `crates/tabdat-runtime/tests/use_contract.rs`;
- the pinned contract/evidence artifacts in this workspace slice;
- current-state README/SPEC/ARCHITECTURE/roadmap wording once updated;
- pinned Python authority, focused/full oracle results, local locked checks,
  policy checks, hosted workflows, merge, and branch cleanup.

## Review questions

1. Does the direct two-argument parser match the pinned Python model, tests,
   generic quote behavior, and frozen diagnostics without broadening tokenizer
   or execution scope?
2. Does the Rust command remain owned and backend-independent, with exhaustive
   runtime dispatch and an explicit unsupported-command result?
3. Are contract citations, oracle provenance, local results, workflow links,
   current-state docs, and deferred semantics reproducible and non-stale?
4. Are the required checks green at the exact head being reviewed, with no
   unrelated changes or hidden state mutation?

## Independent findings

- Parser review: approved at `f376c84`; no findings. The direct two-argument
  parser, quote handling, exact diagnostics, and explicit colon boundary stay
  within the bounded syntax-only scope; execution remains deferred.
- Contract review: approved after the `parser.py:127` inventory citation and
  heredoc probe were corrected. The pinned authority, observed outputs, Rust
  mapping, and deferred behavior have no remaining parity or scope defects.
- Workspace review: approved after the current-state notes and all three
  artifacts were tracked. The code-bearing hosted set at `f376c84` is green;
  the review/evidence closeout is documentation-only.

Verification recorded by the reviewers and task owner:

- pinned oracle focused selection: `419 passed, 70 deselected`;
- pinned parser/script suite: `516 passed`;
- local locked workspace, formatting, clippy, diff, dependency, advisory, and
  unsafe-code policy checks: passed;
- seven hosted checks at `f376c84`: passed (links in `02-evidence-migration.md`).

## Disposition

Approved for PR readiness and squash merge once the current draft head's
required checks pass. Final acceptance still requires marking PR #25 ready,
merging it, deleting the temporary branch locally/remotely, and recording
post-merge `main` checks.

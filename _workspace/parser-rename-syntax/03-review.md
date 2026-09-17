# `rename` syntax slice review

Status: review requested; final disposition is pending independent parser,
contract, and workspace review plus hosted acceptance.

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

Pending reviewer responses. Findings and resolutions will be recorded here
before PR #25 is marked ready.

## Disposition

Pending. The bounded slice is not accepted until parser, contract, and
workspace reviewers approve it and the hosted/merge/post-merge gates pass.

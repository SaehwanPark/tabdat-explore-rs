# Review: bounded `status` sign-token diagnostics

Status: implementation and hosted checks passed; independent review is pending.

Reviewer: independent read-only parser reviewer, reconciled by task owner

## Acceptance checks

- [x] Freeze the pinned Python contract for spaced and attached sign forms.
- [x] Preserve exact `status if` missing-expression behavior.
- [x] Keep generic status arguments, assignments, options, and runtime behavior
  unchanged.
- [x] Add unit and public integration regressions.
- [x] Run the pinned focused and parser/script oracle suites.
- [x] Run locked Rust, formatting, Clippy, diff, advisory, license, and
  first-party unsafe-code checks locally.
- [x] Confirm current-head hosted Rust, policy, and runtime jobs pass.
- [ ] Complete independent review, mark PR ready, squash-merge PR #30, and
  delete the temporary branch locally and remotely.
- [ ] Record post-merge `main` verification.

## Evidence reviewed

- `01-contract.md` and the pinned source/test/doc paths;
- implementation revision `ce73613`;
- `crates/tabdat-language/src/lib.rs` and
  `crates/tabdat-language/tests/parser_contract.rs`;
- `docs/migration/decisions.md` MIG-0002;
- local and hosted evidence in `02-evidence-migration.md`.

## Findings

Pending the independent read-only parser review. The bounded contract must not
be expanded into whole-tokenizer parity during this PR; broader status
punctuation, malformed numbers, and malformed condition expressions remain
explicitly unresolved.

## Required follow-up

Complete the independent review, then record ready/merge/branch-cleanup and
post-merge `main` checks. Keep MIG-0002 unresolved for the broader matrix and
keep the roadmap tokenizer checkbox unchecked.

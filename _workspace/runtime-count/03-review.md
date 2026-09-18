# Review: bounded runtime `count`

Status: independent review complete; PR #32 remains draft pending current-head
hosted checks.

Reviewer: independent read-only runtime reviewer, reconciled by task owner

## Acceptance checks

- [x] Freeze the pinned Python active-dataset count success/error contract.
- [x] Keep the result read-only over the existing eager `DatasetInfo.row_count`
  cache.
- [x] Preserve failed-replacement atomicity and backend non-initialization.
- [x] Add exact runtime regressions for fresh, active, repeated, and failed
  replacement states.
- [x] Run the pinned focused oracle probe.
- [x] Run locked Rust, formatting, Clippy, diff, advisory, license, and
  first-party unsafe-code checks locally.
- [x] Complete independent review with no P0/P1/P2/P3 findings.
- [ ] Confirm current-head hosted Rust, policy, and runtime jobs pass.
- [ ] Mark PR ready, squash-merge PR #32, and delete the temporary branch
  locally and remotely.
- [ ] Record post-merge `main` verification.

## Evidence reviewed

The reviewer read `01-contract.md`, implementation revision `6e47ab0`,
`crates/tabdat-runtime/src/lib.rs`,
`crates/tabdat-runtime/tests/use_contract.rs`, and
`02-evidence-migration.md`, and compared the behavior with the pinned Python
paths listed in the contract.

## Findings

No P0/P1/P2/P3 findings. The implementation matches the frozen contract:

- `count` returns an owned result from the cached eager row count;
- a fresh session produces the exact no-active-dataset diagnostic without
  initializing DuckDB;
- repeated count is read-only and stable; and
- a failed replacement load leaves the prior active dataset and count visible.

The review also confirmed that parser behavior was already covered by the
existing public language tests, and that the cached eager implementation does
not expand the slice into lazy materialization or a new backend failure mode.

## Required follow-up

Keep the slice bounded to eager local-Parquet row counts. Do not mark the broad
inspection/session gate complete, and do not imply lazy count, status tracking,
transforms, labels, formatting, CLI, or MCP parity. Hosted checks and the
post-merge branch/`main` closeout are still required.

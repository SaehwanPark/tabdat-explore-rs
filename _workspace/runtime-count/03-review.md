# Review: bounded runtime `count`

Status: independent review complete; PR #32 was marked ready and squash-merged
as `2287fff`. Post-merge `main` verification is green; this closeout push will
run one docs-only workflow.

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
- [x] Confirm current-head hosted Rust, policy, and runtime jobs pass.
- [x] Mark PR ready, squash-merge PR #32, and delete the temporary branch
  locally and remotely.
- [x] Record post-merge `main` verification.

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
transforms, labels, formatting, CLI, or MCP parity. The current-head hosted
checks and post-merge `main` jobs all passed; only the docs-only closeout run
remains to be recorded.

# Review: bounded runtime `head`

Status: independent review complete; PR #33 was marked ready and squash-merged
as `b107251`. Post-merge `main` verification and docs-inclusive hosted
workflows are green.

Reviewer: independent read-only runtime reviewer, reconciled by task owner

## Acceptance checks

- [x] Freeze the pinned Python eager-preview success/error contract.
- [x] Keep the public result Rust-owned with no DuckDB/Arrow lifetimes.
- [x] Preserve schema order, source insertion order, nulls, and exact decimals.
- [x] Define deterministic zero-limit, `i64::MAX`, and overflow behavior.
- [x] Preserve failed-replacement atomicity and backend non-initialization.
- [x] Add exact runtime regressions for fresh, active, repeated, ordered,
  zero, oversized, value-conversion, and failed-replacement states.
- [x] Run the pinned focused oracle probe.
- [x] Run locked Rust, formatting, Clippy, diff, advisory, license, and
  first-party unsafe-code checks locally.
- [x] Complete independent review with no remaining P0/P1/P2/P3 findings.
- [x] Confirm current-head hosted Rust, policy, runtime, and auxiliary jobs
  pass.
- [x] Mark PR ready, squash-merge PR #33, and delete the temporary branch
  locally and remotely.
- [x] Record post-merge `main` verification.

## Evidence reviewed

The reviewer read `01-contract.md`, implementation revisions `414d8da`,
`e9a869c`, and `a79c815`, `crates/tabdat-runtime/src/lib.rs`,
`crates/tabdat-runtime/tests/use_contract.rs`, and
`02-evidence-migration.md`, and compared behavior with the pinned Python
paths listed in the contract.

## Findings and resolution

The initial review found four bounded issues, all resolved before readiness:

- the preview query now uses `row_number() OVER ()` with explicit positional
  ordering, matching the oracle's insertion-order contract;
- a deliberately reordered `[42, 30, 54]` fixture and exact `i64::MAX` limit
  regression now guard the two boundary cases;
- the `Command::Head` language comment no longer claims execution is deferred;
  and
- `GEOMETRY` is rejected as an unsupported logical type instead of being
  silently classified as ordinary bytes.

The re-review of `e9a869c`/`a79c815` reported no remaining merge-blocking
findings. The final implementation matches the frozen contract: fresh sessions
return the exact no-active diagnostic without backend initialization; successful
previews are owned and read-only; query/conversion failures display `head
failed`; and failed replacements preserve the prior dataset.

## Required follow-up

Keep this slice bounded to eager local-Parquet previews. Do not imply `tail`,
lazy materialization, transforms, labels, formatting, CLI, JSON, MCP, or broad
inspection/session parity. The roadmap `head` item is accepted; `tail` remains
the next bounded preview target.

## Hosted verification

Current-head PR checks:

- [dependency and unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35347629750/job/105607848665);
- [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35347629750/job/105607848905); and
- [tabdat-runtime on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35347629813/job/105607850297).

Post-merge `main` checks:

- [main dependency and unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35349781739/job/105614878164);
- [main Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35349781739/job/105614877899); and
- [main tabdat-runtime](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35349781741/job/105614877774).

The merge push also passed the [ReadStat feasibility spike](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35349781792), [libgretl OLS Rust spike](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35349781724), and [libgretl feasibility spike](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35349781803).

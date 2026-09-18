# Review: bounded runtime `describe`

Status: independent review complete; PR #31 was marked ready and squash-merged
as `6fccd5d`. Final post-merge `main` verification and docs-only CI are green.

Reviewer: independent read-only runtime reviewer, reconciled by task owner

## Acceptance checks

- [x] Freeze the pinned Python active-dataset success/error contract.
- [x] Keep the result read-only over the existing eager `DatasetInfo` cache.
- [x] Preserve failed-replacement atomicity and backend non-initialization.
- [x] Add exact runtime regressions for fresh, active, repeated, and failed
  replacement states.
- [x] Run the pinned focused oracle probe.
- [x] Run locked Rust, formatting, Clippy, diff, advisory, license, and
  first-party unsafe-code checks locally.
- [x] Complete independent review with no P0/P1/P2/P3 findings.
- [x] Confirm current-head hosted Rust, policy, and runtime jobs pass.
- [x] Mark PR ready, squash-merge PR #31, and delete the temporary branch
  locally and remotely.
- [x] Record post-merge `main` verification.

## Evidence reviewed

The reviewer read `01-contract.md`, implementation revision `cea90b1` and its
test-tightening follow-up `6e40f0f`,
`crates/tabdat-runtime/src/lib.rs`,
`crates/tabdat-runtime/tests/use_contract.rs`, and
`02-evidence-migration.md`, and compared the behavior with the pinned Python
paths listed in the contract.

## Findings

No P0/P1/P2/P3 findings. The implementation matches the frozen contract:

- `describe` returns an owned clone of the cached `DatasetInfo`;
- a fresh session produces the exact no-active-dataset diagnostic;
- dispatch does not initialize DuckDB or issue a query;
- a failed replacement load leaves the prior active dataset visible; and
- parser-to-runtime and repeated-read behavior are covered.

The reviewer noted that backend non-initialization was initially inferred by
the integration test. The task owner added a private unit assertion to the
existing session test so the implementation now directly verifies
`session.backend.is_none()` after the fresh-session `describe` error.

## Required follow-up

Keep the slice bounded to cached eager local-Parquet metadata. Do not mark the
roadmap's broad inspection/session gate complete, and do not imply `count`,
`head`, `tail`, lazy materialization, labels, formatting, CLI, or MCP parity.
Hosted checks passed on docs-inclusive head `402b6e5`:

- [dependency and unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35328639245/job/105547554847), 19m38s;
- [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35328639245/job/105547555160), 20m18s; and
- [tabdat-runtime on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35328639397/job/105547556014), 20m45s.

The PR was marked ready, squash-merged as
`6fccd5ded1e6d45d3f77534bc507a511adfe0c41`, and its temporary branch was
deleted locally and remotely. Post-merge `main` verification passed at
`e737897d6f94dedcca2444267577145e4ee4e6bd`:

- [main dependency and unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35330635949/job/105553999096), 19m49s;
- [main Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35330635949/job/105553999315), 20m28s; and
- [main tabdat-runtime](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35330482045/job/105553443539), 20m57s.

The closeout push also passed ReadStat and both libgretl feasibility workflows.
The final docs-only head `7018ef11de2ae6ece6331d4a275e636f8cede87b` passed
[main CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35332474871).

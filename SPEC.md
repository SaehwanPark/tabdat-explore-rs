# TabDat Rust implementation state

## Current behavior

The Rust 2024 binary is a scaffold: it prints `Hello, world!` and exits successfully.
It does not implement the TabDat language, data commands, or statistical models.
The [proposal](docs/TABDAT_RUST_PORT_PROJECT_PROPOSAL.md) and
[roadmap](docs/TABDAT_RUST_PORT_ROADMAP.md) describe planned work, not support.

## Verified slice: reproducible build baseline

- Pin a Rust toolchain and commit the binary's lockfile.
- Enforce two-space indentation using rustfmt and EditorConfig.
- Forbid unsafe code in the ordinary binary and smoke-test its existing output,
  empty stderr, and successful exit without third-party runtime dependencies.
- Run formatting, all-target check/test, and warnings-as-errors Clippy in GitHub
  Actions on pull requests and pushes to `main`, using the lockfile.
- Document exact local checks and distinguish build health from migration parity.

Evidence: the smoke test passed locally; the two-space formatter rejected the
original four-space source before correction. All four baseline commands passed
locally and in [PR #2 CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/34186271522)
on the initial implementation revision. Every subsequent revision must also pass
CI before merge. The smoke test characterizes the scaffold, not a migrated public
CLI contract.

This build slice excluded parser/runtime/backend implementation, Python oracle
recovery, statistical parity, security auditing, benchmarks, packaging, and a
minimum-supported-Rust claim.

## Active slice: Python migration authority

Pin an upstream-identifiable Python commit/tree, inventory behavioral authority and
fixture entry points, and define conflict/deviation and baseline-update rules.
Record the separate Rust repository decision and current/proposed architecture.
Acceptance: local clean checkout and GitHub commit/tree agree; every inventoried
path exists at that revision; bounded parser/script oracle checks are recorded
without implying full-suite or Rust parity; guidance has no stale missing-pin claim.
No Python source edits, dependency installation, backend work, or migrated commands.

## Next

Complete Phase 0 security tooling before backend feasibility and integration.

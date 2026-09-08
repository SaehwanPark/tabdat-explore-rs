# TabDat Rust implementation state

## Current behavior

The Rust 2024 binary is a scaffold: it prints `Hello, world!` and exits successfully.
It does not implement the TabDat language, data commands, or statistical models.
The [proposal](docs/TABDAT_RUST_PORT_PROJECT_PROPOSAL.md) and
[roadmap](docs/TABDAT_RUST_PORT_ROADMAP.md) describe planned work, not support.

## Active slice: reproducible build baseline

- Pin a Rust toolchain and commit the binary's lockfile.
- Enforce two-space indentation using rustfmt and EditorConfig.
- Forbid unsafe code in the ordinary binary and smoke-test its existing output,
  empty stderr, and successful exit without third-party runtime dependencies.
- Run formatting, all-target check/test, and warnings-as-errors Clippy in GitHub
  Actions on pull requests and pushes to `main`, using the lockfile.
- Document exact local checks and distinguish build health from migration parity.

Acceptance: the smoke test passes locally; formatting rejects four-space Rust;
all four baseline commands pass locally and on the PR's latest hosted CI revision.
The smoke test characterizes the scaffold, not a migrated public CLI contract.

Excluded: parser/runtime/backend implementation, Python oracle recovery, statistical
parity, security auditing, benchmarks, packaging, and a minimum-supported-Rust claim.

## Next

Pin the Python migration baseline and authority/deviation policy, then complete
Phase 0 security tooling and architecture decisions before backend integration.

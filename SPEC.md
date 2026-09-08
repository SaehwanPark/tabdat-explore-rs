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

## Verified slice: Python migration authority

Pin an upstream-identifiable Python commit/tree, inventory behavioral authority and
fixture entry points, and define conflict/deviation and baseline-update rules.
Record the separate Rust repository decision and current/proposed architecture.
Acceptance: local clean checkout and GitHub commit/tree agree; every inventoried
path exists at that revision; bounded parser/script oracle checks are recorded
without implying full-suite or Rust parity; guidance has no stale missing-pin claim.
No Python source edits, dependency installation, backend work, or migrated commands.

## Verified slice: dependency and unsafe-code checks

Pin the security-tool versions used by CI, configure dependency license/advisory
policy for the current scaffold, run `cargo deny`, `cargo audit`, and `cargo geiger`
in CI, and document that these checks cover the Rust workspace rather than Python
or future native backends. Keep runtime dependencies unchanged.

Evidence: each tool has an explicit version and locked installation; local runs
passed on the pinned toolchain; [PR #4 CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/34187594609)
ran all three policy checks and the Rust baseline successfully on the latest
revision. Policy scope and expected future review are recorded in ADR 0002. No
dependency, backend, FFI, or product-command implementation was added.

## Verified slice: architecture ownership

Create `ARCHITECTURE.md` as the current-state and target-boundary authority. Distinguish
the implemented scaffold from proposed modules, define language → execution → backend
direction, typed state/effect boundaries, lazy capabilities, and unsafe/FFI ownership.
Evidence: `ARCHITECTURE.md` links the proposal, roadmap, ADRs, and migration policy;
it names current scaffold evidence and explicit exclusions; it introduces no crate,
command, backend, or parity claim. Documentation-only change; no code or dependency
change.

## Verified slice: DuckDB feasibility prototype

Evaluate an isolated `duckdb-rs` candidate for local CSV/Parquet loading, repeated
active-relation inspection, and Arrow result batches. Measure release orientation
costs and record ownership/unsafe/license/platform evidence without adding DuckDB to
the root runtime. Evidence: tiny inspectable tests passed locally and in the Linux path-scoped
[PR #6 workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/34194987287);
local macOS release evidence is recorded. Remote/S3, production session integration,
and parity remain explicitly deferred.

## Verified slice: ReadStat feasibility reconnaissance

Evaluate the pinned ReadStat `v1.0.0` release as a native DTA-ingestion candidate
without adding a Rust binding or product support. Verify the release archive,
record the macOS Apple Silicon build/test evidence and hosted Linux build/test
result, and document callback ownership, label/missingness surfaces, licensing,
and explicit adapter prerequisites. Do not claim DTA ingestion, labels,
missingness parity, or FFI safety from the native upstream test suite.

Evidence is recorded in [the ReadStat feasibility report](docs/feasibility/readstat.md)
and [ADR 0004](docs/adr/0004-readstat-feasibility.md); the path-scoped [PR #7
workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/34197952522)
built the pinned source and passed all four upstream tests on Linux.

## Verified slice: libgretl feasibility reconnaissance

Evaluate a pinned gretl/libgretl release as a native estimator-backend candidate
without adding a Rust binding or statistical product support. Verify the source
archive, record macOS Apple Silicon library/NIST evidence and hosted Linux
build/link/test evidence, and document initialization, native-handle ownership,
thread/dependency, and GPLv3 licensing constraints. Do not claim estimator parity,
FFI safety, or accepted backend status from native tests.

Evidence is recorded in [the libgretl feasibility report](docs/feasibility/libgretl.md)
and [ADR 0005](docs/adr/0005-libgretl-feasibility.md); the path-scoped [PR #8
workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/34201353211)
built the pinned source and passed all 11 NIST tests on Linux.

## Next

Resolve the DuckDB prototype's remaining platform/ownership/semantic gaps before
any production backend integration, or continue ReadStat/libgretl only through
fixture-backed low-level adapter contracts.

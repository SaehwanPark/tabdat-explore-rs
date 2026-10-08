# Closeout: bounded linear-regression VIF runtime

## Outcome: partial

PR #154 adds typed `estat vif` parsing and a library-runtime implementation for successful full-rank OLS/WLS/GLS model states. The runtime retains the complete-case ordered predictor design, runs unweighted auxiliary OLS models using the original intercept convention, and returns an owned `TableResult` with ordered predictor rows and `mean_vif` where defined. Pure-statistics tests preserve `+inf` for exact auxiliary dependence.

The broader roadmap item remains unchecked: the Rust main regression kernel rejects rank-deficient designs, so Python's end-to-end exact-collinearity/infinite-VIF case cannot currently be reached. This slice does not change regression rank handling. Other `estat` commands and CLI/JSON/MCP rendering remain deferred.

## Evidence and validation

- Oracle identity: TabDat 0.25.0, commit `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, clean checkout, matching pinned lockfile digest; statsmodels 0.14.6 is the trusted implementation reference.
- Pinned oracle selection: **3 passed, 597 deselected**; exact-collinear test produces the expected infinity warning. Synthetic OLS/WLS/GLS, missing-weight sample, no-intercept, one-predictor, projection, and failure probes match the oracle and independent closed-form values (`27/7`, `171/46`, and `6600/2879`).
- Focused Rust: 4 statistics tests, 7 VIF runtime tests, and 3 `estat` parser tests pass; existing deferred-`estat` runtime test passes.
- Local `cargo fmt --all -- --check`, locked workspace `cargo check`, and warnings-as-errors Clippy pass. Serial Windows full-workspace testing reaches seven unrelated existing `export_contract` failures because test paths contain backslashes that the current parser rejects; hosted Linux full-workspace tests pass.
- `cargo deny check`, `cargo audit -D warnings`, and equivalent first-party `cargo-geiger` checks pass. Local `jq` absence prevented running the documented shell loop verbatim; a Python JSON-validation equivalent checked all four packages. Hosted policy workflow passes.
- Hosted checks on implementation head `7eb6ea4d400c53e83c9940511b7644ec75dacc9d` passed: Rust baseline, runtime boundary, dependency/unsafe policy, and configured ReadStat/libgretl feasibility workflows. The documentation-closeout head is rechecked before merge; see [PR #154](https://github.com/SaehwanPark/tabdat-explore-rs/pull/154) for the authoritative review, checks, and merge/handoff state.

See [contract](01-contract.md), [oracle and reference evidence](02-evidence-migration.md), and [review](03-review.md). `SPEC.md`, `ARCHITECTURE.md`, and the Phase 7 roadmap distinguish the implemented full-rank boundary from the remaining parity gap.

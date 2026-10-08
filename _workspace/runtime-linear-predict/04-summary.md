# Closeout: bounded linear-regression prediction runtime

## Delivered

PR #153 implements `predict <target> [, xb]` and `predict <target>, residuals` in the library-only eager runtime after OLS/WLS/GLS. The session retains the regression outcome, ordered predictors, intercept setting, and least-squares result. Prediction columns are evaluated from the stored model over the current active dataset, staged and validated in DuckDB, then published; an owned `PredictionResult` returns updated dataset metadata. SQL NULL propagation, row/schema order, row counts, failure-state preservation, and active named-table synchronization are covered.

Other prediction kinds and estimator families, lazy prediction, CLI/JSON/MCP rendering, and broad post-estimation parity remain deferred.

## Evidence and validation

- Pinned oracle: Python TabDat v0.25.0, commit `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`; selected executor prediction tests: **3 passed, 414 deselected**.
- Focused Rust prediction/prerequisite tests: **10 passed** (8 runtime prediction tests + 2 parser/runtime tests).
- Trusted reference: all 16 fitted values and residuals are compared to the NIST/ITL certified Longley coefficient vector using predeclared hybrid fitted-value tolerance and absolute residual tolerance; fixture SHA-256 is recorded in the runtime test and evidence record.
- Local format, locked workspace check, and Clippy checks pass. Windows serial full-workspace testing encounters unrelated existing `export_contract` failures caused by raw backslash paths in command strings.
- Hosted checks at PR head `530dce5` passed: full Rust baseline (format, check, full workspace tests, Clippy), the runtime boundary workflow, dependency/unsafe policy (cargo-deny, cargo-audit, cargo-geiger), and the configured feasibility-spike workflows. The final docs-closeout head is rechecked before merge.

See [contract](01-contract.md), [migration and reference evidence](02-evidence-migration.md), and [review](03-review.md). `SPEC.md`, `ARCHITECTURE.md`, and the Phase 7 roadmap track the implemented boundary and explicit deferrals.

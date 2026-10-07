# Review: linear-regression prediction runtime

- **Scope:** implementation commit `c1c5e07`, tests in `crates/tabdat-runtime/tests/predict_contract.rs` and `predict_runtime_contract.rs`, the pinned command contract, and staged publication/session helpers.
- **Review type:** focused maintainer self-review before hosted PR checks; GitHub PR #153 remains draft pending checks.
- **Result:** No actionable issues found in the reviewed slice.

## Reviewed invariants

- `Session::last_regression()` retains its existing public return type while the private session state now pairs the result with its outcome, predictor order, and intercept setting.
- Only `xb` and residual kinds dispatch; unsupported kinds remain explicitly unsupported. Dataset prerequisite ordering remains ahead of model lookup.
- Target and required-column checks happen before touching DuckDB state. Residuals additionally require the original outcome. Errors preserve the active dataset and regression state.
- SQL identifiers are quoted, only finite fitted coefficients are rendered as SQL numeric literals, and terms follow the original predictor order. DuckDB stages the appended `DOUBLE` column, validates exact schema order/type and row count, and uses the existing transaction publication helper before session metadata synchronization.
- Tests cover NULL propagation, excluded estimation rows, OLS/WLS/GLS states, target and model-variable errors, unsupported kinds, a named active table on success and failure, and independent NIST Longley values.

## Residual validation status

Focused Rust tests, formatter, workspace `cargo check`, and Clippy pass locally. The Python oracle selection passes. The serial full Rust workspace test on Windows is blocked by unchanged `export_contract` tests parsing native backslash paths; full Linux hosted PR-head checks are still pending and remain required before merge. No backend-injected publication-failure test was added; staged inspection and transaction failures use the same cleanup/publication pattern as adjacent runtime transforms.

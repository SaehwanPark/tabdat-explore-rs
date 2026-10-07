# Contract: linear-model `predict, xb` and `predict, residuals`

## Status and scope

- **Producer / consumer:** main agent / implementation, reviewer, and next maintainer.
- **State:** implementation and focused validation complete; full hosted PR-head checks and independent review are pending.
- **Rust base:** `origin/main` at `8d18678` (`8d18678…`); working branch `feat/runtime-linear-predict`.
- **Python oracle:** TabDat `0.25.0`, commit `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree `601b236788872323af9277d2276a236154a0f129`; clean checkout and pinned `uv.lock` SHA-256 verified against `docs/migration/python-baseline.toml`.
- **Roadmap:** Phase 7 §9.2 (`predict, xb`, `predict, residuals`) and §9.3 (prediction validation).
- **Selected skills:** `tabdat-statistical-validation`, `tabdat-migration`, `simple-code-writer`, `code-reviewer`.
- **Risk:** medium — this adds a public post-estimation command that mutates a DuckDB relation and depends on model/sample state.

## Python contract recovered

Authoritative source is the pinned Python checkout:

- `src/tabdat/executor.py::_execute_predict` (linear regression dispatch and state requirements).
- `src/tabdat/backend.py::add_linear_prediction_column`, `_linear_prediction_sql`, and `_require_columns` (target checks, casts, SQL behavior, and missing values).
- `tests/test_executor.py::test_phase_13_predict_adds_linear_predictions_and_residuals` and `test_phase_13_predict_supports_weighted_regression_states`.
- `docs/commands/predict.md` describes the wider command; this slice intentionally ports only linear-model `xb` and residuals.

Recovered behavior for the bounded forms:

1. `predict <target>` defaults to `xb`; `predict <target>, xb` is equivalent. `predict <target>, residuals` computes observed outcome minus the linear prediction. Both use the prior regression model's stored parameters, including WLS/GLS fits, and evaluate every row in the current active relation (not only the estimation sample).
2. The generated target must not already exist. The model's predictor columns must still be present; the outcome column is additionally required for residuals. Errors leave the active data and model state unchanged.
3. The prediction uses the stored intercept (when present) plus ordered coefficient-times-predictor terms. Inputs are cast to double. SQL NULL in any predictor yields a NULL prediction; residuals are also NULL when the observed outcome is NULL. The result column is appended as a numeric double, with original column order, row order, and row count preserved.
4. A successful operation returns `TransformResult` with message `Predicted <target>` and publishes the new active dataset. Model state remains available for subsequent predictions.
5. Without an active dataset the command fails before model lookup. With an active dataset but no compatible preceding linear model, Python reports that `predict` requires a prior regression-family model. Other prediction families and kinds have their own state rules and are out of scope.

The pinned Python tests were run with `PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider tests/test_executor.py -k 'phase_13_predict_adds_linear_predictions_and_residuals or phase_13_predict_supports_weighted_regression_states or phase_13_predict_requires_prior_regression'`: **3 passed, 414 deselected**. Three statsmodels divide-by-zero warnings arise from the tiny perfect-fit fixture; the tests pass. Raw command output is not currently saved separately.

## Rust contract to implement

- Extend `Session::execute` for parsed `Command::Predict`.
- Implement only `PredictKind::Xb` and `PredictKind::Residuals` after the existing OLS/WLS/GLS regression state. Do not imply support for logit/probit, Bayesian, spatial, or any other parser-accepted predict kind.
- Store the linear model's outcome and ordered predictor names together with `LeastSquaresResult` as one session state value; keep `Session::last_regression()`'s existing public return shape.
- Stage and validate a new numeric column before publication; use existing quoted-identifier, DuckDB staging, failure-atomic publication, label reconciliation, and active named-table synchronization conventions.
- Return an owned `PredictionResult` containing the updated `DatasetInfo`. Use typed deterministic errors for missing model, target collision, missing required columns, and backend failures.
- Preserve stored model state and active dataset metadata if parsing, validation, query construction, staged inspection, or publication fails.
- Machine/terminal rendering beyond the runtime's typed result is not part of this slice; the current CLI does not render ordinary runtime result values.

## Test contract and acceptance

Write tests before the implementation and confirm the new positive tests fail because runtime prediction is currently unsupported. Cover:

1. `xb` default and explicit forms after OLS, plus residuals; inspect appended column name/type, exact row order and values, and preservation of source columns/count.
2. WLS and GLS model states are used without re-estimating or requiring weight columns at prediction time.
3. Missing predictor values produce NULL predictions; missing outcome additionally produces NULL residuals; all active rows remain present even when excluded from estimation.
4. Target collision, missing model, no active dataset, and missing predictor/outcome variables return deterministic errors and preserve dataset/model state.
5. A named active table is synchronized after success and unchanged on failure.
6. A NIST Longley runtime fixture compares `xb` and residuals to independently evaluated fitted values from the published NIST/ITL certified Longley coefficient vector. Declare a hybrid absolute/relative tolerance before judging; use a separate absolute tolerance for near-zero residuals. Preserve the certified parameter provenance and fixture hash in the evidence report.
7. Run the pinned Python prediction selections above and compare the same bounded semantic outputs where fixtures overlap.

Acceptance requires the focused runtime contract tests and all root locked Rust checks to pass; dependency/advisory/unsafe-code checks and hosted PR-head CI must pass before merge. Passing tests establish only this eager linear-regression prediction slice.

## Non-goals and stop conditions

- No support for `pr`, `spatial_lag`, `posterior_predictive`, other estimators, confidence intervals, prediction standard errors, saved draws, lazy relations, CLI presentation, JSON/MCP output, or comprehensive `predict` parity.
- Do not add a new dependency or change existing parser syntax.
- Stop and record a blocker rather than guess if the oracle and trusted reference disagree, a public state/error behavior is ambiguous, or a publication path cannot preserve the existing relation on failure.

## Initial repository evidence

- `tabdat-runtime` has a parser-only prediction test in `tests/predict_contract.rs` and currently returns `UnsupportedCommand` for `Command::Predict`.
- `Session` stores `Option<LeastSquaresResult>` but not the model's outcome/predictor mapping, which is necessary for residual and out-of-sample prediction.
- `tabdat-stats` already retains fitted values, residuals, ordered parameter names, and coefficients; `tabdat-runtime` has staged relation publication and active named-table synchronization helpers.
- Baseline `cargo fmt --all -- --check` and `cargo check --locked --workspace --all-targets` pass on this Rust base revision.
- Test-first evidence: `cargo test --locked -p tabdat-runtime --test predict_runtime_contract` compiled and reached runtime before implementation. The first attempt exposed a DuckDB fixture issue (the table-function input path was incorrectly bound inside `COPY`); the test then quoted that generated local path and still bound the output path. Three positive prediction tests failed with `UnsupportedCommand { name: "predict" }` as expected; the no-active and failure-atomicity guard tests ran. The tests now assert exact typed errors.
- Implemented validation: focused Rust prediction/parser suites pass (9 tests total); the pinned Python selection passes (3 passed, 414 deselected); the Longley runtime comparison passes against the NIST certified coefficient vector. `cargo fmt`, workspace `cargo check`, and workspace `cargo clippy -D warnings` pass locally.
- Platform note: serial full-workspace tests on Windows stop in the unchanged `export_contract` suite because existing tests feed absolute backslash paths to the language parser (`unsupported token in command: \\`). This failure is outside the changed files; Linux hosted checks are still required for acceptance.
- See [evidence and migration record](02-evidence-migration.md); hosted PR-head checks and independent review remain pending.

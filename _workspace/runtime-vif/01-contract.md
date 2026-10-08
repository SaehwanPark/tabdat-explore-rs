# Contract: bounded linear-regression VIF runtime

## Status and scope

- **Producer / consumer:** task owner / implementer, reviewer, and next maintainer.
- **State:** partial; the delivered runtime slice covers VIF for currently supported full-rank OLS/WLS/GLS fits. End-to-end exact-collinearity parity is blocked by the existing Rust estimator's rank-deficiency error and remains explicitly deferred.
- **Rust base:** `main` at squash merge `d44f38b046e082dc22d8e4640ce22f747f044476`; working branch `feat/runtime-vif`.
- **Python oracle:** TabDat 0.25.0, commit `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree `601b236788872323af9277d2276a236154a0f129`; clean checkout and lock digest reverified.
- **Reference:** statsmodels `0.14.6`, `statsmodels.stats.outliers_influence.variance_inflation_factor`; the pinned Python reference-validation matrix records `estat` VIF as reference-validated.
- **Roadmap:** Phase 7 §9.2, `Port VIF` (full roadmap item remains unchecked unless the rank-deficient end-to-end gap is resolved).
- **Selected skills:** `tabdat-migration`, `tabdat-statistical-validation`, `simple-code-writer`, `code-reviewer`.
- **Risk:** medium — VIF is a numerical post-estimation diagnostic based on the retained estimation design/sample.

## Python contract recovered

Authoritative implementation/tests:

- `src/tabdat/executor.py::_execute_estat` routes `estat vif` only to a previous `regress` state.
- `src/tabdat/executor.py::_estat_vif_table` obtains `fitted_model.model.exog`, then calls statsmodels `variance_inflation_factor` for each ordered predictor column (skipping the intercept when present).
- `tests/test_executor.py::test_phase_13_estat_residuals_ovtest_and_vif`, `test_phase_13_estat_vif_preserves_infinite_values`, and `test_phase_13_estat_supports_weighted_regression_states` characterize the supported cases; `tests/test_cli.py::test_cli_runs_phase_13_estat_flow` covers the visible table.
- `docs/reference_validation_matrix.json` cites the statsmodels VIF implementation as the trusted reference.

Recovered behavior:

1. `estat vif` requires an active dataset first and a preceding `regress` model. It returns a `TableResult` with headers `("Variable", "VIF")`, rows in ordered predictor order, and a final `("mean_vif", mean)` row when at least one VIF is non-null.
2. For each predictor, the auxiliary model regresses that predictor on all remaining predictor columns and the intercept iff the original model had one. The calculation uses the fitted regression model's retained `exog` rows; rows excluded from estimation are not reintroduced. The variance inflation factor is `1 / (1 - R²)`.
3. WLS/GLS VIF uses the same retained design matrix but does not apply observation weights to the auxiliary regressions; this is directly how the pinned helper passes `model.exog` to statsmodels.
4. Infinity is preserved (`R² == 1` gives `+inf`); NaN/unavailable VIF values become `None`. The mean includes non-null values and preserves positive infinity.
5. Failures in matrix inspection or auxiliary fits become `ExecutionError("estat vif failed for current model")`; no active model becomes `ExecutionError("estat requires a prior regress model")`.

Pinned-oracle selection executed with the recovered checkout/environment:

```sh
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider tests/test_executor.py tests/test_cli.py -k 'phase_13_estat_residuals_ovtest_and_vif or phase_13_estat_vif_preserves_infinite_values or cli_runs_phase_13_estat_flow'
```

Observed: **3 passed, 597 deselected**, one expected divide-by-zero warning for the infinite-VIF fixture. A separate pinned-oracle probe on a full-rank two-predictor fixture returned `x1=3.8571428571428563`, `x2=3.8571428571428563`, `mean_vif=3.8571428571428563`; the analytically independent value is `27/7`.

## Rust contract to implement

- Parse the already-described `estat vif` form into a typed `EstatSubcommand::Vif`; preserve existing diagnostics and keep other `estat` subcommands deferred.
- Retain the ordered complete-case predictor design alongside private linear-regression state so VIF remains based on the fitted estimation sample even if the active dataset is subsequently transformed.
- Add a pure `tabdat-stats` helper implementing auxiliary OLS fits with the original intercept convention and `1/(1-R²)`; return optional values for unavailable/non-finite `R²`, preserving `+inf` when the denominator is zero.
- Dispatch only `estat vif` for the currently supported linear regression state, returning the existing owned `ExecutionResult::Table` with Python-compatible headers, predictor order, and optional `mean_vif` row.
- Add deterministic typed errors for no prior linear regression and VIF calculation failure; check active-data prerequisite first. Leave every other `estat` form unsupported at runtime.
- Keep the result read-only: it must not mutate active dataset or model state and must not initialize a new statistical backend.

## Known blocker and deliberate boundary

Python's `test_phase_13_estat_vif_preserves_infinite_values` obtains infinite VIF after fitting an exactly collinear regression; statsmodels permits that fit. The current Rust `tabdat-stats` kernel explicitly rejects collinear predictors (`tests/estimation_contract.rs::fit_least_squares_rejects_collinear_predictors`). Changing rank-deficient regression behavior would be a separate statistical decision and is out of this slice. The pure VIF helper should still characterize `+inf` on an exact auxiliary dependence, but the runtime cannot expose that value when the preceding Rust `regress` is rejected. Document this as a parity gap; do not claim complete VIF parity or check the broad roadmap item.

## Test contract and acceptance

1. Parser accepts `estat vif` and rejects invalid subcommands/options with the existing diagnostics.
2. Runtime returns the exact typed prerequisite failures (no active data, then no prior regression); other `estat` subcommands remain unsupported.
3. A full-rank two-predictor fixture gives `VIF=27/7` for both predictors, correct order, and matching mean; compare Rust with pinned TabDat and statsmodels plus the closed-form correlation calculation using a declared absolute/relative tolerance.
4. A one-predictor fixture produces VIF 1. On multi-predictor WLS/GLS fixtures, extreme finite weights must not enter auxiliary fits, while a row with a missing weight/sigma is excluded with the original fitted sample.
5. A model's VIF uses the saved estimation design after the current dataset is projected/transformed; active dataset/model state remains unchanged.
6. Pure statistics tests preserve `+inf` for exact auxiliary collinearity and cover malformed dimensions/unavailable R²; end-to-end exact-collinear regression stays explicitly blocked as above.
7. Run focused Rust tests, pinned Python selection, root locked checks, dependency/advisory/unsafe policy, independent code review, and hosted PR-head CI before merge.

No CLI/JSON/MCP/table rendering, residual diagnostics, `ovtest`, other `estat` families, or rank-deficient main-regression changes are in scope.

## Initial repository and test-first evidence

- Rust `EstatSubcommand` currently includes only `FirstStage`, `Overid`, `Endogenous`, and `Hausman`; although the diagnostic text mentions `vif`, `parse_command("estat vif")` rejects it.
- `Session::execute` has no VIF runtime route, and private `LinearRegressionState` does not yet retain the fitted predictor design matrix.
- The existing pure QR kernel rejects rank-deficient predictor matrices (`crates/tabdat-stats/tests/estimation_contract.rs::fit_least_squares_rejects_collinear_predictors`), confirming the exact-collinearity parity boundary above.
- Test-first run: `cargo test --locked -p tabdat-language --test parser_contract estat_vif_is_recognized_by_the_parser` failed at `parse_command("estat vif").is_ok()`; `cargo test --locked -p tabdat-runtime --test estat_vif_contract` failed while parsing the accepted Python form with `ParseError("estat subcommand must be ...")`. These failures establish that the selected language/runtime slice is not currently implemented.

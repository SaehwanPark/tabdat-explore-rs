# Evidence and migration record: linear-regression VIF runtime

## Producer, consumer, revisions

- **Producer:** task owner / implementation author. **Consumer:** reviewer and next maintainer.
- **Rust base:** `d44f38b046e082dc22d8e4640ce22f747f044476`; branch `feat/runtime-vif`, first WIP test commit `bf4a206`; final implementation head is recorded at closeout.
- **Python oracle:** TabDat 0.25.0, commit `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree `601b236788872323af9277d2276a236154a0f129`.
- **Checkout:** `../tabdat-python-oracle`; clean at verification.
- **Lock:** `uv.lock` SHA-256 `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`, matches `docs/migration/python-baseline.toml`.
- **Reference implementation:** statsmodels `0.14.6`, `statsmodels.stats.outliers_influence.variance_inflation_factor`; the pinned Python `docs/reference_validation_matrix.json` records the VIF behavior as reference-validated.
- **Relevant oracle code:** `src/tabdat/executor.py::_execute_estat` (active dataset and regression-state routing), `_estat_vif_table` (ordered predictors, statsmodels call, infinity/mean handling), and `tests/test_executor.py::test_phase_13_estat_residuals_ovtest_and_vif`, `test_phase_13_estat_vif_preserves_infinite_values`, `test_phase_13_estat_supports_weighted_regression_states`; CLI coverage is `tests/test_cli.py::test_cli_runs_phase_13_estat_flow`.

## Oracle execution and observed values

Pinned selection, from `../tabdat-python-oracle`:

```sh
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider tests/test_executor.py tests/test_cli.py -k 'phase_13_estat_residuals_ovtest_and_vif or phase_13_estat_vif_preserves_infinite_values or cli_runs_phase_13_estat_flow'
```

Observed: **3 passed, 597 deselected**. The exact-collinearity fixture emits the expected statsmodels divide-by-zero warning and returns `+inf` for both predictor rows and `mean_vif`.

Additional one-off probes used synthetic Parquet fixtures and the pinned `Executor`/statsmodels path:

- Full-rank two-predictor OLS, WLS with highly unequal positive weights, and GLS with highly unequal sigma values each returned `x1=x2=mean_vif=3.8571428571428563`. Independent centered-correlation algebra gives `R²=20/27`, hence `VIF=27/7`.
- Removing the row whose WLS weight / GLS sigma is NULL gave `3.717391304347826` for each VIF and the mean. The retained four-row design gives `R²=125/171`, hence `VIF=171/46`.
- A no-intercept two-predictor model returned `2.2924626606460574` for each VIF and the mean; the uncentered dot-product reference gives `R²=3721/6600`, hence `VIF=6600/2879`.
- One predictor returns `VIF=1`. After `select y` removes predictor columns from the current active relation, VIF still uses the fitted model's retained design. A single-predictor no-intercept model normalizes its failed empty auxiliary fit to `ExecutionError("estat vif failed for current model")`.

The committed Rust fixtures use these independent closed-form values with a declared `1e-12` absolute tolerance for the small deterministic inputs; no tolerance was widened to obtain a pass.

## Rust changes and focused tests

Changed product files:

- `crates/tabdat-language/src/lib.rs`: adds the typed no-option `EstatSubcommand::Vif` while preserving other `estat` parser diagnostics and option rejection.
- `crates/tabdat-stats/src/diagnostics.rs` (new) and `src/lib.rs`: adds pure ordered auxiliary OLS VIF computation, same intercept convention, `None` for undefined R², and preserved infinity for exact auxiliary dependence.
- `crates/tabdat-runtime/src/lib.rs`: stores the fitted complete-case predictor design in private regression state; read-only `estat vif` dispatch checks active data then a prior regression, returns the Python-compatible owned table, and normalizes auxiliary errors.

Focused tests:

```sh
cargo test --locked -p tabdat-stats --test vif_contract
cargo test --locked -p tabdat-runtime --test estat_vif_contract
cargo test --locked -p tabdat-language --test parser_contract estat_
```

Observed after implementation: **4 statistics tests, 7 runtime VIF tests, and 3 parser `estat` tests pass**. The runtime tests cover ordered rows/mean, an analytically known two-predictor value, retained fitted design after active projection, OLS/WLS/GLS (including ignored auxiliary weights), complete-case exclusion for missing weights/sigma, no-intercept behavior, one-predictor VIF, prerequisite order, and normalized auxiliary-fit failure. Statistics tests cover finite closed-form values, exact auxiliary dependence returning infinity, undefined R², and invalid dimensions. The existing deferred-runtime `estat firststage` test also passes.

Test-first evidence is recorded in `01-contract.md`: before implementation the parser contract test failed because `estat vif` was rejected, and the runtime test failed on that parser error.

## Repository checks and residual local environment issues

- `cargo fmt --all -- --check`: passed.
- `cargo check --locked --workspace --all-targets`: passed.
- `cargo clippy --locked --workspace --all-targets -- -D warnings`: passed.
- `CARGO_BUILD_JOBS=1 cargo test --locked --workspace --all-targets`: compiled and ran, but seven existing `export_contract` cases failed on Windows because native backslashes in test command strings are rejected as `unsupported token in command: \\`; two other export tests passed. These tests do not touch VIF code. A first parallel Cargo invocation produced a transient missing-`rlib` error; the serial rerun proceeded to the unrelated path failures.
- `cargo deny check` and `cargo audit -D warnings`: passed; deny prints existing duplicate-lock-entry warnings.
- `cargo-geiger 0.13.0`: the repository's exact JSON validation loop could not run because local `jq` is not installed. An equivalent Python JSON check invoked `cargo geiger --manifest-path ... --all-dependencies --all-targets --locked --output-format Json` for all four workspace packages and confirmed each first-party package forbids unsafe and reports zero first-party unsafe functions/expressions (all geiger invocations exited 0).
- Hosted PR-head CI remains the acceptance gate for full Linux tests and the repository policy workflow.

## Explicit parity boundary

The Rust statistics helper can return `+inf` when an auxiliary predictor is exactly explained by the remaining predictors. The Python oracle can first fit an exactly collinear main regression and then expose infinite VIF end to end. Rust's pre-existing main regression kernel rejects rank-deficient designs (`crates/tabdat-stats/tests/estimation_contract.rs::fit_least_squares_rejects_collinear_predictors`), so that exact end-to-end case remains unavailable. This slice does not change rank-deficient estimator behavior; the full roadmap item remains unchecked. CLI/JSON/MCP table rendering and non-linear model routing are also not included.

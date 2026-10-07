# Evidence and migration record: linear-regression prediction runtime

## Source and oracle identity

- Python TabDat: v0.25.0, commit `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree `601b236788872323af9277d2276a236154a0f129`.
- Oracle checkout: `../tabdat-python-oracle`; worktree clean at verification.
- `uv.lock` SHA-256: `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239` (matches `docs/migration/python-baseline.toml`).
- Python source/tests: `src/tabdat/executor.py::_execute_predict`, `src/tabdat/backend.py::add_linear_prediction_column`, and `tests/test_executor.py` prediction tests at lines 4137, 4172, and 4218.
- Rust base: `8d18678` (`origin/main` when the branch was created); implementation branch `feat/runtime-linear-predict`.

## Oracle execution

Command, from the pinned checkout:

```sh
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider tests/test_executor.py -k 'phase_13_predict_adds_linear_predictions_and_residuals or phase_13_predict_supports_weighted_regression_states or phase_13_predict_requires_prior_regression'
```

Observed: **3 passed, 414 deselected**, with three statsmodels divide-by-zero/invalid warnings from the perfect-fit three-row fixture. The tests confirm the default `xb` result, residual result, WLS/GLS model state support, and no-prior-regression failure. The linear fixture's expected fitted values are `100`, `150`, `200`, with residuals `0`, `0`, and SQL NULL for the missing outcome.

The recovered backend contract additionally confirms that the existing target is rejected, predictors (and the outcome for residuals) must be available in the active schema, numeric columns are cast to `DOUBLE`, every current active row is evaluated, and nullable arithmetic produces SQL NULL. The Rust contract records the exact state/error scope and the intentional typed-result adaptation.

## Rust and trusted-reference validation

Focused command:

```sh
cargo test --locked -p tabdat-runtime --test predict_runtime_contract --test predict_contract
```

Observed after implementation: **10 passed** (8 runtime prediction tests and 2 parser/runtime prerequisite tests). Coverage includes default `xb`, explicit `xb`, residuals, row order/count, missing predictors and outcomes, target collisions, missing model/data, WLS/GLS after their weight columns are removed, unsupported `pr`, failure-state preservation (including an active named table), named-table synchronization, and an independent Longley reference.

The Longley fixture is `crates/tabdat-stats/fixtures/longley.csv`, SHA-256 `4ed9507415d5453fc13e8309e425fcb63b02f322cba132b2cba068d0c9558c3b`, with 16 observations. The independent certified coefficient vector is the NIST/ITL Statistical Reference Dataset (StRD), Longley values, recorded in `crates/tabdat-runtime/tests/predict_runtime_contract.rs` and already cross-checked for the fit in `crates/tabdat-stats/tests/longley_contract.rs`. The declared comparator is `max(abs_tol, rel_tol * abs(expected))`: fitted values use `abs_tol=1e-6`, `rel_tol=1e-9`; residuals use `abs_tol=1e-6`, `rel_tol=0` because near-zero residuals need an absolute criterion. All 16 rows are evaluated.

Local `cargo fmt --all -- --check`, `cargo check --locked --workspace --all-targets`, and `cargo clippy --locked --workspace --all-targets -- -D warnings` pass. The serial full workspace test command on Windows reaches pre-existing `export_contract` failures because those tests interpolate native backslash paths into the command parser (`unsupported token in command: \\`). No changed code touches export or parsing; Linux hosted full checks are the acceptance gate and remain to be recorded after implementation is pushed.

## Delimitations

This evidence establishes only eager linear-regression `xb` and residual prediction after OLS/WLS/GLS using the stored model coefficients. It does not establish other estimator families/kinds, lazy prediction, broader model-sample parity, or CLI/JSON/MCP rendering. Python returns a user-facing `TransformResult`; Rust intentionally returns the library-owned `PredictionResult`/`DatasetInfo` and does not claim presentation parity.

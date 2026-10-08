# Evidence: bounded linear-regression `test` runtime

## Status

Implementation and focused numerical validation are complete on branch `feat/runtime-test`. Independent review, local policy checks, and hosted PR-head/merge-head workflows remain pending.

## Verified inputs

- Rust base: `0b4ae1fd37a20c8bbc0f730b816f743166cd8b54`; changes are in the worktree based on that `origin/main` commit.
- Python oracle: TabDat 0.25.0, commit `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree `601b236788872323af9277d2276a236154a0f129`; checkout clean and lock digest verified.
- Oracle lockfile: `uv.lock` SHA-256 `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.
- Numerical environment: Python 3.13.15, statsmodels 0.14.6, SciPy 1.17.1.
- Fixture: the six observations and four cluster groups defined in [the contract](01-contract.md).

## Python contract and reference output

The pinned Python source is `Executor._execute_test` and the linear-expression helpers in `src/tabdat/executor.py`; result and command types are in `src/tabdat/models.py`. The pinned `tests/test_statistical_testing.py` module passed:

```sh
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider tests/test_statistical_testing.py
```

Result: **8 passed**.

Additional `Executor` probes on the pinned checkout covered parsed joint, equality, and multiple restrictions; OLS, WLS, current GLS, HC1, clustered, and no-intercept covariance modes; and the exact formatted result fields. Direct statsmodels F tests and SciPy F survival probabilities produced the reference values recorded in [the contract](01-contract.md). On the same four-cluster fixture, the clustered equality test returned F=8.789886334525322 and p=0.05931620014242662 from SciPy; Python returned F=8.78988633452532 and p=0.059316200142426334.

A scaled-outcome probe multiplied all outcome values by `1e-8`. Python returned F=8.789886334525155, p=0.05931620014242778, df=1/3, showing the Wald result is unchanged by this scale transformation.

## Test-first and Rust evidence

Before implementation, `cargo test --locked -p tabdat-runtime --test test_runtime_contract` failed to compile because `TestResult` and `ExecutionResult::Test` did not exist. After implementation:

- `cargo test --locked -p tabdat-runtime --test test_runtime_contract`: **4 passed**, including OLS joint/equality/multiple restrictions, all six supported regression modes, errors, state preservation, and scaled covariance.
- `cargo test --locked -p tabdat-stats --test f_distribution_contract`: **2 passed**, including SciPy reference values, invalid inputs, and a positive extreme-tail probability at F=1e20.
- `cargo test --locked -p tabdat-stats --test linear_combination_inference_contract`: **5 passed**, including a nonsingular restriction covariance below the previous absolute pivot threshold.
- `cargo fmt --all -- --check`: passed.
- `cargo check --locked --workspace --all-targets`: passed.
- `cargo clippy --locked --workspace --all-targets -- -D warnings`: passed.

The full Windows workspace test command reaches existing failures in `crates/tabdat-runtime/tests/export_contract.rs`: seven path cases pass native backslashes through the command parser and receive `unsupported token in command: \\\\ `. The failing file and path cases are outside this change. The focused runtime and statistics test targets above pass locally. Hosted Linux acceptance is still required.

## Implementation mapping

- `crates/tabdat-runtime/src/lib.rs`: read-only `ExecutionResult::Test`, affine restriction conversion in fitted parameter order, Python-compatible labels/errors, and F-test metadata for stored linear-regression states.
- `crates/tabdat-stats/src/matrix.rs`: F-distribution survival probability using the regularized incomplete beta tail, avoiding CDF subtraction.
- `crates/tabdat-stats/src/post_estimation.rs`: normalize the restriction covariance matrix by its largest absolute element before inversion. This preserves the Wald quadratic form while avoiding the old absolute \(10^{-12}\) pivot rejection for valid small-scale covariance matrices.
- DuckDB is used only by integration fixtures. The command reads the stored fit and does not mutate data or model state.
- Unsupported estimator families and the zero-residual-df chi-square fallback remain deferred.

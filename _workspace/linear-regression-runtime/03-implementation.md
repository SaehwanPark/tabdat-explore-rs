# Implementation: Linear Regression Runtime Execution (`regress`)

## 1. Overview of Changes

We implemented the complete runtime execution pipeline for `regress` across `tabdat-stats` and `tabdat-runtime`:

1. **`crates/tabdat-stats`**:
   - `matrix.rs`:
     - Added `sandwich(a, b)` computing $A B A^T$ matrix quadratic forms.
     - Added `lgamma(z)` using Lanczos $g=7$ approximation.
     - Added `regularized_incomplete_beta(a, b, x)` via Lentz's method continued fraction.
     - Added `student_t_pvalue(t, df)` for exact two-tailed Student's $t$ p-values.
   - `least_squares.rs`:
     - Added `LeastSquaresOptions { covariance_type: CovarianceType }`.
     - Implemented `fit_least_squares_with_options` supporting:
       - Classical `CovarianceType::NonRobust`: $s^2 (X^T W X)^{-1}$.
       - Robust HC1 `CovarianceType::RobustHc1`: $\frac{n}{n - k} (X^T W X)^{-1} \left(\sum_i (w_i e_i)^2 x_i x_i^T\right) (X^T W X)^{-1}$.
       - Cluster-robust `CovarianceType::Cluster(var)`: $\frac{G}{G - 1} \frac{n - 1}{n - k} (X^T W X)^{-1} \left(\sum_g u_g u_g^T\right) (X^T W X)^{-1}$.
     - Accurately computes coefficient standard errors, $t$-statistics, two-tailed p-values, and 95% confidence intervals.
   - `error.rs`:
     - Added `StatsError::InsufficientObservations(String)`.
   - `tests/robust_cluster_tests.rs`:
     - Rigorously validates OLS non-robust, OLS robust HC1, OLS clustered covariance, WLS robust HC1, and WLS clustered covariance against reference values from statsmodels and the Python TabDat oracle (`16b45d9`). Observed relative errors are $< 10^{-13}$.

2. **`crates/tabdat-runtime`**:
   - `Cargo.toml`:
     - Added dependency on `tabdat-stats`.
   - `src/lib.rs`:
     - Added `RegressionResult` containing outcome, predictors, estimator, covariance, observation count, degrees of freedom, $R^2$, adjusted $R^2$, root MSE, coefficient estimates, and full `LeastSquaresResult`.
     - Added `ExecutionResult::Regression(Box<RegressionResult>)` to keep enum variant sizes compact.
     - Added `RuntimeError` variants: `RegressUnknownVariable`, `RegressRequiresNumeric`, `RegressNoObservations`, `RegressRequiresPositiveWeights`, `RegressRequiresPositiveSigma`, `RegressFailed`.
     - Added `last_regression: Option<tabdat_stats::LeastSquaresResult>` in `Session` and accessor `Session::last_regression(&self)`.
     - Implemented `Session::execute_regress(&mut self, command: &RegressCommand)`:
       - Validates active dataset prerequisite.
       - Validates variable existence and numeric types for outcome, predictors, and weights.
       - Queries DuckDB table for observation values.
       - Evaluates missingness/nulls across outcome, predictors, weights, and cluster variables while preserving row identity in `EstimationSample`.
       - Validates positive weights ($w_i > 0$) for WLS and positive sigma ($\sigma_i > 0$) for GLS.
       - Converts GLS 1D sigma to precision weights $w_i = 1 / \sigma_i$.
       - Solves regression via `tabdat_stats::least_squares::fit_least_squares_with_options`.
       - Stores estimation state in `self.last_regression`.
       - Returns `ExecutionResult::Regression(Box::new(result))`.
     - Dispatched `Command::Regress(ref regress)` in `Session::execute`.
   - `tests/regress_contract.rs`:
     - Comprehensive integration tests covering:
       - Missing active dataset rejection.
       - Unknown variable reporting.
       - Non-numeric variable rejection.
       - Empty or all-missing sample rejection.
       - Non-positive weights rejection for WLS.
       - Non-positive sigma rejection for GLS.
       - Classical OLS linear fit and session state persistence.
       - Robust HC1 covariance.
       - Clustered covariance.
       - WLS and GLS modes.
       - `noconstant` intercept suppression.
       - Missing row filtering and `EstimationSample` provenance tracking.

## 2. Verification Summary

- `cargo fmt --all -- --check`: PASSED.
- `cargo check --locked --workspace --all-targets`: PASSED.
- `cargo test --locked --workspace --all-targets`: PASSED (all tests in `tabdat-stats`, `tabdat-runtime`, `tabdat-language`, `tabdat-explore-rs`).
- `cargo clippy --locked --workspace --all-targets -- -D warnings`: PASSED.

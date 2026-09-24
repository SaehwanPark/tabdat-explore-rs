# Summary: Linear Regression Runtime Execution (`regress`)

## 1. Accomplishments

This slice implements the end-to-end runtime execution for linear regression (`regress`), fulfilling Roadmap Phase 7 §9.2 items:
- **OLS `regress`**: Classical ordinary least squares regression via Householder QR decomposition with back-substitution.
- **WLS (`wls(weight_var)`)**: Weighted least squares regression using precision weights $w_i > 0$.
- **GLS (`gls(sigma_var)`)**: Generalized least squares regression using 1D variance specification $\sigma_i > 0$ translated to precision weights $w_i = 1 / \sigma_i$ matching statsmodels/Python TabDat semantics.
- **Robust Covariance (`robust`)**: Stata-compatible HC1 robust sandwich covariance matrix $\frac{n}{n - k} (X^T W X)^{-1} \left(\sum_i (w_i e_i)^2 x_i x_i^T\right) (X^T W X)^{-1}$.
- **Clustered Covariance (`cluster(cluster_var)`)**: Cluster-robust sandwich covariance matrix with degrees-of-freedom correction $\frac{G}{G - 1} \frac{n - 1}{n - k} (X^T W X)^{-1} \left(\sum_g u_g u_g^T\right) (X^T W X)^{-1}$.

## 2. Key Technical Additions

1. **`crates/tabdat-stats`**:
   - `matrix.rs`:
     - Added `sandwich(a, b)` computing $A B A^T$ matrix quadratic forms.
     - Added Lanczos log-gamma `lgamma` ($g=7$) and regularized incomplete beta continued fraction `regularized_incomplete_beta` for Student's $t$ two-tailed p-values (`student_t_pvalue`).
   - `least_squares.rs`:
     - Added `LeastSquaresOptions { covariance_type: CovarianceType }`.
     - Implemented `fit_least_squares_with_options` supporting classical `NonRobust`, `RobustHc1`, and `Cluster(String)` covariance structures.
     - Computes standard errors, $t$-statistics, exact two-tailed p-values, and 95% confidence intervals.
   - `tests/robust_cluster_tests.rs`:
     - Validates OLS non-robust, OLS robust HC1, OLS clustered covariance, WLS robust HC1, and WLS clustered covariance against statsmodels and Python TabDat oracle (`16b45d9`) certified reference values ($< 10^{-13}$ observed relative error).

2. **`crates/tabdat-runtime`**:
   - Added dependency on `tabdat-stats`.
   - Added `RegressionResult` and `ExecutionResult::Regression(Box<RegressionResult>)`.
   - Added `RuntimeError` variants for missing/invalid variables and non-positive weights/sigma.
   - Added `last_regression: Option<LeastSquaresResult>` to `Session` and accessor `Session::last_regression(&self)`.
   - Implemented `Session::execute_regress` handling tabular extraction from DuckDB, missingness filtering across variables with sample identity preservation in `EstimationSample`, strict positive weights/sigma validation, and post-estimation session state persistence.
   - `tests/regress_contract.rs`:
     - 12 comprehensive integration tests covering input validation, missingness handling, sample tracking, OLS, WLS, GLS, robust HC1, clustered covariance, intercept suppression (`noconstant`), and session state storage.

## 3. Evidence & Verification

- PR #151 squashed and merged to `main` (`9476053`).
- All 3 CI workflow checks passed:
  - `CI/Rust baseline` (cargo fmt, cargo check, cargo test, cargo clippy -D warnings)
  - `TabDat runtime boundary`
  - `CI/Dependency and unsafe-code policy` (cargo deny, cargo audit, cargo geiger)
- All crates maintain `#![forbid(unsafe_code)]`.
- Zero runtime backend dependencies in `tabdat-stats`.

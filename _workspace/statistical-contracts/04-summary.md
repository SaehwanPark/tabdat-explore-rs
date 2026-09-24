# Summary: Statistical Contracts Substrate (`tabdat-stats`)

## 1. Slice Overview

- **Objective**: Roadmap Phase 7 §9.1 Statistical Contracts and Phase 2 §4.1 Crate Layout.
- **Pull Request**: [#149](https://github.com/SaehwanPark/tabdat-explore-rs/pull/149) (squash-merged to `main` as `8370622`).
- **Crate Created**: `crates/tabdat-stats` (`#![forbid(unsafe_code)]`, zero backend runtime dependencies).

## 2. Implemented Capabilities

1. **Explicit Sample Tracking (`EstimationSample`)**:
   - `retained_indices`: 0-based indices of original candidate rows retained in the estimation sample.
   - `total_observations`: Total candidate rows before missingness or condition filtering.
   - `dropped_observations`: Count of dropped rows (`retained_indices.len() + dropped_observations == total_observations`).
   - `weights`: Optional observation weights aligned with retained rows ($w_i > 0$).
   - `cluster_groups`: Optional cluster entity identifiers aligned with retained rows.
2. **Estimation Problem Specification (`EstimationProblem`)**:
   - Outcome and predictor variable names, response vector $y$, design matrix $X$.
   - Intercept inclusion options (`include_intercept`, `intercept_name`).
   - Explicit sample tracking (`sample`) and weights (`weights`).
   - Validation against empty dimensions, non-positive weights, and non-finite values (`NaN`, `Inf`).
3. **Parameter Estimates & Covariance Representation**:
   - `CoefficientEstimate`: Point estimate, standard error, test statistic ($t$/$z$), p-value.
   - `CovarianceMatrix` & `CovarianceType`: Full symmetric covariance representation supporting `NonRobust`, `RobustHc1`, and `Cluster(var)`.
   - `EstimationDiagnostics`: Method, convergence boolean, iteration count, objective value, residual sum of squares.
   - `FitStatistics`: Observation count, degrees of freedom, R², adjusted R², Root MSE, RSS, TSS, F-statistic, log-likelihood.
   - `LeastSquaresResult`: Complete owned result struct for linear estimation.
4. **Prediction Contract (`predict_linear_response`)**:
   - Pure, out-of-sample and in-sample linear prediction $\hat{y} = X \hat{\beta}$ with conformability checks.
5. **Post-Estimation State Contract (`PostEstimationModel`)**:
   - Parameter lookup by name: `parameter(name)`.
   - Linear combinations (`lincom`): $c^T \hat{\beta}$, standard error $\sqrt{c^T V c}$, and $t$-statistic.
   - General linear hypothesis tests (`test`): $R \beta = r$, computing Wald $F$-statistic and $\chi^2$-statistic.
6. **Backend Capability Trait & Error Normalization**:
   - `Estimator<Problem, Result>` capability trait.
   - `StatsError`: Normalized errors for singular matrices, zero degrees of freedom, empty samples, non-positive weights, and collinearity.
7. **Pure Baseline Numerical Kernel (`fit_least_squares`)**:
   - Householder QR decomposition avoiding condition-number squaring $\kappa(X)^2 \approx 10^{18}$.
   - Back-substitution and triangular inverse computation for $(X^T X)^{-1} = R^{-1} (R^{-1})^T$.

## 3. Verification & Validation Evidence

1. **NIST Longley Certified Reference Benchmark**:
   - Evaluated on the ill-conditioned NIST Longley dataset (`longley.csv`, condition number $\approx 10^9$).
   - Certified NIST reference values matched to $< 10^{-9}$ relative tolerance (observed: $\sim 10^{-13}$ to $10^{-15}$).
   - Pinned Python TabDat oracle (`16b45d9`) matched to $< 10^{-9}$ relative tolerance.
   - All 7 coefficients, standard errors, $R^2$, Root MSE, and $F$-statistic verified.
2. **Post-Estimation & Linear Restrictions**:
   - Joint linear restrictions $H_0: B_1 = 0, B_2 = 0$ verified on Longley.
   - Linear combinations evaluated with point estimates and standard errors.
3. **Continuous Integration**:
   - `CI/Rust baseline`: passed on macOS.
   - `CI/Dependency and unsafe-code policy`: passed (cargo-deny, cargo-audit, and cargo-geiger verifying `#![forbid(unsafe_code)]` across all packages).
   - `TabDat runtime boundary/smoke`: passed on Linux.

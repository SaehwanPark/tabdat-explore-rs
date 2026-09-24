# Implementation: Statistical Contracts Substrate (`tabdat-stats`)

## 1. Overview

Implements Roadmap Phase 7 §9.1 Statistical Contracts and Phase 2 §4.1 Crate Layout:
- Created safe crate `crates/tabdat-stats` with `#![forbid(unsafe_code)]` as an independent workspace member.
- Defined explicit problem, sample, estimate, covariance, diagnostics, and result types:
  - `EstimationProblem`: Typed model problem specification with outcome, predictor names, response, design matrix, intercept options, sample tracking, and optional weights.
  - `EstimationSample`: Explicit sample provenance tracking (`retained_indices`, `total_observations`, `dropped_observations`, `weights`, `cluster_groups`).
  - `CoefficientEstimate`: Individual parameter estimate with name, value, standard_error, statistic, p_value.
  - `CovarianceMatrix` & `CovarianceType`: Full parameter variance-covariance representation supporting nonrobust, robust HC1, and cluster modes.
  - `EstimationDiagnostics`: Estimation convergence metadata (method, converged, iterations, objective value, residual sum of squares).
  - `FitStatistics`: Observation count, degrees of freedom, R², adjusted R², Root MSE, RSS, TSS, F-statistic, log-likelihood.
  - `LeastSquaresResult`: Owned result of linear estimation.
  - `PredictionContract` (`predict_linear_response`): In-sample and out-of-sample linear prediction.
  - `PostEstimationModel`: Post-estimation parameter access, linear combinations (`lincom`), and Wald linear hypothesis tests ($R \beta = r$).
  - `Estimator` trait: Backend capability trait.
  - `StatsError`: Normalized typed statistical error hierarchy.
  - `fit_least_squares`: Pure Rust baseline linear least squares fitting using Householder QR decomposition with back-substitution and triangular inverse computation for numerical backward stability.

## 2. Numerical Stability & Algorithms

- **Householder QR Decomposition**:
  Rather than forming the normal equations $(X^T X) \beta = X^T y$ which squares the condition number $\kappa(X)^2 \approx 10^{18}$ on ill-conditioned data, `tabdat-stats` uses Householder QR decomposition:
  1. $X = Q R$ where $Q$ is orthogonal and $R$ is upper triangular.
  2. Solves $R \beta = Q^T y$ via back-substitution.
  3. Computes $(X^T X)^{-1} = R^{-1} (R^{-1})^T$ directly from upper-triangular $R^{-1}$.
  4. Yields 15 digits of agreement with certified NIST reference values on the notoriously collinear Longley benchmark.
- **Explicit Sample Tracking**:
  `EstimationSample` maintains `retained_indices` mapping each estimated row back to original dataset rows, ensuring sample composition is inspectable and identical sample sizes do not mask differing samples.

## 3. Verification & Validation Evidence

1. **NIST Longley Benchmark Three-Way Differential Validation**:
   - Compared against NIST/ITL Certified Reference Values to $< 10^{-9}$ relative tolerance (observed: $\sim 10^{-13}$ to $10^{-15}$).
   - Compared against pinned Python TabDat oracle (`16b45d9`) to $< 10^{-9}$ relative tolerance.
   - All 7 coefficients ($B_0 \dots B_6$), standard errors ($SE_0 \dots SE_6$), $R^2$, Root MSE, and $F$-statistic verified.
2. **Post-Estimation & Linear Hypothesis Testing**:
   - Tested Wald test $F$-statistic and $\chi^2$-statistic on Longley joint restrictions.
   - Tested linear combinations (`lincom`) point estimate, standard error, and test statistic.
3. **Workspace Policy & Checks**:
   - `cargo fmt --all -- --check`: PASS
   - `cargo check --locked --workspace --all-targets`: PASS
   - `cargo test --locked --workspace --all-targets`: PASS (all 10 new tests pass, existing 80+ tests pass)
   - `cargo clippy --locked --workspace --all-targets -- -D warnings`: PASS
   - `cargo deny check`: PASS
   - `cargo audit -D warnings`: PASS

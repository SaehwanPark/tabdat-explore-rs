# Contract: Statistical Contracts Substrate (`tabdat-stats`)

## 1. Scope & Objective

Implement Roadmap Phase 7 §9.1 Statistical Contracts and Phase 2 §4.1 Crate Layout:
- Create `crates/tabdat-stats` as a safe, backend-independent workspace member crate with `#![forbid(unsafe_code)]`.
- Define core statistical contract types:
  - `EstimationProblem`: Typed problem specification (outcome, predictors, response, design matrix, intercept options, sample, weights).
  - `EstimationSample`: Explicit, inspectable sample identity tracking (retained row indices, original observations count, dropped missing observations, weights, cluster groups).
  - `CoefficientEstimate`: Individual parameter estimate with name, value, standard_error, statistic, p_value.
  - `CovarianceMatrix` & `CovarianceType`: Symmetric parameter covariance representation (nonrobust, robust HC1, cluster).
  - `ModelMetadata` / `FitStatistics`: Observation count, degrees of freedom, R², adjusted R², Root MSE, ESS, TSS, F statistic, log-likelihood.
  - `EstimationDiagnostics`: Convergence and estimation metadata (method, converged, iterations, objective value, residual sum of squares).
  - `LeastSquaresResult`: Owned result of linear estimation.
  - `PredictionContract`: Out-of-sample and in-sample linear prediction `predict_linear_response`, fitted values, residuals.
  - `PostEstimationModel`: Post-estimation parameter access, linear hypothesis testing ($R \beta = r$), and linear combinations (`lincom`).
  - `Estimator` trait: Pure, typed backend capability trait (`fit(&self, problem) -> Result<ResultT, StatsError>`).
  - `StatsError`: Normalized typed statistical error hierarchy.
  - Pure Rust OLS/WLS baseline estimation (`fit_least_squares`, pure safe matrix inversion and operations).
  - Three-way differential validation against NIST Longley certified reference values, Python TabDat oracle (`16b45d9`), and Rust baseline.

Zero DuckDB or native FFI runtime dependencies exist in `tabdat-stats`.

## 2. Reference & Authority

- **Python Oracle Revision**: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`
  - `src/tabdat/estimation.py` (LeastSquaresProblem, LeastSquaresResult, CoefficientEstimate, EstimationDiagnostics, fit_least_squares, predict_linear_response, matrix operations)
  - `tests/test_estimation.py`
  - `src/tabdat/models.py` (RegressionResult, CoefficientEstimate)
- **External Trusted Reference**:
  - NIST/ITL Certified Regression Statistics (Longley benchmark dataset)
  - Relative tolerance declared: $10^{-9}$ for certified coefficients, standard errors, $R^2$, and residual standard deviation.
- **Skill**:
  - `.agents/skills/tabdat-statistical-validation/SKILL.md`

## 3. Contract Specifications

### 3.1 Types & Invariants

1. **`EstimationSample`**:
   - `retained_indices: Vec<usize>`: 0-based indices of original rows retained.
   - `total_observations: usize`: Total observations before filtering.
   - `dropped_observations: usize`: Count of rows excluded due to missingness/filtering.
   - Invariant: `retained_indices.len() + dropped_observations == total_observations`.
   - `weights: Option<Vec<f64>>`: Positive non-null observation weights.
   - `cluster_groups: Option<Vec<String>>`: Cluster identifiers aligned with retained rows.

2. **`EstimationProblem`**:
   - `outcome: String`: Dependent variable name.
   - `predictor_names: Vec<String>`: Predictor names in column order.
   - `response: Vec<f64>`: Numeric response vector ($N \times 1$).
   - `design_matrix: Vec<Vec<f64>>`: Matrix ($N \times K$).
   - `include_intercept: bool`: Defaults to `true`.
   - `intercept_name: String`: Defaults to `"intercept"`.
   - `weights: Option<Vec<f64>>`: Optional weights vector ($N \times 1$).
   - `sample: Option<EstimationSample>`: Explicit provenance sample.
   - Validation rejects:
     - Empty response or design matrix.
     - Dimension mismatches between response length, design matrix rows, predictor count.
     - Non-positive weights ($w_i \le 0$).
     - Non-finite numbers (`NaN`, `Inf`).
     - Empty variable names or whitespace-only names.

3. **`CoefficientEstimate`**:
   - `name: String`
   - `value: f64`
   - `standard_error: Option<f64>`
   - `statistic: Option<f64>`
   - `p_value: Option<f64>`

4. **`CovarianceMatrix` & `CovarianceType`**:
   - Matrix dimension matches parameter count ($P \times P$, where $P = K + 1$ with intercept or $K$ without).
   - `CovarianceType`: `NonRobust`, `RobustHc1`, `Cluster(String)`.
   - Diagonal elements yield parameter variances and standard errors: $SE_j = \sqrt{\max(V_{jj}, 0)}$.

5. **`FitStatistics`**:
   - `observation_count: usize` ($N$)
   - `parameter_count: usize` ($P$)
   - `degrees_of_freedom: usize` ($N - P$)
   - `r_squared: Option<f64>` ($1 - RSS / TSS$)
   - `adjusted_r_squared: Option<f64>` ($1 - \frac{RSS / (N - P)}{TSS / (N - 1)}$)
   - `root_mse: Option<f64>` ($\sqrt{RSS / (N - P)}$)
   - `residual_sum_of_squares: Option<f64>`
   - `total_sum_of_squares: Option<f64>`
   - `f_statistic: Option<f64>`
   - `log_likelihood: Option<f64>`

6. **`EstimationDiagnostics`**:
   - `method: String` ("least_squares", etc.)
   - `converged: bool`
   - `iterations: usize`
   - `objective_value: Option<f64>`
   - `residual_sum_of_squares: Option<f64>`

7. **`StatsError`**:
   - `EmptySample`: Sample size is 0.
   - `DimensionMismatch(String)`: Mismatched rows, columns, or vector lengths.
   - `SingularMatrix(String)`: Non-invertible design matrix / rank deficiency.
   - `ZeroDegreesOfFreedom { observations, parameters }`: When $N \le P$.
   - `NonPositiveWeights`: Weights $\le 0$.
   - `InvalidParameterName(String)`: Empty or malformed parameter names.
   - `NonFiniteValue(String)`: Encountered NaN or infinite value.
   - `UnknownParameter(String)`: Referenced parameter not in model.
   - `IncompatibleHypothesis(String)`: Malformed linear restriction matrix.

### 3.2 Pure Numerical Baseline (`fit_least_squares`)

Using pure safe Rust linear algebra:
1. Augment design matrix with intercept column (vector of $1.0$ at index 0) if `include_intercept` is true.
2. If weights $w$ are supplied:
   $\tilde{X}_i = \sqrt{w_i} X_i$, $\tilde{y}_i = \sqrt{w_i} y_i$.
3. Compute $X^T X$ and $X^T y$.
4. Compute $(X^T X)^{-1}$ using Gauss-Jordan elimination with partial pivoting. If maximum pivot $< 10^{-12}$, return `StatsError::SingularMatrix`.
5. Solve parameters $\hat{\beta} = (X^T X)^{-1} X^T y$.
6. Calculate fitted values $\hat{y} = X \hat{\beta}$ and residuals $e = y - \hat{y}$.
7. Calculate residual sum of squares: $RSS = \sum w_i e_i^2$.
8. Residual degrees of freedom $df = N - P$. If $df \le 0$, return `StatsError::ZeroDegreesOfFreedom`.
9. Compute $\hat{\sigma}^2 = RSS / df$.
10. Covariance matrix $V = \hat{\sigma}^2 (X^T X)^{-1}$.
11. Compute standard errors $SE_j = \sqrt{\max(V_{jj}, 0)}$, $t$-statistics $\hat{\beta}_j / SE_j$.

### 3.3 Post-Estimation & Hypothesis Testing

1. Linear prediction:
   `predict_linear_response(design_matrix, parameters) -> Result<Vec<f64>, StatsError>`
2. Hypothesis test ($R \beta = r$):
   $W = (R \hat{\beta} - r)^T (R V R^T)^{-1} (R \hat{\beta} - r)$
   $F = W / q$ where $q = \text{rank}(R)$.
3. Linear combination ($c^T \beta$):
   Estimate = $c^T \hat{\beta}$
   $SE = \sqrt{c^T V c}$

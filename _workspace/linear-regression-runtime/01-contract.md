# Contract: Linear Regression Runtime Execution (`regress`)

## 1. Scope & Objective

Port the runtime execution of linear regression (`regress`) connecting tabular data extraction in `tabdat-runtime` with the numerical estimation kernels in `tabdat-stats`:
- **Roadmap Target**: Phase 7 §9.2:
  - Port OLS `regress`.
  - Port WLS.
  - Port current GLS semantics.
  - Port robust covariance (`robust` -> HC1).
  - Port clustered covariance (`cluster(var)`).
- **Language Command**: `tabdat_language::RegressCommand`:
  - `outcome: String`
  - `predictors: Vec<String>`
  - `estimator: RegressEstimator` (`Ols`, `Wls`, `Gls`)
  - `weight_variable: Option<String>`
  - `robust: bool`
  - `cluster_variable: Option<String>`
  - `include_intercept: bool`

## 2. Behavioral Contract & Invariants

1. **Active Dataset Prerequisite**:
   - `regress` requires an active dataset in session state.
   - If no dataset is active: return `RuntimeError::NoActiveDataset { command: "regress".into() }`.
2. **Variable Validation**:
   - Outcome and all predictors must exist in the active dataset schema.
   - If `weight_variable` is specified (for WLS or GLS), it must exist in the active dataset schema.
   - If `cluster_variable` is specified (for cluster covariance), it must exist in the active dataset schema.
   - Missing variable error: returns `RuntimeError::ColumnNotFound { command: "regress".into(), column: <var> }`.
   - All outcome, predictor, and weight variables must be numeric types (INTEGER, BIGINT, DOUBLE, FLOAT, etc.). If non-numeric: `RuntimeError::InvalidColumnType { command: "regress".into(), column: <var>, expected: "numeric", actual: <type> }`.
3. **Missingness & Sample Selection (`EstimationSample`)**:
   - An observation is dropped if ANY of the following are null/NaN:
     - Outcome variable
     - Any predictor variable
     - Weight variable (if WLS/GLS)
     - Cluster variable (if cluster)
   - Non-finite values (`Inf`, `-Inf`, `NaN`) are excluded.
   - Sample identity must be preserved in `EstimationSample`:
     - `retained_indices`: 0-based row indices in original table.
     - `total_observations`: Total rows in active relation.
     - `dropped_observations`: Count of excluded rows.
     - `weights`: Retained observation weights.
     - `cluster_groups`: Retained cluster identifier strings.
   - If 0 observations remain: `RuntimeError::ExecutionError("regress requires at least one complete observation".into())`.
4. **Weights Validation**:
   - For WLS: weights must be strictly positive ($w_i > 0$). If any $w_i \le 0$:
     `RuntimeError::ExecutionError("regress requires positive weights values".into())`.
   - For GLS: sigma must be strictly positive ($\sigma_i > 0$). If any $\sigma_i \le 0$:
     `RuntimeError::ExecutionError("regress requires positive sigma values".into())`.
   - GLS weight translation: In statsmodels/Python TabDat, GLS with a 1D sigma vector scales observations by $1 / \sqrt{\sigma_i}$, which is mathematically equivalent to WLS with weights $w_i = 1 / \sigma_i$.
5. **Numerical Estimation (`tabdat-stats`)**:
   - Constructs `EstimationProblem` from the extracted outcome vector, design matrix, intercept options, sample provenance, and weights.
   - Solves for coefficients $\hat{\beta}$ using backward-stable Householder QR decomposition avoiding $\kappa(X)^2$ condition squaring.
   - Parameter Covariance Modes:
     - **NonRobust**: $s^2 (X^T X)^{-1}$.
     - **RobustHc1**: $\frac{n}{n - k} (X^T X)^{-1} \left(\sum_i w_i^2 e_i^2 x_i x_i^T\right) (X^T X)^{-1}$.
     - **Cluster(var)**: $\frac{G}{G - 1} \frac{n - 1}{n - k} (X^T X)^{-1} \left(\sum_g u_g u_g^T\right) (X^T X)^{-1}$ where $u_g = \sum_{i \in g} w_i e_i x_i$.
   - Test statistics: $t = \hat{\beta} / \text{se}(\hat{\beta})$.
   - Goodness of fit: $N$, degrees of freedom, $R^2$, adjusted $R^2$, Root MSE, RSS, TSS, $F$-statistic.
6. **Session State Invariants**:
   - On successful regression, stores `LeastSquaresResult` in session state (`last_regression`).
   - Clears prior post-estimation states.
   - On error, prior session dataset and state remain unmodified.
7. **Execution Result**:
   - `ExecutionResult::Regression(RegressionResult)`.

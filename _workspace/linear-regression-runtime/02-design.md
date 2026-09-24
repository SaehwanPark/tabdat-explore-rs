# Design: Linear Regression Runtime Execution (`regress`)

## 1. Architectural Distribution

Following the strict dependency architecture:
- `tabdat-stats` (no backend dependencies, `#![forbid(unsafe_code)]`):
  - Pure numerical estimation kernels, matrix operations, and covariance calculations.
  - Exposes `LeastSquaresOptions` with `covariance_type: CovarianceType`.
  - Implements HC1 robust covariance matrix and cluster covariance matrix in `least_squares.rs`.
  - Computes p-values using Student's $t$ CDF / normal CDF.
- `tabdat-runtime` (DuckDB session, relation lifecycle, `#![forbid(unsafe_code)]`):
  - Depends on `tabdat-language` and `tabdat-stats`.
  - Implements `execute_regress(&mut self, command: &RegressCommand) -> Result<ExecutionResult, RuntimeError>`.
  - Validates active dataset schema and numeric types.
  - Queries active DuckDB relation to fetch raw rows for outcome, predictors, cluster (optional), and weight (optional).
  - Filters missing/null/NaN values while tracking `EstimationSample` (`retained_indices`, `total_observations`, `dropped_observations`, `weights`, `cluster_groups`).
  - Validates positive weights/sigma values, returning exact error messages.
  - Invokes `tabdat_stats::least_squares::fit_least_squares_with_options`.
  - Saves the resulting `LeastSquaresResult` in `self.last_regression`.
  - Returns `ExecutionResult::Regression(RegressionResult)`.

## 2. Mathematical Formulations

### 2.1 Least Squares & QR Decomposition
Given design matrix $X \in \mathbb{R}^{n \times k}$ (with optional prepended column of 1s for intercept) and response $y \in \mathbb{R}^n$:
- With weights $w_i > 0$, define $W = \text{diag}(w_i)$, $\tilde{X} = W^{1/2} X$, $\tilde{y} = W^{1/2} y$.
- Householder QR decomposition of $\tilde{X} = Q R$, solving $R \beta = Q^T \tilde{y}$.
- $(X^T W X)^{-1} = R^{-1} (R^{-1})^T$.
- Residuals: $e_i = y_i - x_i^T \hat{\beta}$.
- Weighted RSS: $\text{RSS} = \sum_{i=1}^n w_i e_i^2$.

### 2.2 Covariance Modes
1. **NonRobust**:
   $$V = s^2 (X^T W X)^{-1} \quad \text{where } s^2 = \frac{\text{RSS}}{n - k}$$
2. **Robust HC1**:
   $$S_{HC1} = \frac{n}{n - k} \sum_{i=1}^n (w_i e_i)^2 x_i x_i^T$$
   $$V_{HC1} = (X^T W X)^{-1} S_{HC1} (X^T W X)^{-1}$$
3. **Cluster Robust**:
   For cluster entities $g \in \{1, \dots, G\}$:
   $$u_g = \sum_{i \in \text{cluster } g} w_i e_i x_i$$
   $$S_{cluster} = \frac{G}{G - 1} \frac{n - 1}{n - k} \sum_{g=1}^G u_g u_g^T$$
   $$V_{cluster} = (X^T W X)^{-1} S_{cluster} (X^T W X)^{-1}$$

### 2.3 Two-Tailed P-Values
For test statistic $t_j = \hat{\beta}_j / \text{se}(\hat{\beta}_j)$:
- When $df = n - k > 0$: compute Student's $t$ two-tailed p-value via regularized incomplete beta function $I_x\left(\frac{df}{2}, \frac{1}{2}\right)$ where $x = \frac{df}{df + t_j^2}$.

## 3. Data Extraction Pipeline in `tabdat-runtime`

1. Schema Check:
   - Identify all referenced variables: `outcome`, `predictors`, `weight_variable` (if any), `cluster_variable` (if any).
   - Verify every variable exists in `active_dataset.columns`.
   - Verify `column.is_numeric()` for outcome, predictors, and weight.
2. DuckDB Query:
   - Construct safe quoted select: `SELECT "outcome", "p1", "p2", ... FROM <active_table>`.
   - Iterate rows via DuckDB statement execution.
3. Row-by-Row Coercion & Missingness Filtering:
   - Null or NaN in any of the required columns causes the row to be marked dropped.
   - For retained rows, push 0-based index to `retained_indices`.
   - For weights: if `w <= 0.0`, immediately abort with descriptive error.
   - For GLS: compute $w_i = 1.0 / \sigma_i$. If $\sigma_i \le 0.0$, immediately abort with descriptive error.
4. Problem Assembly:
   - Construct `EstimationSample` with explicit tracking.
   - Construct `EstimationProblem` with outcome, predictors, response vector, design matrix rows, intercept options, sample, and weights.
5. Solve & Store:
   - Execute `fit_least_squares_with_options`.
   - Store in `self.last_regression = Some(result.clone())`.
   - Construct and return `RegressionResult`.

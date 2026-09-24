# Design: Statistical Contracts Substrate (`tabdat-stats`)

## 1. Crate Architecture & Layout

Create `crates/tabdat-stats`:
- `crates/tabdat-stats/Cargo.toml`
- `crates/tabdat-stats/src/lib.rs` (public facade and re-exports)
- `crates/tabdat-stats/src/problem.rs` (`EstimationProblem`, `EstimationSample`)
- `crates/tabdat-stats/src/estimates.rs` (`CoefficientEstimate`, `CovarianceMatrix`, `CovarianceType`, `FitStatistics`, `EstimationDiagnostics`)
- `crates/tabdat-stats/src/result.rs` (`LeastSquaresResult`)
- `crates/tabdat-stats/src/error.rs` (`StatsError`)
- `crates/tabdat-stats/src/traits.rs` (`Estimator` trait)
- `crates/tabdat-stats/src/matrix.rs` (safe, pure Rust linear algebra primitives: transpose, multiply, invert, scale, sample covariance/variance)
- `crates/tabdat-stats/src/least_squares.rs` (`fit_least_squares`, `predict_linear_response`)
- `crates/tabdat-stats/src/post_estimation.rs` (`PostEstimationModel`, linear hypothesis testing, linear combinations)
- `crates/tabdat-stats/tests/longley_contract.rs` (differential validation with certified NIST reference & Python TabDat)
- `crates/tabdat-stats/tests/estimation_contract.rs` (unit and contract tests matching Python `test_estimation.py`)

All files strictly enforce `#![forbid(unsafe_code)]`.

## 2. Invariants & Edge Cases

1. **Explicit Sample Tracking (`EstimationSample`)**:
   - Explicitly records `retained_indices` mapping each estimated row back to original dataset rows.
   - Preserves `dropped_observations` count.
   - Prevents conflating identical sample sizes with differing sample compositions.

2. **Rank Deficiency & Singular Matrices**:
   - Gauss-Jordan elimination detects near-zero pivots ($|\text{pivot}| \le 10^{-12}$).
   - Emits structured `StatsError::SingularMatrix` with descriptive error details rather than returning NaNs or panicking.

3. **Saturated Models & Degrees of Freedom**:
   - Requires $df = N - P > 0$.
   - When $N \le P$, returns `StatsError::ZeroDegreesOfFreedom { observations, parameters }`.

4. **Numerical Stability**:
   - Uses `f64` throughout.
   - Matrix multiplication and inversion check dimension conformability before computation.
   - Standard errors guard variance diagonals: $\sqrt{\max(V_{jj}, 0.0)}$. Zero standard errors yield `None` for $t$-statistic to avoid division by zero.

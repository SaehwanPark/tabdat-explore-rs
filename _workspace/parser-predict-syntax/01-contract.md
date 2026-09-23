# Bounded Contract: Direct `predict` Post-Estimation Syntax Slice

## 1. Context and Scope

This slice implements the syntax-only parser and AST representations for TabDat's
post-estimation prediction command: `predict`. It fulfills Phase 3 / Phase 8
prediction language parsing requirements while keeping post-estimation state,
dataset mutation, and model scoring deferred to future execution slices.

- **Status**: Active development
- **Branch**: `feature/parser-predict-syntax`
- **Related PRs**: Prior syntax slices (#11–#20, #45, #47, #59–#67, #69, #77, #79, #81, #83, #85, #87, #89, #91, #93, #95, #97, #99, #101, #103, #105)
- **Oracle Revision**: Pinned Python oracle checkout at `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`

---

## 2. Python Oracle Contract

### 2.1 Grammar and Accepted Forms
```stata
predict <newvar> [, xb | residuals | pr | spatial_lag | posterior_predictive] [interval] [level(<float>)] [std] [saving(<path>)]
```
- `<newvar>`: Single target variable name for the prediction output. Backtick or double-quote quoting is supported.
- Prediction kinds (mutually exclusive, default is `xb`):
  - `xb`: Linear prediction / fitted values ($X\hat{\beta}$)
  - `residuals`: Residuals ($y - X\hat{\beta}$)
  - `pr`: Predicted probabilities (for binary choice models like logit/probit)
  - `spatial_lag`: Predicted spatial lag (for spatial models)
  - `posterior_predictive`: Posterior predictive draws (for Bayesian models)
- Auxiliary options:
  - `interval`: Request credible/prediction interval bounds (requires `posterior_predictive`)
  - `level(<float>)`: Interval confidence level strictly between 0 and 100 (exclusive, default `95.0`, requires `interval` and `posterior_predictive`)
  - `std`: Standard deviation of posterior predictive draws (requires `posterior_predictive`)
  - `saving(<path>)`: File path to save posterior draws (requires `posterior_predictive`; cannot be combined with `std` or `interval`)

### 2.2 Oracle Sources and References
- Python parser: `tabdat-explore/src/tabdat/parser.py:2036-2087` (`_parse_predict`)
- Python model: `tabdat-explore/src/tabdat/models.py:585-592` (`PredictCommand`)
- Python tests: `tabdat-explore/tests/test_parser.py:706-780, 1575-1592`

### 2.3 Diagnostic Rules and Precedence
1. Missing arguments / multiple arguments / if conditions / expressions:
   - `predict expects syntax: predict <newvar>`
2. Assignment / delimiter guards:
   - `predict=` or `predict = 1`: `predict assignment requires a target before =`
   - `predict==` or `predict == 1`: `unsupported token in command: ==`
   - `predict:`: `unsupported token in command: :`
3. Trailing comma without options:
   - `comma must be followed by at least one option`
4. Unsupported options:
   - `predict unsupported option: <sorted, comma-separated names>`
5. Flag validation:
   - `predict option <name> does not accept a value` (for `xb`, `residuals`, `pr`, `spatial_lag`, `posterior_predictive`, `interval`, `std`)
6. Mutually exclusive prediction kinds:
   - `predict options xb, residuals, pr, spatial_lag, and posterior_predictive cannot be combined`
7. Single-occurrence options:
   - `predict option level may only be supplied once`
   - `predict option saving may only be supplied once`
8. Option values:
   - Bare `level` or non-numeric: `predict option level expects a numeric value`
   - Bare `saving`: `predict option saving expects a path`
9. Interdependency validation:
   - `(interval || level) && kind != posterior_predictive`: `predict interval options require posterior_predictive`
   - `level && !interval`: `predict option level requires interval`
   - `level <= 0.0 || level >= 100.0`: `predict option level must be between 0 and 100`
   - `(std || saving) && kind != posterior_predictive`: `predict std and saving options require posterior_predictive`
   - `saving && (std || interval)`: `predict saving option cannot be combined with std or interval options`

---

## 3. Rust Contract

### 3.1 AST Types in `tabdat-language`
```rust
/// The requested prediction mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PredictKind {
  /// Linear prediction (fitted values $X\hat{\beta}$).
  Xb,
  /// Residuals ($y - X\hat{\beta}$).
  Residuals,
  /// Predicted probabilities for binary choice models.
  Pr,
  /// Predicted spatial lag for spatial models.
  SpatialLag,
  /// Posterior predictive draws for Bayesian models.
  PosteriorPredictive,
}

/// The parser-only `predict` form retained for a later post-estimation
/// runtime slice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PredictCommand {
  /// Target variable name to store predictions.
  pub target_variable: String,
  /// Prediction kind.
  pub kind: PredictKind,
  /// Whether interval bounds are requested.
  pub interval: bool,
  /// Credible interval level, retained as string spelling.
  pub level: String,
  /// Whether standard deviation of posterior predictive draws is requested.
  pub std: bool,
  /// Optional file path to save posterior draws.
  pub saving: Option<String>,
}
```
In `enum Command`:
```rust
  /// Predict post-estimation values (execution is deferred).
  Predict { command: PredictCommand },
```

### 3.2 Runtime Behavior in `tabdat-runtime`
`tabdat-runtime` intercepts `Command::Predict` in `unsupported_command_name` and reports:
`"tabdat-runtime does not yet support predict execution"`.

---

## 4. Test Contract

- Unit tests in `crates/tabdat-language/src/lib.rs`:
  - `predict <newvar>` defaults to `xb`, `interval = false`, `level = "95.0"`, `std = false`, `saving = None`
  - Explicit kinds: `residuals`, `pr`, `spatial_lag`, `posterior_predictive`
  - Bayesian options: `interval`, `level(90)`, `std`, `saving(path.parquet)`
  - Case insensitivity for command name
  - Quoted target variable names (backtick and double quotes)
  - Full diagnostic error test suite matching Python oracle exactly
- Integration tests in `crates/tabdat-runtime/tests/predict_contract.rs`:
  - End-to-end command parsing via runtime session facade returning deferred execution error

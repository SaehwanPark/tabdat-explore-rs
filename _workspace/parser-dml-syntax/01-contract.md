# Bounded Contract: Double Machine Learning Syntax Slice (`dml`)

## 1. Context and Scope

This slice implements the syntax-only parser and AST representations for TabDat's
double machine learning estimation command: `dml`. It fulfills Phase 6 / Phase 8
causal inference language parsing requirements while keeping cross-fitting, Neyman-orthogonal score
estimation, Lasso/regularized nuisance estimation, and parameter inference deferred to future runtime slices.

- **Status**: Active development
- **Branch**: `feature/parser-dml-syntax`
- **Related PRs**: Prior syntax slices (#11–#20, #45, #47, #59–#67, #69, #77, #79, #81, #83, #85, #87, #89, #91, #93, #95, #97, #99, #101, #103, #105, #107, #109, #111, #113, #115)
- **Oracle Revision**: Pinned Python oracle checkout at `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`

---

## 2. Python Oracle Contract

### 2.1 Grammar and Accepted Forms
```stata
dml linear <y> <controls>, treat(<var>) [folds(<int>) alpha(<num>) robust seed(<int>) noconstant]
```
- `linear`: Required estimation model identifier (**required** strictly as first argument).
- `<y>`: Dependent outcome variable (**required** as second argument).
- `<controls>`: Control variables (**required** $\ge 1$ control variable, so total arguments $\ge 3$).
- Options:
  - `treat(<var>)`: Binary/continuous treatment variable (**required**, strictly 1 variable name).
  - `folds(<int>)`: Number of cross-fitting folds (optional integer $\ge 2$, default `5`).
  - `alpha(<num>)`: Regularization penalty parameter (optional positive float $> 0.0$, default `1.0`).
  - `robust`: Robust standard errors flag (optional, default `false`).
  - `seed(<int>)`: Random seed for cross-fitting splits (optional integer $\ge 0$).
  - `noconstant`: Suppress intercept in nuisance regressions (optional flag, default `false`).
- Disallowed / Semantic validation rules:
  - Conditions (`if ...`): Not accepted.
  - Expressions (`= ...`): Not accepted.
  - Colons (`dml: ...`): Rejected with `unsupported token in command: :`.
  - Leading assignment (`dml = ...`): Rejected with `dml assignment requires a target before =`.
  - Double equal (`dml == ...`): Rejected with `unsupported token in command: ==`.
  - Arguments < 3:
    - Rejected with `dml expects syntax: dml linear <y> <controls>, treat(<var>) [folds(<int>) alpha(<num>) robust seed(<int>) noconstant]`.
  - Model not `linear`:
    - Rejected with `dml model must be linear`.
  - Treatment variable validation:
    - If `treatment_variable == outcome`: `dml treatment variable must differ from outcome`
    - If `controls.contains(&treatment_variable)`: `dml treatment variable must not appear in controls`

### 2.2 Oracle Sources and References
- Python parser: `tabdat-explore/src/tabdat/parser.py:2768-2814` (`_parse_dml`)
- Python model: `tabdat-explore/src/tabdat/models.py:776-784` (`DmlCommand`)
- Python tests: `tabdat-explore/tests/test_parser.py:1377-1428`

### 2.3 Diagnostic Rules and Precedence
1. Condition or Expression present / Arguments < 3:
   - `dml expects syntax: dml linear <y> <controls>, treat(<var>) [folds(<int>) alpha(<num>) robust seed(<int>) noconstant]`
2. Model specification:
   - If argument 0 is not `linear`: `dml model must be linear`
3. Assignment / Delimiter guards:
   - `dml = ...`: `dml assignment requires a target before =`
   - `dml == ...`: `unsupported token in command: ==`
   - `dml: ...`: `unsupported token in command: :`
4. Trailing comma without options:
   - `comma must be followed by at least one option`
5. Unsupported options:
   - `dml unsupported option: <sorted, comma-separated names>`
6. Flag validation:
   - `dml option robust does not accept a value`
   - `dml option noconstant does not accept a value`
7. Option `treat`:
   - Missing: `dml option treat expects one variable`
   - Flag without parens (`treat`): `dml option treat expects variables`
   - Empty parens (`treat()`): `option treat expects at least one value`
   - Multiple variables (`treat(d1 d2)`): `dml option treat expects one variable`
   - Multiple occurrences: `dml option treat may only be supplied once`
8. Variable relationship constraints:
   - `dml treatment variable must differ from outcome`
   - `dml treatment variable must not appear in controls`
9. Option `folds`:
   - Flag without value: `dml option folds expects an integer value`
   - Multiple occurrences: `dml option folds may only be supplied once`
   - Non-integer: `dml option folds expects an integer value`
   - Value < 2: `dml option folds must be at least 2`
10. Option `alpha`:
    - Flag without value: `dml option alpha expects a numeric value`
    - Multiple occurrences: `dml option alpha may only be supplied once`
    - Non-numeric: `dml option alpha expects a numeric value`
    - Value $\le 0.0$: `dml option alpha must be positive`
11. Option `seed`:
    - Flag without value: `dml option seed expects an integer value`
    - Multiple occurrences: `dml option seed may only be supplied once`
    - Non-integer: `dml option seed expects an integer value`
    - Value < 0: `dml option seed must be at least 0`

---

## 3. Rust AST and Language Architecture

### 3.1 AST Representation (`crates/tabdat-language/src/lib.rs`)
```rust
/// The parser-only `dml` double machine learning estimator form retained for a later
/// statistical runtime slice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DmlCommand {
  /// The dependent variable.
  pub outcome: String,
  /// Control variables.
  pub controls: Vec<String>,
  /// Treatment indicator variable.
  pub treatment_variable: String,
  /// Number of cross-fitting folds (default: 5).
  pub folds: i64,
  /// Regularization penalty parameter, retained as string spelling (default: "1.0").
  pub alpha: String,
  /// Request robust covariance in the eventual runtime.
  pub robust: bool,
  /// Random seed for cross-fitting splits.
  pub seed: Option<i64>,
  /// Whether the eventual runtime should include an intercept.
  pub include_intercept: bool,
}
```
Add enum variant to `Command`:
```rust
  /// Fit a double machine learning model (execution is deferred).
  Dml { command: DmlCommand },
```

### 3.2 Parser Implementation
- Implement `parse_dml_command(body: &str) -> Result<Command, ParseError>`.
- Wire `dml` into `parse_named_command`.
- Add colon guard `command.as_bytes().get(..3) == b"dml"` with byte 3 == `b':'`.
- Add delimiter `=` guard `name.eq_ignore_ascii_case("dml") && delimiter == '='`.

### 3.3 Runtime Wiring (`crates/tabdat-runtime/src/lib.rs`)
- In `command_name(&Command)`:
  - `Command::Dml { .. } => "dml"`
- In `execute(&mut self, command: Command)`:
  - Falls through to `_ => Err(RuntimeError::UnsupportedCommand { name: command_name })`.

---

## 4. Verification Plan

1. Unit tests in `crates/tabdat-language`:
   - Valid standard forms: outcome and 1 control, multiple controls, with `folds`, with `alpha`, with `robust`, with `seed`, with `noconstant`, combination of all options.
   - Exact diagnostic preservation for all invalid syntax, argument counts (< 3), invalid model, options, relationship constraints, numeric limits, and delimiters.
2. Integration contract test in `crates/tabdat-runtime/tests/dml_contract.rs`:
   - Verifies runtime deferred execution returns `RuntimeError::UnsupportedCommand { name: "dml" }`.
3. Workspace quality checks:
   - `cargo fmt --check`
   - `cargo check --locked --workspace --all-targets`
   - `cargo test --locked --workspace --all-targets`
   - `cargo clippy --locked --workspace --all-targets -- -D warnings`
   - `cargo deny check`
   - `cargo audit`

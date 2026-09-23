# Bounded Contract: Control Function Regression Syntax Slice (`cfregress`)

## 1. Context and Scope

This slice implements the syntax-only parser and AST representations for TabDat's
control function regression estimation command: `cfregress`. It fulfills Phase 6 / Phase 14
endogeneity and instrumental variable language parsing requirements while keeping first-stage
residual generation, control function augmentation, bootstrapping, standard error corrections,
and parameter inference deferred to future runtime slices.

- **Status**: Active development
- **Branch**: `feature/parser-cfregress-syntax`
- **Related PRs**: Prior syntax slices (#11–#20, #45, #47, #59–#67, #69, #77, #79, #81, #83, #85, #87, #89, #91, #93, #95, #97, #99, #101, #103, #105, #107, #109, #111, #113, #115, #117)
- **Oracle Revision**: Pinned Python oracle checkout at `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`

---

## 2. Python Oracle Contract

### 2.1 Grammar and Accepted Forms
```stata
cfregress <y> [exog_vars], endog(<var>) iv(<vars>) [robust cluster(<var>) noconstant]
```
- `<y>`: Dependent outcome variable (**required** as first argument).
- `[exog_vars]`: Optional exogenous explanatory variables ($\ge 0$ variables, so total arguments $\ge 1$).
- Options:
  - `endog(<var>)`: Endogenous regressor variable (**required**, strictly 1 variable name).
  - `iv(<vars>)`: Instrumental variables (**required**, $\ge 1$ variable names).
  - `robust`: Robust standard errors flag (optional, default `false`).
  - `cluster(<var>)`: Clustered standard errors on a cluster variable (optional, strictly 1 variable name).
  - `noconstant`: Suppress intercept (optional flag, default `false`).
- Disallowed / Semantic validation rules:
  - Conditions (`if ...`): Not accepted.
  - Expressions (`= ...`): Not accepted.
  - Colons (`cfregress: ...`): Rejected with `unsupported token in command: :`.
  - Leading assignment (`cfregress = ...`): Rejected with `cfregress assignment requires a target before =`.
  - Double equal (`cfregress == ...`): Rejected with `unsupported token in command: ==`.
  - Arguments < 1:
    - Rejected with `cfregress expects syntax: cfregress <y> [exog_vars], endog(<var>) iv(<vars>)`.
  - Options combination:
    - Cannot combine `robust` and `cluster`: `cfregress cannot combine robust and cluster`.
  - Variable relationship constraints:
    - If `endogenous` appears in `exogenous`: `cfregress endog variable must not appear in exogenous variables`.

### 2.2 Oracle Sources and References
- Python parser: `tabdat-explore/src/tabdat/parser.py:2823-2860` (`_parse_cfregress`)
- Python model: `tabdat-explore/src/tabdat/models.py:787-796` (`CfRegressCommand`)
- Python tests: `tabdat-explore/tests/test_parser.py:1191-1220, 1799-1807`

### 2.3 Diagnostic Rules and Precedence
1. Condition or Expression present / Arguments < 1:
   - `cfregress expects syntax: cfregress <y> [exog_vars], endog(<var>) iv(<vars>)`
2. Assignment / Delimiter guards:
   - `cfregress = ...`: `cfregress assignment requires a target before =`
   - `cfregress == ...`: `unsupported token in command: ==`
   - `cfregress: ...`: `unsupported token in command: :`
3. Trailing comma without options:
   - `comma must be followed by at least one option`
4. Unsupported options:
   - `cfregress unsupported option: <sorted, comma-separated names>`
5. Flag validation:
   - `cfregress option robust does not accept a value`
   - `cfregress option noconstant does not accept a value`
6. Option `endog`:
   - Missing: `cfregress option endog expects one variable`
   - Flag without parens (`endog`): `cfregress option endog expects variables`
   - Empty parens (`endog()`): `option endog expects at least one value`
   - Multiple variables (`endog(d1 d2)`): `cfregress option endog expects one variable`
   - Multiple occurrences: `cfregress option endog may only be supplied once`
7. Option `iv`:
   - Missing: `cfregress option iv expects at least one variable`
   - Flag without parens (`iv`): `cfregress option iv expects variables`
   - Empty parens (`iv()`): `option iv expects at least one value`
   - Multiple occurrences: `cfregress option iv may only be supplied once`
8. Option `cluster`:
   - Flag without parens (`cluster`): `cfregress option cluster expects variables`
   - Empty parens (`cluster()`): `option cluster expects at least one value`
   - Multiple variables (`cluster(c1 c2)`): `cfregress option cluster expects one variable`
   - Multiple occurrences: `cfregress option cluster may only be supplied once`
9. Conflict between `robust` and `cluster`:
   - `cfregress cannot combine robust and cluster`
10. Variable relationship constraints:
    - `cfregress endog variable must not appear in exogenous variables`

---

## 3. Rust AST and Language Architecture

### 3.1 AST Representation (`crates/tabdat-language/src/lib.rs`)
```rust
/// The parser-only `cfregress` control function estimator form retained for a later
/// statistical runtime slice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CfRegressCommand {
  /// The dependent variable.
  pub outcome: String,
  /// Exogenous covariate variables.
  pub exogenous: Vec<String>,
  /// Endogenous variable.
  pub endogenous: String,
  /// Instrumental variables.
  pub instruments: Vec<String>,
  /// Request robust covariance in the eventual runtime.
  pub robust: bool,
  /// Cluster identifier variable.
  pub cluster_variable: Option<String>,
  /// Whether the eventual runtime should include an intercept.
  pub include_intercept: bool,
}
```
Add enum variant to `Command`:
```rust
  /// Fit a control function regression model (execution is deferred).
  CfRegress { command: CfRegressCommand },
```

### 3.2 Parser Implementation
- Implement `parse_cfregress_command(body: &str) -> Result<Command, ParseError>`.
- Wire `cfregress` into `parse_named_command`.
- Add colon guard `command.as_bytes().get(..9) == b"cfregress"` with byte 9 == `b':'`.
- Add delimiter `=` guard `name.eq_ignore_ascii_case("cfregress") && delimiter == '='`.

### 3.3 Runtime Wiring (`crates/tabdat-runtime/src/lib.rs`)
- In `command_name(&Command)`:
  - `Command::CfRegress { .. } => "cfregress"`
- In `execute(&mut self, command: Command)`:
  - Falls through to `_ => Err(RuntimeError::UnsupportedCommand { name: command_name })`.

---

## 4. Verification Plan

1. Unit tests in `crates/tabdat-language`:
   - Valid standard forms: outcome and no exog, outcome and multiple exog, single/multiple instruments, robust, cluster, noconstant, combinations.
   - Exact diagnostic preservation for all invalid syntax, missing/invalid options, option conflicts (robust + cluster), variable relationship constraints, delimiters, and colons.
2. Integration contract test in `crates/tabdat-runtime/tests/cfregress_contract.rs`:
   - Verifies runtime deferred execution returns `RuntimeError::UnsupportedCommand { name: "cfregress" }`.
3. Workspace quality checks:
   - `cargo fmt --check`
   - `cargo check --locked --workspace --all-targets`
   - `cargo test --locked --workspace --all-targets`
   - `cargo clippy --locked --workspace --all-targets -- -D warnings`
   - `cargo deny check`
   - `cargo audit`

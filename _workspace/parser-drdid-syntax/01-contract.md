# Bounded Contract: Doubly Robust Difference-in-Differences Syntax Slice (`drdid`)

## 1. Context and Scope

This slice implements the syntax-only parser and AST representations for TabDat's
doubly robust difference-in-differences estimation command: `drdid`. It fulfills Phase 6 / Phase 8
causal inference language parsing requirements while keeping non-parametric estimation,
inverse probability weighting, outcome regression, bootstrapping, and parameter inference deferred to future runtime slices.

- **Status**: Active development
- **Branch**: `feature/parser-drdid-syntax`
- **Related PRs**: Prior syntax slices (#11–#20, #45, #47, #59–#67, #69, #77, #79, #81, #83, #85, #87, #89, #91, #93, #95, #97, #99, #101, #103, #105, #107, #109, #111, #113)
- **Oracle Revision**: Pinned Python oracle checkout at `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`

---

## 2. Python Oracle Contract

### 2.1 Grammar and Accepted Forms
```stata
drdid <y> [covariates], treat(<var>) post(<var>) [method(or|ipw|aipw) robust bootstrap(<n>) seed(<n>)]
```
- `<y>`: Dependent outcome variable (**required**). Total arguments must be at least 1 (`len(arguments) >= 1`).
- `[covariates]`: Optional covariate control variables following outcome.
- Options:
  - `treat(<var>)`: Binary treatment indicator variable (**required**, strictly 1 variable name).
  - `post(<var>)`: Binary post-treatment time period indicator variable (**required**, strictly 1 variable name).
  - `method(or|ipw|aipw)`: Estimation method (optional, default `"aipw"`). Accepted values: `"or"`, `"ipw"`, `"aipw"`.
  - `robust`: Robust standard errors flag (optional, default `false`).
  - `bootstrap(<n>)`: Number of bootstrap replications (optional, integer $\ge 1$).
  - `seed(<n>)`: Random seed for bootstrap (optional, integer $\ge 0$; requires `bootstrap`).
- Disallowed / Semantic validation rules:
  - Conditions (`if ...`): Not accepted.
  - Expressions (`= ...`): Not accepted.
  - Colons (`drdid: ...`): Rejected with `unsupported token in command: :`.
  - Leading assignment (`drdid = ...`): Rejected with `drdid assignment requires a target before =`.
  - Double equal (`drdid == ...`): Rejected with `unsupported token in command: ==`.
  - Treatment and post variable distinctness:
    - If `treatment_variable == post_variable`: `drdid treatment and post variables must be distinct`
  - Difference from outcome:
    - If `treatment_variable == outcome || post_variable == outcome`: `drdid treatment and post variables must differ from outcome`
  - Absence from covariates:
    - If `covariates.contains(&treatment_variable) || covariates.contains(&post_variable)`: `drdid treatment and post variables must not appear in covariates`
  - Seed dependency:
    - If `seed` is present without `bootstrap`: `drdid option seed requires option bootstrap`

### 2.2 Oracle Sources and References
- Python parser: `tabdat-explore/src/tabdat/parser.py:2719-2766` (`_parse_drdid`)
- Python model: `tabdat-explore/src/tabdat/models.py:764-773` (`DrDidCommand`)
- Python tests: `tabdat-explore/tests/test_parser.py:1324-1375`

### 2.3 Diagnostic Rules and Precedence
1. Condition or Expression present / Arguments < 1:
   - `drdid expects syntax: drdid <y> [covariates], treat(<var>) post(<var>) [method(or|ipw|aipw) robust bootstrap(<n>) seed(<n>)]`
2. Assignment / Delimiter guards:
   - `drdid = ...`: `drdid assignment requires a target before =`
   - `drdid == ...`: `unsupported token in command: ==`
   - `drdid: ...`: `unsupported token in command: :`
3. Trailing comma without options:
   - `comma must be followed by at least one option`
4. Unsupported options:
   - `drdid unsupported option: <sorted, comma-separated names>`
5. Flag validation:
   - `drdid option robust does not accept a value`
6. Option `treat`:
   - Missing: `drdid option treat expects one variable`
   - Flag without parens (`treat`): `drdid option treat expects variables`
   - Empty parens (`treat()`): `option treat expects at least one value`
   - Multiple variables (`treat(d1 d2)`): `drdid option treat expects one variable`
   - Multiple occurrences: `drdid option treat may only be supplied once`
7. Option `post`:
   - Missing: `drdid option post expects one variable`
   - Flag without parens (`post`): `drdid option post expects variables`
   - Empty parens (`post()`): `option post expects at least one value`
   - Multiple variables (`post(t1 t2)`): `drdid option post expects one variable`
   - Multiple occurrences: `drdid option post may only be supplied once`
8. Variable relationship constraints:
   - `drdid treatment and post variables must be distinct`
   - `drdid treatment and post variables must differ from outcome`
   - `drdid treatment and post variables must not appear in covariates`
9. Option `method`:
   - Flag without value (`method`): `drdid option method expects a value`
   - Empty parens (`method()`): `option method expects at least one value`
   - Multiple values (`method(or ipw)`): `drdid option method expects one value`
   - Multiple occurrences: `drdid option method may only be supplied once`
   - Value validation: `drdid option method must be one of: or, ipw, aipw`
10. Option `bootstrap`:
    - Flag without value: `drdid option bootstrap expects an integer value`
    - Multiple occurrences: `drdid option bootstrap may only be supplied once`
    - Non-integer: `drdid option bootstrap expects an integer value`
    - Value < 1: `drdid option bootstrap must be at least 1`
11. Option `seed`:
    - Flag without value: `drdid option seed expects an integer value`
    - Multiple occurrences: `drdid option seed may only be supplied once`
    - Non-integer: `drdid option seed expects an integer value`
    - Value < 0: `drdid option seed must be at least 0`
    - Missing bootstrap when seed is provided: `drdid option seed requires option bootstrap`

---

## 3. Rust AST and Language Architecture

### 3.1 AST Representation (`crates/tabdat-language/src/lib.rs`)
```rust
/// The doubly robust difference-in-differences estimator methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrDidMethod {
  /// Outcome regression.
  Or,
  /// Inverse probability weighting.
  Ipw,
  /// Augmented inverse probability weighting (doubly robust).
  Aipw,
}

/// The parser-only `drdid` doubly robust difference-in-differences estimator form retained
/// for a later statistical runtime slice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrDidCommand {
  /// The dependent variable.
  pub outcome: String,
  /// Optional covariate control variables.
  pub covariates: Vec<String>,
  /// Treatment indicator variable.
  pub treatment_variable: String,
  /// Post-treatment time period indicator variable.
  pub post_variable: String,
  /// Estimation method.
  pub method: DrDidMethod,
  /// Request robust covariance in the eventual runtime.
  pub robust: bool,
  /// Number of bootstrap replications.
  pub bootstrap: Option<i64>,
  /// Random seed for bootstrap.
  pub seed: Option<i64>,
}
```
Add enum variant to `Command`:
```rust
  /// Fit a doubly robust difference-in-differences model (execution is deferred).
  DrDid { command: DrDidCommand },
```

### 3.2 Parser Implementation
- Implement `parse_drdid_command(body: &str) -> Result<Command, ParseError>`.
- Wire `drdid` into `parse_named_command`.
- Add colon guard `command.as_bytes().get(..5) == b"drdid"` with byte 5 == `b':'`.
- Add delimiter `=` guard `name.eq_ignore_ascii_case("drdid") && delimiter == '='`.

### 3.3 Runtime Wiring (`crates/tabdat-runtime/src/lib.rs`)
- In `command_name(&Command)`:
  - `Command::DrDid { .. } => "drdid"`
- In `execute(&mut self, command: Command)`:
  - Falls through to `_ => Err(RuntimeError::UnsupportedCommand { name: command_name })`.

---

## 4. Verification Plan

1. Unit tests in `crates/tabdat-language`:
   - Valid standard forms: with/without covariates, with each method (`or`, `ipw`, `aipw`), default method (`aipw`), with robust, with bootstrap, with bootstrap + seed.
   - Exact diagnostic preservation for all invalid syntax, argument counts, options, relationship constraints, bootstrap/seed conditions, and delimiters.
2. Integration contract test in `crates/tabdat-runtime/tests/drdid_contract.rs`:
   - Verifies runtime deferred execution returns `RuntimeError::UnsupportedCommand { name: "drdid" }`.
3. Workspace quality checks:
   - `cargo fmt --check`
   - `cargo check --locked --workspace --all-targets`
   - `cargo test --locked --workspace --all-targets`
   - `cargo clippy --locked --workspace --all-targets -- -D warnings`
   - `cargo deny check`
   - `cargo audit`

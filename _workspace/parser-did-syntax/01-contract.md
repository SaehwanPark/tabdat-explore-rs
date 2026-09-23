# Bounded Contract: Difference-in-Differences Syntax Slice (`did`)

## 1. Context and Scope

This slice implements the syntax-only parser and AST representations for TabDat's
difference-in-differences estimation command: `did`. It fulfills Phase 6 / Phase 8
causal inference language parsing requirements while keeping two-way fixed effects estimation,
interaction modeling, and parameter inference deferred to future runtime slices.

- **Status**: Active development
- **Branch**: `feature/parser-did-syntax`
- **Related PRs**: Prior syntax slices (#11–#20, #45, #47, #59–#67, #69, #77, #79, #81, #83, #85, #87, #89, #91, #93, #95, #97, #99, #101, #103, #105, #107, #109, #111)
- **Oracle Revision**: Pinned Python oracle checkout at `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`

---

## 2. Python Oracle Contract

### 2.1 Grammar and Accepted Forms
```stata
did <y> [controls], treat(<var>) post(<var>) [robust]
```
- `<y>`: Dependent outcome variable (**required**). Total arguments must be at least 1 (`len(arguments) >= 1`).
- `[controls]`: Optional control variables following outcome.
- Options:
  - `treat(<var>)`: Binary treatment indicator variable (**required**, strictly 1 variable name).
  - `post(<var>)`: Binary post-treatment time period indicator variable (**required**, strictly 1 variable name).
  - `robust`: Robust standard errors flag (optional, default `false`).
- Disallowed / Semantic validation rules:
  - Conditions (`if ...`): Not accepted.
  - Expressions (`= ...`): Not accepted.
  - Colons (`did: ...`): Rejected with `unsupported token in command: :`.
  - Leading assignment (`did = ...`): Rejected with `did assignment requires a target before =`.
  - Double equal (`did == ...`): Rejected with `unsupported token in command: ==`.
  - Treatment and post variable distinctness:
    - If `treatment_variable == post_variable`: `did treatment and post variables must be distinct`
  - Difference from outcome:
    - If `treatment_variable == outcome || post_variable == outcome`: `did treatment and post variables must differ from outcome`
  - Absence from controls:
    - If `controls.contains(&treatment_variable) || controls.contains(&post_variable)`: `did treatment and post variables must not appear in controls`

### 2.2 Oracle Sources and References
- Python parser: `tabdat-explore/src/tabdat/parser.py:2684-2717` (`_parse_did`)
- Python model: `tabdat-explore/src/tabdat/models.py:754-760` (`DidCommand`)
- Python tests: `tabdat-explore/tests/test_parser.py:1280-1322`

### 2.3 Diagnostic Rules and Precedence
1. Condition or Expression present / Arguments < 1:
   - `did expects syntax: did <y> [controls], treat(<var>) post(<var>)`
2. Assignment / Delimiter guards:
   - `did = ...`: `did assignment requires a target before =`
   - `did == ...`: `unsupported token in command: ==`
   - `did: ...`: `unsupported token in command: :`
3. Trailing comma without options:
   - `comma must be followed by at least one option`
4. Unsupported options:
   - `did unsupported option: <sorted, comma-separated names>`
5. Flag validation:
   - `did option robust does not accept a value`
6. Option `treat`:
   - Missing: `did option treat expects one variable`
   - Flag without parens (`treat`): `did option treat expects variables`
   - Empty parens (`treat()`): `option treat expects at least one value`
   - Multiple variables (`treat(d1 d2)`): `did option treat expects one variable`
   - Multiple occurrences: `did option treat may only be supplied once`
7. Option `post`:
   - Missing: `did option post expects one variable`
   - Flag without parens (`post`): `did option post expects variables`
   - Empty parens (`post()`): `option post expects at least one value`
   - Multiple variables (`post(t1 t2)`): `did option post expects one variable`
   - Multiple occurrences: `did option post may only be supplied once`
8. Variable relationship constraints:
   - `did treatment and post variables must be distinct`
   - `did treatment and post variables must differ from outcome`
   - `did treatment and post variables must not appear in controls`

---

## 3. Rust AST and Language Architecture

### 3.1 AST Representation (`crates/tabdat-language/src/lib.rs`)
```rust
/// The parser-only `did` difference-in-differences estimator form retained for a later
/// statistical runtime slice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DidCommand {
  /// The dependent variable.
  pub outcome: String,
  /// Optional control variables.
  pub controls: Vec<String>,
  /// Treatment indicator variable.
  pub treatment_variable: String,
  /// Post-treatment time period indicator variable.
  pub post_variable: String,
  /// Request robust covariance in the eventual runtime.
  pub robust: bool,
}
```
Add enum variant to `Command`:
```rust
  /// Fit a difference-in-differences model (execution is deferred).
  Did { command: DidCommand },
```

### 3.2 Parser Implementation
- Implement `parse_did_command(body: &str) -> Result<Command, ParseError>`.
- Wire `did` into `parse_named_command`.
- Add colon guard `command.as_bytes().get(..3) == b"did"` with byte 3 == `b':'`.
- Add delimiter `=` guard `name.eq_ignore_ascii_case("did") && delimiter == '='`.

### 3.3 Runtime Wiring (`crates/tabdat-runtime/src/lib.rs`)
- In `command_name(&Command)`:
  - `Command::Did { .. } => "did"`
- In `execute(&mut self, command: Command)`:
  - Falls through to `_ => Err(RuntimeError::UnsupportedCommand { name: command_name })`.

---

## 4. Verification Plan

1. Unit tests in `crates/tabdat-language`:
   - Valid standard forms: without controls, with controls, with robust, order-independence of options.
   - Exact diagnostic preservation for all invalid syntax, argument counts, options, relationship constraints, and delimiters.
2. Integration contract test in `crates/tabdat-runtime/tests/did_contract.rs`:
   - Verifies runtime deferred execution returns `RuntimeError::UnsupportedCommand { name: "did" }`.
3. Workspace quality checks:
   - `cargo fmt --check`
   - `cargo check --locked --workspace --all-targets`
   - `cargo test --locked --workspace --all-targets`
   - `cargo clippy --locked --workspace --all-targets -- -D warnings`
   - `cargo deny check`
   - `cargo audit`

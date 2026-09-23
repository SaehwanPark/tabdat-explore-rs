# Bounded Contract: Panel Fixed-Effects Logit Syntax Slice (`xtlogit`)

## 1. Context and Scope

This slice implements the syntax-only parser and AST representations for TabDat's
panel fixed-effects logit estimation command: `xtlogit`. It fulfills Phase 5 / Phase 8
panel econometrics language parsing requirements while keeping numerical optimization,
fixed-effects conditioning, and parameter estimation deferred to future runtime slices.

- **Status**: Active development
- **Branch**: `feature/parser-xtlogit-syntax`
- **Related PRs**: Prior syntax slices (#11–#20, #45, #47, #59–#67, #69, #77, #79, #81, #83, #85, #87, #89, #91, #93, #95, #97, #99, #101, #103, #105, #107)
- **Oracle Revision**: Pinned Python oracle checkout at `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`

---

## 2. Python Oracle Contract

### 2.1 Grammar and Accepted Forms
```stata
xtlogit <y> <xvars>, fe [robust]
```
- `<y>`: Dependent binary variable (outcome).
- `<xvars>`: One or more predictor variables. Total arguments must be at least 2 (`len(arguments) >= 2`).
- Options:
  - `fe`: Fixed-effects estimator flag (**required**).
  - `robust`: Robust standard errors flag (optional, default `false`).
- Disallowed:
  - If conditions (`if ...`): Not allowed in this command form.
  - Expressions (`= ...`): Not allowed.
  - Colons (`xtlogit: ...`): Rejected with `unsupported token in command: :`.
  - Leading assignment (`xtlogit = ...`): Rejected with `xtlogit assignment requires a target before =`.
  - Double equal (`xtlogit == ...`): Rejected with `unsupported token in command: ==`.

### 2.2 Oracle Sources and References
- Python parser: `tabdat-explore/src/tabdat/parser.py:2616-2633` (`_parse_xtlogit`)
- Python model: `tabdat-explore/src/tabdat/models.py:730-734` (`XtLogitCommand`)
- Python tests: `tabdat-explore/tests/test_parser.py:1202-1240`

### 2.3 Diagnostic Rules and Precedence
1. Condition or Expression present / Arguments < 2:
   - `xtlogit expects syntax: xtlogit <y> <xvars>, fe [robust]`
2. Assignment / Delimiter guards:
   - `xtlogit = ...`: `xtlogit assignment requires a target before =`
   - `xtlogit == ...`: `unsupported token in command: ==`
   - `xtlogit: ...`: `unsupported token in command: :`
3. Trailing comma without options:
   - `comma must be followed by at least one option`
4. Unsupported options:
   - `xtlogit unsupported option: <sorted, comma-separated names>`
5. Flag validation:
   - `xtlogit option <name> does not accept a value` (for `fe` or `robust`)
6. Required option:
   - If `fe` is not present: `xtlogit requires option fe`

---

## 3. Rust AST and Language Architecture

### 3.1 AST Representation (`crates/tabdat-language/src/lib.rs`)
```rust
/// The parser-only `xtlogit` fixed-effects panel logit estimator form retained for a later
/// statistical runtime slice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XtLogitCommand {
  /// The dependent variable.
  pub outcome: String,
  /// Ordered predictor variables.
  pub predictors: Vec<String>,
  /// Request robust covariance in the eventual runtime.
  pub robust: bool,
}
```
Add enum variant to `Command`:
```rust
  /// Fit a fixed-effects panel logit model (execution is deferred).
  XtLogit { command: XtLogitCommand },
```

### 3.2 Parser Implementation
- Implement `parse_xtlogit_command(body: &str) -> Result<Command, ParseError>`.
- Wire `xtlogit` into `parse_named_command`.
- Add colon guard `command.as_bytes().get(..7) == b"xtlogit"` with byte 7 == `b':'`.
- Add delimiter `=` guard `name.eq_ignore_ascii_case("xtlogit") && delimiter == '='`.

### 3.3 Runtime Wiring (`crates/tabdat-runtime/src/lib.rs`)
- In `command_name(&Command)`:
  - `Command::XtLogit { .. } => "xtlogit"`
- In `execute(&mut self, command: Command)`:
  - Falls through to `_ => Err(RuntimeError::UnsupportedCommand { name: command_name })`.

---

## 4. Verification Plan

1. Unit tests in `crates/tabdat-language/tests/parser_contract.rs`:
   - Valid single predictor: `xtlogit y x, fe` -> `outcome: "y", predictors: ["x"], robust: false`.
   - Valid multiple predictors: `xtlogit y x1 x2, fe` -> `outcome: "y", predictors: ["x1", "x2"], robust: false`.
   - Valid with robust: `xtlogit y x, fe robust` -> `robust: true`.
   - Valid option ordering: `xtlogit y x, robust fe` -> `robust: true`.
   - Duplicate `fe`: `xtlogit y x, fe fe` -> accepted.
   - Missing `fe`: `xtlogit y x` -> `xtlogit requires option fe`.
   - Only robust: `xtlogit y x, robust` -> `xtlogit requires option fe`.
   - Missing predictors: `xtlogit y, fe` -> `xtlogit expects syntax: xtlogit <y> <xvars>, fe [robust]`.
   - Missing arguments: `xtlogit, fe` -> `xtlogit expects syntax: xtlogit <y> <xvars>, fe [robust]`.
   - Bare command: `xtlogit` -> `xtlogit expects syntax: xtlogit <y> <xvars>, fe [robust]`.
   - Unsupported option: `xtlogit y x, fe extra` -> `xtlogit unsupported option: extra`.
   - Multiple unsupported options sorted: `xtlogit y x, fe foo bar` -> `xtlogit unsupported option: bar, foo`.
   - Option with value: `xtlogit y x, fe(a)` -> `xtlogit option fe does not accept a value`.
   - Robust with value: `xtlogit y x, fe robust=1` -> `xtlogit option robust does not accept a value`.
   - Condition rejection: `xtlogit y x if y > 0, fe` -> `xtlogit expects syntax: xtlogit <y> <xvars>, fe [robust]`.
   - Delimiter / colon guards: `xtlogit = 1`, `xtlogit == 1`, `xtlogit: regress y x`.
2. Integration contract test in `crates/tabdat-runtime/tests/xtlogit_contract.rs`:
   - Verifies runtime deferred execution returns `RuntimeError::UnsupportedCommand { name: "xtlogit" }`.
3. Repository quality checks:
   - `cargo fmt --check`
   - `cargo check`
   - `cargo test`
   - `cargo clippy`
   - `cargo deny check`
   - `cargo audit`

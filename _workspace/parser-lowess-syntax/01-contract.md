# Bounded Contract: Locally Weighted Scatterplot Smoothing Syntax Slice (`lowess`)

## 1. Context and Scope

This slice implements the syntax-only parser and AST representations for TabDat's
locally weighted regression smoothing command: `lowess`. It fulfills Phase 6 / Phase 8
non-parametric regression language parsing requirements while keeping kernel weighting,
polynomial degree evaluation, nearest-neighbor sorting, and column generation deferred to future runtime slices.

- **Status**: Active development
- **Branch**: `feature/parser-lowess-syntax`
- **Related PRs**: Prior syntax slices (#11–#20, #45, #47, #59–#67, #69, #77, #79, #81, #83, #85, #87, #89, #91, #93, #95, #97, #99, #101, #103, #105, #107, #109)
- **Oracle Revision**: Pinned Python oracle checkout at `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`

---

## 2. Python Oracle Contract

### 2.1 Grammar and Accepted Forms
```stata
lowess <y> <x>, gen(<newvar>) [bandwidth=<0,1>]
```
- `<y>`: Dependent variable (outcome).
- `<x>`: Single predictor variable. Total arguments must be strictly 2 (`len(arguments) == 2`).
- Options:
  - `gen(<newvar>)`: Target smoothed variable name (**required**, strictly 1 variable name).
  - `bandwidth=<0,1>`: Smoothing bandwidth fraction strictly between 0 and 1 exclusive (optional, default $2/3$, formatted as `"0.6666666666666666"`).
- Disallowed:
  - Conditions (`if ...`): Not accepted.
  - Expressions (`= ...`): Not accepted.
  - Colons (`lowess: ...`): Rejected with `unsupported token in command: :`.
  - Leading assignment (`lowess = ...`): Rejected with `lowess assignment requires a target before =`.
  - Double equal (`lowess == ...`): Rejected with `unsupported token in command: ==`.

### 2.2 Oracle Sources and References
- Python parser: `tabdat-explore/src/tabdat/parser.py:2660-2682` (`_parse_lowess`)
- Python model: `tabdat-explore/src/tabdat/models.py:746-752` (`LowessCommand`)
- Python tests: `tabdat-explore/tests/test_parser.py:1242-1278`

### 2.3 Diagnostic Rules and Precedence
1. Condition or Expression present / Arguments != 2:
   - `lowess expects syntax: lowess <y> <x>, gen(<newvar>) [bandwidth=<0,1>]`
2. Assignment / Delimiter guards:
   - `lowess = ...`: `lowess assignment requires a target before =`
   - `lowess == ...`: `unsupported token in command: ==`
   - `lowess: ...`: `unsupported token in command: :`
3. Trailing comma without options:
   - `comma must be followed by at least one option`
4. Unsupported options:
   - `lowess unsupported option: <sorted, comma-separated names>`
5. Option `gen`:
   - Missing: `lowess option gen expects one variable`
   - Flag without parens (`gen`): `lowess option gen expects variables`
   - Empty parens (`gen()`): `option gen expects at least one value`
   - Multiple variables (`gen(y1 y2)`): `lowess option gen expects one variable`
   - Multiple occurrences (`gen(a) gen(b)`): `lowess option gen may only be supplied once`
6. Option `bandwidth`:
   - Flag without value (`bandwidth`): `lowess option bandwidth expects a numeric value`
   - Non-numeric value (`bandwidth=abc`): `lowess option bandwidth expects a numeric value`
   - Out of range ($val \le 0$ or $val \ge 1$): `lowess option bandwidth must be between 0 and 1`
   - Multiple occurrences (`bandwidth=0.5 bandwidth=0.6`): `lowess option bandwidth may only be supplied once`

---

## 3. Rust AST and Language Architecture

### 3.1 AST Representation (`crates/tabdat-language/src/lib.rs`)
```rust
/// The parser-only `lowess` locally weighted regression smoother form retained for a later
/// statistical runtime slice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LowessCommand {
  /// The dependent variable.
  pub outcome: String,
  /// The predictor variable.
  pub predictor: String,
  /// The target smoothed variable name.
  pub target_variable: String,
  /// The bandwidth smoothing parameter, retained as string spelling.
  pub bandwidth: String,
}
```
Add enum variant to `Command`:
```rust
  /// Fit a locally weighted regression smoother (execution is deferred).
  Lowess { command: LowessCommand },
```

### 3.2 Parser Implementation
- Implement `parse_lowess_command(body: &str) -> Result<Command, ParseError>`.
- Wire `lowess` into `parse_named_command`.
- Add colon guard `command.as_bytes().get(..6) == b"lowess"` with byte 6 == `b':'`.
- Add delimiter `=` guard `name.eq_ignore_ascii_case("lowess") && delimiter == '='`.

### 3.3 Runtime Wiring (`crates/tabdat-runtime/src/lib.rs`)
- In `command_name(&Command)`:
  - `Command::Lowess { .. } => "lowess"`
- In `execute(&mut self, command: Command)`:
  - Falls through to `_ => Err(RuntimeError::UnsupportedCommand { name: command_name })`.

---

## 4. Verification Plan

1. Unit tests in `crates/tabdat-language`:
   - Valid standard call: `lowess y x, gen(y_hat)` -> `outcome: "y", predictor: "x", target_variable: "y_hat", bandwidth: "0.6666666666666666"`.
   - Valid with explicit bandwidth: `lowess y x, gen(y_hat) bandwidth=0.5` -> `bandwidth: "0.5"`.
   - Valid with quoted variables: `lowess \`y var\` \`x var\`, gen(\`y hat\`)`.
   - Exact diagnostic preservation for all invalid syntax, argument counts, options, and delimiters.
2. Integration contract test in `crates/tabdat-runtime/tests/lowess_contract.rs`:
   - Verifies runtime deferred execution returns `RuntimeError::UnsupportedCommand { name: "lowess" }`.
3. Workspace quality checks:
   - `cargo fmt --check`
   - `cargo check --locked --workspace --all-targets`
   - `cargo test --locked --workspace --all-targets`
   - `cargo clippy --locked --workspace --all-targets -- -D warnings`
   - `cargo deny check`
   - `cargo audit`

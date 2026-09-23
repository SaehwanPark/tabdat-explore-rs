# Bounded Contract: Linear Combination Hypothesis Testing Syntax Slice (`lincom`)

## 1. Context and Scope

This slice implements the syntax-only parser and AST representations for TabDat's
post-estimation linear combination hypothesis testing command: `lincom`. It fulfills Phase 9
classical hypothesis testing language parsing requirements while keeping post-estimation parameter
retrieval, linear expression symbolic differentiation, covariance matrix transformation, standard
error computation, t/z test statistics, p-values, and confidence intervals deferred to future runtime slices.

- **Status**: Active development
- **Branch**: `feature/parser-lincom-syntax`
- **Related PRs**: Prior syntax slices (#11–#20, #45, #47, #59–#67, #69, #77, #79, #81, #83, #85, #87, #89, #91, #93, #95, #97, #99, #101, #103, #105, #107, #109, #111, #113, #115, #117, #119)
- **Oracle Revision**: Pinned Python oracle checkout at `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`

---

## 2. Python Oracle Contract

### 2.1 Grammar and Accepted Forms
```stata
lincom <expression>
```
- `<expression>`: Linear combination expression over parameter names, numbers, arithmetic operators (`+`, `-`, `*`, `/`, unary `-`), parentheses, strings, nulls, and functions.
- Empty command body (e.g. `lincom`, `lincom   `):
  - Rejected with `lincom command expects a linear combination expression`.
- Incomplete expression (e.g. `lincom x1 +`):
  - Rejected with `incomplete expression after +` (or operator name).
- Missing closing parenthesis (e.g. `lincom (x1 + x2`):
  - Rejected with `missing closing ) in expression`.
- Unsupported tokens in expression (e.g. `,`, `)`, `+` where operand expected, `.`):
  - Rejected with `unsupported token in expression: <token>`.
- Unsupported characters / tokens in command (e.g. `[`, `;`):
  - Rejected with `unsupported token in command: <char>`.
- Delimiter guards:
  - Attached colon (`lincom:`): Rejected with `unsupported token in command: :`.
  - Attached assignment (`lincom=`): Rejected with `lincom assignment requires a target before =`.
  - Attached double equal (`lincom==`): Rejected with `unsupported token in command: ==`.
  - Attached comma without options (`lincom,`): Rejected with `comma must be followed by at least one option`.
  - Attached comma with options (`lincom, level(95)`): Rejected with `unknown command: lincom`.

### 2.2 Oracle Sources and References
- Python parser: `tabdat-explore/src/tabdat/parser.py:3601-3608` (`_parse_lincom`)
- Python model: `tabdat-explore/src/tabdat/models.py:805-807` (`LincomCommand`)
- Python tests: `tabdat-explore/tests/test_parser.py:1861-1864`
- Python executor: `tabdat-explore/src/tabdat/executor.py:6799-6850` (`_execute_lincom`)

### 2.3 Diagnostic Rules and Precedence
1. Attached colon:
   - `unsupported token in command: :`
2. Attached assignment / double equal:
   - `lincom=...`: `lincom assignment requires a target before =`
   - `lincom==...`: `unsupported token in command: ==`
3. Attached comma:
   - `lincom,`: `comma must be followed by at least one option`
   - `lincom, ...`: `unknown command: lincom`
4. Empty body:
   - `lincom command expects a linear combination expression`
5. Expression syntax errors:
   - Incomplete expression: `incomplete expression after <operator>`
   - Missing closing paren: `missing closing ) in expression`
   - Unsupported token in expression: `unsupported token in expression: <token>`
   - Unsupported token in command: `unsupported token in command: <char>`

---

## 3. Rust AST and Language Architecture

### 3.1 AST Representation (`crates/tabdat-language/src/lib.rs`)
```rust
/// Parsed `lincom` linear combination specification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LincomCommand {
  /// The linear combination expression to estimate.
  pub expression: GenerateExpression,
}
```
Add enum variant to `Command`:
```rust
  /// Compute linear combination of model parameters (execution is deferred).
  Lincom { command: LincomCommand },
```

### 3.2 Parser Implementation
- Implement `parse_lincom_command(body: &str) -> Result<Command, ParseError>` using `tokenize` and `GenerateExpressionParser`.
- Wire `lincom` into `parse_named_command`.
- Add colon guard `command.as_bytes().get(..6) == b"lincom"` with byte 6 == `b':'`.
- Add delimiter `=` guard `name.eq_ignore_ascii_case("lincom") && delimiter == '='`.
- Add delimiter `,` guard `name.eq_ignore_ascii_case("lincom") && delimiter == ','`.
- Export `LincomCommand` from `tabdat_language`.

### 3.3 Runtime Wiring (`crates/tabdat-runtime/src/lib.rs`)
- In `command_name(&Command)`:
  - `Command::Lincom { .. } => "lincom"`
- In `execute(&mut self, command: Command)`:
  - Falls through to `_ => Err(RuntimeError::UnsupportedCommand { name: command_name })`.

---

## 4. Verification Plan

1. Unit tests in `crates/tabdat-language`:
   - Basic arithmetic linear combinations: `lincom x1 - x2`, `lincom x1 + 2*x2`, `lincom (x1 + x2) * 3`.
   - Quoted identifiers: ``lincom `wage rate` + 2 * `hours worked` ``.
   - Case-insensitivity: `LINCOM x1 + x2`.
   - Prefix commands: `by group: lincom x1 + x2`.
   - Exact diagnostics: empty body, incomplete expressions, missing paren, unsupported tokens, colons, assignments, commas.
2. Integration contract tests in `crates/tabdat-language/tests/parser_contract.rs`:
   - High-level parse checks matching Python oracle AST.
3. Integration contract test in `crates/tabdat-runtime/tests/lincom_contract.rs`:
   - Verifies runtime deferred execution returns `RuntimeError::UnsupportedCommand { name: "lincom" }`.
4. Workspace quality checks:
   - `cargo fmt --all -- --check`
   - `cargo check --locked --workspace --all-targets`
   - `cargo test --locked --workspace --all-targets`
   - `cargo clippy --locked --workspace --all-targets -- -D warnings`

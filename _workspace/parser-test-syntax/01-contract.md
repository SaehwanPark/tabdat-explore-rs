# Bounded Contract: Linear Hypothesis Testing Syntax Slice (`test`)

## 1. Context and Scope

This slice implements the syntax-only parser and AST representations for TabDat's
classical linear hypothesis testing command: `test`. It fulfills Phase 3 and Phase 9
linear hypothesis testing language parsing requirements while keeping post-estimation parameter
retrieval, linear restriction matrix $R$ and vector $r$ construction, Wald/F/chi-squared test statistics,
and p-value computation deferred to future runtime slices.

- **Status**: Active development
- **Branch**: `feature/parser-test-syntax`
- **Related PRs**: Prior syntax slices (#11–#20, #45, #47, #59–#67, #69, #77, #79, #81, #83, #85, #87, #89, #91, #93, #95, #97, #99, #101, #103, #105, #107, #109, #111, #113, #115, #117, #119, #121)
- **Oracle Revision**: Pinned Python oracle checkout at `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`

---

## 2. Python Oracle Contract

### 2.1 Grammar and Accepted Forms
```stata
test <varlist>
test <lhs> = <rhs>
test <lhs> == <rhs>
test (<constraint>) [(<constraint>) ...]
```
- Variable list form (e.g. `test x1`, `test x1 x2`):
  - Every token must be an identifier; each is converted to `GenerateExpression::Identifier(name)`.
  - If a non-identifier token is encountered (e.g. `test 123` or `test x1, replace`), rejected with:
    `test command: expected variable name, got '<token>'`.
- Single unparenthesized constraint form (e.g. `test x1 = x2`, `test x1 == x2`, `test x1 + 2*x2 = 0`):
  - Exactly one `=` or `==` delimiter separating `<lhs>` and `<rhs>`.
  - Parsed into `lhs - rhs` via `GenerateExpression::Binary { left: lhs, operator: GenerateBinaryOperator::Subtract, right: rhs }`.
  - If multiple `=` or `==` are present without parentheses (e.g. `test x1 = x2 = x3`), rejected with:
    `test command: multiple '=' in a single constraint (use parentheses for multiple constraints)`.
  - If missing left-hand side (e.g. `test = x2`), rejected with:
    `test command: missing left-hand side of constraint`.
  - If missing right-hand side (e.g. `test x1 =`), rejected with:
    `test command: missing right-hand side of constraint`.
- Parenthesized constraints form (e.g. `test (x1 = x2) (x3 = 0)`, `test (x1)`):
  - Multiple constraints wrapped in parentheses `(...)`.
  - Any tokens outside parentheses (e.g. `test (x1) extra`) rejected with:
    `test command: unexpected tokens outside parentheses`.
  - Mismatched parentheses (e.g. `test (x1) (x2`, `test (x1))`) rejected with:
    `test command: mismatched parentheses`.
  - Empty constraint in parentheses (e.g. `test ()`) rejected with:
    `test command: empty constraint inside parentheses`.
  - Multiple `=` or `==` inside a constraint (e.g. `test (x1 = x2 = x3)`) rejected with:
    `test command: multiple '=' in a constraint`.
  - Missing lhs or rhs inside a constraint (e.g. `test (x1 = )` or `test ( = x1)`) rejected with:
    `test command: malformed constraint`.
  - Single expression inside parentheses without `=` (e.g. `test (x1)`, `test (x1 + x2)`):
    parsed directly via `GenerateExpressionParser`.
- Empty command body (e.g. `test`, `test   `):
  - Rejected with `test command expects a list of variables or constraints`.
- Delimiter guards:
  - Attached colon (`test:`): Rejected with `unsupported token in command: :`.
  - Attached assignment (`test=`): Rejected with `test assignment requires a target before =`.
  - Attached double equal (`test==`): Rejected with `unsupported token in command: ==`.
  - Attached comma without options (`test,`): Rejected with `comma must be followed by at least one option`.
  - Attached comma with options (`test, replace`): Rejected with `unknown command: test`.

### 2.2 Oracle Sources and References
- Python parser: `tabdat-explore/src/tabdat/parser.py:3517-3600` (`_parse_test`, `_parse_single_constraint`)
- Python model: `tabdat-explore/src/tabdat/models.py:801-803` (`TestCommand`)
- Python tests: `tabdat-explore/tests/test_parser.py:1842-1860`
- Python executor: `tabdat-explore/src/tabdat/executor.py:6727-6798` (`_execute_test`)

### 2.3 Diagnostic Rules and Precedence
1. Attached colon:
   - `unsupported token in command: :`
2. Attached assignment / double equal:
   - `test=...`: `test assignment requires a target before =`
   - `test==...`: `unsupported token in command: ==`
3. Attached comma:
   - `test,`: `comma must be followed by at least one option`
   - `test, ...`: `unknown command: test`
4. Empty body:
   - `test command expects a list of variables or constraints`
5. Parenthesized constraint validation:
   - Tokens outside parentheses: `test command: unexpected tokens outside parentheses`
   - Mismatched parentheses: `test command: mismatched parentheses`
   - Empty constraint: `test command: empty constraint inside parentheses`
   - Multiple equals inside constraint: `test command: multiple '=' in a constraint`
   - Missing operand around equals: `test command: malformed constraint`
6. Unparenthesized constraint validation:
   - Multiple equals: `test command: multiple '=' in a single constraint (use parentheses for multiple constraints)`
   - Missing lhs: `test command: missing left-hand side of constraint`
   - Missing rhs: `test command: missing right-hand side of constraint`
   - Non-identifier in varlist: `test command: expected variable name, got '<token>'`
7. Inner expression errors:
   - Forwarded from `GenerateExpressionParser` (e.g. `incomplete expression after <op>`, `unsupported token in expression: <token>`).

---

## 3. Rust AST and Language Architecture

### 3.1 AST Representation (`crates/tabdat-language/src/lib.rs`)
```rust
/// Parsed `test` linear hypothesis testing specification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestCommand {
  /// The linear constraints to test.
  pub constraints: Vec<GenerateExpression>,
}
```
Add enum variant to `Command`:
```rust
  /// Test linear hypotheses after estimation (execution is deferred).
  Test { command: TestCommand },
```

### 3.2 Parser Implementation
- Implement `parse_test_command(body: &str) -> Result<Command, ParseError>` using `tokenize`, paren-grouping, and `GenerateExpressionParser`.
- Wire `test` into `parse_named_command`.
- Add colon guard `command.as_bytes().get(..4) == b"test"` with byte 4 == `b':'`.
- Add delimiter `=` guard `name.eq_ignore_ascii_case("test") && delimiter == '='`.
- Add delimiter `,` guard `name.eq_ignore_ascii_case("test") && delimiter == ','`.
- Export `TestCommand` from `tabdat_language`.

### 3.3 Runtime Wiring (`crates/tabdat-runtime/src/lib.rs`)
- In `command_name(&Command)`:
  - `Command::Test { .. } => "test"`
- In `execute(&mut self, command: Command)`:
  - Falls through to `_ => Err(RuntimeError::UnsupportedCommand { name: command_name })`.

---

## 4. Verification Plan

1. Unit tests in `crates/tabdat-language/src/lib.rs`:
   - Valid variable lists (`test x1`, `test x1 x2`).
   - Valid single constraints (`test x1 = x2`, `test x1 == x2`, `test x1 + 2*x2 = 0`).
   - Valid parenthesized constraints (`test (x1 = x2) (x3 = 0)`, `test (x1)`, `test (x1 + x2)`).
   - All diagnostic error cases (empty body, colon, delimiter `=`, delimiter `,`, multiple `=`, missing lhs/rhs, unexpected tokens, mismatched parens, malformed constraint, non-identifier).
2. Integration contract tests in `crates/tabdat-language/tests/parser_contract.rs`.
3. Runtime contract test in `crates/tabdat-runtime/tests/test_contract.rs`:
   - Parse and execute `test x1 = x2` and verify `RuntimeError::UnsupportedCommand { name: "test" }`.
4. Workspace checks:
   - `cargo fmt --all -- --check`
   - `cargo check --locked --workspace --all-targets`
   - `cargo test --locked --workspace --all-targets`
   - `cargo clippy --locked --workspace --all-targets -- -D warnings`

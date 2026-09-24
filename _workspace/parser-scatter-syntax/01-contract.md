# Bounded Contract: Scatter Plot Syntax Slice (`scatter`)

## 1. Context and Scope

This slice implements the syntax-only parser and AST representations for TabDat's
visualization command: `scatter`. It fulfills Phase 7 (§7.4) visualization language
parsing requirements while keeping plot generation, DuckDB coordinate extraction,
SVG/PNG rendering, artifact management, and browser/viewer interaction deferred to future
runtime slices.

- **Status**: Active development
- **Branch**: `feature/parser-scatter-syntax`
- **Related PRs**: Prior syntax slices (#11–#20, #45, #47, #59–#67, #69, #77, #79, #81, #83, #85, #87, #89, #91, #93, #95, #97, #99, #101, #103, #105, #107, #109, #111, #113, #115, #117, #119, #121, #123, #125)
- **Oracle Revision**: Pinned Python oracle checkout at `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`

---

## 2. Python Oracle Contract

### 2.1 Grammar and Accepted Forms
```stata
scatter <y_variable> <x_variable> [, saving(<path>) noopen]
```
- Exactly two variable arguments (y followed by x):
  - Valid: `scatter y x`, `scatter price weight`, `scatter price weight, saving(plot.png)`.
  - Zero variables (e.g. `scatter`, `scatter, saving(plot.png)`): rejected with
    `scatter expects syntax: scatter y_var x_var`.
  - One variable (e.g. `scatter price`): rejected with
    `scatter expects syntax: scatter y_var x_var`.
  - More than two variables (e.g. `scatter price weight extra`): rejected with
    `scatter expects syntax: scatter y_var x_var`.
- Predicates and assignment syntax:
  - If clause (e.g. `scatter y x if y > 0`): rejected with
    `scatter does not accept if clauses or assignment syntax`.
  - Assignment syntax (e.g. `scatter y x = 2`): rejected with
    `scatter does not accept if clauses or assignment syntax`.
  - Assignment without target (e.g. `scatter =`, `scatter = 1`, `scatter=1`): rejected with
    `scatter assignment requires a target before =`.
  - Assignment without expression (e.g. `scatter y x =`): rejected with
    `scatter assignment requires an expression after =`.
  - Double equal (e.g. `scatter==1`): rejected with
    `unsupported token in command: ==`.
- Supported options:
  - `saving`: file path to save the generated plot artifact.
    - Syntax: `saving(<path>)` (e.g. `saving(plot.png)`, `saving("my plot.png")`),
      or `saving = <path>` / `saving = "path"`.
    - If no value provided (e.g. `scatter y x, saving`): rejected with
      `scatter option saving expects a path`.
    - Duplicate specification (e.g. `saving(a) saving(b)`): rejected with
      `scatter option saving may only be supplied once`.
  - `noopen`: flag option instructing not to automatically open the artifact.
    - Default value for `open_artifact` is `true`. When `noopen` is passed, `open_artifact` becomes `false`.
    - If a value is provided (e.g. `noopen=1` or `noopen(true)`): rejected with
      `scatter option noopen does not accept a value`.
    - Duplicate flag (e.g. `noopen noopen`) is accepted and sets `open_artifact = false`.
- Unsupported options:
  - Any options other than `saving`, `noopen` (e.g. `scatter y x, foo`): rejected with
    `scatter unsupported option: foo`.
  - Multiple unsupported options are sorted alphabetically (e.g. `scatter y x, zebra apple`):
    `scatter unsupported option: apple, zebra`.
- Delimiter guards:
  - Attached colon (`scatter:`): Rejected with `unsupported token in command: :`.
  - Attached assignment (`scatter=`): Rejected with `scatter assignment requires a target before =`.
  - Attached double equal (`scatter==`): Rejected with `unsupported token in command: ==`.
  - Attached comma with nothing following (`scatter,`): Rejected with `comma must be followed by at least one option`.

### 2.2 Oracle Sources and References
- Python parser: `tabdat-explore/src/tabdat/parser.py:1550-1568` (`_parse_scatter`)
- Python model: `tabdat-explore/src/tabdat/models.py:393-398` (`ScatterCommand`)
- Python tests: `tabdat-explore/tests/test_parser.py:650-675`
- Python executor: `tabdat-explore/src/tabdat/executor.py:5442-5490` (`_execute_scatter`)

### 2.3 Diagnostic Rules and Precedence
1. Attached colon:
   - `unsupported token in command: :`
2. Attached assignment / double equal:
   - `scatter=...`: `scatter assignment requires a target before =`
   - `scatter==...`: `unsupported token in command: ==`
3. Trailing comma without options:
   - `scatter,`: `comma must be followed by at least one option`
4. Predicate or assignment syntax:
   - If clauses or assignment expressions: `scatter does not accept if clauses or assignment syntax`
   - Missing assignment target before `=`: `scatter assignment requires a target before =`
   - Missing assignment expression after `=`: `scatter assignment requires an expression after =`
5. Arity check:
   - `scatter expects syntax: scatter y_var x_var`
6. Unsupported options:
   - `scatter unsupported option: <sorted_opts>`
7. Flag option value check:
   - `scatter option noopen does not accept a value`
8. Option value checks:
   - `scatter option saving expects a path`
   - `scatter option saving may only be supplied once`

---

## 3. Rust AST and Language Architecture

### 3.1 AST Representation (`crates/tabdat-language/src/lib.rs`)
```rust
/// Parsed `scatter` visualization specification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScatterCommand {
  /// The y-axis variable to plot.
  pub y_variable: String,
  /// The x-axis variable to plot.
  pub x_variable: String,
  /// Optional file path to save the generated plot.
  pub saving: Option<String>,
  /// Whether to open the generated artifact in the browser/viewer (default true).
  pub open_artifact: bool,
}
```
Add enum variant to `Command`:
```rust
  /// Compute a scatter plot of two variables (visualization execution is deferred).
  Scatter { command: ScatterCommand },
```

### 3.2 Parser Implementation
- Implement `parse_scatter_command(body: &str) -> Result<Command, ParseError>` using `first_unquoted_comma`, `parse_simple_body(..., false)`, and `parse_use_options`.
- Wire `scatter` into `parse_named_command`.
- Add colon guard `command.as_bytes().get(..7) == b"scatter"` with byte 7 == `b':'`.
- Add delimiter `=` guard `name.eq_ignore_ascii_case("scatter") && delimiter == '='`.
- Export `ScatterCommand` from `tabdat_language`.

### 3.3 Runtime Wiring (`crates/tabdat-runtime/src/lib.rs`)
- In `command_name(&Command)`:
  - `Command::Scatter { .. } => "scatter"`
- In `execute(&mut self, command: Command)`:
  - Falls through to `_ => Err(RuntimeError::UnsupportedCommand { name: command_name })`.

---

## 4. Verification Plan

1. Unit tests in `crates/tabdat-language/src/lib.rs`:
   - Basic `scatter y x` with default options (`saving: None`, `open_artifact: true`).
   - `scatter price weight, saving(plot.png)`.
   - `scatter price weight, saving("my plot.png")`.
   - `scatter price weight, noopen` (`open_artifact: false`).
   - Combined options: `scatter price weight, saving(out.png) noopen`.
   - Error cases:
     - Arity errors: `scatter`, `scatter price`, `scatter price weight extra`, `scatter, noopen`.
     - Condition clause: `scatter y x if y > 0`.
     - Assignment syntax: `scatter y x = 2`, `scatter = 1`, `scatter y x =`.
     - Double equal: `scatter==1`.
     - Attached colon: `scatter:`.
     - Trailing comma: `scatter,`.
     - Unsupported options: `scatter y x, foo`, `scatter y x, zebra apple`.
     - Malformed saving: `scatter y x, saving`.
     - Duplicate saving: `scatter y x, saving(a) saving(b)`.
     - Flag with value: `scatter y x, noopen=1`.
2. Integration contract tests in `crates/tabdat-language/tests/parser_contract.rs`.
3. Runtime contract test in `crates/tabdat-runtime/tests/scatter_contract.rs`:
   - Parse and execute `scatter price weight` and verify `RuntimeError::UnsupportedCommand { name: "scatter" }`.
4. Workspace checks:
   - `cargo fmt --all -- --check`
   - `cargo check --locked --workspace --all-targets`
   - `cargo test --locked --workspace --all-targets`
   - `cargo clippy --locked --workspace --all-targets -- -D warnings`

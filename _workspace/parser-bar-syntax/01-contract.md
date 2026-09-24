# Bounded Contract: Bar Chart Syntax Slice (`bar`)

## 1. Context and Scope

This slice implements the syntax-only parser and AST representations for TabDat's
visualization command: `bar`. It fulfills Phase 7 (§7.4) visualization language
parsing requirements while keeping plot generation, DuckDB category aggregation,
SVG/PNG rendering, artifact management, and browser/viewer interaction deferred to future
runtime slices.

- **Status**: Active development
- **Branch**: `feature/parser-bar-syntax`
- **Related PRs**: Prior syntax slices (#11–#20, #45, #47, #59–#67, #69, #77, #79, #81, #83, #85, #87, #89, #91, #93, #95, #97, #99, #101, #103, #105, #107, #109, #111, #113, #115, #117, #119, #121, #123, #125, #127)
- **Oracle Revision**: Pinned Python oracle checkout at `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`

---

## 2. Python Oracle Contract

### 2.1 Grammar and Accepted Forms
```stata
bar <variable> [, missing saving(<path>) noopen]
```
- Exactly one variable argument:
  - Valid: `bar sex`, `bar sex, missing noopen`, `bar sex, saving(plot.png)`.
  - Zero variables (e.g. `bar`, `bar, missing`): rejected with
    `bar expects exactly one variable`.
  - More than one variable (e.g. `bar sex age`): rejected with
    `bar expects exactly one variable`.
- Predicates and assignment syntax:
  - If clause (e.g. `bar sex if age > 18`): rejected with
    `bar does not accept if clauses or assignment syntax`.
  - Assignment syntax (e.g. `bar sex = 2`): rejected with
    `bar does not accept if clauses or assignment syntax`.
  - Assignment without target (e.g. `bar =`, `bar = 1`, `bar=1`): rejected with
    `bar assignment requires a target before =`.
  - Assignment without expression (e.g. `bar sex =`): rejected with
    `bar assignment requires an expression after =`.
  - Double equal (e.g. `bar==1`): rejected with
    `unsupported token in command: ==`.
- Supported options:
  - `saving`: file path to save the generated plot artifact.
    - Syntax: `saving(<path>)` (e.g. `saving(plot.png)`, `saving("my plot.png")`),
      or `saving = <path>` / `saving = "path"`.
    - If no value provided (e.g. `bar sex, saving`): rejected with
      `bar option saving expects a path`.
    - Duplicate specification (e.g. `saving(a) saving(b)`): rejected with
      `bar option saving may only be supplied once`.
  - `missing`: flag option to include missing values as a separate category in the bar chart.
    - Default value for `include_missing` is `false`. When `missing` is passed, `include_missing` becomes `true`.
    - If a value is provided (e.g. `missing=true` or `missing(1)`): rejected with
      `bar option missing does not accept a value`.
    - Duplicate flag (e.g. `missing missing`) is accepted and sets `include_missing = true`.
  - `noopen`: flag option instructing not to automatically open the artifact.
    - Default value for `open_artifact` is `true`. When `noopen` is passed, `open_artifact` becomes `false`.
    - If a value is provided (e.g. `noopen=1` or `noopen(true)`): rejected with
      `bar option noopen does not accept a value`.
    - Duplicate flag (e.g. `noopen noopen`) is accepted and sets `open_artifact = false`.
- Unsupported options:
  - Any options other than `saving`, `missing`, `noopen` (e.g. `bar sex, foo`, `bar sex, bins=20`): rejected with
    `bar unsupported option: <unsupported_option>`.
  - Multiple unsupported options are sorted alphabetically (e.g. `bar sex, zebra apple`):
    `bar unsupported option: apple, zebra`.
- Delimiter guards:
  - Attached colon (`bar:`): Rejected with `unsupported token in command: :`.
  - Attached assignment (`bar=`): Rejected with `bar assignment requires a target before =`.
  - Attached double equal (`bar==`): Rejected with `unsupported token in command: ==`.
  - Attached comma with nothing following (`bar,`): Rejected with `comma must be followed by at least one option`.

### 2.2 Oracle Sources and References
- Python parser: `tabdat-explore/src/tabdat/parser.py:1570-1588` (`_parse_bar`)
- Python model: `tabdat-explore/src/tabdat/models.py:401-406` (`BarCommand`)
- Python tests: `tabdat-explore/tests/test_parser.py:1276-1280`, `1536-1539`
- Python executor: `tabdat-explore/src/tabdat/executor.py:5492-5538` (`_execute_bar`)

### 2.3 Diagnostic Rules and Precedence
1. Attached colon:
   - `unsupported token in command: :`
2. Attached assignment / double equal:
   - `bar=...`: `bar assignment requires a target before =`
   - `bar==...`: `unsupported token in command: ==`
3. Trailing comma without options:
   - `bar,`: `comma must be followed by at least one option`
4. Predicate or assignment syntax:
   - If clauses or assignment expressions: `bar does not accept if clauses or assignment syntax`
   - Missing assignment target before `=`: `bar assignment requires a target before =`
   - Missing assignment expression after `=`: `bar assignment requires an expression after =`
5. Arity check:
   - `bar expects exactly one variable`
6. Unsupported options:
   - `bar unsupported option: <sorted_opts>`
7. Flag option value checks:
   - `bar option missing does not accept a value`
   - `bar option noopen does not accept a value`
8. Option value checks:
   - `bar option saving expects a path`
   - `bar option saving may only be supplied once`

---

## 3. Rust AST and Language Architecture

### 3.1 AST Representation (`crates/tabdat-language/src/lib.rs`)
```rust
/// Parsed `bar` visualization specification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BarCommand {
  /// The variable to plot frequency categories for.
  pub variable: String,
  /// Optional file path to save the generated plot.
  pub saving: Option<String>,
  /// Whether to include missing values as a category in the bar chart.
  pub include_missing: bool,
  /// Whether to open the generated artifact in the browser/viewer (default true).
  pub open_artifact: bool,
}
```
Add enum variant to `Command`:
```rust
  /// Compute a bar chart of a categorical variable (visualization execution is deferred).
  Bar { command: BarCommand },
```

### 3.2 Parser Implementation
- Implement `parse_bar_command(body: &str) -> Result<Command, ParseError>` using `first_unquoted_comma`, `parse_simple_body(..., false)`, and `parse_use_options`.
- Wire `bar` into `parse_named_command`.
- Add colon guard `command.as_bytes().get(..3) == b"bar"` with byte 3 == `b':'`.
- Add delimiter `=` guard `name.eq_ignore_ascii_case("bar") && delimiter == '='`.
- Export `BarCommand` from `tabdat_language`.

### 3.3 Runtime Wiring (`crates/tabdat-runtime/src/lib.rs`)
- In `command_name(&Command)`:
  - `Command::Bar { .. } => "bar"`
- In `execute(&mut self, command: Command)`:
  - Falls through to `_ => Err(RuntimeError::UnsupportedCommand { name: command_name })`.

---

## 4. Verification Plan

1. Unit tests in `crates/tabdat-language/src/lib.rs`:
   - Basic `bar sex` with default options (`saving: None`, `include_missing: false`, `open_artifact: true`).
   - `bar sex, missing noopen`.
   - `bar sex, saving(out.png)`.
   - `bar sex, saving("my bar.png")`.
   - `bar sex, missing`.
   - `bar sex, noopen`.
   - `bar sex, saving(out.png) missing noopen`.
   - Error cases:
     - Arity errors: `bar`, `bar sex age`, `bar, missing`.
     - Condition clause: `bar sex if age > 18`.
     - Assignment syntax: `bar sex = 1`, `bar = 1`, `bar sex =`.
     - Double equal: `bar==1`.
     - Attached colon: `bar:`.
     - Trailing comma: `bar,`.
     - Unsupported options: `bar sex, bins=20`, `bar sex, zebra apple`.
     - Malformed saving: `bar sex, saving`.
     - Duplicate saving: `bar sex, saving(a) saving(b)`.
     - Flags with value: `bar sex, missing=true`, `bar sex, noopen=1`.
2. Integration contract tests in `crates/tabdat-language/tests/parser_contract.rs`.
3. Runtime contract test in `crates/tabdat-runtime/tests/bar_contract.rs`:
   - Parse and execute `bar sex` and verify `RuntimeError::UnsupportedCommand { name: "bar" }`.
4. Workspace checks:
   - `cargo fmt --all -- --check`
   - `cargo check --locked --workspace --all-targets`
   - `cargo test --locked --workspace --all-targets`
   - `cargo clippy --locked --workspace --all-targets -- -D warnings`

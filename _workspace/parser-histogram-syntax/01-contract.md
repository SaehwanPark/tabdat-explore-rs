# Bounded Contract: Histogram Syntax Slice (`histogram`)

## 1. Context and Scope

This slice implements the syntax-only parser and AST representations for TabDat's
visualization command: `histogram`. It fulfills Phase 7 (§7.4) visualization language
parsing requirements while keeping plot generation, DuckDB binned frequency aggregation,
SVG/PNG rendering, artifact management, and browser/viewer interaction deferred to future
runtime slices.

- **Status**: Active development
- **Branch**: `feature/parser-histogram-syntax`
- **Related PRs**: Prior syntax slices (#11–#20, #45, #47, #59–#67, #69, #77, #79, #81, #83, #85, #87, #89, #91, #93, #95, #97, #99, #101, #103, #105, #107, #109, #111, #113, #115, #117, #119, #121, #123)
- **Oracle Revision**: Pinned Python oracle checkout at `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`

---

## 2. Python Oracle Contract

### 2.1 Grammar and Accepted Forms
```stata
histogram <variable> [, bins=<int> saving(<path>) noopen]
```
- Exactly one variable argument:
  - Valid: `histogram x`, `histogram price`, `histogram weight, bins=20`.
  - Zero variables (e.g. `histogram`, `histogram, bins=10`): rejected with
    `histogram expects exactly one variable`.
  - Multiple variables (e.g. `histogram x y`): rejected with
    `histogram expects exactly one variable`.
- Predicates and assignment syntax:
  - If clause (e.g. `histogram x if x > 0`): rejected with
    `histogram does not accept if clauses or assignment syntax`.
  - Assignment syntax (e.g. `histogram x = 2`): rejected with
    `histogram does not accept if clauses or assignment syntax`.
  - Assignment without target (e.g. `histogram =`, `histogram = 1`, `histogram=1`): rejected with
    `histogram assignment requires a target before =`.
  - Assignment without expression (e.g. `histogram x =`): rejected with
    `histogram assignment requires an expression after =`.
  - Double equal (e.g. `histogram==1`): rejected with
    `unsupported token in command: ==`.
- Supported options:
  - `bins`: positive integer specifying number of histogram bins.
    - Syntax: `bins=<int>` (e.g. `bins=10`, `bins = 20`, or parenthesized backtick identifier `bins(`10`)`).
    - Minimum value is 1: `bins=0` or negative values rejected with
      `histogram option bins must be at least 1`.
    - Non-integer or float values (e.g. `bins=1.5`, `bins=abc`): rejected with
      `histogram option bins expects an integer value`.
    - Note on parenthesized numbers (e.g. `bins(10)`): In Python oracle tokenizer,
      `10` is classified as `number`, not `identifier`, and `_parenthesized_option_value`
      does not include `"bins"` in its numeric list, so `bins(10)` yields:
      `option bins values must be identifiers`.
    - Duplicate specification (e.g. `bins=10 bins=20`): rejected with
      `histogram option bins may only be supplied once`.
  - `saving`: file path to save the generated plot artifact.
    - Syntax: `saving(<path>)` (e.g. `saving(plot.png)`, `saving("my plot.png")`),
      or `saving = <path>` / `saving = "path"`.
    - If no value provided (e.g. `histogram x, saving`): rejected with
      `histogram option saving expects a path`.
    - Duplicate specification (e.g. `saving(a) saving(b)`): rejected with
      `histogram option saving may only be supplied once`.
  - `noopen`: flag option instructing not to automatically open the artifact.
    - Default value for `open_artifact` is `true`. When `noopen` is passed, `open_artifact` becomes `false`.
    - If a value is provided (e.g. `noopen=1` or `noopen(true)`): rejected with
      `histogram option noopen does not accept a value`.
    - Duplicate flag (e.g. `noopen noopen`) is accepted and sets `open_artifact = false`.
- Unsupported options:
  - Any options other than `bins`, `saving`, `noopen` (e.g. `histogram x, foo`): rejected with
    `histogram unsupported option: foo`.
  - Multiple unsupported options are sorted alphabetically (e.g. `histogram x, zebra apple`):
    `histogram unsupported option: apple, zebra`.
- Delimiter guards:
  - Attached colon (`histogram:`): Rejected with `unsupported token in command: :`.
  - Attached assignment (`histogram=`): Rejected with `histogram assignment requires a target before =`.
  - Attached double equal (`histogram==`): Rejected with `unsupported token in command: ==`.
  - Attached comma with nothing following (`histogram,`): Rejected with `comma must be followed by at least one option`.

### 2.2 Oracle Sources and References
- Python parser: `tabdat-explore/src/tabdat/parser.py:1530-1548` (`_parse_histogram`)
- Python model: `tabdat-explore/src/tabdat/models.py:385-389` (`HistogramCommand`)
- Python tests: `tabdat-explore/tests/test_parser.py:650-675`
- Python executor: `tabdat-explore/src/tabdat/executor.py:5390-5440` (`_execute_histogram`)

### 2.3 Diagnostic Rules and Precedence
1. Attached colon:
   - `unsupported token in command: :`
2. Attached assignment / double equal:
   - `histogram=...`: `histogram assignment requires a target before =`
   - `histogram==...`: `unsupported token in command: ==`
3. Trailing comma without options:
   - `histogram,`: `comma must be followed by at least one option`
4. Predicate or assignment syntax:
   - If clauses or assignment expressions: `histogram does not accept if clauses or assignment syntax`
   - Missing assignment target before `=`: `histogram assignment requires a target before =`
   - Missing assignment expression after `=`: `histogram assignment requires an expression after =`
5. Arity check:
   - `histogram expects exactly one variable`
6. Unsupported options:
   - `histogram unsupported option: <sorted_opts>`
7. Flag option value check:
   - `histogram option noopen does not accept a value`
8. Option value checks:
   - `histogram option bins must be at least 1`
   - `histogram option bins expects an integer value`
   - `histogram option bins may only be supplied once`
   - `histogram option saving expects a path`
   - `histogram option saving may only be supplied once`

---

## 3. Rust AST and Language Architecture

### 3.1 AST Representation (`crates/tabdat-language/src/lib.rs`)
```rust
/// Parsed `histogram` visualization specification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistogramCommand {
  /// The single variable to plot.
  pub variable: String,
  /// Optional bin count.
  pub bins: Option<i64>,
  /// Optional file path to save the generated plot.
  pub saving: Option<String>,
  /// Whether to open the generated artifact in the browser/viewer (default true).
  pub open_artifact: bool,
}
```
Add enum variant to `Command`:
```rust
  /// Compute a histogram of a variable (visualization execution is deferred).
  Histogram { command: HistogramCommand },
```

### 3.2 Parser Implementation
- Implement `parse_histogram_command(body: &str) -> Result<Command, ParseError>` using `first_unquoted_comma`, `parse_simple_body(..., false)`, and `parse_use_options`.
- Wire `histogram` into `parse_named_command`.
- Add colon guard `command.as_bytes().get(..9) == b"histogram"` with byte 9 == `b':'`.
- Add delimiter `=` guard `name.eq_ignore_ascii_case("histogram") && delimiter == '='`.
- Export `HistogramCommand` from `tabdat_language`.

### 3.3 Runtime Wiring (`crates/tabdat-runtime/src/lib.rs`)
- In `command_name(&Command)`:
  - `Command::Histogram { .. } => "histogram"`
- In `execute(&mut self, command: Command)`:
  - Falls through to `_ => Err(RuntimeError::UnsupportedCommand { name: command_name })`.

---

## 4. Verification Plan

1. Unit tests in `crates/tabdat-language/src/lib.rs`:
   - Basic `histogram x` with default options (`bins: None`, `saving: None`, `open_artifact: true`).
   - `histogram x, bins=20`.
   - `histogram x, saving(plot.png)`.
   - `histogram x, saving("my plot.png")`.
   - `histogram x, noopen` (`open_artifact: false`).
   - Combined options: `histogram x, bins=15 saving(out.png) noopen`.
   - Error cases:
     - Empty variable (`histogram`, `histogram, bins=10`).
     - Multiple variables (`histogram x y`).
     - Condition clause (`histogram x if x > 0`).
     - Assignment syntax (`histogram x = 2`, `histogram = 1`, `histogram x =`).
     - Double equal (`histogram==1`).
     - Attached colon (`histogram:`).
     - Trailing comma (`histogram,`).
     - Unsupported options (`histogram x, foo`, `histogram x, zebra apple`).
     - Malformed bins (`histogram x, bins=0`, `histogram x, bins=-1`, `histogram x, bins=1.5`, `histogram x, bins=abc`).
     - Duplicate bins (`histogram x, bins=5 bins=10`).
     - Malformed saving (`histogram x, saving`).
     - Duplicate saving (`histogram x, saving(a) saving(b)`).
     - Flag with value (`histogram x, noopen=1`).
2. Integration contract tests in `crates/tabdat-language/tests/parser_contract.rs`.
3. Runtime contract test in `crates/tabdat-runtime/tests/histogram_contract.rs`:
   - Parse and execute `histogram x` and verify `RuntimeError::UnsupportedCommand { name: "histogram" }`.
4. Workspace checks:
   - `cargo fmt --all -- --check`
   - `cargo check --locked --workspace --all-targets`
   - `cargo test --locked --workspace --all-targets`
   - `cargo clippy --locked --workspace --all-targets -- -D warnings`

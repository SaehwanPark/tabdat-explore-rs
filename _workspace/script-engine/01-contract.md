# Bounded Contract: Script Engine Parsing & Directives (`.td` Scripts)

## 1. Context and Scope

This slice implements the backend-independent script parsing, macro expansion, and directive
evaluation engine for TabDat's `.td` script files. It fulfills Phase 5 (§5.2) script engine
requirements by porting `tabdat/script.py` behavior into `crates/tabdat-language/src/script.rs`
(exposed via `tabdat_language::script`).

- **Status**: Active development
- **Branch**: `feature/script-engine`
- **Related PRs**: Prior syntax slices (#11–#20, #45, #47, #59–#67, #69, #77, #79, #81, #83, #85, #87, #89, #91, #93, #95, #97, #99, #101, #103, #105, #107, #109, #111, #113, #115, #117, #119, #121, #123, #125, #127, #129, #131)
- **Oracle Revision**: Pinned Python oracle checkout at `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`

---

## 2. Python Oracle Contract

### 2.1 Script Structure & Line Handling
- Empty lines and lines starting with `#` (after optional whitespace) are ignored.
- Non-empty, non-comment lines are retained with their original 1-based start line numbers.
- Multiline SQL blocks:
  - Lines starting with `sql` followed by triple quotes `"""` are grouped into a single command spanning multiple lines.
  - Balanced closing `"""` completes the SQL command block.
  - Unterminated multiline SQL queries are rejected with:
    `{path}:{start_line}: sql multiline query is missing closing """`.

### 2.2 File Reading (`read_script`)
- Reading from disk verifies:
  - Missing file: `{path}:1: script file not found`.
  - Path is a directory: `{path}:1: script path is a directory`.
  - Non-UTF-8 content: `{path}:{line}: script file must be UTF-8 text`, where `line` is the 1-based line of the first invalid byte.
  - Valid files are passed to `parse_script`.

### 2.3 Macro Expansion (`expand_script_macros`)
- Macro references match `$([A-Za-z_][A-Za-z0-9_]*)`.
- If a macro reference is defined in `context.macros`, it is replaced with its string value.
- If undefined: rejected with `{path}:{line}: undefined macro: {name}`.
- Dollar signs not followed by identifier characters (e.g. `$1`, `$$foo`, bare `$`) are preserved.

### 2.4 Script Directives (`parse_script_directive`)
- `seed <integer>`:
  - Valid: `seed 123` -> `SeedDirective(123)`.
  - Invalid value or arity (e.g. `seed`, `seed 1.5`, `seed 1 2`): rejected with
    `{path}:{line}: seed expects an integer`.
- `let <name> = <value>`:
  - Valid: `let data = patients.parquet` -> `LetDirective("data", "patients.parquet")`.
  - Macro names must match `^[A-Za-z_][A-Za-z0-9_]*$`:
    - Invalid syntax (missing `=` or empty name): `{path}:{line}: let expects syntax: let <name> = <value>`.
    - Non-identifier name (e.g. `1data`): `{path}:{line}: macro name must be an identifier: {name}`.
    - Empty value (e.g. `let data = `): `{path}:{line}: macro value cannot be empty: {name}`.
    - Duplicate definition: `{path}:{line}: macro already defined: {name}`.

### 2.5 Control Flow Directives (`parse_control_flow_directive`)
- `if <condition>`:
  - Missing condition: `{path}:{line}: if expects a condition`.
  - Evaluates condition via `evaluate_script_condition`.
- `else`:
  - With condition/arguments: `{path}:{line}: else does not accept a condition`.
- `end`:
  - With arguments: `{path}:{line}: end does not accept arguments`.

### 2.6 Condition Evaluation (`evaluate_script_condition`)
- Truthy values (case-insensitive): `true`, `on`, `1` -> `true`.
- Falsy values (case-insensitive): `false`, `off`, `0` -> `false`.
- String equality and inequality comparisons:
  - `<left> == <right>`: true if stripped left equals stripped right, otherwise false.
  - `<left> != <right>`: true if stripped left does not equal stripped right, otherwise false.
  - Missing left or right operand: rejected with
    `{path}:{line}: if condition expects true/false, 1/0, on/off, ==, or !=`.
- Any other condition string: rejected with
  `{path}:{line}: if condition expects true/false, 1/0, on/off, ==, or !=`.

### 2.7 Oracle Sources and References
- Python script engine: `tabdat-explore/src/tabdat/script.py:1-335`
- Python tests: `tabdat-explore/tests/test_script.py:1-204` (27 passing tests)
- Python runner integration: `tabdat-explore/src/tabdat/cli.py:1600-1730`

---

## 3. Rust Architecture

### 3.1 Module Placement
- Create `crates/tabdat-language/src/script.rs`.
- Expose via `pub mod script;` in `crates/tabdat-language/src/lib.rs`.

### 3.2 Types & Structs
```rust
pub struct ScriptCommand {
  pub text: String,
  pub start_line: usize,
}

pub struct ScriptContext {
  pub macros: HashMap<String, String>,
  pub seed: Option<i64>,
}

pub struct ScriptBlockState {
  pub start_line: usize,
  pub condition_active: bool,
  pub in_else: bool,
}

pub struct SeedDirective {
  pub value: i64,
}

pub struct LetDirective {
  pub name: String,
  pub value: String,
}

pub enum ScriptDirective {
  Seed(SeedDirective),
  Let(LetDirective),
}

pub struct IfDirective {
  pub active: bool,
}

pub struct ElseDirective;

pub struct EndDirective;

pub enum ControlFlowDirective {
  If(IfDirective),
  Else(ElseDirective),
  End(EndDirective),
}

pub struct ScriptError {
  pub path: PathBuf,
  pub line: usize,
  pub message: String,
}
```

### 3.3 Public Functions
```rust
pub fn parse_script(text: &str, path: &Path) -> Result<Vec<ScriptCommand>, ScriptError>;
pub fn read_script(path: &Path) -> Result<Vec<ScriptCommand>, ScriptError>;
pub fn expand_script_macros(text: &str, context: &ScriptContext, path: &Path, line: usize) -> Result<String, ScriptError>;
pub fn parse_script_directive(text: &str, context: &ScriptContext, path: &Path, line: usize) -> Result<Option<ScriptDirective>, ScriptError>;
pub fn parse_control_flow_directive(text: &str, path: &Path, line: usize) -> Result<Option<ControlFlowDirective>, ScriptError>;
pub fn evaluate_script_condition(condition: &str, path: &Path, line: usize) -> Result<bool, ScriptError>;
```

---

## 4. Verification Plan

1. Unit tests in `crates/tabdat-language/src/script.rs`:
   - Characterizing all 27 test scenarios from `test_script.py`.
   - Blank lines and comment filtering.
   - Multiline SQL grouping and closing quote validation.
   - UTF-8 validation and non-existent file handling.
   - Macro reference expansion, duplicate definition rejection, undefined reference rejection.
   - Seed and Let directive parsing and validation.
   - If/else/end control flow directives and condition evaluations.
2. Integration contract test in `crates/tabdat-language/tests/script_contract.rs`.
3. Workspace checks:
   - `cargo fmt --all -- --check`
   - `cargo check --locked --workspace --all-targets`
   - `cargo test --locked --workspace --all-targets`
   - `cargo clippy --locked --workspace --all-targets -- -D warnings`

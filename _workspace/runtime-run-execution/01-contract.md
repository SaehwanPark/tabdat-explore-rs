# Bounded runtime `run` script execution contract

## Scope and purpose

Implement the bounded runtime execution slice for `run <script-path>` in `crates/tabdat-runtime`.
This slice connects the parsed `Command::Run { path }` language variant to the typed `Session` execution engine, fulfilling Phase 5 §5.2 checklist items for:
- Nested `run` script execution
- Recursion rejection
- Relative path resolution against parent script directory
- Shared and inherited macro definitions and random seed state via `ScriptContext`
- Control flow (`if` / `else` / `end`) execution
- Directive evaluation (`seed`, `let`)
- Source file and line diagnostics for script-originating errors (`<path>:<line>: <message>`)
- Script early termination via `exit`

## Architecture and interfaces

1. **Language dependency:**
   `tabdat-runtime` depends on `tabdat-language` and utilizes `tabdat_language::script`:
   - `parse_script`, `read_script`, `ScriptCommand`
   - `ScriptContext` (macros and seed)
   - `ScriptBlockState`
   - `ScriptDirective`, `parse_script_directive`
   - `ControlFlowDirective`, `parse_control_flow_directive`
   - `ScriptError`

2. **Runtime AST & Result Model:**
   - Add `RunResult` struct:
     ```rust
     #[derive(Debug, Clone, PartialEq, Eq)]
     pub struct RunResult {
       pub path: PathBuf,
       pub executed_commands: usize,
     }
     ```
   - Add `Run(RunResult)` variant to `ExecutionResult`.
   - Add `ScriptError(tabdat_language::script::ScriptError)` variant to `RuntimeError`.

3. **Session execution:**
   - In `Session::execute(&mut self, command: Command) -> Result<ExecutionResult, RuntimeError>`:
     `Command::Run { path } => self.execute_run(path),`
   - `execute_run`:
     Starts with empty `active_stack: Vec<PathBuf>` and fresh `ScriptContext::empty()`.
     Resolves path and calls `execute_script_file(&mut self, path, base_dir, active_stack, context)`.
   - Recursion guard:
     If the canonicalized script path is already present in `active_stack`:
     Return `Err(RuntimeError::ScriptError(ScriptError::new(canonical_path, 1, "recursive script inclusion is not supported")))`.
   - Nested `run`:
     When a command inside a script parses to `Command::Run { path: child_path }`:
     Calls `execute_script_file` with `base_dir = Some(parent_path.parent().unwrap_or(Path::new(".")))`, inheriting the current `active_stack` and mutable `context`.
   - `exit` in script:
     When a command parses to `Command::Exit`:
     Terminates the script's command loop cleanly and returns `Ok(RunResult)`.
   - Line diagnostics:
     Any syntax parse error or runtime execution error encountered during script execution is converted into a `ScriptError` with the current script path and 1-based start line.

## Error diagnostics parity matrix

| Scenario | Diagnostic |
|---|---|
| Missing script file | `<path>:1: script file not found` |
| Directory path | `<path>:1: script path is a directory` |
| Non-UTF-8 script | `<path>:<line>: script file must be UTF-8 text` |
| Recursive script inclusion | `<path>:1: recursive script inclusion is not supported` |
| Unterminated multiline SQL | `<path>:<line>: sql multiline query is missing closing """` |
| Undefined macro | `<path>:<line>: undefined macro: <name>` |
| Unmatched `else` | `<path>:<line>: else without matching if` |
| Duplicate `else` | `<path>:<line>: if block already has an else branch` |
| Unmatched `end` | `<path>:<line>: end without matching if` |
| Unclosed `if` at EOF | `<path>:<line>: if block is missing end` |
| Command execution failure | `<path>:<line>: <command-failure-message>` |
| Command parse error | `<path>:<line>: <parse-error-message>` |

## Non-goals and explicit deferrals

- Interactive CLI driver loop (`src/main.rs` CLI wiring remains separate).
- Terminal output formatting / printing of script progress lines (`. <command>`).
- Signal handling / Ctrl-C cancellation during script execution.

# Bounded runtime `run` script execution slice summary

## Outcome

Accepted bounded runtime script execution slice (`run <script-path>`, Phase 5 §5.2). PR
[#135](https://github.com/SaehwanPark/tabdat-explore-rs/pull/135) was merged to
`main` as `4717a09`.

The runtime layer now exposes typed script execution in `tabdat-runtime`:
- `RunResult { path: PathBuf, executed_commands: usize }` struct representing the canonical path and executed command count.
- `ExecutionResult::Run(RunResult)` variant added to the typed public result model.
- `RuntimeError::ScriptError(ScriptError)` wrapping script errors with source file and line diagnostics.
- `Session::execute_run(&mut self, path)` entry point and internal recursive execution via `execute_script_file`.

The implementation enforces exact Python-compatible behavior and diagnostics:
- Script loading and parsing via `tabdat_language::script::read_script` with comment stripping and multiline SQL grouping.
- Sequential command execution on mutable active `Session` state.
- Nested `run <nested_path>` execution with relative path resolution against the enclosing parent script's directory.
- Recursion rejection: tracks active canonical script call stack and returns exact diagnostic (`<path>:1: recursive script inclusion is not supported`).
- Macro expansion and script context inheritance: `ScriptContext` holding macros and seed is shared and mutated across nested script calls.
- Control flow execution: `if` / `else` / `end` directives evaluate conditions; inactive branches skip execution and macro expansion while honoring nested block constraints.
- Directive evaluation: `seed` updates context random seed state; `let` defines macros.
- Early exit: `exit` directive cleanly stops script execution without error.
- Exact diagnostics: syntax errors and runtime execution failures encountered inside scripts are wrapped in `ScriptError` with the exact source path and 1-based start line.

Contract and implementation notes are recorded in:

- [01-contract.md](01-contract.md)

The pinned oracle, local locked/policy checks, PR-head workflows, and squash merge all passed. The
runtime script execution and recursion rejection boundary is closed; interactive REPL/CLI driver loop
remains unchecked in the roadmap.

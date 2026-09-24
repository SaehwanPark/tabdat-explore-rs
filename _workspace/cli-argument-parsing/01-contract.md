# Bounded CLI Argument Parsing and Execution Routing Contract

## Scope and Intent

This slice initiates **Phase 5 §7.1 (CLI, REPL, Reporting, and Machine Interfaces)** by implementing bounded CLI argument parsing, flag validation, and batch execution dispatch for the `tabdat` root binary (`src/main.rs`).

Until this slice, `tabdat-explore-rs` was a binary scaffold printing `Hello, world!` while all language and runtime execution remained library-only. This slice wires the root binary to the underlying `tabdat-language` and `tabdat-runtime` engines for:
1. Version reporting: `-v` / `--version`
2. Usage help: `-h` / `--help`
3. Batch command execution: repeated `-c <cmd>` / `--command <cmd>`
4. Script file execution: `-f <path>` / `--file <path>` and positional `<script>`
5. Exact error diagnostics, exit code conventions, and argument conflict rejection matching Python TabDat parity.

Interactive REPL shell (§7.2), JSON serialization and terminal table rendering (§7.3), visualization (§7.4), MCP server (§7.5), and discovery flags (`--list-commands`, `--list-command-effects`, `--help-topic`, `--explain`, `--describe-command`) remain explicitly deferred.

---

## Command-Line Interface Syntax

```text
usage: tabdat [-h] [-v] [-c COMMAND] [-f FILE] [--config CONFIG] [--json]
              [--list-commands] [--list-command-effects] [--help-topic TOPIC]
              [--explain] [--describe-command COMMAND] [--mcp]
              [script]
```

### Supported Arguments in this Slice

| Argument | Description | Parity Rule |
| :--- | :--- | :--- |
| `-v`, `--version` | Print version string and exit 0 | Prints `tabdat 0.1.0\n` matching package version |
| `-h`, `--help` | Print help/usage text and exit 0 | Displays usage and options summary |
| `-c`, `--command <CMD>` | Execute a single TabDat command string; can be repeated | Executes in sequential order on a single `Session`; stops on first error |
| `-f`, `--file <PATH>` | Execute a TabDat `.td` script file | Resolves path and executes via `Session::execute_script` |
| `<script>` | Positional script file path | Equivalent to `-f <PATH>` |
| *(no arguments)* | Scaffold behavior | Preserves `Hello, world!\n` exit 0 until REPL is implemented |

---

## Validation and Mutual Exclusivity Rules

1. **Conflict between `-c` and script execution**:
   - If both `-c/--command` and `-f/--file` or positional `<script>` are provided:
   - Stderr: `usage: tabdat ...\ntabdat: error: -c/--command cannot be combined with script execution\n`
   - Exit code: `2`
2. **Conflict between `-f` and positional script**:
   - If both `-f/--file` and positional `<script>` are provided:
   - Stderr: `usage: tabdat ...\ntabdat: error: -f/--file cannot be combined with a positional script\n`
   - Exit code: `2`
3. **Missing argument**:
   - If `-c` or `-f` is passed without an argument:
   - Stderr: `usage: tabdat ...\ntabdat: error: argument -c/--command: expected one argument\n`
   - Exit code: `2`
4. **Unrecognized option**:
   - If an unknown flag or option is passed:
   - Stderr: `usage: tabdat ...\ntabdat: error: unrecognized arguments: <args>\n`
   - Exit code: `2`

---

## Execution and Exit Code Semantics

| Condition | Exit Code | Stdout | Stderr |
| :--- | :--- | :--- | :--- |
| `--version` / `-v` | `0` | `tabdat 0.1.0\n` | *(empty)* |
| `--help` / `-h` | `0` | Usage text | *(empty)* |
| All `-c` commands succeed | `0` | Command outputs / silent | *(empty)* |
| `-c` command has parse error | `2` | *(empty)* | `Error: <parse_error>\n` |
| `-c` command has runtime error | `1` | *(empty)* | `Error: <runtime_error>\n` |
| Script file not found | `3` | *(empty)* | `Error: <path>:1: script file not found\n` |
| Script execution parse/syntax error | `2` | *(empty)* | `Error: <script_error>\n` |
| Script execution runtime error | `1` | *(empty)* | `Error: <script_error>\n` |
| CLI flag validation failure | `2` | *(empty)* | Usage + `tabdat: error: ...\n` |

---

## Architecture and Safety

- **Safe Rust Only**: `#![forbid(unsafe_code)]` in binary and all modules.
- **Zero New Dependencies**: Implemented using Rust standard library argument iterator (`std::env::args_os()`).
- **Workspace Wiring**: Root `Cargo.toml` adds path dependencies on `crates/tabdat-language` and `crates/tabdat-runtime`.
- **Directionality**: CLI binary depends on runtime and language; domain and backend logic remain independent.

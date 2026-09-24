# Bounded CLI argument parsing and batch execution slice summary

## Outcome

Accepted bounded CLI argument parsing and batch execution routing slice (Phase 5 §7.1). PR
[#145](https://github.com/SaehwanPark/tabdat-explore-rs/pull/145) was merged to
`main` as `231613f`.

The root binary (`src/main.rs`) and CLI layer (`src/cli.rs`) now wire `tabdat-explore-rs` to `tabdat-language` and `tabdat-runtime`:
- **Workspace Wiring**: Wires root package `tabdat-explore-rs` to `tabdat-language` and `tabdat-runtime` using path dependencies with explicit versions to comply with `deny.toml` wildcard policies.
- **CLI Argument Parser (`src/cli.rs`)**:
  - `-v`, `--version`: Prints `tabdat 0.1.0\n` and exits 0.
  - `-h`, `--help`: Prints standard usage and option summaries and exits 0.
  - `-c`, `--command <CMD>`: Repeated batch command execution against a `tabdat_runtime::Session`.
  - `-f`, `--file <PATH>` and positional `<script>`: Executes TabDat `.td` script files via `Session::execute_run`.
  - Conflict detection: Rejects `-c` combined with script execution, `-f` combined with positional scripts, missing arguments, and unrecognized flags with exact Python-compatible diagnostics (`tabdat: error: ...`) on stderr and exit code 2.
- **Execution Dispatch**:
  - Dispatches batch commands and script execution with exact error diagnostics and exit code conventions:
    - Exit code 0: Successful execution.
    - Exit code 1: Runtime execution error.
    - Exit code 2: Command/script parse or syntax error, or CLI flag error.
    - Exit code 3: Script file not found.
  - Preserves scaffold greeting (`Hello, world!\n`) when run with no arguments until interactive shell REPL is implemented in Phase 5 §7.2.
- **Testing**:
  - 10 unit tests in `src/cli.rs`.
  - 8 integration tests in `tests/cli_contract.rs`.
  - Original scaffold smoke test in `tests/scaffold.rs` continues to pass.

Contract and implementation notes are recorded in:

- [01-contract.md](01-contract.md)

The pinned oracle, local locked/policy checks, PR-head workflows, and squash merge all passed. The
bounded CLI argument parsing and batch execution slice is closed; interactive REPL shell (§7.2), JSON serialization and terminal table rendering (§7.3), visualization (§7.4), MCP server (§7.5), and discovery flags remain explicitly deferred.

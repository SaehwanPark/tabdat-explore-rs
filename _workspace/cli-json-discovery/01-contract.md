# Contract: CLI JSON & Command Discovery Interfaces

## 1. Scope & Objective

Implement Roadmap Phase 5 §7.1 CLI discovery and JSON interfaces:
- `--json`: emit versioned JSON envelopes for command execution, explain, and discovery; emit JSON error envelopes on failure.
- `--list-commands`: emit the complete canonical 81-command catalog (`CommandCatalogResult`) in versioned JSON format.
- `--list-command-effects`: emit declared command effects (`CommandEffectCatalogResult`) for all 81 commands in canonical order (`read`, `write`, `control`, `plot`, `unknown`).
- `--help-topic <topic>`: emit packaged markdown documentation for any canonical in-app help topic (`HelpTopicResult`).
- `--explain`: parse one batch command passed via `-c`/`--command` without starting a session or executing backend code (`CommandExplainResult`).
- `--describe-command <cmd>`: emit one command's syntax, arguments, and options schema (`CommandSchemaResult`).

All discovery and explain interfaces must execute as pure metadata/syntax operations without constructing a `Session` or initializing `duckdb`.

## 2. Python Oracle Authority & Evidence

- Pinned Python revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`
- Source references:
  - `src/tabdat/cli.py`
  - `src/tabdat/formatter.py`
  - `src/tabdat/models.py`
  - `src/tabdat/help/__init__.py` and `src/tabdat/help/topics/*.md`
  - `tests/test_cli.py`

### Exit Codes & Diagnostics Parity

1. Argument parsing & validation errors (Exit code 2, printed to `stderr` with `tabdat: error: ...`):
   - `--list-commands requires --json`
   - `--list-command-effects requires --json`
   - `--help-topic requires --json`
   - `--explain requires --json`
   - `--describe-command requires --json`
   - `--list-commands cannot be combined with command, script, or describe execution`
   - `--list-command-effects cannot be combined with another execution mode`
   - `--help-topic cannot be combined with command, script, command discovery, or describe execution`
   - `--explain cannot be combined with command discovery, help-topic retrieval, or describe execution`
   - `--describe-command cannot be combined with another execution mode`
   - `--explain requires exactly one -c/--command`
   - `--json requires a command execution, script path, explain, or discovery/describe flag`

2. Discovery and Explain Execution Errors (Exit code 1):
   - Output channel: `Error: <message>` to `stderr`, and JSON error envelope to `stdout`:
     ```json
     {"error":{"message":"<message>","type":"TabDatError"},"schema_version":1}
     ```
   - For `--describe-command`:
     - Empty/blank name: `command name cannot be empty` (`type: TabDatError`)
     - Unknown command: `unknown command name: <name>` (`type: TabDatError`)
   - For `--help-topic`:
     - Empty/blank topic: `help topic cannot be empty` (`type: TabDatError`)
     - Unknown topic: `unknown help topic: <topic>` (`type: TabDatError`)
   - For `--explain`:
     - Parse error: `Error: <parse_err>` to `stderr`, and `{"error":{"message":"<parse_err>","type":"ParseError"},"schema_version":1}` to `stdout` (`type: ParseError`).

3. Success Output (Exit code 0):
   - `--list-commands`:
     ```json
     {"data":{"commands":[{"help_topic":"...","name":"..."},...]},"result_type":"CommandCatalogResult","schema_version":1}
     ```
   - `--list-command-effects`:
     ```json
     {"data":{"commands":[{"effects":["..."],"name":"..."},...]},"result_type":"CommandEffectCatalogResult","schema_version":1}
     ```
   - `--describe-command <cmd>`:
     ```json
     {"data":{"arguments":[{"name":"...","required":true|false}],"help_topic":"...","name":"...","options":[{"name":"...","required":true|false}],"syntax":"..."},"result_type":"CommandSchemaResult","schema_version":1}
     ```
   - `--help-topic <topic>`:
     ```json
     {"data":{"help_topic":"...","text":"..."},"result_type":"HelpTopicResult","schema_version":1}
     ```
   - `--explain -c "<cmd>"`:
     ```json
     {"data":{"command_name":"...","execution":"not_run"},"result_type":"CommandExplainResult","schema_version":1}
     ```

## 3. Rust Implementation Design

- Root package `Cargo.toml`: Add `serde = { version = "1.0", features = ["derive"] }` and `serde_json = "1.0"` (both already locked in `Cargo.lock`).
- `src/catalog.rs`:
  - Static 81-command catalog array `COMMAND_NAMES`.
  - Effect categories enum `EffectCategory`: `Read`, `Write`, `Control`, `Plot`, `Unknown`.
  - Static command effects table `COMMAND_EFFECTS` mapping each of the 81 commands to its sorted canonical effect slice.
  - Static command schema table `COMMAND_SCHEMAS` mapping each of the 81 commands to syntax, arguments, options, and help topic.
  - Serialization types: `CommandCatalogEntry`, `CommandCatalogResult`, `CommandEffectEntry`, `CommandEffectCatalogResult`, `CommandSchemaResult`, `ArgumentDescriptor`, `OptionDescriptor`, `CommandExplainResult`, `HelpTopicResult`.
  - JSON envelope wrappers producing exact key-sorted JSON matching Python's `separators=(",", ":")` and `schema_version = 1`.
- `src/help.rs`:
  - Static map or match dispatch embedding all 79 packaged help topic markdown files.
  - Helper functions `available_help_topics() -> &'static [&'static str]` and `load_help_topic_text(topic: &str) -> Option<&'static str>`.
- `src/cli.rs`:
  - Update `CliArgs` with `json`, `list_commands`, `list_command_effects`, `help_topic`, `explain`, `describe_command`.
  - Update `parse_args` to parse these flags and enforce all exact mutual exclusivity and requirement rules.
  - Update `run_cli` to handle all discovery and explain paths without initializing a `Session` or database connection.
  - Support JSON error envelopes when `--json` is specified with `-c` or script execution.

## 4. Test Strategy

1. Unit tests in `src/cli.rs`, `src/catalog.rs`, and `src/help.rs`:
   - All 81 commands present in catalog, effects, and schemas.
   - All 79 help topics present and loadable.
   - Exact CLI argument conflict and requirement error strings.
   - Flag precedence (`--help` and `--version` precedence over validation).
2. Integration contract tests in `tests/cli_discovery_contract.rs`:
   - `--json --list-commands` output format and content.
   - `--list-commands` requires `--json`.
   - `--json --list-command-effects` output format and content.
   - `--list-command-effects` requires `--json`.
   - `--json --describe-command summarize` and `--json --describe-command does-not-exist`.
   - `--describe-command` requires `--json`.
   - `--json --help-topic summarize`, case-insensitivity (`SuMmArIzE`), and unknown topic error.
   - `--help-topic` requires `--json`.
   - `--json --explain -c "summarize age"`, stable command name extraction, and parse error handling.
   - `--explain` requires `--json` and exactly one `-c`.
   - Incompatible argument rejections for each discovery mode.
   - No session construction: metadata commands do not touch duckdb or filesystem.

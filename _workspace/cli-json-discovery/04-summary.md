# Summary: CLI JSON & Command Discovery Interfaces

## 1. Objective and Merged Artifacts

Implemented Roadmap Phase 5 §7.1 CLI discovery and JSON interfaces:
- `--json`: emit versioned JSON envelopes for command execution, explain, and discovery; emit JSON error envelopes on failure.
- `--list-commands`: emit the complete canonical 81-command catalog (`CommandCatalogResult`) in versioned JSON format.
- `--list-command-effects`: emit declared command effects (`CommandEffectCatalogResult`) for all 81 commands in canonical order (`read`, `write`, `control`, `plot`, `unknown`).
- `--help-topic <topic>`: emit packaged markdown documentation for any canonical in-app help topic (`HelpTopicResult`).
- `--explain`: parse one batch command passed via `-c`/`--command` without starting a session or executing backend code (`CommandExplainResult`).
- `--describe-command <cmd>`: emit one command's syntax, arguments, and options schema (`CommandSchemaResult`).

Merged in PR #147 (`c3ff323`).

## 2. Key Implementations

- **Root Dependency Wiring (`Cargo.toml`, `Cargo.lock`)**:
  - Added `serde` (`derive`) and `serde_json` with locked versions compliant with `deny.toml`.
- **In-App Help Topics (`src/help.rs`, `src/help/topics/`)**:
  - Packaged all 79 canonical help topic markdown files.
  - Implemented `available_help_topics()` and `load_help_topic_text()`.
- **Command Discovery & Schemas (`src/catalog.rs`)**:
  - Defined 81-command catalog `COMMAND_NAMES` and effect mapping `COMMAND_EFFECTS` matching Python oracle `16b45d9`.
  - Defined schemas for all commands with typed arguments and options.
  - Defined `ResultEnvelope<T>` and `ErrorEnvelope` with exact key order and field serialization matching Python's `separators=(",", ":")` and `schema_version = 1`.
- **CLI Argument Parsing & Dispatch (`src/cli.rs`)**:
  - Added `--json`, `--list-commands`, `--list-command-effects`, `--help-topic`, `--explain`, `--describe-command` flags.
  - Enforced exact mutual exclusivity and `requires --json` validation rules matching Python `argparse`.
  - Dispatched metadata/explain commands directly without database/session initialization.
  - Emitted JSON error envelopes on failure when `--json` is active.

## 3. Verification & Evidence

- 19 unit tests across `cli.rs`, `catalog.rs`, and `help.rs`.
- 17 integration contract tests in `tests/cli_discovery_contract.rs`.
- Backwards compatibility confirmed with `tests/cli_contract.rs` and `tests/scaffold.rs`.
- CI workflows passed:
  - `Rust baseline` (cargo fmt, check all targets, test all targets, clippy with `-D warnings`)
  - `Dependency and unsafe-code policy` (cargo deny, cargo audit, unsafe report)
  - `TabDat runtime boundary` on Linux (check runtime targets, test runtime targets, clippy)

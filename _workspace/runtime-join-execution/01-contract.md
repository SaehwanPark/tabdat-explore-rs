# Bounded runtime `join` command execution contract

Status: contract checkpoint; implementation and validation are in progress.

Producer: task owner, using `tabdat-data-semantics`, `tabdat-migration`, and
`preferred-workflow`.
Consumer: the bounded Rust runtime execution implementation and its focused review.

## Scope

Add the runtime execution boundary for the already-parsed
`join <table> on <keylist> [, how=inner|left suffix(_right)]` command
(Roadmap Phase 4 §6.4 & §6.1).

The runtime boundary requires an active dataset and a registered named table,
validates that all specified join keys exist in both the active and named-table
relations, preserves active and match row sequences via collision-free internal
ordering columns, handles column collisions with the specified or default suffix,
retains left-table label metadata for surviving columns, synchronizes any active
named table, and publishes the joined dataset atomically.

Supported forms in this slice:

- `join <table> on <keylist>` — default `how=inner`, `suffix="_right"`;
- `join <table> on <keylist>, how=left` — left join preserving unmatched active rows with NULL right values;
- `join <table> on <keylist>, suffix(<str>)` — custom suffix for colliding right-side columns;
- `join <table> on <keylist>, how=left suffix(<str>)` — combined mode and custom suffix;
- multiple join keys — `join <table> on key1 key2`;
- error handling:
  - no active dataset: returns typed `NoActiveDataset { command: "join" }`;
  - unknown named table: returns typed `UnknownTable { name: table_name }` ("unknown table: <name>");
  - unknown key in active dataset: returns typed `JoinUnknownVariable { variables }` ("join unknown variable: <vars>");
  - unknown key in named table: returns typed `JoinUnknownVariableInTable { table_name, variables }` ("join unknown variable in <name>: <vars>").

Remote DuckDB sessions, external databases, right/full outer joins (not supported in TabDat language),
and CLI/JSON/MCP rendering remain explicitly deferred.

## Python contract

Pinned authority:

- repository: `https://github.com/SaehwanPark/tabdat-explore.git`;
- revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`;
- tree: `601b236788872323af9277d2276a236154a0f129`;
- package: `0.25.0`; and
- `uv.lock` SHA-256:
  `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.

Relevant authority paths:

- `src/tabdat/executor.py:1490-1503` — `_execute_join` execution logic;
- `src/tabdat/backend.py:481-541` — `join_named_table` and `validate_join`;
- `src/tabdat/backend.py:3545-3605` — `_right_join_selects`, `_join_order_column_names`, `_unique_internal_name`, `_unique_output_name`;
- `src/tabdat/models.py:321-326` — `JoinCommand` model;
- `src/tabdat/models.py:1099-1104` — `TransformResult` shape;
- `tests/test_executor.py:5836-6145` — end-to-end inner/left join, multiple keys, collision suffixing, ordering, and validation error tests.

## Rust contract

In `tabdat-runtime`:

1. Public result models:
   - `JoinResult { dataset: DatasetInfo }`
   - `ExecutionResult::Join(JoinResult)`

2. Public error models:
   - `RuntimeError::JoinUnknownVariable { variables: Vec<String> }`
   - `RuntimeError::JoinUnknownVariableInTable { table_name: String, variables: Vec<String> }`
   - (Reuses existing `RuntimeError::NoActiveDataset { command: "join" }` and `RuntimeError::UnknownTable { name: String }`)

3. Session execution:
   - `Session::execute_join(&mut self, command: &JoinCommand) -> Result<ExecutionResult, RuntimeError>`
   - `Session::execute` routes `Command::Join { command }` to `execute_join(&command)`.
   - `DuckDbBackend::join_named_table` generates and executes the ordered join query into staging table, publishes staging to active, and returns new `DatasetInfo`.
   - Active named table synchronization via `self.sync_active_dataset(next_dataset)`.
   - Preserves left variable labels for surviving columns via `self.retain_label_metadata(&next_dataset)`.

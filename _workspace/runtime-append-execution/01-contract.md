# Bounded runtime `append` command execution contract

Status: contract checkpoint; implementation and validation are in progress.

Producer: task owner, using `tabdat-data-semantics`, `tabdat-migration`, and
`preferred-workflow`.
Consumer: the bounded Rust runtime execution implementation and its focused review.

## Scope

Add the runtime execution boundary for the already-parsed `append <table>` command
(Roadmap Phase 4 §6.4 & §6.1).

The runtime boundary requires an active dataset and a registered named table,
validates that the named table and active relation have the exact same set of
columns with compatible canonical types, preserves row order (all active relation
rows primary, followed by all named-table rows secondary, each preserving internal
row sequence), retains surviving active-table label metadata, detaches the active
relation from any underlying named table (`active_table_name = None`), and publishes
the combined dataset atomically.

Supported forms in this slice:

- `append <table>` — appends rows from `<table>` to the active dataset;
- schema validation:
  - no active dataset: returns typed `NoActiveDataset { command: "append" }` ("append requires an active dataset");
  - unknown named table: returns typed `UnknownTable { name: table_name }` ("unknown table: <name>");
  - extra variable in table: returns typed `AppendUnknownVariable { variables }` ("append unknown variable: <vars>");
  - missing variable in table: returns typed `AppendUnknownVariableInTable { table_name, variables }` ("append unknown variable in <name>: <vars>");
  - type mismatch between active and append table: returns typed `AppendTypeMismatch { variable, left_type, right_type }` ("append type mismatch for <var>: <left> vs <right>").
- row order preservation:
  - active table rows always precede appended table rows (`__tabdat_append_side = 0` vs `1`);
  - rows within each side preserve original sequence via `row_number() OVER ()` (`__tabdat_append_row`);
  - internal order column collision-avoidance via `unique_internal_name`.
- state transition:
  - active dataset updated with combined row count and active column schema;
  - detached transform: `active_table_name` is set to `None`, ensuring subsequent mutations do not overwrite the named table from which active was originally loaded;
  - named table registry remains unmodified;
  - failed validation leaves all session state completely untouched.

Remote DuckDB sessions, external databases, schema evolution/union of mismatched columns,
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

- `src/tabdat/executor.py:1505-1515` — `_execute_append` and detached transform recording;
- `src/tabdat/executor.py:1839-1842` — `_record_detached_transform`;
- `src/tabdat/backend.py:542-575` — `append_named_table`;
- `src/tabdat/backend.py:576-596` — `validate_append`;
- `src/tabdat/backend.py:3444-3480` — `_require_columns`, `_require_matching_types`, `_canonical_data_type`;
- `src/tabdat/models.py:328-330` — `AppendCommand`;
- `tests/test_executor.py:6160-6234` — Phase 11 append execution, snapshot preservation, schema error tests;
- `tests/test_executor.py:8570-8686` — row order preservation and failure rollbacks across engines.

## Rust contract

In `tabdat-runtime`:

1. Public result models:
   - `AppendResult { dataset: DatasetInfo }`
   - `ExecutionResult::Append(AppendResult)`

2. Public error models:
   - `RuntimeError::AppendUnknownVariable { variables: Vec<String> }`
   - `RuntimeError::AppendUnknownVariableInTable { table_name: String, variables: Vec<String> }`
   - `RuntimeError::AppendTypeMismatch { variable: String, left_type: String, right_type: String }`
   - `RuntimeError::AppendFailed`
   - Reuses `RuntimeError::NoActiveDataset { command: "append" }` and `RuntimeError::UnknownTable { name: String }`.

3. Session execution:
   - `Session::execute_append(&mut self, table_name: &str) -> Result<ExecutionResult, RuntimeError>`
   - `Session::execute` routes `Command::Append { table_name }` to `execute_append(&table_name)`.
   - `DuckDbBackend::append_named_table(&mut self, dataset: &DatasetInfo, append_dataset: &DatasetInfo, table_name: &str) -> Result<DatasetInfo, RuntimeError>`.
   - On success:
     - `self.retain_label_metadata(&next_dataset)`.
     - Detach active table name: `self.active_table_name = None`.
     - `self.active_dataset = Some(next_dataset)`.
     - Returns `ExecutionResult::Append(AppendResult { dataset })`.

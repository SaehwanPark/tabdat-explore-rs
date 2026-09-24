# Bounded runtime `reshape` command execution contract

Status: contract checkpoint; implementation and validation are in progress.

Producer: task owner, using `tabdat-data-semantics`, `tabdat-migration`, and
`preferred-workflow`.
Consumer: the bounded Rust runtime execution implementation and its focused review.

## Scope

Add the runtime execution boundary for the already-parsed `reshape long|wide <variables>, i(<identifiers>) j(<j_variable>)` command
(Roadmap Phase 4 §6.4 & §6.1).

The runtime boundary requires an active dataset, validates that identifiers and participating columns exist,
enforces collision rules on generated output column names, preserves deterministic row and column ordering
(for `long`: source row order primary, stub j value sequence secondary; for `wide`: min source row order across
the id group), retains surviving active-table label metadata, detaches the active relation from any underlying
named table (`active_table_name = None`), and publishes the reshaped dataset atomically.

Supported forms in this slice:

- `reshape long <variables>, i(<identifiers>) j(<j_variable>)`:
  - converts repeated wide columns matching `<variable>_<j>` into rows;
  - validates all identifiers exist in the active relation;
  - validates `j_variable` does NOT exist in the active relation;
  - discovers `j_values` in order of appearance across `variables` (stubs);
  - requires that every stub column has matching columns for all discovered `j_values`;
  - constructs long-format rows preserving `(row_order, j_order)` sequence;
  - produces schema: `[identifiers..., j_variable, variables...]`.
- `reshape wide <variables>, i(<identifiers>) j(<j_variable>)`:
  - converts repeated rows into suffixed wide columns `<variable>_<j>`;
  - validates `identifiers`, `j_variable`, and `variables` all exist in the active relation;
  - discovers non-null distinct `j_values` cast to string, ordered lexicographically;
  - requires at least one non-null `j_value`;
  - validates that generated output columns `<variable>_<j>` do not collide with existing columns not participating in the reshape;
  - aggregates values per identifier group via `MAX(CASE WHEN ... END)`;
  - preserves group ordering by `MIN(source_row_order)`;
  - produces schema: `[identifiers..., (for v in variables: for j in j_values: v_j)...]`.
- schema and state validation:
  - no active dataset: returns typed `NoActiveDataset { command: "reshape" }` ("reshape requires an active dataset");
  - missing variables or identifiers: returns typed `ReshapeUnknownVariable { variables }` ("reshape unknown variable: <vars>");
  - long output `j_variable` collision: returns typed `ReshapeOutputColumnExists { variable }` ("reshape output column already exists: <var>");
  - long stub without matching columns: returns typed `ReshapeLongFoundNoColumnsForStub { stub }` ("reshape long found no columns for stub: <stub>");
  - long missing column across stubs: returns typed `ReshapeLongMissingColumn { column }` ("reshape long missing column: <col>");
  - wide found no j values: returns typed `ReshapeWideFoundNoJValues` ("reshape wide found no j values");
  - wide output column collision: returns typed `ReshapeWideOutputColumnExists { variable }` ("reshape wide output column already exists: <var>").
- internal column collision-avoidance:
  - `unique_internal_name` ensures internal ordering columns (`__tabdat_reshape_row_order`, `__tabdat_reshape_j_order`, `__tabdat_reshape_group_order`, `__tabdat_reshape_source_order`) never overwrite user columns.
- state transition:
  - active dataset updated with reshaped schema and row count;
  - detached transform: `active_table_name` is set to `None`, preserving the underlying named table snapshot if loaded from one;
  - failed validation leaves all session state completely untouched.

Remote DuckDB sessions, external databases, and CLI/JSON/MCP rendering remain explicitly deferred.

## Python contract

Pinned authority:

- repository: `https://github.com/SaehwanPark/tabdat-explore.git`;
- revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`;
- tree: `601b236788872323af9277d2276a236154a0f129`;
- package: `0.25.0`; and
- `uv.lock` SHA-256:
  `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.

Relevant authority paths:

- `src/tabdat/executor.py:1517-1534` — `_execute_reshape` and detached transform recording;
- `src/tabdat/backend.py:598-660` — `reshape_long` and `validate_reshape_long`;
- `src/tabdat/backend.py:661-719` — `reshape_wide` and `validate_reshape_wide`;
- `src/tabdat/backend.py:2775-2795` — `_reshape_wide_j_values`;
- `src/tabdat/backend.py:3492-3543` — `_reject_existing_column`, `_long_j_values`, `_require_long_stub_columns`, `_require_wide_output_names`;
- `src/tabdat/backend.py:3576-3586` — `_reshape_long_order_column_names`;
- `tests/test_executor.py:6235-6635` — reshape long/wide roundtrip, source/group order preservation, collision avoidance, null handling, and validation failure tests.

## Rust contract

In `tabdat-runtime`:

1. Public result models:
   - `ReshapeResult { dataset: DatasetInfo }`
   - `ExecutionResult::Reshape(ReshapeResult)`

2. Public error models:
   - `RuntimeError::ReshapeUnknownVariable { variables: Vec<String> }`
   - `RuntimeError::ReshapeOutputColumnExists { variable: String }`
   - `RuntimeError::ReshapeLongFoundNoColumnsForStub { stub: String }`
   - `RuntimeError::ReshapeLongMissingColumn { column: String }`
   - `RuntimeError::ReshapeWideFoundNoJValues`
   - `RuntimeError::ReshapeWideOutputColumnExists { variable: String }`
   - `RuntimeError::ReshapeFailed`
   - Reuses `RuntimeError::NoActiveDataset { command: "reshape" }`.

3. Session execution:
   - `Session::execute_reshape(&mut self, command: &ReshapeCommand) -> Result<ExecutionResult, RuntimeError>`
   - `Session::execute` routes `Command::Reshape { command }` to `execute_reshape(&command)`.
   - `DuckDbBackend::reshape_long(&mut self, dataset: &DatasetInfo, variables: &[String], identifiers: &[String], j_variable: &str) -> Result<DatasetInfo, RuntimeError>`.
   - `DuckDbBackend::reshape_wide(&mut self, dataset: &DatasetInfo, variables: &[String], identifiers: &[String], j_variable: &str) -> Result<DatasetInfo, RuntimeError>`.
   - On success:
     - `self.retain_label_metadata(&next_dataset)`.
     - Detach active table name: `self.active_table_name = None`.
     - `self.active_dataset = Some(next_dataset)`.
     - Returns `ExecutionResult::Reshape(ReshapeResult { dataset })`.

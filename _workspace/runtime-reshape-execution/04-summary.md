# Bounded runtime `reshape` command execution slice summary

## Outcome

Accepted bounded runtime `reshape` execution slice (`reshape long|wide <stubs>, i(<identifiers>) j(<j_variable>)`, Phase 4 §6.4 & §6.1). PR
[#143](https://github.com/SaehwanPark/tabdat-explore-rs/pull/143) was merged to
`main` as `4d90584`.

The runtime layer now exposes typed `reshape` execution against an active eager relation in `tabdat-runtime`:
- `ReshapeResult { dataset: DatasetInfo }` representing the transformed dataset after reshape execution.
- `ExecutionResult::Reshape(ReshapeResult)` variant added to the typed public result model.
- Typed runtime error variants matching exact Python parity:
  - `RuntimeError::ReshapeUnknownVariable { variables: Vec<String> }` ("reshape unknown variable: <vars>")
  - `RuntimeError::ReshapeOutputColumnExists { variable: String }` ("reshape output column already exists: <var>")
  - `RuntimeError::ReshapeLongFoundNoColumnsForStub { stub: String }` ("reshape long found no columns for stub <stub>")
  - `RuntimeError::ReshapeLongMissingColumn { stub: String, j_value: String }` ("reshape long missing column <stub><j> for stub <stub>")
  - `RuntimeError::ReshapeWideFoundNoJValues` ("reshape wide found no j values")
  - `RuntimeError::ReshapeWideOutputColumnExists { variable: String }` ("reshape wide output column already exists: <var>")
  - `RuntimeError::ReshapeFailed` ("reshape failed")
- `Session::execute_reshape(&mut self, command: &ReshapeCommand)` entry point, also routed from `Session::execute`.

The implementation enforces exact Python-compatible behavior and invariants:
- **Reshape Long (`reshape long`)**:
  - Discovers stub `j_values` in order of column appearance in the active relation schema.
  - Validates that each stub column is present across all discovered `j_values`; returns exact error `reshape long missing column <stub><j> for stub <stub>` on ragged stub occurrences.
  - Returns `reshape long found no columns for stub <stub>` if any stub has no matching columns.
  - Returns `reshape output column already exists: <j_var>` if the target `j_var` name already exists in the active relation.
  - Unpivots stubs via `UNION ALL` branches preserving row order primary (`row_order`) and discovered stub order secondary (`j_order`).
- **Reshape Wide (`reshape wide`)**:
  - Extracts distinct non-null `j_values` ordered lexicographically via DuckDB scalar query.
  - Returns `reshape wide found no j values` if all `j` values are null or empty.
  - Checks for output column collisions (`<stub><j>`) against non-participating columns; returns exact error `reshape wide output column already exists: <col>`.
  - Aggregates values using `MAX(CASE WHEN <j_var> = <j_val> THEN <stub> END)` grouped by identifier columns (`id_vars`), ordered by `MIN(source_order)` to preserve the initial appearance order of identifier groups.
- **Shared Invariants**:
  - Collision-free internal ordering columns: uses `unique_internal_name` to prevent colliding with existing columns named `__tabdat_reshape_row_order`, `__tabdat_reshape_j_order`, `__tabdat_reshape_group_order`, or `__tabdat_reshape_source_order`.
  - Detached transform behavior: sets `active_table_name = None` so that subsequent mutations on the active relation do not overwrite the named table from which active was originally loaded, preserving the named table snapshot.
  - Variable label retention: preserves surviving variable labels from the active relation via `self.retain_label_metadata`.
  - Atomic staging table lifecycle: builds the reshaped relation in `__tabdat_staging` and atomically publishes it to `__tabdat_active`, cleaning up staging on failure without altering session state.
  - Multi-statement `.td` script integration: scripts can execute `reshape` commands seamlessly alongside other transformation and inspection commands.

Contract and implementation notes are recorded in:

- [01-contract.md](01-contract.md)

The pinned oracle, local locked/policy checks, PR-head workflows, and squash merge all passed. The
bounded `reshape` command execution slice is closed; remote DuckDB sessions, external databases, complex nested stubs, and CLI/JSON/MCP rendering remain explicitly deferred.

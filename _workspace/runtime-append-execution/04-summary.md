# Bounded runtime `append` command execution slice summary

## Outcome

Accepted bounded runtime `append` execution slice (`append <table>`, Phase 4 §6.4 & §6.1). PR
[#141](https://github.com/SaehwanPark/tabdat-explore-rs/pull/141) was merged to
`main` as `800a231`.

The runtime layer now exposes typed `append` execution against an active relation and a registered named table in `tabdat-runtime`:
- `AppendResult { dataset: DatasetInfo }` representing the combined dataset after append execution.
- `ExecutionResult::Append(AppendResult)` variant added to the typed public result model.
- Typed runtime error variants matching exact Python parity:
  - `RuntimeError::AppendUnknownVariable { variables: Vec<String> }` ("append unknown variable: <vars>")
  - `RuntimeError::AppendUnknownVariableInTable { table_name: String, variables: Vec<String> }` ("append unknown variable in <name>: <vars>")
  - `RuntimeError::AppendTypeMismatch { variable: String, left_type: String, right_type: String }` ("append type mismatch for <var>: <left> vs <right>")
  - `RuntimeError::AppendFailed` ("append failed")
- `Session::execute_append(&mut self, table_name: &str)` entry point, also routed from `Session::execute`.

The implementation enforces exact Python-compatible behavior and invariants:
- Row order preservation: preserves active table row order primary (`side = 0`) and append table row sequence secondary (`side = 1`), preserving internal row sequence within each side via `row_number() OVER ()`.
- Column alignment: projects columns by explicit active dataset schema name on both sides before `UNION ALL`, guaranteeing correct alignment even if the named table columns were defined in a different order.
- Collision-free internal ordering columns: uses `unique_internal_name` to avoid colliding with existing columns named `__tabdat_append_side` or `__tabdat_append_row`.
- Detached transform behavior: sets `active_table_name = None` so that subsequent mutations on the active relation do not overwrite the named table from which active was originally loaded, preserving the named table snapshot.
- Variable label retention: preserves surviving variable labels from the active relation via `self.retain_label_metadata`.
- Atomic staging table lifecycle: builds the appended relation in `__tabdat_staging` and atomically publishes it to `__tabdat_active`, cleaning up staging on failure without altering session state.
- Multi-statement `.td` script integration: scripts can execute `append` commands seamlessly alongside `sql ... into <table>` and other transformation commands.

Contract and implementation notes are recorded in:

- [01-contract.md](01-contract.md)

The pinned oracle, local locked/policy checks, PR-head workflows, and squash merge all passed. The
bounded `append` command execution slice is closed; remote DuckDB sessions, external databases, schema evolution/mismatched column union, and CLI/JSON/MCP rendering remain explicitly deferred.

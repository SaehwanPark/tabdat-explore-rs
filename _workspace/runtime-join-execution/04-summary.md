# Bounded runtime `join` command execution slice summary

## Outcome

Accepted bounded runtime `join` execution slice (`join <table> on <keylist> [, how=inner|left suffix(_right)]`, Phase 4 §6.4 & §6.1). PR
[#139](https://github.com/SaehwanPark/tabdat-explore-rs/pull/139) was merged to
`main` as `56babda`.

The runtime layer now exposes typed `join` execution against an active relation and a registered named table in `tabdat-runtime`:
- `JoinResult { dataset: DatasetInfo }` representing the transformed dataset after join execution.
- `ExecutionResult::Join(JoinResult)` variant added to the typed public result model.
- `RuntimeError::JoinUnknownVariable { variables: Vec<String> }` ("join unknown variable: <vars>") and `RuntimeError::JoinUnknownVariableInTable { table_name: String, variables: Vec<String> }` ("join unknown variable in <name>: <vars>") error variants with exact Python parity.
- `Session::execute_join(&mut self, command: &JoinCommand)` entry point, also routed from `Session::execute`.

The implementation enforces exact Python-compatible behavior and invariants:
- Row order preservation: preserves active table row order primary and matching named-table row sequence secondary using collision-free internal row order identifiers (`__tabdat_join_order`, `__tabdat_join_right_order`).
- Right-side column collision handling: renames colliding columns from the right-hand relation using default suffix `_right` or user-specified `suffix(...)`, ensuring collision-free output identifiers via incremental suffix dedup.
- Inner and Left joins: supports `how=inner` (filtering to matching keys) and `how=left` (preserving all active rows, filling missing right columns with NULL).
- Multi-key joins: supports joining on multiple key columns (`join <table> on key1 key2`).
- Surviving label metadata retention: preserves left-table variable labels for surviving columns via `self.retain_label_metadata`.
- Active named table synchronization: updates underlying named table and registry if the active dataset was loaded from a named table.
- Atomic staging table lifecycle: builds the joined relation in `__tabdat_staging` and atomically publishes it to `__tabdat_active`, cleaning up staging on failure without altering session state.
- Multi-statement `.td` script integration: scripts can execute `join` commands seamlessly alongside `sql ... into <table>` and other transformation commands.

Contract and implementation notes are recorded in:

- [01-contract.md](01-contract.md)

The pinned oracle, local locked/policy checks, PR-head workflows, and squash merge all passed. The
bounded `join` command execution slice is closed; remote DuckDB sessions, external databases, right/full outer joins (not supported in TabDat language), and CLI/JSON/MCP rendering remain explicitly deferred.

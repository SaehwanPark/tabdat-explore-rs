# Bounded runtime `sql` command execution slice summary

## Outcome

Accepted bounded runtime SQL query and named table slice (`sql <query> [into <table>]`, Phase 4 §6.5 & §6.1). PR
[#137](https://github.com/SaehwanPark/tabdat-explore-rs/pull/137) was merged to
`main` as `44686be`.

The runtime layer now exposes typed SQL execution and named table lifecycle management in `tabdat-runtime`:
- `TableResult { headers: Vec<String>, rows: Vec<Vec<CellValue>> }` representing typed tabular query results.
- `SqlCreateResult { table_name: String, dataset: DatasetInfo }` representing target named table creation and dataset publishing.
- `ActivateResult { table_name: String, dataset: DatasetInfo }` representing named table activation into active relation.
- `ExecutionResult::Table`, `ExecutionResult::SqlCreate`, and `ExecutionResult::Activate` variants added to the typed public result model.
- `RuntimeError::SqlNotSelectOrWith`, `RuntimeError::SqlFailed`, `RuntimeError::SqlNoTableResult`, `RuntimeError::UseOptionsNotSupportedForNamedTable`, and `RuntimeError::UnknownTable { name }` error variants with exact Python-parity diagnostics.
- `Session::execute_sql(&mut self, query, into)` entry point, `Session::named_tables()`, and `Session::active_table_name()` accessors.

The implementation enforces exact Python-compatible behavior and invariants:
- Direct query execution (`sql <query>`) evaluates against DuckDB with the `active` view bound, returning headers and cell values without modifying the active dataset.
- Target query execution (`sql <query> into <table>`) evaluates the query into temporary table `__tabdat_named_<table>`, updates `__tabdat_active`, registers the table in `self.named_tables`, and tracks `self.active_table_name`.
- Active named table synchronization: subsequent transforms (`keep`, `drop`, `generate`, `replace`, etc.) that mutate `__tabdat_active` automatically update the underlying `__tabdat_named_<name>` table and in-memory registry.
- Named table activation (`use <table>`) resolves against `self.named_tables`, publishes the relation as active, and rejects loader options (`delimiter`, `lazy`, etc.) with `use options are not supported for named table activation`.
- Query validation: ensures queries begin with `select` or `with` (case-insensitive); non-query statements return `sql only supports select or with queries in Phase 4`.
- Unresolved table diagnostics: `use <missing>` without file extension or path separators produces `unknown table: <name>`.
- Multi-statement `.td` script integration: scripts can execute multiline triple-quoted SQL queries and `into <table>` workflows seamlessly.

Contract and implementation notes are recorded in:

- [01-contract.md](01-contract.md)

The pinned oracle, local locked/policy checks, PR-head workflows, and squash merge all passed. The
bounded SQL query execution and named-table lifecycle slice is closed; multi-database connections, remote DuckDB sessions, and CLI/JSON/MCP rendering remain explicitly deferred.

# Bounded runtime `sql` command execution contract

Status: contract checkpoint; implementation and validation are in progress.

Producer: task owner, using `tabdat-data-semantics`, `tabdat-migration`, and
`preferred-workflow`.
Consumer: the bounded Rust runtime execution implementation and its focused review.

## Scope

Add the runtime execution boundary for the already parsed `sql <query> [into <table>]`
command (Roadmap Phase 4 §6.5 & §6.1). The runtime boundary requires an active dataset,
validates that the query begins with `select` or `with` (case-insensitive), binds the
`active` view in DuckDB, and either returns the tabular query results or registers
and activates a named table target.

Supported forms in this slice:

- `sql <query>` — direct query execution against DuckDB with `active` view bound,
  returning typed `TableResult` (`headers`, `rows`) without modifying the active dataset;
- `sql <query> into <table>` — executes query into named table `__tabdat_named_<table>`,
  registers the named table in the session, and updates the active dataset to the new relation;
- `use <table>` — activates a previously registered named table when given a table name
  without file extension that exists in the session's named table registry;
- error handling:
  - no active dataset: returns typed `NoActiveDataset { command: "sql" }`;
  - non-`select`/`with` query: returns typed `SqlNotSelectOrWith` ("sql only supports select or with queries in Phase 4");
  - query execution failure: returns typed `SqlFailed` ("sql failed");
  - empty table results: returns typed `SqlNoTableResult` ("sql must produce a table result");
  - activating unknown named table: returns typed `UnknownTable { name }` ("unknown table: <name>");
  - options provided to named table activation: returns typed `UseOptionsNotSupportedForNamedTable` ("use options are not supported for named table activation").

Multi-database connections, remote DuckDB sessions, SQL mutation outside `into`, and CLI/JSON/MCP
rendering remain explicitly deferred.

## Python contract

Pinned authority:

- repository: `https://github.com/SaehwanPark/tabdat-explore.git`;
- revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`;
- tree: `601b236788872323af9277d2276a236154a0f129`;
- package: `0.25.0`; and
- `uv.lock` SHA-256:
  `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.

Relevant authority paths:

- `src/tabdat/executor.py:1476-1488` — `_execute_sql` execution logic;
- `src/tabdat/backend.py:797-808` — `run_sql` and `_bind_active_view`;
- `src/tabdat/backend.py:439-459` — `create_named_table_from_sql` and `activate_named_table`;
- `src/tabdat/models.py:1668-1677` — `SqlCreateResult` and `TableResult` shapes;
- `src/tabdat/models.py:1124-1127` — `ActivateResult` shape;
- `tests/test_executor.py:9273-9378` — end-to-end SQL query, into, activation, and error tests.

## Rust contract

In `tabdat-runtime`:

1. Public result models:
   - `TableResult { headers: Vec<String>, rows: Vec<Vec<CellValue>> }`
   - `SqlCreateResult { table_name: String, dataset: DatasetInfo }`
   - `ActivateResult { table_name: String, dataset: DatasetInfo }`
   - `ExecutionResult::Table(TableResult)`
   - `ExecutionResult::SqlCreate(SqlCreateResult)`
   - `ExecutionResult::Activate(ActivateResult)`

2. Public error models:
   - `RuntimeError::SqlNotSelectOrWith`
   - `RuntimeError::SqlFailed`
   - `RuntimeError::SqlNoTableResult`
   - `RuntimeError::UseOptionsNotSupportedForNamedTable`
   - `RuntimeError::UnknownTable { name: String }`

3. Session execution:
   - `Session::execute_sql(&mut self, query: &str, into: Option<&str>)`
   - `Session::named_tables(&self) -> &HashMap<String, DatasetInfo>`
   - `Session::execute_use` updated to check for named table activation.

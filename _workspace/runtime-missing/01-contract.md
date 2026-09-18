# Contract: bounded eager-runtime `missing`

Status: contract recovered; implementation pending

Selected skills: `tabdat-migration`, `tabdat-data-semantics`,
`simple-code-writer`

Boundary: pinned Python execution contract → Rust-owned eager-session missingness report

## Scope

Execute the existing syntax-only `Command::Missing { variables }` against the
active dataset held by the bounded eager local-Parquet `tabdat-runtime` session.
A successful request returns one owned row per selected variable. An empty
variable list selects all active columns in schema order; an explicit list
preserves requested order and duplicates. Each row reports the exact schema
type, total rows, SQL-NULL missing rows, nonmissing rows, and missingness
percentage. The operation is read-only: active metadata and the private
relation remain unchanged.

This slice builds on the accepted eager local-Parquet `use` session and the
read-only `describe`, `count`, `head`, `tail`, `summarize`, and `codebook`
runtime slices. It does not add lazy loading/materialization, transforms,
labels, formatting, JSON/MCP output, CLI/REPL behavior, or execution for
`duplicates`, `isid`, `datasignature`, or `assert`.

The result owns only Rust scalar metadata and counts; it exposes no DuckDB
`Value`/`ValueRef`, Arrow value, connection, statement, raw pointer, or foreign
lifetime. Missingness is explicit SQL NULL only. Empty strings, sentinel codes,
and NaN values are nonmissing under this contract.

## Python authority

The Python oracle is the clean sibling checkout `../tabdat-explore` at pinned
commit `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, package `0.25.0`, and Python
`3.13.3`. Its `uv.lock` SHA-256 is
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`; the
checkout is clean and no dependency synchronization or source edits are
performed. Authority paths are:

- `src/tabdat/models.py:179-184,1201-1213` — `MissingCommand`, `MissingRow`,
  and `MissingResult`;
- `src/tabdat/parser.py:574-580` — direct syntax and exact parser diagnostics;
- `src/tabdat/executor.py:914-916` — active-dataset dispatch;
- `src/tabdat/backend.py:899-929` — eager aggregate query and row construction;
- `src/tabdat/backend.py:2156-2180` — lazy aggregate path (explicitly deferred
  by this eager-only slice);
- `src/tabdat/backend.py:2955-2971` — missing percentage calculation;
- `tests/test_missing.py:1-126` — focused parser, result, ordering, unknown,
  empty, lazy, CLI, and no-active coverage; and
- `docs/commands/missing.md:1-25` plus `docs/language-semantics.md:50-56` —
  public semantics and deferred presentation behavior.

Focused oracle validation at the pinned revision:

```sh
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_missing.py
```

Observed result: `9 passed` (Python `3.13.3`; no environment synchronization).

## Rust contract

Add the following owned result types beside the existing inspection results:

```rust
pub struct MissingRow {
    pub variable: String,
    pub data_type: String,
    pub total: u64,
    pub missing: u64,
    pub nonmissing: u64,
    pub missing_percent: f64,
}

pub struct MissingResult {
    pub rows: Vec<MissingRow>,
}
```

Expose `ExecutionResult::Missing` and typed runtime diagnostics for unknown
variables and aggregate failure. Preserve the exact display forms:

- no active dataset: `missing requires an active dataset; run use <path> first`;
- unknown variables: `missing unknown variable: <comma-separated names>`;
- backend failure: `missing failed`.

Validate all unknown variables before opening a query, preserving their request
order and duplicates. If the variable list is empty, use active schema order.
The backend should use one aggregate query with `count(*)` plus one
`count(quoted_identifier)` per requested variable. Alias counts by request index
so duplicate names cannot collide. Convert count results with checked `u64`
conversion. Use the existing deliberate identifier quoting helper; never
interpolate untrusted identifiers unquoted.

For each requested variable:

```text
total = count(*)
nonmissing = count(variable)
missing = total - nonmissing
missing_percent = 0.0 when total == 0, otherwise 100.0 * missing / total
```

The command must not update `active_dataset`, row-count metadata, relation
ownership, or backend initialization state. A failed validation or aggregate
must leave the prior active dataset and metadata unchanged. Because no values
are copied, logical/container columns are supported as long as DuckDB can count
them; this is intentionally broader than preview/codebook value ownership.

## Test contract

Test first and preserve semantic state separately from result values:

- fresh-session exact no-active error and no DuckDB initialization;
- parser-to-session eager `use` followed by explicit `missing age cost`;
- default schema order, explicit order, and duplicate variables;
- exact schema types, total/missing/nonmissing counts, and percentages;
- all-null column and empty relation (zero percentage);
- unknown-variable diagnostics preserving duplicate request order;
- quoted identifiers and SQL NULL-only missingness (empty text and NaN remain
  nonmissing where the fixture supports them);
- repeated read-only execution and failed/dropped relation preserving metadata;
- a list/struct column count succeeds because no backend value conversion is
  required; and
- unsupported lazy/materialized configuration remains outside this bounded
  eager contract.

Use the existing local-Parquet fixture and tiny temporary Parquet fixtures. Do
not sort rows or normalize percentages beyond the explicit zero-total rule.

## Implementation mapping and completion state

- Native Rust: typed `MissingRow`/`MissingResult`, result enum, diagnostics, and
  session validation/state boundary;
- DuckDB: one quoted aggregate query over the private eager relation;
- Existing parser/session: reuse `Command::Missing`, active metadata, and
  atomic eager-load relation lifecycle; and
- Deferred: lazy/materialized execution, labels, wildcard/range varlists,
  formatting, JSON/MCP/CLI surfaces, and broad session/registry APIs.

Contract recovery is `complete` for the pinned Python behavior. Rust
implementation and hosted parity evidence remain `partial` until the bounded
slice is built, reviewed, merged, and documented.

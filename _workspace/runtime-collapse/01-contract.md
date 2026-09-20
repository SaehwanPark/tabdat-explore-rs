# Bounded eager-runtime `collapse` contract

Status: contract checkpoint; implementation and hosted verification are
pending.

## Authority

The behavior authority is the pinned Python checkout recorded by the prior
runtime migration slices: tabdat-explore revision
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, with the recorded `uv.lock`
SHA-256
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.

Focused oracle checks at this checkpoint:

    uv run --no-sync pytest -q -p no:cacheprovider tests/test_parser.py -k collapse
    uv run --no-sync pytest -q -p no:cacheprovider tests/test_executor.py -k collapse

The checks cover the direct parser form, required `by(...)` grouping, the
supported statistic set, and replacement of the active dataset with grouped
aggregate columns.

## Accepted command form

The Rust language layer will own this bounded case-insensitive form:

```text
collapse <statistic> <variable> [<variable> ...], by(<group> [<group> ...])
```

Supported statistics are `count`, `mean`, `sum`, `min`, and `max`. The command
requires one or more aggregate variables and exactly one non-empty `by(...)`
option containing one or more grouping variables. The parser rejects `if`
clauses, assignment syntax, missing or repeated `by(...)`, unsupported
statistics, and additional options with command-specific diagnostics.

## Runtime semantics

- Execution requires an active eager local-Parquet DuckDB relation. Parse-only
  validation and no-active-dataset validation do not initialize DuckDB.
- Grouping uses the listed group variables in listed order. SQL NULL group
  values form an explicit group, and grouped rows follow DuckDB ascending
  ordering for those group variables, with NULL values last.
- For every aggregate variable, the result column is named
  `<statistic>_<variable>`. Group columns precede aggregate columns in the
  published schema. `count(variable)` counts non-NULL values; the other
  aggregates use DuckDB's NULL-ignoring aggregate semantics and may produce
  NULL for an all-missing group.
- `count` accepts variables of any DuckDB type. `mean`, `sum`, `min`, and
  `max` require numeric aggregate variables and preserve the backend's owned
  scalar result types.
- Successful execution stages the grouped relation, then atomically replaces
  the active relation. The returned `CollapseResult` owns the resulting
  `DatasetInfo`; it exposes no DuckDB statement, row handle, or connection
  lifetime.
- The source path and eager execution metadata remain attached to the new
  dataset. Session-local label metadata is pruned to surviving group columns;
  no labels are synthesized for aggregate columns.
- Validation, type checking, query failure, and publication failure leave the
  prior active relation, dataset metadata, and label metadata unchanged.

## Explicit deferrals

This slice does not claim `if` predicates, weights, named-table execution,
lazy or materialized execution, panel metadata propagation, persistence,
formatting, CLI, JSON, MCP, or broad Python `collapse` parity. Those require
separate contracts and evidence.

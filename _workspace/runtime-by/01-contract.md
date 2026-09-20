# Bounded eager-runtime `by` contract

Status: implementation checkpoint on `feat/runtime-by`; the contract is
intentionally bounded and will be marked accepted only after local and hosted
verification. The pinned Python authority and focused evidence are recorded
below.

## Authority

The behavior authority is the pinned Python checkout recorded by the prior
runtime migration slices: tabdat-explore revision
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, with the recorded `uv.lock`
SHA-256
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.

Focused oracle checks at this checkpoint:

    uv run --no-sync pytest -q -p no:cacheprovider tests/test_parser.py -k by
    uv run --no-sync pytest -q -p no:cacheprovider tests/test_executor.py -k 'by_summarize_and_count or unsupported_by_command'

The parser check covers direct and quoted grouping identifiers, child-command
validation, and command-specific diagnostics. The executor check covers
grouped `summarize` and `count`, default numeric selection, unsupported child
commands, and preservation of the active dataset.

## Accepted command form

The Rust language layer will own this bounded case-insensitive form:

```text
by <group> [<group> ...]: summarize [<variable> ...]
by <group> [<group> ...]: count
```

The parser retains the child as a typed command so the runtime can report a
bounded capability error for children outside this slice. Grouping and child
variables may be backtick-quoted identifiers, including embedded whitespace or
punctuation. A colon is required outside quoted identifiers, at least one
grouping variable is required, and a child command is required. Nested `by`
commands and `help`, `status`, or `doctor` children are rejected during parse.

## Runtime semantics

- Execution requires an active eager local-Parquet DuckDB relation. Parse-only
  validation and no-active-dataset validation do not initialize DuckDB.
- This slice executes only grouped `summarize` and grouped `count`. Other
  parsed child commands, including `tabulate`, return an explicit bounded
  capability error and do not mutate the session.
- Grouping uses the listed variables in listed order. SQL NULL group values
  form an explicit group, and output rows use ascending group ordering with
  NULL values last for each grouping variable.
- `by ...: summarize vars` emits owned table headers consisting of the group
  variables followed by `mean_<variable>` for each requested numeric variable.
  With no variable list, all numeric active columns except grouping variables
  are selected in schema order. An empty selection is an error; explicit
  nonnumeric or unknown variables are rejected before querying DuckDB.
- `by ...: count` emits the group variables followed by `Count`. `COUNT(*)`
  counts every row in each group, including rows whose grouping values are
  NULL.
- Successful grouped queries are read-only. The returned `ByResult` owns all
  headers and scalar cells; the active relation, dataset metadata, and label
  metadata remain unchanged.
- Validation and query failure leave the prior active relation and session
  metadata unchanged.

## Explicit deferrals

This slice does not claim grouped `tabulate` execution, `if` predicates,
weights, named-table execution, lazy or materialized execution, panel metadata,
persistence, formatting, CLI, JSON, MCP, or broad Python `by` parity. Those
require separate contracts and evidence.

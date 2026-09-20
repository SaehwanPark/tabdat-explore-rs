# Bounded eager-runtime `tabulate` contract

Status: accepted and verified on main at merge commit 24405a6878034506e18086d5092d665b4dd25159.

## Authority

The behavior authority is the pinned Python checkout recorded by the prior
slice artifacts: tabdat-explore revision
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, with the recorded `uv.lock`
SHA-256
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.

Focused oracle checks at this checkpoint:

    uv run --no-sync pytest -q -p no:cacheprovider tests/test_tabulate_labels.py
    uv run --no-sync pytest -q -p no:cacheprovider tests/test_executor.py -k "tabulate_one_way_and_two_way or tabulate_missing_option_controls_missing_categories"

The checks cover parser flags, one-way frequencies, two-way frequencies,
percentages, missing-category inclusion, and value-label display.

## Accepted command forms

The Rust language layer will own these case-insensitive direct forms:

```text
tabulate <rowvar>
tabulate <rowvar> <columnvar>
tabulate <rowvar> [, missing] [, nolabel]
tabulate <rowvar> <columnvar> [, row] [, col] [, missing] [, nolabel]
```

Options are flag-only and may be written in any order after one comma. The
bounded parser rejects repeated options, option values, assignment syntax,
conditions, and unsupported options with command-specific diagnostics. A row
and column variable must be distinct. The `row` and `col` percentage flags are
valid only for a two-way table.

## Runtime semantics

- Execution requires an active eager local-Parquet DuckDB relation and does not
  initialize DuckDB for parse-only or no-active-dataset validation.
- One-way output has headers `(row_variable, "Count", "Percent")` and one row
  per observed category. Percent is the category count divided by the included
  table count, expressed in percentage points.
- Two-way output uses the row variable as the first column and emits one count
  column for each observed column category. `row` and `col` add adjacent
  `"<category> Row %"` and `"<category> Col %"` columns respectively.
- SQL NULL categories are excluded by default. `missing` includes them as an
  explicit category and fills absent two-way cells with zero counts and zero
  percentages.
- Categories and table rows follow DuckDB's native ascending order with NULL
  values last; tied rows have deterministic output order.
- A variable's attached session-local value-label set is applied to displayed
  row categories and two-way column headers unless `nolabel` is present.
  Underlying counted values remain native owned result values where they are
  returned as cells.
- A failed validation or query leaves the active relation and label metadata
  unchanged.

## Typed boundary

The runtime will expose an owned `TabulateResult` containing headers and rows
of owned scalar cells. It will not expose DuckDB statements, row handles, or
backend-specific result lifetimes. Runtime errors will distinguish no active
dataset, unknown variables, invalid percentage usage, and query failure.

## Explicit deferrals

This slice does not claim `values()/stat()` aggregation, `if` predicates, `by`
prefixes, multi-row or multi-column dimensions, named-table execution,
lazy/materialized execution, persistence, formatting, CLI, JSON, MCP, or broad
Python tabulate parity. Those require separate contracts and evidence.

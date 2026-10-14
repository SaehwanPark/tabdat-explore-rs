# tabulate

How to invoke:
`tabulate rowvar [colvar] [if expr] [, row col missing nolabel]`

`tabulate [if expr], rows(rowvar) [columns(colvar)] [row col missing nolabel]`

What it does:
Show an eager one-way frequency/percent table or a two-way frequency table.
`row` and `col` add explicitly headed row and column percentages to two-way
cells; they are rejected for one-way tables. Equivalent single-variable
`rows()`/`columns()` forms execute with or without a condition.

What problem it answers:
How are categorical values distributed in the current data or a selected cohort,
without filtering the active dataset or refitting a model?

A bounded `if` predicate selects TRUE rows only; FALSE and NULL/UNKNOWN rows are
excluded. Arithmetic, comparisons, boolean columns and NULL are supported;
numeric/string truthiness, function calls and broader logical expressions are
not installed. Explicit comparisons with NULL test missingness. Arithmetic that
becomes UNKNOWN is excluded; direct comparisons retain native DuckDB semantics.

Category SQL NULLs are excluded by default (either key in a two-way table).
`missing` includes them, last in native key order. Stored empty strings, NaN and
infinities are not SQL NULL. One-way totals and two-way row/column denominators
come only from the selected rows surviving these category rules—not the original
dataset or a cached estimation sample. Empty selections return empty tables.

Keys use native numeric/text/boolean ordering. When a dimension has attached
value labels (`label values`), category cells and wide column headers show those
labels by default; `nolabel` displays raw keys. Ordering/counting still use stored
values. This read-only operation preserves the active data, source/schema/order,
labels, named relation and cached model/sample.

Readable shell/`--text` and CLI JSON use the same owned `TabulateResult`.
JSON provides `data.headers` and `data.rows`, with preview scalar encoding
(including null missing/nonfinite cells and exact integer numbers). Consumers
need lossless integer parsing for wide keys; JSON null does not merge distinct
native nonfinite and SQL-NULL groups.

Limits: exactly one row and at most one column variable. Multi-dimensional
`rows()`/`columns()`, `values()`/`stat()` aggregation and `by ...: tabulate` remain
parser-only/unsupported. No category repair, data mutation or model-sample
reinterpretation is performed.

Examples:
- `tabulate sex`
- `tabulate sex outcome, row col`
- `tabulate sex if age >= 18`
- `tabulate sex if age >= 18, missing nolabel`
- `tabulate if age >= 18, rows(sex) columns(outcome) row col`

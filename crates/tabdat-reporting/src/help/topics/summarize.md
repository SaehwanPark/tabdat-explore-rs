# summarize

How to invoke:
`summarize [varlist]`

What it does:
Show descriptive statistics for one or more numeric variables. Each variable uses its own
finite, non-NULL sample: count, mean, sample SD, minimum and maximum all exclude SQL NULL,
NaN and positive/negative infinity. Sample SD uses N-1 and is undefined below two observations.
An empty finite sample has count zero and undefined mean/SD/extrema. Finite arithmetic overflow
can still fail native aggregates; this is not a general numerical-stability guarantee.
`missing` counts SQL NULL only and can therefore report a larger nonmissing count.

`--json` batch/scripts support `SummarizeResult` with ordered `data.rows` containing
`variable`, `count`, `mean`, `std_dev`, `minimum`, and `maximum`. Explicit variable order
and duplicates are preserved; omitted variables select numeric columns in schema order.
Missing/undefined and non-finite statistics serialize as null. Extrema use the existing
preview scalar encoding (integer numbers, exact scaled decimal strings).

What problem it answers:
How are the variables distributed, and how much missingness is there?

Examples:
- `summarize`
- `summarize age bmi`

Links:
- `docs/command_glossary_v0.md`

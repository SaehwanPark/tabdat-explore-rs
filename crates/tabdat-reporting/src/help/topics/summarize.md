# summarize

How to invoke:
`summarize [varlist]`

What it does:
Show descriptive statistics for one or more numeric variables. Counts exclude SQL NULLs;
standard deviation is sample SD and is undefined below two observations.

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

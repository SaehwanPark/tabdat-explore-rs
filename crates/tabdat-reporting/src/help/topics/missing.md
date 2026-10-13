# missing

How to invoke:
`missing [varlist]`

What it does:
Show total, missing, nonmissing, and missing-percentage counts for active dataset columns. Missing
means SQL NULL in the loaded relation. Stored empty strings, user-defined sentinel codes,
NaN and infinity are not treated as missing. Statistical commands can separately exclude
non-finite numeric values, so their estimation count need not equal this nonmissing count.
CSV loading may turn empty fields into NULL before this report runs.

What problem it answers:
Which columns have missing data, and how much?

Examples:
- `missing`
- `missing age income`

The report preserves schema or requested variable order, including repeated explicit variables.
The current runtime uses eager local CSV/Parquet through DuckDB; broader lazy/remote forms are not
installed. An empty relation reports zero counts and 0% by the existing convention.
Interactive output and CLI-v1 JSON expose these owned counts without recalculation.
Use `codebook` when you also need types, distinct counts, and example values.

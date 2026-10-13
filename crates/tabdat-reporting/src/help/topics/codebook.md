# codebook

How to invoke:
`codebook [varlist]`

What it does:
Show compact owned profiles: variable/type, non-NULL and SQL-NULL counts, native
non-NULL distinct count, and up to three non-NULL scan examples. Examples can
repeat; they are not a distinct-value list. Default order follows the schema;
explicit order and repeated variables are preserved. Empty/all-NULL columns
have no examples. Stored empty strings and non-finite numbers are nonmissing.

Interactive output and CLI-v1 JSON report these values without recalculation.
JSON `CodebookResult.data.rows` contains `variable`, `data_type`, `nonmissing`,
`missing`, `distinct`, and `examples`; metadata states SQL-NULL missingness,
example limit 3, and non-finite numeric example encoding as null. Examples use
the existing preview scalar policy (integer numbers, exact decimal strings,
base64-prefixed bytes). A JSON null example can therefore represent NaN/infinity,
not a SQL NULL selected by the backend. Use a lossless JSON parser for large integers.

Current execution is eager local CSV/Parquet through DuckDB. Composite example
values outside the owned scalar boundary can still fail the profile; broader
lazy/remote modes and new scalar conversions are not installed.

What problem it answers:
What do these columns look like at a glance?

Examples:
- `codebook`
- `codebook sex cost`

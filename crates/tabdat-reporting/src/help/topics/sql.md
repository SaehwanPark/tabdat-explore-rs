# sql

How to invoke:
`sql query`, `sql """ ... """`, or either form followed by `into name`.

What it does:
Run an installed SELECT/WITH query against the active relation exposed as `active`. Without `into`, return owned table rows; with `into`, create/replace and activate a session-local named table. Requires an active dataset even for a constant query.

What problem it answers:
How do I filter a cohort, aggregate or derive a table using SQL before a supported summary/model/persistence workflow?

Examples:
- `sql select sex, avg(bmi) from active group by sex order by sex`
- `sql """select x,y from active where y is not null order by x""" into cohort`
- `use cohort`
- `describe`
- `regress y x`

Notes:
- Explicit `ORDER BY` controls query-key sequence; include tie-breaker keys for a reproducible total order. Into preserves that query sequence in the active named relation. Without an order, no row-order guarantee is implied.
- CLI-v1 JSON returns `TableResult` with `data.headers`/`rows` for queries, retaining owned projection/row order and duplicate headers. Empty results retain headers. Existing scalar policy applies: exact decimal strings, losslessly parsed wide integers, base64-prefixed bytes, nonfinite/unavailable numbers as null. Not all native composite cell conversions are supported. No invented type/affected-row metadata is added to query results.
- All owned query rows are reported, without a default row cap. Use SQL `LIMIT` deliberately when bounded output is wanted. SQL/native functions and source-type coercion semantics are unchanged; reporting does not execute or rescan the query.
- Into returns `SqlCreateResult`, registered `use name` returns `ActivateResult`, with owned `data.table_name` and `data.dataset` metadata. Dataset source is the named-table identifier, not the loaded filename. Named tables are session-local; active transformations update the stored named relation. Into replaces an existing named destination under current backend rules. Parser names must be identifiers; `active` and the `__tabdat_` prefix are reserved. Column aliases may use native quoted SQL identifiers.
- Ordinary read-only queries retain session/data/label/model state. Into and named activation clear labels but retain the original cached fit/sample; explicitly refit for cohort estimation. Prediction uses current rows with the retained fit. Dataset label/panel metadata remains null placeholders, not session snapshots.
- Missing/binder/ordinary validation or unsupported-cell errors preserve prior state in tested cases and stop later script/files. Invalid named-use options reject. Native named publication is multi-step, not a general atomic rollback guarantee. Earlier successful effects/files are not rolled back on later failure, including output failure (MIG-0008).
- Existing query validation only checks the leading SELECT/WITH word. This is not a SQL security sandbox or a guarantee that every accepted statement/function is read-only. Execute only trusted SQL; no whole-SQL, filesystem, composite-type, lazy/remote, or platform/security certification is implied.

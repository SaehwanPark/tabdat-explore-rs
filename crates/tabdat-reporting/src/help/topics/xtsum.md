# xtsum

`xtsum [varlist], i(panelvar)`

Compute overall, between-panel, and within-panel summaries for numeric data.
The panel identifier is explicit because this runtime does not retain an
`xtset` declaration. No time variable or unique panel-time key is required.

Examples:

- `xtsum wage hours, i(id)`
- `xtsum, i(firm)` reports all numeric columns in schema order.

Each variable uses its own finite nonmissing sample. Overall N counts valid
rows; between `n` counts identifiers that contribute at least one valid value;
`T-bar` is the average valid observations per contributing identifier. Between
statistics use equally weighted identifier means. Within values subtract each
identifier's mean and add the pooled overall mean. Missing values in one
variable do not remove rows from another variable's summary.

The result is read-only and preserves the active data and retained estimates.
Numeric and string panel identifiers are supported but must be nonmissing;
numeric identifiers must also be finite. Explicit variables must be numeric.
Empty and singleton samples retain counts and return `null` for undefined
statistics.

Structured output uses `XtSumResult` with one ordered row per variable and
separate overall, between, and within counts and moments. This bounded command
does not add time-series operators, weights, `if`/`in`, a persistent panel
declaration, or inferential output. See the [full command contract](../../../../../docs/commands/xtsum.md).

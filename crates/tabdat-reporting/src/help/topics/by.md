# by

How to invoke:
`by groupvars: count` or `by groupvars: summarize [numeric_vars]`

What it does:
Run the installed read-only grouped count or native grouped mean, not an arbitrary
command runner. Both forms support readable terminal/--text and CLI-v1 JSON.

What problem it answers:
How do I inspect subgroup sizes or native means before analysis/reporting without
filtering data or refitting the cached model?

Examples:
- `by sex: count`
- `by firm_id year: count`
- `by sex: summarize age`
- `by firm_id year: summarize cost visits`

Groups and output:
- Group names must exist. Multiple/backtick-quoted/repeated groups are supported;
  headers preserve request order and duplicates. Native group order is ascending
  NULLS LAST for each key. Empty input yields headers and no groups.
- Raw keys are reported, not substituted value labels. SQL-NULL key groups are
  included; stored empty strings are distinct. Existing scalar policy applies:
  exact decimal strings, losslessly parsed wide integers, base64-prefixed bytes,
  and nonfinite keys encoded as null. Native NaN and SQL NULL can consequently
  have indistinguishable encoded keys while remaining separate result rows.
- JSON returns owned `ByResult` with `data.headers`/`rows`, not a dataset or
  fabricated group diagnostics. No header-based type inference or rescan is added.
- COUNT(*) includes every row in each group, regardless of unrelated missing or
  nonfinite values. Its aggregate column is `Count`.

Grouped means are NOT ordinary finite-sample summarize:
- Only native AVG means are returned, not counts/SD/min/max or inference. Explicit
  numeric targets preserve order/duplicates; omitted targets use numeric columns
  in schema order excluding group columns. Headers are `mean_<variable>`.
- Each target's native mean excludes SQL NULL independently but retains NaN and
  positive/negative infinity. All-NULL means are SQL NULL; nonfinite results encode
  as JSON null under the existing scalar policy. Those nulls are not evidence of
  a finite-only sample or a zero count. Terminal output keeps native NaN/infinity
  explicit and SQL NULL as `NA`, with a native-aggregation heading.
- Ordinary `summarize` excludes nonfinite values. For example, a group containing
  x=[3, infinity] has a native grouped mean of infinity (JSON null), whereas the
  corresponding finite summary has N=1 and mean=3. No implicit reinterpretation
  or general finite-arithmetic overflow/stability guarantee is installed.
- Independent reference: `tests/fixtures/grouped-mean-reference.json`, generated
  by development-only Python standard-library `statistics.mean` over explicitly
  recorded per-variable non-NULL row indices, including tagged IEEE results.

State and limits:
Success/failure and serialization retain data/source/schema/order, labels, named
state and cached fit/sample. Unknown/no-active groups, nonnumeric/unknown targets
or no numeric default targets fail. Conditional/options-bearing grouped summaries,
other children such as tabulate/sort, and nested by remain unsupported in JSON.
Later effects/files stop on failure; completed exports survive later output failure
(MIG-0008). No general by runner, mutation, lazy/remote execution, new sample counts,
full descriptive statistics or whole-command parity is implied.

# collapse

Replace the active dataset with a grouped aggregate relation.

```text
collapse count x y, by(group)
collapse mean age cost, by(sex)
```

The eager runtime executes count, mean, sum, min and max with one statistic applied to every
requested variable and exactly one nonempty `by()` group list. Count accepts numeric or text
variables; the other statistics require numeric variables. These are native aggregate semantics,
not the ordinary finite-sample `summarize` contract. No new grammar or aggregate implementation
is installed by reporting. `by()` takes identifier tokens, not quoted names with spaces.

## Count/mean workflows and JSON

```text
use observations.parquet
collapse count x y, by(group)
head 10
save group-counts.parquet
```

CLI-v1 JSON admits **count and mean** and emits `CollapseResult` with owned `data.dataset`
metadata. Sum/min/max JSON remains unsupported. Terminal reporting provides the owned
row/column outcome (`Collapsed: N rows, K columns`) for executed collapse results; inspect values
with `head`/`describe`. No statistic, group-rule, affected-row or original-size fields are invented.
Dataset label/panel fields retain their existing null-placeholder policy.

Output columns are requested group keys followed by `count_<variable>` in request order.
Each count is `COUNT(variable)`: SQL NULL is excluded independently for each requested variable;
stored empty strings, NaN and infinity are non-NULL and counted. This differs from
`by group: count`, which uses `COUNT(*)` and includes every row. SQL-NULL keys form native groups.
Groups sort ascending with NULL last; wide integer keys retain native values. An empty grouped
input produces an empty relation with count-column schema, not an invented zero-count group.
Repeated/colliding output names may fail native staged schema validation; no deduplication or
alias repair is performed. Grouping equality and ordering are native, not cross-package guarantees.

For numeric group means and explicit aggregate-model fitting:

```text
use observations.parquet
collapse mean x y, by(group)
head 10
regress mean_y mean_x
predict fitted
save group-means.parquet
```

Mean output columns are requested group keys followed by DOUBLE `mean_<variable>` in
request order. Native AVG excludes SQL NULL independently per target but retains NaN and
infinities. All-NULL target groups have NULL means; this is **not** finite-sample `summarize`.
`head` JSON uses the existing scalar policy: nonfinite means also encode as null, so null
alone cannot distinguish all-NULL from nonfinite samples. No sample diagnostics or weights
are supplied. Numeric targets are required; empty input retains the aggregate schema with
zero rows. Grouping/order/collision rules above also apply to mean. No numerical overflow
or stability guarantee is added.

The source path is retained. Labels/attachments prune to surviving names, retaining dictionaries;
active named relations synchronize to the collapsed data. Cached fits, original parameter names
and estimation samples remain until explicit refit. If old predictors were removed, prediction
fails: fit a model using the aggregate columns before predicting. Group counts are not individual
observations or automatic statistical weights; neither are group means.

Validation and covered staging failures preserve prior data/state. Nested scripts stream the same
leaf results and retain path/line errors; interactive failures can recover. Noninteractive output
write/flush failure stops later commands/files, retaining already completed aggregation and exports.
There is no general native publication/named-sync/filesystem rollback promise. CSV can re-infer
schema/missingness and neither CSV nor Parquet archives session-local label dictionaries.

# Command and CLI-v1 result contracts

This reference preserves the detailed supported command, sample, state and JSON
contracts formerly in the root README. Source/tests remain authoritative. Paths to
source and fixtures below are relative to the repository root; use in-session
`help <command>` for syntax. This is not full-command JSON or parser-catalog support.

- [Session status](#session-status)
- [Summaries and percentiles](#summaries-and-percentiles)
- [Frequencies and conditional cohorts](#frequencies-and-conditional-cohorts)
- [Selected-cohort regression](#selected-cohort-regression)
- [Expressions and recoding](#expressions-and-recoding)
- [Grouped collapse](#grouped-collapse)
- [Assertions and fingerprints](#assertions-and-fingerprints)
- [Labels and category coding](#labels-and-category-coding)
- [SQL and named relations](#sql-and-named-relations)
- [Joins and appends](#joins-and-appends)
- [Key integrity](#key-integrity)
- [Conditional summaries](#conditional-summaries)
- [Conditional selection](#conditional-selection)
- [Reshape](#reshape)
- [Grouped counts and means](#grouped-counts-and-means)
- [Rename and column projection](#rename-and-column-projection)
- [Sorting](#sorting)
- [Persistence](#persistence)
- [VIF](#vif)
- [Linear inference fields and conventions](#linear-inference-fields-and-conventions)
- [Script exit categories](#script-exit-categories)
- [Mean comparisons](#mean-comparisons)
- [Missingness and finite samples](#missingness-and-finite-samples)
- [Column profiles](#column-profiles)

Use `--json -c "use data.csv" -c "summarize"` (or `--json -f analysis.td`)
for versioned line-delimited envelopes. Successful execution output currently
supports use, describe, count, codebook, summarize, missing, tabulate,
generate/replace, save/export,
head/tail, `estat vif` and the bounded
regress/predict/lincom/test workflow plus ttest; nested scripts stream leaf results.
Existing result shapes remain unchanged. `describe` uses `schema_version: 1`,
`result_type: "DescribeResult"` and `data.dataset`, reusing the load/prediction
metadata shape: source `path`, integer `row_count`, ordered `columns` with names
and native type strings, `execution_mode` and `lazy_engine`. It reports the
current schema (including generated prediction columns) without rescanning or
changing data/model state. The source path remains the load source, not a new
output destination. `label_metadata` and `panel_metadata` remain the existing
`null` placeholders, not a full session-label/panel snapshot.

## Session status

`status` reports current dataset/named-table metadata separately from the retained
linear fit's names, intercept convention and original sample counts. It is read-only,
works before loading without initializing DuckDB, and never clones model arrays.
CLI-v1 adds `StatusResult` with `data_initialized`, nullable `dataset`, nullable
`active_table_name`, sorted `named_tables` and nullable `linear_model`. Status is
not a current-data compatibility check or an automatic refit/stale flag. It does
not invent estimator, covariance, operation or materialization tracking. See `help status`.

## Summaries and percentiles

A summary uses `schema_version: 1`,
`result_type: "SummarizeResult"` and `data.rows`, with each row containing
`variable`, `count`, `mean`, `std_dev`, `minimum` and `maximum`.

Summary rows preserve requested order and duplicate variables; omitted variables
select numeric columns in schema order. `count` is the finite, non-NULL count and
`std_dev` is sample standard deviation (undefined below two observations).
Missing/undefined and non-finite values serialize as `null`, consistent with
CLI-v1 preview output. Extrema reuse preview scalar encoding: integers remain
JSON numbers, decimals are exact scaled strings. Consumers needing large-integer
precision should use a lossless JSON number parser. This is an additive result
surface, not full-command JSON support.

`summarize [varlist] [if <predicate>], detail` additionally reports percentiles
1/5/10/25/50/75/90/95/99 on that same per-variable finite sample. It uses type-7
linear interpolation after DOUBLE conversion, not an exact integer/decimal quantile
or a universal Stata percentile convention. Terminal output pairs each ordinary
summary with a percentile table. CLI-v1 adds `SummarizeDetailResult` with
`data.quantile_method: "linear_type_7"` and ordered rows containing `summary`
(the unchanged ordinary fields) and `percentiles` (`percentile`, nullable `value`).
Empty samples have undefined percentiles; singleton percentiles equal the one value.
Ordinary `SummarizeResult` is unchanged. No higher moments, weights, extreme-value
lists, configurable percentiles, grouped detail or other option combinations are
installed. Native finite-arithmetic limits remain. See `help summarize` and the
independent NumPy fixture `tests/fixtures/summary-detail-reference.json`.

## Frequencies and conditional cohorts

Bounded one/two-way `tabulate` uses `result_type: "TabulateResult"` with
`data.headers` and `data.rows`. Header/cell order, value labels, counts and
percentages are supplied by execution without recomputation. Missing categories
are excluded by default and included last with `missing`; their cells serialize
as `null`. Two-way row/column percentage headers remain explicit. This does not
install the broader parser-only tabulation/aggregation forms.

Read-only `count if expr` returns existing `CountResult.data.row_count` for all
TRUE predicate rows, including rows with unrelated NULL/nonfinite values. FALSE
and UNKNOWN are excluded; empty selections return zero. It preserves the active
dataset, labels, named relation and cached fit/sample. Use it to check cohort size
before `summarize` (per-variable finite samples) or `tabulate` (category exclusions).
Bare count is unchanged; variable lists/options, grouped conditional counts and
function/logical-expression breadth remain unsupported. See `help count`.

Read-only `tabulate r [c] if expr` and equivalent single-variable
`rows()`/`columns()` forms select only TRUE predicate rows. FALSE/NULL rows are
excluded; category SQL NULLs are then excluded unless `missing`. All percentage
denominators use that selected category sample, not original dataset totals or a
cached estimation sample. Arithmetic/comparisons/boolean columns/NULL reuse the
bounded predicate subset; functions/logical-expression breadth, multiple keys
per dimension and value aggregation remain unsupported. Active data, labels,
named relations and cached fits/samples are preserved. Selected counts and
percentages are checked against a development-only pandas reference with exact
counts and absolute percentage tolerance `1e-10` (no production Python dependency).

## Selected-cohort regression

`regress y xvars if expr` now fits the existing bounded linear estimators
(OLS/WLS/GLS with nonrobust/HC1/cluster covariance and intercept conventions)
on the TRUE predicate cohort without filtering active rows. FALSE/UNKNOWN rows
are excluded before finite outcome/predictor, cluster and weight validation, and
the estimation sample keeps original active-row indices, total rows and
exclusion counts alongside aligned weights/clusters. Active data, schema, order,
labels, named relations and source stay read-only; a successful fit replaces the
cached model, while a failed attempted regression clears it under existing
policy. `lincom`/`test`/`estat vif` consume the retained selected design and
`predict` remains current-data prediction with fitted coefficients. Predicates
reuse the bounded TRUE subset (no numeric/string truthiness, functions or
logical breadth); temporary filtering and automatic refits stay unsupported.
Coefficients, inference, VIF and predictions are checked against a
development-only statsmodels selected-row reference with exact sample/cluster/
weight membership (no production Python dependency).

## Expressions and recoding

`generate` and `replace` return additive `GenerateResult`/`ReplaceResult`
envelopes with `data.dataset` post-transform metadata. Source path, total row
count and ordered native schema are supplied, not changed-cell counts or overflow
diagnostics. Inspect values with `head`/`tail`. Numeric arithmetic generate and
numeric/string/NULL replace with boolean/NULL conditions use existing runtime
rules; function calls remain unsupported. Failed transforms preserve prior state.
Successful transforms retain the cached fit without refitting; replace removes
the target value-label attachment under existing rules. Metadata label/panel
fields remain null placeholders. See `help generate`/`help replace`.

`recode` returns additive `RecodeResult` with owned `data.dataset` metadata,
not rules, changed-cell counts or overflow diagnostics. Exactly one `generate()`
(new outputs in requested order) or `replace` target is required. Conditional
rules match original values in order (first match, no cascade); numeric ranges
are inclusive, `min`/`max` unbounded, and `else` is the final fallback. Unmatched
values remain unless an `else` rule supplies a value. `missing`/`nonmissing` mean
SQL NULL/non-NULL, not empty strings or general non-finiteness. Native output
schema/promotion is reported; text output uses text conversion. Replacing removes
selected value-label attachments, retains variable labels/definitions and cached
fits without refitting. Metadata label/panel fields remain null placeholders.
Parquet retains stored empty strings; CSV reload may turn empty fields into NULL.
See `help recode` for bounded eager syntax and failure limits.

## Grouped collapse

`collapse count x y, by(group)` and `collapse mean x y, by(group)` emit
`CollapseResult` with owned `data.dataset` metadata in CLI-v1 JSON and a readable
terminal row/column outcome. They replace active data with ordered group keys and
native BIGINT `count_x`/`count_y` or DOUBLE `mean_x`/`mean_y` columns. Counts exclude
SQL NULL independently per variable, unlike `by group: count` row totals; stored
empty/nonfinite values count as non-NULL. Means require numeric targets and use native
AVG with per-target SQL-NULL exclusion, retaining NaN/infinities, **not** finite-sample
`summarize`. Inspect values with `head`; its existing JSON policy maps both nonfinite
means and all-NULL means to null, without sample counts/diagnostics. NULL keys group
natively, NULL-last; empty input remains empty. Group labels survive, aggregate-source
labels prune, dictionaries remain and active named relations synchronize. Cached fits/
samples are not refitted: use aggregate columns for explicit refit/prediction before
persistence. Group aggregates are not individual observations or automatic model weights.
Repeated/colliding names can fail staged schema validation without repair. Sum/min/max
JSON remains unsupported; see `help collapse` for syntax and state/output limitations.

## Assertions and fingerprints

Read-only `assert` and `datasignature` now report in terminal/CLI-v1 JSON, enabling
script preconditions and current-data fingerprint records before analysis/persistence.
Successful `AssertResult` owns `data.checked`/`failed`; FALSE or SQL-NULL predicates
fail (empty data passes), and failure is an execution error, not a successful
assertion envelope. `DatasignatureResult` owns `data.algorithm`, `signature`,
`row_count` and `column_count`: the existing versioned SHA-256 protocol covers
public names/canonical types, row order and values, excluding labels/source paths.
Both inspect current data without changing named ownership, labels or cached models/
samples. No functions/logical breadth/Boolean literal keywords, `if`/options or `by:`
execution is added. Fingerprints are not file-byte checksums, authentication or
cross-package signatures; CSV re-inference may change them. See `help assert` and
`help datasignature` for bounded syntax and output-failure rules.

## Labels and category coding

Interactive `encode`/`decode` now report owned row/column outcomes, and `label`
responses show the action/message plus readable variable labels, attachments and
value-label sets. Names/text are quoted/control-safe. Mapping keys are explicitly
`integer`, `number` (exact source spelling) or `text`, retaining owned order and
wide integer precision without converting categories. Empty/absent dictionaries
are explicit. Filtered `label list names` still includes all variable labels.
These displays do not rescan data, change dictionaries/models or change JSON;
labels are not automatic factors or general metadata-aware formatting for other
commands. Nested terminal scripts stream the same leaf displays and can recover
from located errors; output failure stops later effects without rolling back
completed encoding, labels or files.

Bounded session-local `label variable`, `define`, `values`, `list` and `drop`
return `LabelResult` with `data.action`, `message` and owned `metadata` (null
when empty). Metadata contains ordered `variable_labels` and `attachments`
`[name, text-or-set-name]` pairs, and `value_sets` with `name` and ordered
`mappings` pairs `[key, display_text]`. Keys retain tagged `{kind, payload}`
forms: `integer` with an i64 JSON number (use lossless consumer parsing),
`number` with an exact numeric spelling string, or `text` with a string.
Names/attachments/sets are producer-normalized; mappings retain definition
order. `label list names` filters sets and matching attachments but still
includes all variable labels. Defining/replacing/clearing/dropping labels
changes metadata only, not data or cached models. Other dataset envelopes
retain their existing null label/panel placeholders. Labels are session-local:
a new `use` clears them, and CSV/Parquet save/export does not archive them.
`label save`/`label use` are not installed. See `help label`.

`encode`/`decode` report additive `EncodeResult`/`DecodeResult` with owned
`data.dataset` metadata, not code/mapping/count diagnostics. Encode maps sorted
non-NULL strings to 1..K, preserving SQL NULL and treating stored empty strings
as categories. Default/custom label dictionaries are inspected with `label list`;
custom names replace existing definitions, preserving other attachments to them.
Decode requires attached integer-key mappings and numeric input, uses native
SQL numeric equality (no explicit fractional truncation), and emits SQL NULL
for unknown/missing codes. Cross-type wide comparisons follow native coercion,
not a lossless DOUBLE/i64 equality guarantee.
Source columns/order/count/path and cached fits remain; generated variable labels
inherit their source label. Native schema is reported (small nonempty mappings
use INTEGER; empty/all-NULL encode uses BIGINT). Other dataset label/panel placeholders remain
null. Numeric codes alone do not install factor/dummy/ordinal model semantics;
choose a supported explicit design/transform. Dictionary persistence remains
uninstalled; CSV reload may turn empty strings into NULL. See `help encode`/`decode`.

## SQL and named relations

`sql SELECT/WITH ...` returns `TableResult` with owned `data.headers`/`rows`
using the existing scalar policy; headers can repeat, empty queries retain
headers, nonfinite numbers encode as null, and decimals stay exact strings.
Rows are not automatically truncated: specify SQL `LIMIT` when appropriate.
`sql ... into name` returns `SqlCreateResult`, and `use name` for a registered
named table returns `ActivateResult`, each with owned `data.table_name` and
`dataset`. Their source path is the named-table identifier, not the original
file. Publication/activation retains the cached fit/sample without refitting,
clears session labels, and reports native schema/count; transformations update
an active named table under existing rules. Dataset label/panel fields stay null.
Use explicit `ORDER BY` plus tie keys for reproducible query/cohort ordering.
Existing statement validation only checks the leading SELECT/WITH word; this
is not a SQL sandbox or general read-only guarantee. Native named publication
uses multiple steps, with no general atomic rollback guarantee. Ordinary
validation/binder/unsupported-cell failures are tested; do not infer whole-SQL,
composite-type, filesystem, lazy/remote or security support. See `help sql`.

## Joins and appends

Bounded `join lookup on keys [, how=inner|left suffix(...)]` returns
`JoinResult` with owned `data.dataset` metadata, not keys/matching/cardinality
or affected-row diagnostics. Join a registered named table after SQL setup and
primary activation. Inner is the default; left retains unmatched rows with
NULL right fields. SQL NULL keys do not match each other. Duplicate matches
multiply rows, grouped by prior active-row order and stored right-match order.
Right keys are omitted; nonkeys append with existing suffix/collision rules;
final native names/types/count/source are reported, without a wide cross-type
matching or case-collision naming guarantee. Labels/definitions/attachments for
retained left variables and the original cached fit/sample remain; explicitly
refit on enriched rows. Active named tables are updated under existing rules.
Dataset label/panel fields stay null placeholders; no cardinality validation,
right-label propagation, by/lazy/remote execution or general merge parity is
implied. See `help join`.

`append followup` returns `AppendResult` with owned `data.dataset` metadata,
not input paths or inferred added-row diagnostics. It requires the same column
names and equal canonical native types (including numeric width/decimal scale),
aligns right columns by name into left schema order, and preserves duplicates,
NULLs and left-then-stored-right row order. No schema union, automatic coercion,
sorting or deduplication is installed. Left labels/definitions/attachments and
the original cached fit/sample remain; explicitly refit combined data.
Append detaches active named-table ownership while retaining its source
identifier; registered prior named relations are unchanged, and `use name`
restores the preserved snapshot, not the stacked data. Metadata label/panel
fields stay null placeholders. Empty/self/repeated appends and ordinary
validation/type/file/output boundaries are checked; native publication faults
are not injected and no general atomicity guarantee is added. See `help append`.

## Key integrity

Read-only `duplicates [report] [varlist]` and `isid varlist [, missok]` now
report readable terminal diagnostics and additive CLI-v1 `DuplicatesResult` /
`IsidResult` fields, connecting key diagnosis/validation to lookup preparation.
Counters are owned current-data aggregates; `unique_groups` means all distinct
combinations (including duplicate/NULL groups), not only singletons. Default
isid rejects any SQL-NULL component; missok permits unique NULL-containing
combinations but never duplicates. Failures remain execution errors, with no
success-shaped or invented failure counters. Native grouping and stored-empty/
nonfinite distinctions are unchanged. Data/labels/named/model state is read-only;
no duplicate repair, key coercion, persistent cardinality rule or lazy loading
is installed. See `help duplicates` and `help isid` for exact fields/limits.

## Conditional summaries

Read-only `summarize [varlist] if <predicate>` (no options) now supports the
terminal and CLI-v1 `SummarizeResult` fields. It selects TRUE rows, then uses
per-variable finite/non-NULL samples for all aggregates; counts can differ,
not a fabricated global/model sample. Exact extrema, variable order/duplicates,
empty/singleton policy and native finite-overflow limits remain unchanged.
Predicates reuse the bounded keep-if rules, including explicit SQL-NULL and
native direct nonfinite versus safe-arithmetic handling. Data/labels/named state
and cached fits are untouched. One bare `detail` flag adds the percentile report
above. Functions/logical-expression breadth, other summary options and conditional
grouped summaries remain uninstalled. See `help summarize` and the
development-only NumPy/SciPy conditional-summary reference.

## Conditional selection

Conditional `keep if <predicate>` / `drop if <predicate>` now execute in the
CLI/REPL and CLI-v1 JSON, using existing KeepResult/DropResult dataset metadata.
Keep retains TRUE rows; drop removes TRUE rows and keeps FALSE/UNKNOWN, so they
partition input. Schema/types/source/relative order and labels survive; active
named tables synchronize. Cached inference/sample stay original until explicit
refit, while prediction uses current rows. Predicates reuse bounded comparisons,
arithmetic, Boolean columns, strings and SQL-NULL equality; no numeric truthiness,
functions, logical conjunction/disjunction or extra options. Direct NaN/infinity
comparisons remain native, not finite-sample filtering; safe arithmetic can turn
nonfinite/failed arithmetic into NULL/UNKNOWN. Stored empty strings are not NULL.
No predicate/affected/overflow counts or general native atomicity are implied;
column keep/drop remains unchanged. See `help keep`, `help drop`, MIG-0009.

## Reshape

Bounded `reshape long|wide` returns additive `ReshapeResult` with owned
`data.dataset` metadata, not inferred direction/stubs/changed-row fields.
Long requires complete `stub_suffix` columns and emits identifier/j/stub columns
in source-row then requested-stub/schema first-appearance suffix order; native
UNION promotion and NULL values are retained. Wide casts non-NULL j keys to text,
orders generated suffixes lexically, and uses native MAX for duplicate cells,
not uniqueness rejection or a general lossless pivot; NULL j rows contribute no
cell. Unrelated columns are omitted. Source identity is retained, named ownership
detaches (registered snapshots survive), and only surviving label names/attachments
remain; definitions and cached original fit/sample persist until explicit refit.
Missing old predictors fail rather than silently fitting a new model. Native
publication is not generally atomic; no panel/factor state is inferred. Empty
long works with valid stubs; empty/all-NULL-j wide fails. See `help reshape`.

## Grouped counts and means

`by groups: count` and `by groups: summarize [numeric_vars]` support CLI-v1 JSON
`ByResult` with owned `data.headers`/`rows`. Raw group keys preserve requested
order/duplicates; native group order is ascending NULLS LAST. SQL-NULL keys form
groups and stored empty strings remain distinct; no value-label substitution is
performed. Counts use COUNT(*) over all group rows. Means use existing native
AVG: SQL NULLs are excluded independently per target, but NaN/infinity remain.
Explicit targets preserve order/duplicates; defaults are numeric non-group columns
in schema order. This returns `mean_<variable>` columns only, not ordinary finite
`SummarizeResult` counts/SD/extrema or inferred sample diagnostics.

Empty input has headers and no groups. Existing scalar policy applies: exact
scaled decimals/wide integers and nonfinite cells encoded as null. Distinct native
NaN and SQL NULL keys can encode identically without merging groups; an all-NULL
mean and a native nonfinite mean can both encode null without implying the same
sample. Terminal output preserves native NaN/infinity explicitly. For x=[3,infinity],
the native mean is infinity (JSON null), while finite summarize reports N=1, mean=3.
A development-only standard-library `statistics.mean` fixture records per-variable
non-NULL input indices and tagged IEEE outputs; raw results are checked before JSON
encoding. No overflow/stability guarantee or numerical/backend change is added.

Inspection/reporting retains data/schema/source/order, labels, named state and
cached fits/samples. Conditional/options-bearing grouped summaries, other `by`
children and nested by remain unsupported. See `help by` for limits and reference
regeneration; this is not general by execution or full grouped descriptive statistics.

## Rename and column projection

Single-column `rename old new` returns additive `RenameResult` with the same
owned `data.dataset` post-operation metadata: native schema position/type, source
and total row count, not old/new-name or affected-cell fields. Values/NULLs and
row order are retained, including empty data and backtick-quoted names. Existing
targets (including the old name) and native case-insensitive collisions fail
without changing prior data/session state. Variable labels/value-label attachments
move to the new name; definitions remain. Cached fits retain their original names
and sample without refitting: post-estimation uses that fit, but prediction needs
its original predictor columns in current data. Normalize before fitting or
explicitly refit after renaming a predictor. Label/panel fields remain null
placeholders. See `help rename`.

Column-list `keep`, `drop` and `select` return additive `KeepResult`, `DropResult`
and `SelectResult` envelopes with the same `data.dataset` post-projection metadata.
`keep`/`select` use request order; `drop` preserves remaining schema order. Rows,
NULLs, native types and source are retained. Repeated keep/select names get native
unique aliases (e.g. `x`, `x_1`); repeated drop names remove a column once. Empty
data is supported; unknown columns or dropping every column fail without changes.
Labels/attachments for absent names are pruned, definitions and cached fits retained
without refitting. Prediction requires current predictors; post-estimation retains
the owned fit. Label/panel JSON fields remain null placeholders, not full snapshots.
Conditional row filtering also executes; see [conditional selection](#conditional-selection).
See `help keep`/`help drop`/`help select`.

## Sorting

`sort` (ascending) and `gsort` (per-key directions) return additive `SortResult`
and `GsortResult` envelopes with the same owned `data.dataset` metadata. Ordered
previews/predictions and exports use the actual current rows; result metadata
supplies no key list, changed-row count or cell values. NULLs are last for either
direction and full-key ties preserve their prior active-row order. Native schema,
source, row count, labels and the cached fit are retained without refitting;
retained sample indices still refer to the original fit, not reordered data.
Prediction evaluates current rows. Unknown keys fail without changes; unsupported
options/conditions fail before execution. Bounded local eager runtime only, not
lazy/remote or `by:` execution. Label/panel fields remain null placeholders.
See `help sort`/`help gsort`.

## Persistence

`save` (local Parquet) and `export` (local CSV) return additive `SaveResult` and
`ExportResult` envelopes with requested `data.path` and output-oriented
`data.dataset`, reusing the existing metadata shape. The reported dataset path
is the destination, not the unchanged active source shown by `describe`.
Prediction columns are included; data/model state and label/panel placeholder
policy are unchanged. Existing targets require `replace`; refused overwrite
preserves their bytes. Paths with spaces/commas can be quoted.

Persistence is **not atomic file publication**: a native failure may leave
partial output or alter a replaced file, and created parent directories may
remain. CSV is text interchange, not a general type/label archive. Result I/O
failure stops later commands but does not roll back files already written by
the completed command. Roundtrips and these failure boundaries are tested;
no general filesystem rollback or platform guarantee is claimed. See `help save`,
`help export` and MIG-0008.

## VIF

`estat vif` returns an additive `TableResult` envelope with `data.headers`
(`Variable`, `VIF`) and ordered `data.rows`, including the supplied `mean_vif`
aggregate when available. It preserves the owned table rather than identifying
results by header/label heuristics. A predictor may itself be named `mean_vif`;
rows and labels are not deduplicated. Unavailable/non-finite cells encode as
`null`; finite zero remains numeric. Generic table serialization alone does not admit
commands: SQL has separate JSON admission, while other `estat` forms remain unsupported.

VIF uses **unweighted auxiliary regressions on the retained parent-fit sample**,
with the parent's intercept convention, including after WLS/GLS. It does not
rescan/refit changed current data or modify the dataset/model. Parent fit counts,
estimator and sample metadata remain in the preceding `RegressionResult`.
A lone no-intercept predictor now has VIF/mean-VIF 1: its zero-column auxiliary model
predicts zero, giving SSR = uncentered TSS and R² = 0. The library's all-zero design has
undefined R²; invalid/nonfinite input and general regression rank/zero-parameter rules
still reject as before. Other auxiliary failures remain execution errors. This corrects
the former valid-fit diagnostic failure, so subsequent script/batch commands can proceed
(MIG-0011). JSON null does not distinguish unavailable/infinite cells. See `help estat`.

## Linear inference fields and conventions

Linear JSON results reuse the supplied statistics without recalculation:

- `RegressionResult`: `outcome`, ordered `predictors`, `estimator`, `covariance`,
  `observation_count`, `include_intercept`, `df_residual`, `r_squared`,
  `adjusted_r_squared`, `root_mse`, `sample` and ordered `coefficients`.
  `sample` contains `total_observations`/`dropped_observations`, or is null when
  unavailable. Coefficients contain `name`, `value`, `standard_error`, `statistic`
  and two-sided `p_value`. Full model matrices/row arrays are deliberately omitted.
- `PredictionResult`: `dataset`, using the existing load metadata shape; follow
  with head/tail to inspect values. Current prediction kinds are xb/residuals.
- `LincomResult`: `label`, `estimate`, `standard_error`, `statistic`, `p_value`,
  `ci_lower`, `ci_upper`, `ci_level`, `df_residual` (supplied Student-t inference).
- `TestResult`: ordered `constraints`, `statistic`, upper-tail `p_value`, `df`,
  `df_residual`, `is_chi2`; false means F with numerator/residual denominator DF.

The current least-squares engine uses residual DF N − P for coefficient/lincom
Student-t inference and the F denominator, including cluster mode (not G − 1).
`robust` means HC1 (N/(N − P)); `cluster(name)` uses the supplied cluster
covariance with G/(G − 1) × (N − 1)/(N − P) correction. Reporting changes none
of these conventions.

Finite statistics are JSON numbers; unavailable/non-finite values are null,
not zero. Samples and inference remain estimator/mode-specific; reference tests
cover named fixtures, not every accepted mode.

## Script exit categories

Script execution (`-f` or a positional script) exits 0 on success, 1 for command
execution/preflight or recursive inclusion failures, 2 for invalid script syntax,
directives, macros or control structure, and 3 for script-loading failures or a
missing data file. Nested failures retain the child source location and category.
Classification comes from the error source, not words such as “not found” or
“expected” in diagnostics. These deliberate corrections replace the old
message-based heuristic; JSON envelope fields and diagnostic text are unchanged.
Batch `-c` exit behavior is unchanged.

## Mean comparisons

Interactive mean comparisons now display the existing bounded `ttest` results:
`ttest x == 0` (one-sample), `ttest x == y` (paired), `ttest x, by(group)`
(pooled equal-variance), and `ttest x, by(group) welch` (unequal-variance;
`unequal` is an alias). `help ttest`/`?ttest` and `--json --help-topic ttest`
explain these forms, samples and limits; discovery links the packaged topic.
Output includes ordered sample summaries, fixed 95% Student-t intervals, the
mean/difference null, t statistic, reference DF (fractional for Welch), and
left-/two-sided/right-tail probabilities. Paired samples use joint complete
finite observations; independent samples exclude invalid tested values and
missing group keys. Independent group labels are rendered, then sorted as strings;
the contrast is the first label minus the second, not input-row or numeric key order.

Independent difference rows show `NA` for N because the owned legacy field is
not a sample count; real group counts and reference DF are shown separately.
Undefined statistics display `NA`, not invented zeros. Rendering does not change
calculations, samples, data, or the saved regression. Nested interactive `run`
streams these results; default quiet batch output is unchanged, with readable `--text` as a separate opt-in.

CLI-v1 JSON also supports all four bounded `ttest` forms. `TtestResult.data`
contains `varname1`, nullable `varname2`/`by_variable`, `is_paired`, `is_welch`,
`null_value`, `reference_distribution: "student_t"`, `ci_level: 95.0`,
`group1_stats`, nullable `group2_stats`/`combined_stats`, `difference_stats`,
`t_statistic`, `degrees_of_freedom`, and `p_left`/`p_two`/`p_right`.
Each summary contains `name`, `count`, `mean`, `standard_error`,
`standard_deviation`, `ci_lower`, and `ci_upper`. Counts are integer observation
counts, except independent difference `count` is null; its legacy DF/zero field
is deliberately not exposed as N. Sorted rendered-label order defines group1 minus
group2; paired tests use varname1 minus varname2. One-sample differences
report the tested mean, not the mean minus the null. Group intervals use their
own sample DF; the tested interval and probabilities use the supplied reference
DF, including fractional Welch DF. Missing summaries and non-finite statistics
serialize as null; finite zero remains a number. Rendering and serialization do
not change the estimation sample or saved model.

## Missingness and finite samples

`missing [variables]` now reports owned missingness rows in the shell and
CLI-v1 JSON. `MissingResult.data` contains `missing_definition: "sql_null"` and
ordered `rows` with `variable`, `data_type`, `total`, `missing`, `nonmissing`,
and `missing_percent`. Default order follows the loaded schema; explicit order
and repeated variables are preserved. Counts remain integer JSON numbers and
finite percentages are unrounded; non-finite percentages serialize as null.
An empty relation keeps the existing zero-count/0% convention.

Missingness means SQL NULL **after loading**. Stored NaN/infinity, empty strings
and sentinel codes are nonmissing; CSV loading can first turn empty fields into
NULL. `summarize`, `ttest`, and `regress` exclude non-finite numeric inputs
separately. Each summary variable uses one finite, non-NULL sample for count,
mean, sample SD (N-1), and extrema. All-invalid samples report count zero and
undefined statistics; singleton SD is undefined. Filtering preserves exact
integer/decimal extrema without casting the aggregate input. Very large finite
values can still overflow native aggregates; broader numerical stability is not
claimed. This deliberately corrects formerly poisoned/failed non-finite summaries
without changing result field shapes or finite-data behavior. Reporting
is read-only, retains types/data/model state, and streams in nested scripts.
Current runtime support is eager local CSV/Parquet through DuckDB, not broader
lazy/remote modes.

## Column profiles

`codebook [variables]` now reports owned column profiles in the shell and
CLI-v1 JSON. `CodebookResult.data` contains `missing_definition: "sql_null"`,
`example_limit: 3`, `examples_nonfinite_encoding: "null"`, and ordered `rows`
with `variable`, `data_type`, `nonmissing`, `missing`, `distinct`, and `examples`.
Schema/requested order and duplicates are preserved. Distinct counts use native
non-NULL semantics; examples are up to three non-NULL scan values and can repeat.
Empty/all-NULL profiles have empty example arrays.

JSON examples reuse preview scalar encoding: integer numbers, exact decimal
strings, base64-prefixed bytes, and null for non-finite floats. Thus an encoded
null example can be a selected non-NULL NaN/infinity, not SQL missingness. The
terminal displays those non-finite examples directly. Types/counts, scans, data,
and model state are unchanged; unsupported composite example conversions still
fail meaningfully rather than produce a false profile.

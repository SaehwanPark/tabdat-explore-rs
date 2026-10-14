# TabDat Explore Rust

A Rust-native, terminal-first statistical and exploratory data analysis (EDA)
application inspired by Stata-style commands. The goal is a **small, responsive,
trustworthy workflow**, not exhaustive Python or Stata compatibility.

**Current snapshot:** the Rust workspace supports bounded batch/script execution,
local eager DuckDB CSV/Parquet workflows, inspection/transforms, linear statistical
utilities, and selected JSON results. No-argument `tabdat` starts a minimal
persistent line-oriented session with readable core inspection results. The typed
MCP boundary is not an MCP server. Syntax support for other commands does **not**
imply executable statistical features.

## Try what works today

Install the pinned Rust toolchain with [rustup](https://rustup.rs/), then:

```sh
cargo run --locked -- --help
cargo run --locked -- --json --list-commands
cargo test --locked --workspace --all-targets
```

Start a session with `cargo run --locked`, then try:

```text
use data.csv
describe
summarize
head 5
count
exit
```

`use` loads local eager CSV/Parquet files. Enclose paths containing spaces or
commas in matching single or double quotes: `use "data files/survey,2026.csv"`
or `use 'data files/survey.parquet'`. Quotes are removed; backslashes remain
literal, with no shell escape or tilde expansion. The matching quote cannot
occur inside the path (use the other quote for a name containing an apostrophe).
Existing options follow the closing quote, e.g.
`use "semi, data.csv", delimiter(";") has_header(true)`. This does not enable
remote/lazy/DTA loading or add quoted paths to other commands.

Use `help` or `?` for core workflow guidance and `help summarize` or `?regress`
for packaged topic help (case-insensitive). Unknown/malformed help requests
report an error without losing session data. Reference topics may include
parser-only/future syntax; their presence is not an execution-support promise.
`lincom` and `test` have bounded topics describing the implemented inference.
In-session help is a shell feature; batch/scripts retain their execution behavior,
and JSON topic discovery remains available through `--json --help-topic`.

`exit`, `quit`, or EOF leaves the session. Command errors are reported and the
session remains usable. Prompts appear only when stdin and stdout are terminals;
piped input is also accepted, with no greeting or prompts. A clean exit returns
0 even after recoverable command errors; input/output failure returns 1. Use batch
`-c`/`-f` modes when command failures must determine a process exit code.

Noninteractive CLI results and diagnostics use fallible writes with explicit
flush boundaries. A write/flush failure (including a closed JSON output pipe)
returns 1 instead of panicking, stops later batch/script commands, and attempts
one best-effort `Error: CLI I/O failed:` diagnostic on stderr. If stderr itself
fails, exit 1 still applies. A failed argument-error diagnostic also returns 1;
normal argument/syntax/file exit categories remain unchanged when streams work.
A broken stream cannot be promised a JSON error envelope. Prior results/effects,
and the command whose result failed to write, are not rolled back. Plain batch/
script success remains silent; unused streams are not proactively probed. This
is not transactional or atomic file publication; see MIG-0008.

Readable terminal details currently cover load, describe, summarize, count,
head/tail, frequency tables, SQL table results and run metadata. Core
`generate`/`replace`, rename, sort/gsort and keep/drop/select report updated
row/column counts; save/export confirm the written path and row/column counts. Table output
preserves existing result order and labels; SQL-null cells display as `NA`, while
text is quoted to distinguish literal `"NA"` and escape control characters.
For the supported linear model subset, `regress y x` displays the estimator,
covariance, sample size/exclusions, residual degrees of freedom, fit measures and
ordered coefficient inference. Follow with `predict fitted`, `lincom x`, or
`test x` for prediction confirmation, linear-combination inference or a named
reference-distribution test. These displays use existing results, not a second
statistical calculation. Undefined inference displays as `NA`; regression
coefficient confidence intervals are not added by the renderer.
Other executed results explicitly report that terminal details are not yet
supported. Batch/script output contracts are unchanged. Interactive
`run analysis.td` streams successful leaf results, including nested scripts,
then reports the total executed-command count for that run. A script error keeps prior displayed
results and completed state changes and returns to the shell with its source
path/line; scripts are sequential, not atomic. A terminal output failure ends
the session and prevents subsequent data/file operations, but does not
roll back an already completed command.

Capable terminals provide basic line editing and Up/Down command recall, with
at most 500 session-local submitted commands (blank/comments and cancelled input
are excluded; consecutive duplicates are skipped). History is never saved to disk.
Ctrl-C while composing a line cancels that input and preserves data/model state;
Ctrl-D on an empty line exits. Piped input keeps the existing plain line behavior.
Unsupported/dumb terminals may fall back to plain input. Completion and a
multiline continuation prompt are not installed; submitted text is parsed as one command. Ctrl-C during command execution is **not** a
supported cancellation protocol and may terminate the process; in-flight file-write
safety is not promised.

Terminal editing uses Rustyline's safe API with optional/file-history features
disabled. Local Unix PTY verification is reproducible with
`python3 tools/check_terminal_pty.py target/debug/tabdat-explore-rs` after
`cargo build --locked`; this is not cross-platform release certification.

Use `-c` and `-f` for the currently supported command/script subsets. Consult
[SPEC.md](SPEC.md) and the tests before relying on a specific option; discovery
includes commands that parse but do not yet execute.

## Local responsiveness measurements

A development-only Rust example measures fresh-process startup/core JSON workflows
and separate session CSV load, summary, OLS and prediction execution. It validates
results, excludes one warmup per case, and records raw/minimum/median/maximum
milliseconds with supplied build context. It introduces no production dependency
or timing gate. Fresh-process is not OS-cold-cache timing, and debug smoke evidence
is not release-performance evidence. See [the measurement guide](docs/performance/core-workflow.md)
for matching-profile build/run commands, fixture semantics and limitations.
A [local matching release baseline](docs/performance/core-workflow.md#local-release-baseline)
retains raw samples and context for one macOS/aarch64 host; it is not a timing
threshold, OS-cold-cache measurement or release/platform certification.

## Bounded JSON results

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

A summary uses `schema_version: 1`,
`result_type: "SummarizeResult"` and `data.rows`, with each row containing
`variable`, `count`, `mean`, `std_dev`, `minimum` and `maximum`.

Summary rows preserve requested order and duplicate variables; omitted variables
select numeric columns in schema order. `count` is the non-null count and
`std_dev` is sample standard deviation (undefined below two observations).
Missing/undefined and non-finite values serialize as `null`, consistent with
CLI-v1 preview output. Extrema reuse preview scalar encoding: integers remain
JSON numbers, decimals are exact scaled strings. Consumers needing large-integer
precision should use a lossless JSON number parser. This is an additive result
surface, not full-command JSON support.

Bounded one/two-way `tabulate` uses `result_type: "TabulateResult"` with
`data.headers` and `data.rows`. Header/cell order, value labels, counts and
percentages are supplied by execution without recomputation. Missing categories
are excluded by default and included last with `missing`; their cells serialize
as `null`. Two-way row/column percentage headers remain explicit. This does not
install the broader parser-only tabulation/aggregation forms.

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

Only `by groups: count` is admitted as grouped CLI-v1 JSON execution. It returns
owned `ByResult` `data.headers`/`rows`: raw group columns in requested order,
then `Count`; native ascending NULL-last group order; COUNT(*) includes SQL
NULL-key groups, and stored empty strings are distinct. Counts reconcile to
all rows, with no finite-sample filtering or label substitution. Empty input
has headers and no groups. Existing scalar policy applies; distinct native
NaN and SQL NULL keys both encode as null, without merging their counted groups.
Counting/reporting is read-only for data, labels, named state and cached fits.
Other `by` children/nested by commands are not admitted. Grouped summaries are
existing native AVG means, not the ordinary finite summarize contract; their
JSON admission is deferred for separate statistical review. See `help by`.

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
Conditional `keep if`/`drop if` parses but does not execute and is not admitted in
JSON. See `help keep`/`help drop`/`help select`.

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

`estat vif` returns an additive `TableResult` envelope with `data.headers`
(`Variable`, `VIF`) and ordered `data.rows`, including the supplied `mean_vif`
aggregate when available. It preserves the owned table rather than identifying
results by header/label heuristics. A predictor may itself be named `mean_vif`;
rows and labels are not deduplicated. Unavailable/non-finite cells encode as
`null`; finite zero remains numeric. Generic table serialization does not enable
SQL or other `estat` commands in CLI-v1 JSON.

VIF uses **unweighted auxiliary regressions on the retained parent-fit sample**,
with the parent's intercept convention, including after WLS/GLS. It does not
rescan/refit changed current data or modify the dataset/model. Parent fit counts,
estimator and sample metadata remain in the preceding `RegressionResult`.
Auxiliary failure (including single-predictor/no-intercept) is an execution error;
JSON null does not distinguish unavailable/infinite cells. See `help estat`.

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

Script execution (`-f` or a positional script) exits 0 on success, 1 for command
execution/preflight or recursive inclusion failures, 2 for invalid script syntax,
directives, macros or control structure, and 3 for script-loading failures or a
missing data file. Nested failures retain the child source location and category.
Classification comes from the error source, not words such as “not found” or
“expected” in diagnostics. These deliberate corrections replace the old
message-based heuristic; JSON envelope fields and diagnostic text are unchanged.
Batch `-c` exit behavior is unchanged.

Interactive mean comparisons now display the existing bounded `ttest` results:
`ttest x == 0` (one-sample), `ttest x == y` (paired), `ttest x, by(group)`
(pooled equal-variance), and `ttest x, by(group) welch` (unequal-variance).
Output includes ordered sample summaries, fixed 95% Student-t intervals, the
mean/difference null, t statistic, reference DF (fractional for Welch), and
left-/two-sided/right-tail probabilities. Paired samples use joint complete
finite observations; independent samples exclude invalid tested values and
missing group keys. The contrast follows the supplied group order.

Independent difference rows show `NA` for N because the owned legacy field is
not a sample count; real group counts and reference DF are shown separately.
Undefined statistics display `NA`, not invented zeros. Rendering does not change
calculations, samples, data, or the saved regression. Nested interactive `run`
streams these results; non-JSON batch output is unchanged.

CLI-v1 JSON also supports all four bounded `ttest` forms. `TtestResult.data`
contains `varname1`, nullable `varname2`/`by_variable`, `is_paired`, `is_welch`,
`null_value`, `reference_distribution: "student_t"`, `ci_level: 95.0`,
`group1_stats`, nullable `group2_stats`/`combined_stats`, `difference_stats`,
`t_statistic`, `degrees_of_freedom`, and `p_left`/`p_two`/`p_right`.
Each summary contains `name`, `count`, `mean`, `standard_error`,
`standard_deviation`, `ci_lower`, and `ci_upper`. Counts are integer observation
counts, except independent difference `count` is null; its legacy DF/zero field
is deliberately not exposed as N. The supplied order defines group1 minus
group2 (or varname1 minus varname2 for paired tests). One-sample differences
report the tested mean, not the mean minus the null. Group intervals use their
own sample DF; the tested interval and probabilities use the supplied reference
DF, including fractional Welch DF. Missing summaries and non-finite statistics
serialize as null; finite zero remains a number. Rendering and serialization do
not change the estimation sample or saved model.

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

## Development direction

The immediate priority is a usable end-to-end EDA/linear-model workflow:
interactive shell, readable results, supported data operations, and independent
statistical validation. See the [outcome roadmap](docs/TABDAT_RUST_PORT_ROADMAP.md).

- [Current capabilities and guarantees](SPEC.md)
- [Short contributor workflow](CONTRIBUTING.md)
- [Agent instructions](AGENTS.md)
- [Architecture](ARCHITECTURE.md)
- [Python migration and compatibility policy](docs/migration/README.md)

The [original proposal](docs/TABDAT_RUST_PORT_PROJECT_PROPOSAL.md),
[accepted ADRs](docs/adr/), [migration records](docs/migration/) and old
repository history retain detailed rationale; they are **historical/reference
material**, not instructions to implement every proposed capability now.

Core runtime should require neither Python nor R. Native distribution and
redistribution/licensing guarantees are not established merely by successful
local builds. GitHub Actions are disabled; developers run relevant checks locally.

# CLI, scripts and interactive editing

This guide covers supported interaction and recovery. For command/data semantics and
structured fields, see the [command and JSON reference](../reference/command-results.md).

- [Help and session recovery](#help-and-session-recovery)
- [Output failures and completed effects](#output-failures-and-completed-effects)
- [Batch and script output](#batch-and-script-output)
- [Script paths and nested runs](#script-paths-and-nested-runs)
- [JSON batch runs](#json-batch-runs)
- [Editing, history and completion](#editing-history-and-completion)
- [Discovery and uninstalled interfaces](#discovery-and-uninstalled-interfaces)

`use` loads local eager CSV/Parquet files. Enclose paths containing spaces or
commas in matching single or double quotes: `use "data files/survey,2026.csv"`
or `use 'data files/survey.parquet'`. Quotes are removed; backslashes remain
literal, with no shell escape or tilde expansion. The matching quote cannot
occur inside the path (use the other quote for a name containing an apostrophe).
Existing options follow the closing quote, e.g.
`use "semi, data.csv", delimiter(";") has_header(true)`. This does not enable
remote/lazy/DTA loading.

## Help and session recovery

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

## Output failures and completed effects

Noninteractive CLI results and diagnostics use fallible writes with explicit
flush boundaries. A write/flush failure (including a closed JSON output pipe)
returns 1 instead of panicking, stops later batch/script commands, and attempts
one best-effort `Error: CLI I/O failed:` diagnostic on stderr. If stderr itself
fails, exit 1 still applies. A failed argument-error diagnostic also returns 1;
normal argument/syntax/file exit categories remain unchanged when streams work.
A broken stream cannot be promised a JSON error envelope. Prior results/effects,
and the command whose result failed to write, are not rolled back. Default batch/
script success remains silent; unused streams are not proactively probed. This
is not transactional or atomic file publication; see MIG-0008.

## Batch and script output

For opt-in readable one-shot output, use:

```sh
tabdat-explore-rs --text -c "use data.csv" -c "summarize" -c "regress y x"
tabdat-explore-rs --text -f analysis.td
```

`--text` shares the owned terminal renderer without additional scans or changed
calculation/state semantics. It requires explicit batch/script execution and cannot
be combined with `--json`, discovery or explain. Quiet defaults and CLI-v1 JSON
remain unchanged. Text file/positional scripts stream leaf results without a final
container summary; text `-c "run script.td"` streams leaves and the existing run
summary, then continues the shared batch session. File/script mode keeps located
execution/syntax/file exits 1/2/3; a failed `-c run` remains a batch execution error
(exit 1). Write/flush failures return 1, stopping later commands/files while keeping
completed effects and partial prior output. Text is for humans, not a stable machine
schema; use `--json` for supported structured results. Text admission does not imply
that every catalogued command executes or expand bounded JSON admission.

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
Every current owned result variant has an explicit terminal display. Interactive
`run analysis.td` streams successful leaf results, including nested scripts,
then reports the total executed-command count for that run. A script error keeps prior displayed
results and completed state changes and returns to the shell with its source
path/line; scripts are sequential, not atomic. A terminal output failure ends
the session and prevents subsequent data/file operations, but does not
roll back an already completed command.

## Script paths and nested runs

`run "analysis files/model.td"` and `run 'analysis files/model.td'` accept
one nonempty literal path. Matching quotes are removed; internal whitespace,
commas, Unicode and backslashes are preserved, with no shell escapes or tilde
expansion. Use the other quote if the filename contains an apostrophe or double
quote; the matching delimiter cannot occur inside the path. Extra arguments and
options are rejected. Unquoted paths keep their existing one-word literal syntax.
Nested `run` paths resolve relative to the containing script; data/output paths
still resolve from the process working directory. Existing script macro expansion
happens before command parsing. Quoted `run` no longer targets a filename that
includes its surrounding quote characters. JSON leaf streaming remains supported
via `--json -f`/positional scripts and `--json -c "run ..."`.

## JSON batch runs

JSON batch `run` streams the same supported leaf envelopes as file/script mode,
without a `RunResult` wrapper or run summary. Setup before the run and later
`-c` commands share its data/model state. Empty/directive-only runs emit nothing;
`exit` inside a script ends that script, not the surrounding batch. Unsupported
leaf commands are rejected before execution. A failed run emits one located
`ScriptError` envelope after prior leaves, stops later batch commands, and exits
1; direct command-parse errors still exit 2, and `-f`/positional scripts retain
source-classified exits 1/2/3. Output failure stops later effects but cannot undo
completed commands/files or guarantee an error envelope on a broken stream.

## Editing, history and completion

Capable terminals provide basic line editing and Up/Down command recall, with
at most 500 session-local submitted commands (blank/comments and cancelled input
are excluded; consecutive duplicates are skipped). History is never saved to disk.
Ctrl-C while composing a line cancels that input and preserves data/model state;
Ctrl-D on an empty line exits. Piped input keeps the existing plain line behavior.
Unsupported/dumb terminals may fall back to plain input. Tab at the end of a
command-name prefix completes installed core commands: for example, `summ<Tab>`
becomes `summarize`. Ambiguous prefixes extend to their common prefix; another Tab
lists alternatives. `help summ<Tab>` and `?summ<Tab>` complete packaged reference
topics, which can include parser-only/future syntax. Command/topic matching is
case-insensitive; completion preserves leading whitespace and arguments after the
cursor, and never executes a command.

Tab also completes the first local-path argument of `use` and `run`, using literal,
case-sensitive prefixes relative to the process working directory or an absolute path.
For spaces, commas or quotes, type a compatible opening single/double quote first:
`use 'anal<Tab>` can navigate into `analysis files/`, then complete a filename. Files
close an open quote; directories append a separator and leave it open for navigation.
Existing closing quotes and `use` option suffixes are preserved. Hidden names require
a dot prefix. Files/directories (including resolvable symlinks) are listed without
extension or loadability validation; unavailable entries, control/non-UTF-8 names and
names incompatible with the current quote are skipped. No shell escapes, home/environment
or glob expansion or output-path completion is provided. Completion does not
replace the rest of a filename after the cursor. Piped input remains unchanged.

At a bare `use` word end, registered table identifiers also join the sorted,
deduplicated local-path candidates: `use cohort_p<Tab>` can complete `cohort_probe`.
Bare `append` and `join` targets also complete registered names, but offer no
filesystem candidates; a typed suffix after the target (for example,
`join cohort_p<Tab> on x, how=left`) is preserved. Names are literal/case-sensitive
and refresh before each prompt, including partial scripts and failures, from registry
keys without backend scans or model-array copies. For `use`, an exact registered name
retains native precedence over a same-name file/directory; explicit `./`/directory
paths, quotes and option-bearing suffixes keep filesystem-only completion. This does
not validate activation/loadability or add named completion to `run` or piped input.

Basic variable lists also complete current column names for `summarize`, `codebook`,
`missing`, `tabulate`, `regress`, `keep`, `drop`, `select` and `sort`. For example,
`summarize age_s<Tab>` can complete `age_squared`. Names use literal case-sensitive
prefixes, sorted and deduplicated; bare letter/underscore identifiers (including Unicode)
are offered, excluding the reserved `if` keyword. The owned snapshot refreshes from current
schema before every prompt, including after failed commands, partial scripts and named-table
switches; it does not scan data or use cached-model names. Suggestions are not numeric/type,
formula or estimator validation, and repeated variables are not suppressed. For a header with
spaces, punctuation or the reserved `if` name, type an opening backtick before its prefix:
``summarize `annual i<Tab>`` completes to ``summarize `annual income` ``. Backtick completion
closes the name unless a closing backtick already follows the cursor; mixed bare/backtick
lists and existing quote/option suffixes are preserved. Empty, control-bearing and embedded-
backtick names are not offered; this completion requires backticks rather than single/double quotes.
Factors, signed keys, expressions, predicates/options and new output names are not completed.
Completion acts at a variable prefix end, not inside a name. Piped input never completes tabs.
A multiline continuation prompt is not installed; submitted text is parsed as one command. Ctrl-C during command execution is **not** a
supported cancellation protocol and may terminate the process; in-flight file-write
safety is not promised.

Terminal editing uses Rustyline's safe API with optional/file-history features
disabled. Local Unix PTY verification is reproducible with
`python3 tools/check_terminal_pty.py target/debug/tabdat-explore-rs` after
`cargo build --locked`; this is not cross-platform release certification.

## Discovery and uninstalled interfaces

Startup configuration-file loading and an MCP transport/server are not installed:
`--config` and `--mcp` are rejected, not applied or silently ignored. The typed
transport-independent MCP library boundary does not start a stdio server.

Use `-c` and `-f` for the currently supported command/script subsets. Consult
[SPEC.md](../../SPEC.md) and the tests before relying on a specific option; discovery
includes commands that parse but do not yet execute. `--json --describe-command`
now shows `[if expr]` for `count`, `summarize`, `tabulate` and `regress`, plus
one/two-way and `rows()`/`columns()` frequency forms. Regression predictor
requiredness matches the parser: at least one predictor is required. Tabulation
lists installed frequency options alongside retained parser-reference `values`/
`stat` descriptors; those aggregation forms remain uninstalled. Prediction discovery
uses the accepted `pr` reference spelling, not the rejected `probabilities` option;
only xb/residuals execute. `help predict` teaches current-data prediction, retained-fit
and fresh-target rules, inspection/export and deferred forms. Follow the linked
help for supported sample/option limits; `--json --explain -c "..."` checks syntax
only (`execution: "not_run"`), not data validity or execution support.

Every currently owned execution-result variant now has an explicit terminal
representation. Named SQL creation/activation shows the quoted supplied name and
row/column outcome; join/append/reshape/recode show owned post-operation dimensions.
These are metadata outcomes, not invented matched/changed-row or shape-rule reports:
inspect values with `head`/`describe`. Raw `by` tables display supplied keys/aggregates,
including native grouped means with an explicit native-aggregation heading. Those
means use existing SQL-NULL/nonfinite semantics, **not** ordinary finite-sample
`summarize`; bounded native grouped means now share CLI-v1 JSON `ByResult` tables.
Rendering does not recompute, mutate state or broaden parser/backend support. Nested terminal workflows and
output-error guards retain existing sequential effects. This is not whole-command
parity, full model-array reporting or a transactional publication guarantee.

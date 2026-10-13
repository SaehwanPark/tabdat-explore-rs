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

## Bounded JSON results

Use `--json -c "use data.csv" -c "summarize"` (or `--json -f analysis.td`)
for versioned line-delimited envelopes. Successful execution output currently
supports use, count, summarize, tabulate, head and tail; nested scripts stream leaf results.
Existing result shapes remain unchanged. A summary uses `schema_version: 1`,
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

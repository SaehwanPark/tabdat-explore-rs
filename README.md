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
Other executed results explicitly report that terminal details are not yet supported. Batch/script output contracts
are unchanged. There is no history, completion, or multiline interactive input.
Ctrl-C retains default OS process termination; recoverable cancellation and
in-flight file-write safety are not promised.

Use `-c` and `-f` for the currently supported command/script subsets. Consult
[SPEC.md](SPEC.md) and the tests before relying on a specific option; discovery
includes commands that parse but do not yet execute.

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

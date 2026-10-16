# TabDat Explore Rust

A Rust-native, terminal-first statistical and exploratory data analysis (EDA)
application with Stata-inspired commands. The aim is a **small, responsive,
trustworthy workflow**, not exhaustive Python or Stata compatibility.

## What works today

- Persistent interactive sessions, readable batch output and sequential scripts.
- Local eager CSV/Parquet loading through DuckDB, inspection, summaries,
  frequency tables and bounded data transformations.
- Linear regression, prediction, hypothesis tests, VIF and mean comparisons.
- Versioned JSON results for the supported command subset.

**Work in progress:** command discovery and parsing include forms that do not
execute. Remote/lazy/DTA loading, plot execution and an MCP server are not
installed. Consult [SPEC.md](SPEC.md) for current capabilities and limits.

## Build and try

Use [rustup](https://rustup.rs/) with the pinned `rust-toolchain.toml`:

```sh
cargo build --locked
./target/debug/tabdat-explore-rs --help
./target/debug/tabdat-explore-rs
```

Save this small example as `data.csv` in your working directory:

```csv
x,y
1,2
2,5
3,5
4,9
```

Then, in the interactive session:

```text
use data.csv
describe
summarize
regress y x
predict fitted
head 4
exit
```

Use `help` or `?` for workflow guidance and `help regress` for a command topic.
Quote paths containing spaces: `use "data files/survey.csv"`. Supported variable
lists accept backtick-quoted names. Capable terminals provide Tab completion,
editing and session-local history; Ctrl-C cancels pending input, not an executing
command. Errors normally return to the interactive prompt; use batch/script
modes when command failures must determine the process exit code.

## Batch, scripts and JSON

```sh
# Human-readable results; repeat -c to share one session.
./target/debug/tabdat-explore-rs --text -c "use data.csv" -c "summarize y"

# Run a script containing one command per line.
./target/debug/tabdat-explore-rs --text -f analysis.td

# Machine-readable, versioned line-delimited results for supported commands.
./target/debug/tabdat-explore-rs --json -c "use data.csv" -c "count"
```

Batch/script success is quiet unless `--text` or `--json` is selected. Text is a
human display, not a machine contract. Scripts and file writes are sequential,
not transactions: later failures do not undo completed effects.

## Documentation

| Need | Start here |
| --- | --- |
| Supported features and guarantees | [Capability map](SPEC.md) |
| CLI modes, paths, scripts, editing and recovery | [User guide](docs/guides/cli-and-editor.md) |
| Samples, missingness, state and structured fields | [Command and JSON contracts](docs/reference/command-results.md) |
| Current priorities | [Outcome roadmap](docs/TABDAT_RUST_PORT_ROADMAP.md) |
| Build checks and contribution workflow | [Contributing](CONTRIBUTING.md) |
| Module ownership and design boundaries | [Architecture](ARCHITECTURE.md) |
| All guides, evidence and historical material | [Documentation index](docs/README.md) |

## Repository layout

- `src/`, `crates/` — binary entry point and production Rust workspace.
- `tests/` — integration contracts and small reference fixtures.
- `examples/`, `tools/` — development measurements, reference capture and PTY checks.
- `fuzz/` — isolated parser-fuzz workspace.
- `spikes/` — isolated native feasibility prototypes, not production integrations.
- `_workspace/` — historical task evidence, not the current development checklist.

Core runtime needs no Python or R; some development reference/PTY tools use Python.
Local performance observations are documented in the
[measurement guide](docs/performance/core-workflow.md), not platform guarantees.
GitHub Actions are disabled; contributors run relevant checks locally. Native
packaging, redistribution and platform support require separate evidence.

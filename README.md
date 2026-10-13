# TabDat Explore Rust

A Rust-native, terminal-first statistical and exploratory data analysis (EDA)
application inspired by Stata-style commands. The goal is a **small, responsive,
trustworthy workflow**, not exhaustive Python or Stata compatibility.

**Current snapshot:** the Rust workspace supports bounded batch/script execution,
local eager DuckDB CSV/Parquet workflows, inspection/transforms, linear statistical
utilities, and selected JSON results. No-argument `tabdat` still prints
`Hello, world!`: the interactive REPL is not yet implemented. The typed MCP
boundary is not an MCP server. Syntax support for other commands does **not**
imply executable statistical features.

## Try what works today

Install the pinned Rust toolchain with [rustup](https://rustup.rs/), then:

```sh
cargo run --locked -- --help
cargo run --locked -- --json --list-commands
cargo test --locked --workspace --all-targets
```

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

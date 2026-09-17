# TabDat Explore Rust

Rust-native successor to TabDat's terminal-first statistical/EDA environment.
**This repository is a scaffold, not a usable TabDat CLI.** Running the root binary
currently prints `Hello, world!`. The workspace has begun a safe language layer with
a syntax-only parser for a few control, status, inspection, diagnostic, and
configuration commands (`help`, `status`, `exit`, `describe`, `doctor`, `set`,
`datasignature`, `use`, `codebook`, `missing`, `duplicates`, `summarize`, `isid`,
`run`, `count`, `head`, and `tail`). The `missing [varlist]`, `duplicates [report] [varlist]`,
direct `summarize [varlist]`, and direct `isid [varlist] [, missok]` forms are
syntax-only; no data engine or statistical
backend is installed as a supported product surface. Merged PR #22 (`26dba2b`) accepted a bounded
`tabdat-runtime` evaluation that can load an existing local Parquet file eagerly
through its library API; it is not wired into the root binary, does not provide a
general data engine, and is not a supported CLI or statistical backend. The
accepted runtime boundary is documented as an evaluation result, not broad
`use` or DuckDB product support.

PR #23 adds the owned syntax-only `isid` command. Key uniqueness checks,
missing-key handling, active-dataset access, and all execution/reporting remain
deferred to the roadmap's data-runtime work.

Merged PR #24 (`77f4754`) adds the owned syntax-only `run <script-path>` form.
Script loading, line-oriented execution, nested/recursive scripts, and
file/line diagnostics remain deferred to the roadmap's script-engine work.

## Development

Install [rustup](https://rustup.rs/), then:

```sh
cargo run --locked
cargo test --locked --workspace --all-targets
```

`rust-toolchain.toml` pins the compiler, rustfmt, and Clippy used locally and in CI.
The core scaffold and language crate require neither Python nor R. See
[CONTRIBUTING.md](CONTRIBUTING.md) for all checks and the development workflow.

## Project state

- [Implemented state and active scope](SPEC.md)
- [Current and proposed architecture](ARCHITECTURE.md)
- [Proposed project design](docs/TABDAT_RUST_PORT_PROJECT_PROPOSAL.md)
- [Roadmap and acceptance gates](docs/TABDAT_RUST_PORT_ROADMAP.md)
- [Agent rules](AGENTS.md)

The initial Python migration baseline is pinned in
[docs/migration/](docs/migration/). Build checks and the recorded parser/script
oracle run do not establish Rust behavioral or statistical parity. Licensing and
redistribution decisions must be resolved before publication; the proposal's AGPL
intent is not a release license.

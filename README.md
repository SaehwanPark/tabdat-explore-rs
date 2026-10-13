# TabDat Explore Rust

Rust-native successor to TabDat's terminal-first statistical/EDA environment.
**Phase 2 core workspace is implemented; full TabDat product parity is not.**
The binary has bounded batch/script execution, help/discovery and JSON interfaces.
No-argument execution still prints `Hello, world!`; there is no interactive REPL
or MCP server.

## Workspace

- **Domain:** owned typed commands/results/errors, metadata, explicit model-family
  state, effects/capability prerequisites and internal JSON v1 migration contracts.
- **Language:** existing command/expression/tokenizer/script parser contracts.
- **Execution / data / I/O:** existing bounded eager DuckDB CSV/Parquet,
  inspection/transforms/labels/named tables, scripts and local persistence.
- **Statistics:** pure safe Rust linear regression, sample/covariance/inference,
  prediction/hypothesis/VIF and t-test utilities.
- **Reporting / shell:** shared help/catalog/CLI JSON and the existing batch CLI.
- **MCP:** typed transport-independent tool boundary sharing these contracts,
  not an implemented transport/server.
- **Runtime:** compatibility facade for existing library callers and tests.

All eleven first-party workspace packages forbid unsafe code. Domain has no
runtime backend dependency; native DuckDB values/connections stay inside data.
The language parses more command forms than execution supports. The
[implementation state](SPEC.md), [architecture](ARCHITECTURE.md), tests and
individual roadmap entries establish each supported variant—not merely a
compiling enum or a merged PR. Remote/lazy/DTA runtime, broad estimator parity,
native libgretl/ReadStat integration, REPL/MCP transport and shipping remain
later-phase work. Phase 1 backend spikes are feasibility evidence, not installed
production capabilities or redistribution approval.

## Development

Install [rustup](https://rustup.rs/), then:

```sh
cargo run --locked -- --help
cargo run --locked -- --json --list-commands
cargo test --locked --workspace --all-targets
```

`rust-toolchain.toml` pins the compiler, rustfmt and Clippy. Core runtime requires
neither Python nor R; they remain external validation tools. GitHub Actions are
disabled, so required checks run locally. See [CONTRIBUTING.md](CONTRIBUTING.md)
for all checks and the development workflow.

## Project state

- [Implemented state and evidence](SPEC.md)
- [Current and proposed architecture](ARCHITECTURE.md)
- [Project proposal](docs/TABDAT_RUST_PORT_PROJECT_PROPOSAL.md)
- [Roadmap and acceptance gates](docs/TABDAT_RUST_PORT_ROADMAP.md)
- [Phase 2 recovered contract / review](docs/migration/phase2-core-contracts.md)
- [Agent rules](AGENTS.md)

The initial Python migration baseline is pinned in [docs/migration/](docs/migration/).
Cargo checks do not establish whole-command/statistical parity. Licensing,
artifact-specific redistribution and corresponding-source obligations must be
resolved before publication; the proposal's AGPL intent is not a release license.

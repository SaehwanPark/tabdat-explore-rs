# TabDat Explore Rust

Rust-native successor to TabDat's terminal-first statistical/EDA environment.
**This repository is a scaffold, not a usable TabDat CLI.** Running it currently
prints `Hello, world!`; no parser, data engine, or statistical backend is installed.

## Development

Install [rustup](https://rustup.rs/), then:

```sh
cargo run --locked
cargo test --locked --workspace --all-targets
```

`rust-toolchain.toml` pins the compiler, rustfmt, and Clippy used locally and in CI.
The core scaffold requires neither Python nor R. See
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

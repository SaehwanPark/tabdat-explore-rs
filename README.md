# TabDat Explore Rust

Rust-native successor to TabDat's terminal-first statistical/EDA environment.
**This repository is a scaffold, not a usable TabDat CLI.** Running the root binary
currently prints `Hello, world!`. The workspace has begun a safe language layer with
a syntax-only parser for a few control, status, inspection, diagnostic, and
configuration commands (`help`, `status`, `exit`, `describe`, `doctor`, `set`,
`datasignature`, `use`, `codebook`, `missing`, `duplicates`, `summarize`, `isid`,
`select`, `rename`, `run`, `sort`, `gsort`, `save`, `export`, `count`, `head`, and `tail`). The `missing [varlist]`, `duplicates [report] [varlist]`,
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

Merged PR #25 (`89f6c14`) adds the owned syntax-only `rename <old> <new>` form.
Schema lookup, collision checks, relation mutation, and execution remain
deferred.

Merged PR #26 (`5735b43`) adds the owned syntax-only `select <varlist>` form.
Active-schema lookup, wildcard/range expansion, relation mutation, and
execution remain deferred.

Merged PR #27 (`7cf21ae`) adds the owned syntax-only `sort <varlist>` form.
Active-schema lookup, stable/null/descending/expression sorting, relation
mutation, and execution remain deferred.

Merged PR #28 (`fd94133`) adds the owned syntax-only `gsort [+|-]varlist` form.
Direction metadata is owned in typed keys, while active-schema lookup, ordering,
relation mutation, and execution remain deferred.

Merged PR #29 (`8b16223`) adds the owned syntax-only `save <path> [, replace]`
and `export <path> [, replace]` forms. PR #70 (`9bbf804`) adds a separate
library-only eager runtime `save` path for active local-Parquet relations,
including target validation, Parquet writing, and state-preserving round trips.
PR #72 (`5cb5b34`) adds the corresponding bounded CSV-only eager `export` path
with target validation, header-bearing output, and state-preserving round trips.
Parquet aliasing through `export`, broader output formats, and
interface/persistence surfaces remain deferred.

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

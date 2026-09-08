# Contributing

## Toolchain and formatting

Use rustup; entering this repository selects the exact version in
`rust-toolchain.toml` and installs its rustfmt/Clippy components when needed.
The pin is the supported development/CI toolchain, not a promised MSRV. Upgrade it
in a dedicated PR that runs all checks locally and in CI. Commit `Cargo.lock`
for this binary and use `--locked` so builds cannot silently change resolution.

Use two spaces, no hard tabs. EditorConfig covers editable text and rustfmt covers
Rust. Markdown keeps intentional trailing spaces for hard line breaks. Generated
files and formats with mandatory indentation retain their required syntax.

## Development loop

Read [AGENTS.md](AGENTS.md), [SPEC.md](SPEC.md), and the relevant
[roadmap](docs/TABDAT_RUST_PORT_ROADMAP.md) gate. Work on a branch from updated
`main`; one bounded slice corresponds to one PR. Record acceptance and exclusions
before implementation, write behavior tests first and observe their intended
failure, then implement and reconcile documentation with evidence. Characterization
tests for unchanged behavior may pass immediately; record that distinction.
Prefer pure domain functions, explicit state and errors, and effects at boundaries.

## Required checks

Run from the repository root:

```sh
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo test --locked --workspace --all-targets
cargo clippy --locked --workspace --all-targets -- -D warnings
git diff --check
```

For the dependency and unsafe-code policy slice, install the pinned tools and run:

```sh
cargo install cargo-deny --version 0.20.2 --locked
cargo install cargo-audit --version 0.22.2 --locked
cargo install cargo-geiger --version 0.13.0 --locked
cargo deny check
cargo audit -D warnings
cargo geiger --all-dependencies --all-targets --locked
```

`deny.toml` rejects unknown sources, disallowed licenses, wildcard dependencies,
and advisories are checked without local ignores. The current unpublished scaffold
is explicitly excluded from dependency-license resolution until release licensing
is decided; external crates are not. `cargo geiger` reports current workspace and
transitive unsafe usage; it does not prove FFI safety or replace review.

The GitHub Actions `Rust baseline` job runs the four Cargo checks and the
`Dependency and unsafe-code policy` job runs these three policy checks on Linux for
every PR and pushes to `main` (no path filters). Inspect the latest revision's
checks before merging; local success is not a substitute for hosted CI. No branch
protection is configured by this PR.

For docs, also validate relative links, skill frontmatter where affected, and
normal/blocked guidance scenarios. The one scaffold integration test only checks
successful exit, exact greeting, and empty stderr. None of these checks validates
TabDat semantics, statistical accuracy, dependency security, or native packaging.
Those need separate roadmap evidence. The binary and integration test forbid
unsafe code; future ordinary crates must do the same.

The isolated DuckDB feasibility prototype is not part of the root workspace. When
working on `spikes/duckdb-prototype/`, use its committed lockfile and keep build
artifacts under the ignored root target directory:

```sh
cargo fmt --manifest-path spikes/duckdb-prototype/Cargo.toml -- --check
CARGO_TARGET_DIR=target/duckdb-spike cargo test --manifest-path spikes/duckdb-prototype/Cargo.toml --locked
CARGO_TARGET_DIR=target/duckdb-spike cargo build --manifest-path spikes/duckdb-prototype/Cargo.toml --release --locked
target/duckdb-spike/release/measure
```

The path-scoped `DuckDB feasibility spike` workflow covers the Linux build/test;
macOS Apple Silicon evidence, dependency audit, and native unsafe inventory remain
explicitly recorded in `docs/feasibility/duckdb.md`, not presented as a
product-support guarantee.

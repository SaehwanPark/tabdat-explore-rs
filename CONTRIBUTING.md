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

Use `cargo fmt --all` to apply formatting. The GitHub Actions `Rust baseline` job
runs the four Cargo checks on Linux for every PR and pushes to `main` (no path
filters). Inspect the latest revision's checks before merging; local success is
not a substitute for hosted CI. No branch protection is configured by this PR.

For docs, also validate relative links, skill frontmatter where affected, and
normal/blocked guidance scenarios. The one scaffold integration test only checks
successful exit, exact greeting, and empty stderr. None of these checks validates
TabDat semantics, statistical accuracy, dependency security, or native packaging.
Those need separate roadmap evidence. The binary and integration test forbid
unsafe code; future ordinary crates must do the same.

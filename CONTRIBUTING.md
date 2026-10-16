# Contributing to TabDat Explore Rust

This is an incremental, outcome-driven Rust project. Read [AGENTS.md](AGENTS.md)
for invariant boundaries and [SPEC.md](SPEC.md) for supported features. The
[roadmap](docs/TABDAT_RUST_PORT_ROADMAP.md) lists current priorities, not mandatory
phase gates. For small changes, read only the relevant code/tests.

## Development

Use [rustup](https://rustup.rs/) and the repo's pinned `rust-toolchain.toml`
(currently Rust **1.99.1**, including Clippy and rustfmt). The workspace uses
stable APIs added after 1.97, so use the pin rather than the old toolchain.
Commit `Cargo.lock` for dependency changes and use `--locked` in checks.
If package-manager Rust is also installed, `rustup show active-toolchain` can report
this pin while `cargo`/`rustc` on `PATH` still use another installation. If their
versions disagree with the pin, put rustup's shims first before running checks:
`export PATH="$HOME/.cargo/bin:$PATH"`. Avoid unrelated code changes just to satisfy
a different toolchain's new lints. Verify both `rustc --version` and
`cargo --version` after updating `PATH`. Historical performance/reference records
retain the compiler versions on which they were measured; a pin update alone is
not evidence of a compilation or runtime speedup.

Use two-space indentation, no hard tabs or non-breaking spaces; respect syntax
or generated-file formatting requirements.

A practical change should:

1. State what user-visible behavior or internal invariant changes.
2. Add or adjust the few relevant tests; include realistic failure cases.
3. Implement the smallest solution within existing boundaries.
4. Run focused tests, inspect the diff, and summarize residual limitations.
5. Update brief current-status docs only if supported behavior changes.

A new failure-first test is helpful for a bug fix, not a mandatory ceremonial
step. A PR or design note should be as short as the actual complexity allows.

Keep the root README focused on orientation and a runnable quickstart. Follow the
[documentation index](docs/README.md) for detailed guides, result contracts and
historical evidence; put per-slice test logs in the PR, not the README or roadmap.

## Finding the right code and tests

Use [ARCHITECTURE.md](ARCHITECTURE.md) for crate ownership, then follow the
command from `tabdat-language::parse_command` through
`tabdat-execution::Session::execute` to its data/statistics adapter and reporting.
Parser acceptance alone does not imply executable support.

- `crates/tabdat-language/tests/`: syntax, diagnostics, tokenizer/script contracts,
  properties and pinned compatibility fixtures; private parser tests also live in `src/lib.rs`.
- `crates/tabdat-runtime/tests/`: execution contracts, including unsupported-command
  errors, dataset/label preservation and retained model state.
- `crates/tabdat-stats/tests/`: numerical reference fixtures and low-level shape/error contracts.
- Root `tests/`: binary CLI, interactive, JSON/text and cross-crate workflow contracts.
  `tests/common/` provides reusable CLI process and temporary-directory plumbing;
  keep scenario-specific data and assertions beside their tests.
- Crate-local unit tests cover private helpers and failure injection (for example,
  shell output errors); `tests/workspace_boundaries.rs` checks dependency boundaries.

Prefer a focused test beside the behavior it protects. Share repeated plumbing,
not a test DSL that hides commands, expected rows, numerical tolerances or errors.

## Normal Rust checks

```sh
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo test --locked --workspace --all-targets
cargo clippy --locked --workspace --all-targets -- -D warnings
git diff --check
```

Run focused commands while iterating. Before publishing a completed code change,
run the relevant full checks; report any failure rather than hiding it. GitHub
Actions are currently **disabled**: checks do not run automatically. CI status
must not be inferred from old workflow records.

## Proportional additional verification

| Scope | Additional checks |
| --- | --- |
| Parser/script/CLI | Focused valid/invalid examples, line/path diagnostics and existing regression tests |
| Data/session/file effects | Schema and missingness, order, rollback/failure state, output recovery |
| Statistics | Independent reference (e.g., trusted numerical package), sample/design verification, justified tolerances |
| Native/FFI work | Ownership, failure/teardown, sanitizer/leak checks when appropriate, platform and licensing evidence |
| Release/dependency update | `cargo deny check`, `cargo audit -D warnings`, distribution/license review as applicable |

`cargo-geiger` may assist an unsafe-code inventory, but it is not proof of
native safety and **does not need to be run on every documentation or parser PR**.
The previous full geiger procedure and toolchain caveats remain in the
[historical contributor guide](https://github.com/SaehwanPark/tabdat-explore-rs/blob/333b97dd55524dbf5ce55fac510375df99e4649e/CONTRIBUTING.md).

For reproducible parser robustness work, keep the existing isolated
[`fuzz/`](fuzz/README.md) workspace and its reproduced stack-overflow regression.
Do not launch long fuzz campaigns for routine non-parser changes. Use the
[syntax budget ADR](docs/adr/0023-syntax-resource-budgets-and-fuzz-tooling.md)
for current input limits.

## PRs, approvals, and handoffs

Keep scope small and reviewable. Do not overwrite unrelated uncommitted work.
Push/merge only under explicit task authorization; no required automatic PR
creation, review-pass count, delegation, or wait-for-quota loop. On an incomplete
change, record the branch/commit, successful and failing tests, remaining
question and smallest next action. Leave important WIP evidence in the draft PR
when one exists.

For Python compatibility, use [migration guidance](docs/migration/README.md).
Differences affecting mathematical results or supported user workflows deserve
strong evidence; harmless error-message/lexical differences can be documented.

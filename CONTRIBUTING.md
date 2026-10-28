# Contributing to TabDat Explore Rust

This is an incremental, outcome-driven Rust project. Read [AGENTS.md](AGENTS.md)
for invariant boundaries and [SPEC.md](SPEC.md) for supported features. The
[roadmap](docs/TABDAT_RUST_PORT_ROADMAP.md) prioritizes graduate econometrics and
Cameron–Trivedi empirical coverage, not mandatory phase gates. The
[coverage ledger](docs/reference/microeconometrics-coverage.md) distinguishes
required future methods from validated textbook replications. For small changes,
read only the relevant code/tests; not every task requires a textbook replay.

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
5. Update brief current-status docs when supported behavior changes. For a
   textbook slice, refine one exact illustration's data/sample/design/reference
   contract and update the coverage ledger with implementation/test evidence;
   partial method support must not be labeled whole-example or chapter coverage.

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

## Impact-scoped testing

Follow the [agent test execution policy](AGENTS.md#test-execution-policy): inspect
changes and consumers, run the smallest relevant scope first, then finish with
affected-crate tests and relevant integrations. Previously passing runs count
unless later edits could affect them. Do not rerun a verified baseline at startup
or run the entire workspace by habit. When impact is uncertain, broaden scope.

### Runner setup

Use [cargo-nextest](https://nexte.st/docs/installation/) for unit/integration
execution. On macOS, `brew install cargo-nextest`; alternatively,
`cargo install cargo-nextest --locked`. Check `cargo nextest --version` after
installation (this setup was verified with **0.9.149**). This is a developer tool,
not an application dependency; plain Cargo remains supported.

The checked-in [`.config/nextest.toml`](.config/nextest.toml) hides passing-test
noise, emits captured failure output immediately, disables retries, and finishes
all selected tests after a failure. It does **not** exclude slow tests or change
Cargo's compilation profile, test concurrency or statistical tolerances. Use
`--status-level pass` when individual successful-test status is useful. Nextest
runs tests in separate processes; use Cargo to investigate runner-specific
behavior, and do not silently accept a difference.

On macOS, test discovery has occasionally stalled before the test harness starts:
`sample <pid> 1` showed only `_dyld_start`, while a SHA256-identical executable
copied to a fresh temporary path ran normally. This is a diagnosed loader-start
stall, not a test pass or proof that nextest is faulty. Preserve the sample and
failed attempt; if using this fallback, select executables from a successful
current Cargo `--no-run --message-format json` inventory (never a stale filename
glob), verify copy hashes, use each package's manifest working directory and
record complete standard-harness results/counts, including ignored/zero-test
artifacts. Do not repeatedly retry real assertion failures.

The dev profile and its test-profile derivative disable debug information and
incremental compilation to limit build output, especially the bundled DuckDB
objects and the many integration test executables. This trades debugging detail
and potentially slower rebuilds for disk space; release builds are unchanged.
The bundled C/C++ build also defines `NDEBUG` with this profile, disabling native
assertions; Rust debug assertions remain enabled.
For a debugging session, opt in with `CARGO_PROFILE_DEV_DEBUG=2` and, if needed,
`CARGO_PROFILE_DEV_INCREMENTAL=true` (use `CARGO_PROFILE_TEST_DEBUG` and
`CARGO_PROFILE_TEST_INCREMENTAL` for tests). Overrides and changes to toolchains
or features can leave additional artifacts behind. Check `du -sh target` and
run `cargo clean` when necessary; it deletes generated build artifacts and the
next build will recompile them. Do not clean while a build is running. Each
worktree normally has its own target directory; clean the intended worktree.

### Select from the diff, not just test names

Before testing, inspect both committed branch work and uncommitted/new files:

```sh
git diff --stat origin/main...HEAD
git diff origin/main...HEAD
git diff HEAD
git status --short
```

For a PR based elsewhere, substitute its actual base (a fixed base commit is
useful when main advances). Map each behavior to its implementation, direct
unit/module tests, crate integration targets, and callers. Read changed/new tests
and include them explicitly. Use `rg` and [ARCHITECTURE.md](ARCHITECTURE.md) for
consumers; inspect the dependency graph when a shared API changes:

```sh
cargo tree --locked --workspace --invert tabdat-stats
```

The graph identifies candidate consumers, not semantic impact. Account for root
`tests/`, fixtures, scripts and data files: a filename-only changed-crate tool
cannot prove which workflows are unaffected. Changes to shared contracts,
workspace configuration, toolchains or dependencies can require broad validation.
Documentation-only changes need relevant link/example checks, not an invented
application-test obligation.

### Smallest scope first

Use Cargo package/target flags to avoid building unrelated test binaries. Root
integration tests belong to **`tabdat-explore-rs`**, not to the runtime crate.
Filters narrow execution, but are not a substitute for build-target selection.
Examples (replace the selections with the actual changed behavior):

```sh
# Private parser unit test; list first if the exact name is uncertain.
cargo nextest list --locked -p tabdat-language --lib -E 'test(~lincom)'
cargo nextest run --locked -p tabdat-language --lib --no-tests fail -E 'test(~lincom)'

# One numerical integration target, then a root boundary/CLI target when relevant.
cargo nextest run --locked -p tabdat-stats --test gamma_inference_contract --no-tests fail
cargo nextest run --locked -p tabdat-explore-rs --test cli_json_linear_contract --no-tests fail

# Completion: all tests in affected crates (unit and crate integration tests).
cargo nextest run --locked -p tabdat-language -p tabdat-execution --no-tests fail

# Shared API: changed crate plus transitive reverse dependents, including root tests.
cargo nextest run --locked --workspace --no-tests fail -E 'rdeps(=tabdat-stats)'
```

The [`rdeps` filter](https://nexte.st/docs/filtersets/reference/) includes the
named crate itself. Do not combine it with a restrictive `-p` that removes needed
consumers. Add relevant cross-crate integrations even when a dependency edge does
not capture their contract. Include **every new/modified test** rather than
assuming a topic substring matches it. An empty selection is an error, not test
evidence (`--no-tests fail` makes this explicit).

Cargo fallback and documentation examples:

```sh
cargo test --locked -p tabdat-language --lib lincom
cargo test --locked -p tabdat-stats --test gamma_inference_contract
cargo test --locked -p tabdat-language -p tabdat-execution
# Nextest does not run doctests. Run these when Rust documentation examples change.
cargo test --locked -p tabdat-domain --doc
```

Cargo name filters can succeed with zero matches: inspect the reported count.
Do not use `--all-targets` as a substitute for doctests. No automatic selector can
replace inspecting changed behavior, fixtures and consumers.

### Checks and output

For code changes, format changed Rust files (or check their owning package) and
run applicable affected-package checks; for example:

```sh
cargo fmt -p tabdat-language -- --check
cargo check --locked -p tabdat-language --all-targets
cargo clippy --locked -p tabdat-language --all-targets -- -D warnings
git diff --check
```

A successful test build already checks compilation of its selected targets; do
not mechanically repeat equivalent checks unless another target/feature needs
validation. Expand package selection for shared APIs. Reserve these broad checks
for cross-cutting changes or explicitly requested local validation:

```sh
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo nextest run --locked --workspace --all-targets --no-tests fail
cargo test --locked --workspace --doc
cargo clippy --locked --workspace --all-targets -- -D warnings
```

Keep passing output to scope/count/timing summaries. Preserve complete failure
diagnostics: do not pipe test output through `tail`, suppress stderr, or discard
the runner's exit status. If saving a log, use `set -o pipefail` with `tee` and
retain the full log. Report tested commands/scopes, failures, and checks not run
with reasons; filtered-out, ignored and unrun tests are not passes. No runtime
speedup is implied merely by installing nextest.

GitHub Actions are currently **disabled**: checks do not run automatically. CI
status must not be inferred from old workflow records. Scope selection is a local
responsibility, not a reason to assume CI will cover omissions.

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

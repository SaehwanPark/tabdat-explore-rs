# Contributing to TabDat Explore Rust

The [capability map](SPEC.md) describes the supported product and the
[roadmap](docs/TABDAT_RUST_PORT_ROADMAP.md) sets priorities. For agent
development, follow the [default PR workflow](AGENTS.md#default-development-workflow).

## Development

Use [rustup](https://rustup.rs/) and the [pinned toolchain](rust-toolchain.toml)
with Clippy and rustfmt. If package-manager Rust is also installed, compare
`rustc --version` and `cargo --version` with the pin. Put rustup's shims first
with `export PATH="$HOME/.cargo/bin:$PATH"` if necessary. Commit `Cargo.lock` for
dependency changes and use `--locked` in checks.

Use the [architecture](ARCHITECTURE.md) to find crate ownership. Follow a command
from `tabdat-language::parse_command` through `tabdat-execution::Session::execute`
to its data or statistical adapter and renderer. Parser tests live in
`tabdat-language`, numerical fixtures in `tabdat-stats`, and session contracts
in `tabdat-runtime/tests/`. Root `tests/` covers CLI, interactive, machine
output and end-to-end workflows. Crate-local tests exercise private helpers.

Keep detailed contracts in the [documentation index](docs/README.md)'s guides
and references. Update the [coverage ledger](docs/reference/microeconometrics-coverage.md)
when an illustration's evidence changes. Record slice-specific validation in the PR.

## Impact-scoped testing

Use the [test selection policy](AGENTS.md#test-execution-policy). GitHub Actions
are disabled for this repository. Run the relevant checks locally.

### Select targets

Inspect both the committed branch and working changes, including untracked files.
For shared interfaces, trace callers and affected fixtures as well as Cargo edges.

```sh
git diff origin/main...HEAD
git diff HEAD
git status --short
cargo tree --locked --workspace --invert tabdat-stats
```

Use [cargo-nextest](https://nexte.st/docs/installation/) or plain Cargo.
Select packages and integration targets before filtering tests. Root integration
targets belong to `tabdat-explore-rs`. Examples for different scopes:

```sh
# Inspect available names when selecting a private test.
cargo nextest list --locked -p tabdat-language --lib -E 'test(~lincom)'
cargo nextest run --locked -p tabdat-language --lib --no-tests fail -E 'test(~lincom)'

cargo nextest run --locked -p tabdat-stats --test gamma_inference_contract --no-tests fail
cargo nextest run --locked -p tabdat-explore-rs --test cli_json_linear_contract --no-tests fail
cargo nextest run --locked -p tabdat-language -p tabdat-execution --no-tests fail

# When the shared API affects transitive consumers, include the workspace.
cargo nextest run --locked --workspace --no-tests fail -E 'rdeps(=tabdat-stats)'

# Cargo fallback. Inspect the count when using a name filter.
cargo test --locked -p tabdat-language --lib lincom
cargo test --locked -p tabdat-stats --test gamma_inference_contract
# Run doctests separately when Rust documentation examples change.
cargo test --locked -p tabdat-domain --doc
```

The [nextest profile](.config/nextest.toml) suppresses passing-test noise,
prints failures immediately and finishes the selection without retries. Preserve
complete failure diagnostics and the runner's exit code when capturing output.

### Checks and output

Check formatting and lint the affected packages. Broaden to the workspace for
cross-cutting changes. A test build covers compilation of its selected targets.

```sh
cargo fmt -p tabdat-language -- --check
cargo clippy --locked -p tabdat-language --all-targets -- -D warnings
git diff --check
```

| Change | Additional evidence |
| --- | --- |
| Documentation | Check affected links, examples and capability claims |
| Parser, script or CLI | Acceptance and error regressions, including locations where promised |
| Data, session or file effects | Schema, missingness, order, and state after success and failure |
| Statistical results | Independent numerical reference, sample/design, covariance, degrees of freedom and justified tolerances |
| Native or FFI | Ownership and teardown, licensing, platform checks and sanitizer/leak evidence as applicable |
| Dependencies or release | `cargo deny check`, `cargo audit -D warnings` and distribution review as applicable |

Use the [fuzz workspace](fuzz/README.md) and [syntax budgets](docs/adr/0023-syntax-resource-budgets-and-fuzz-tooling.md)
for parser robustness work. For Python compatibility decisions see the
[migration guide](docs/migration/README.md).

### Runner troubleshooting

On macOS, discovery has occasionally stalled in `_dyld_start` before the test
harness starts. Inspect the process with `sample <pid> 1`. For that diagnosed
loader stall, a byte-identical executable at a fresh path has worked. Select
binaries from a successful current Cargo `--no-run --message-format json`
inventory, verify copy hashes and run them from their package directories.
Record standard-harness results, including ignored and zero-test binaries.

Development and test builds disable debug information and incremental compilation
to limit bundled DuckDB and test artifact size. This profile defines `NDEBUG`
for bundled C/C++ while Rust debug assertions remain enabled. To debug, opt into
`CARGO_PROFILE_DEV_DEBUG=2` and optionally `CARGO_PROFILE_DEV_INCREMENTAL=true`;
use the corresponding `CARGO_PROFILE_TEST_*` settings for tests. If reclaiming
space with `cargo clean`, check the target directory and wait for active builds
to finish.

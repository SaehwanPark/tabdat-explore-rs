# Bounded eager-runtime CSV `use` migration evidence

Status: implementation and local validation complete; PR #74 hosted and
post-merge workflow verification are recorded below after acceptance.

Producer: task owner
Consumers: independent reviewer, PR reviewers, and the next maintainer

Boundary: pinned Python eager CSV ingestion behavior → Rust eager local-CSV
`use`. The bounded contract is in [`01-contract.md`](01-contract.md).

## Authority and contract inputs

The Python oracle is the clean sibling checkout `../tabdat-explore` at pinned
revision `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, package `0.25.0`, Python `3.13.3`,
and `uv.lock` SHA-256
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.
The checkout was clean; no dependency synchronization or source edits were
performed.

The execution authority is:

- `src/tabdat/backend.py:290-327` for parameterized local CSV ingestion;
- `src/tabdat/backend.py:3617-3628` for source-path validation; and
- `tests/test_executor.py:10139-10172` for delimiter/header, typed metadata,
  schema, row count, and preview behavior.

## Rust revisions and changed paths

- `f0aa129` — contract checkpoint and draft PR [#74](https://github.com/SaehwanPark/tabdat-explore-rs/pull/74);
- `de98fef` — bounded eager CSV implementation and focused tests;
- `dcd33cf` — compatibility/error-boundary review fixes and expanded focused
  coverage;
- `2dcc9ea` — final compatibility coverage and evidence/review updates; and
- `7a5b8d4` — squash merge of PR #74 onto `main`.

Changed paths:

- `crates/tabdat-runtime/src/lib.rs` — local Parquet/CSV format dispatch,
  parameterized `read_csv_auto`, staged publication, typed CSV errors, and
  failure-preserving state transitions;
- `crates/tabdat-runtime/tests/use_csv_contract.rs` — six focused CSV success,
  option, empty-input, and failure/state tests; and
- `crates/tabdat-runtime/tests/use_contract.rs` — updated format diagnostics,
  Parquet option-precedence coverage, and unsupported-suffix coverage.

No dependency, unsafe code, FFI, backend-selection, parser, or root-binary
surface changed.

## Pinned oracle probes

Focused Python execution probe:

```sh
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_executor.py -k 'test_execute_ingestion_csv_feather_arrow'
```

Observed: `1 passed, 416 deselected`.

The parser/script oracle was not changed by this runtime-only slice; the
existing parser contract tests and workspace test suite remain the source of
truth for syntax behavior.

## Rust implementation evidence

The runtime recognizes `.csv` case-insensitively, validates local paths before
backend initialization, preserves the historical Parquet option-error
precedence, and binds the path, delimiter, and header values through DuckDB
`read_csv_auto`. CSV rows are staged in `__tabdat_next`; schema and row count
are inspected before shared transactional publication. Read, schema, row-count,
and CSV-publication errors clean staging without replacing the active relation.
Successful publication returns the existing owned `LoadResult` and clears label
metadata only after the relation has been published.

Focused command:

```sh
cargo test --locked -p tabdat-runtime --test use_csv_contract
```

Observed: `6 passed, 0 failed`.

Workspace checks:

```sh
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo test --locked --workspace --all-targets
cargo clippy --locked --workspace --all-targets -- -D warnings
git diff --check
```

Observed locally: format, check, the workspace suite (including 65 existing
`use_contract` tests and the six CSV tests), Clippy, and diff checks pass on the
final implementation revision `2dcc9ea`.

Dependency and unsafe-code policy checks:

```sh
cargo deny check
cargo audit -D warnings
# metadata-driven cargo geiger loop from CONTRIBUTING.md
```

Observed locally: `cargo deny` reported advisories, bans, licenses, and sources
`ok`; `cargo audit -D warnings` completed without findings; and the metadata
geiger loop checked one first-party package per manifest, `forbid(unsafe_code)`,
and zero first-party unsafe use. Transitive dependency inventory warnings, if
reported by geiger, remain policy-reporting scope only.

## State and scope review inputs

Focused tests cover default auto-detection, explicit semicolon delimiter and
header options, the combined delimiter/headerless branch, quoted fields,
empty/NULL fields, case-insensitive `.CSV`, header-only and empty files, ordered
schema/count/preview values, missing and non-file paths, unsupported suffixes,
invalid UTF-8 CSV, Parquet/unsupported-suffix option compatibility, successful
label clearing, and failed replacement preserving all prior rows, metadata,
labels, and backend usability.

Known bounded deviations are explicit: no remote or URI source, lazy or
materialized execution, Feather/Arrow/DTA, `~` expansion, broader path
normalization, metadata serialization, temporary-file atomic replacement, or
CLI/JSON/MCP surface. DuckDB's auto-reader remains authoritative for its
accepted empty and malformed-text behavior; the test uses invalid UTF-8 for a
deterministic read failure.

## Hosted acceptance

PR #74 was promoted from draft and its exact-head pull-request workflows passed
on `2dcc9ea`:

- [CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35562235760)
  (`Rust baseline` and `Dependency and unsafe-code policy`); and
- [TabDat runtime boundary](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35562235734).

The same SHA also passed the manually dispatched [CI run](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35562197645)
and [runtime run](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35562198903)
used while the PR was still draft. PR #74 was squash-merged as `7a5b8d4`.

Post-merge `main` verification is recorded by the merge-head [CI run](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35563640913)
and [TabDat runtime boundary run](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35563640908),
both for `7a5b8d4`.

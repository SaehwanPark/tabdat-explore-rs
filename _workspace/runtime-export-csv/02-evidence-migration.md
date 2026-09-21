# Bounded eager-runtime CSV `export` migration evidence

Status: accepted after PR #72 (`5cb5b34`) squash merge; post-merge workflow
verification is recorded below.

Producer: task owner
Consumers: independent reviewer, PR reviewers, and the next maintainer

Boundary: pinned Python export behavior → Rust eager local-Parquet CSV output.
The bounded contract is in [`01-contract.md`](01-contract.md).

## Authority and contract inputs

The Python oracle is the clean sibling checkout `../tabdat-explore` at pinned
revision `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, package `0.25.0`, Python `3.13.3`,
and `uv.lock` SHA-256
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.
The checkout was clean; no dependency synchronization or source edits were
performed.

The execution authority is:

- `src/tabdat/backend.py:778-795` for supported suffixes, overwrite handling,
  parent creation, and CSV dispatch;
- `src/tabdat/backend.py:3617-3628` for output-target checks; and
- `tests/test_executor.py:9472-9530` for transformed output, overwrite
  rejection, replacement, and exact CSV behavior.

## Rust revisions and changed paths

- `3b09266` — contract checkpoint and draft PR [#72](https://github.com/SaehwanPark/tabdat-explore-rs/pull/72);
- `55c805b` — bounded runtime implementation and focused tests;
- `cd3654c` — evidence, review, and closeout artifacts on the PR head; and
- `5cb5b34` — squash merge of PR #72 to `main`.

Changed paths:

- `crates/tabdat-runtime/src/lib.rs` — `ExportResult`,
  `ExecutionResult::Export`, export-specific typed errors, path validation,
  state-preserving dispatch, and parameterized DuckDB CSV copy;
- `crates/tabdat-runtime/tests/export_contract.rs` — nine success/failure,
  exact-byte, transformed, replacement, empty-relation, and recovery tests; and
- `crates/tabdat-runtime/tests/use_contract.rs` — updated no-active export
  regression.

No dependency, unsafe code, FFI, backend selection, or root-binary surface
changed.

## Pinned oracle probes

Focused Python execution probe:

```sh
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_executor.py -k 'phase_9_export_writes_supported_formats'
```

Observed: `3 passed, 414 deselected`.

The broader parser/script oracle was rerun to ensure the syntax contract
remained intact:

```sh
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py tests/test_script.py
```

Observed: `516 passed in 0.48s`.

## Rust implementation evidence

The runtime validates the `.csv` extension, existing target, regular-file
condition, and parent directory before invoking DuckDB. The backend uses a
bound path parameter in
`COPY (SELECT * FROM __tabdat_active) TO ? (FORMAT CSV, HEADER)`. Successful
exports return output-oriented metadata whose source is the output path, while
the session's active metadata, labels, and relation remain unchanged. Failure
paths return typed errors before publication or relation mutation.

Focused command:

```sh
cargo test --locked -p tabdat-runtime --test export_contract
```

Observed: `9 passed, 0 failed`.

Workspace checks:

```sh
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo test --locked --workspace --all-targets
cargo clippy --locked --workspace --all-targets -- -D warnings
git diff --check
```

Observed locally: formatting, check, full workspace tests, Clippy, and diff
checks passed. The full test run includes the nine export tests and the updated
existing runtime regression suite.

Dependency and unsafe-code policy checks:

```sh
cargo deny check
cargo audit -D warnings
# metadata-driven cargo geiger loop from CONTRIBUTING.md
```

Observed locally: deny reported advisories/bans/licenses/sources `ok`; audit
reported no warnings; the metadata geiger loop completed with one first-party
package per manifest, `forbid(unsafe_code)`, and zero first-party unsafe use.
Transitive dependency inventory warnings remain policy-reporting scope only.

## State and scope review inputs

The focused tests cover no-active backend non-initialization, parsed export,
exact header/schema order/insertion row order/decimal formatting/quoted text/
NULL fields, transformed data, case-insensitive `.CSV`, unsupported suffixes,
parent creation, overwrite gating and replacement, directory targets, parent
failures, a valid-parent backend-copy failure, empty relations, and unchanged
active state.

Known bounded deviations are explicit: no `~` expansion or broader path
normalization, no Parquet alias through `export`, no Feather/Arrow writer, no
temporary-file atomic replacement guarantee for a backend write failure, and no
lazy/materialized, label/panel persistence, CLI, JSON, or MCP surface.

## Hosted acceptance

PR [#72](https://github.com/SaehwanPark/tabdat-explore-rs/pull/72) was promoted
from draft and squash merged as `5cb5b34` at `2026-09-21T03:02:46Z`. All
required PR-head workflows passed on `cd3654c`:

- [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35554779193/job/106196209512);
- [dependency and unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35554779193/job/106196209256); and
- [tabdat-runtime on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35554779171/job/106196205667).

The merge-head workflows for `5cb5b34` also passed:

- [main CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35556151175)
  — Rust baseline job `106200078563` and policy job `106200078640`;
- [main tabdat-runtime boundary](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35556151230)
  — Linux job `106200078731`.

The accepted slice is now verified on `main` at `5cb5b34`.

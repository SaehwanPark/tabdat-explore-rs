# Bounded eager-runtime `save` migration evidence

Status: implementation complete locally; hosted acceptance and merge evidence
will be appended before closeout.

Producer: task owner
Consumers: independent reviewer, PR reviewers, and the next maintainer

## Authority and contract inputs

The Python oracle is the clean sibling checkout `../tabdat-explore` at pinned
revision `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, package `0.25.0`, Python `3.13.3`,
and `uv.lock` SHA-256
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.
The checkout was clean; no dependency synchronization or source edits were
performed. The bounded contract is in [`01-contract.md`](01-contract.md).

The execution authority is:

- `src/tabdat/backend.py:758-778` for Parquet-only output, parent creation,
  overwrite handling, and DuckDB copy;
- `src/tabdat/backend.py:3617-3628` for output-target checks; and
- `tests/test_executor.py:9442-9470` for transformed output, overwrite
  rejection, and replacement.

## Rust revisions and changed paths

- `a4bfbc2` — contract checkpoint and draft PR [#70](https://github.com/SaehwanPark/tabdat-explore-rs/pull/70);
- `8a51881` — bounded runtime implementation and focused tests; and
- `e2ac065` — review-gap closure, focused evidence, and migration record; and
- `e37a59f` — direct recovery/read-back evidence after a backend-copy failure.

Changed paths:

- `crates/tabdat-runtime/src/lib.rs` — `SaveResult`, `ExecutionResult::Save`,
  save-specific typed errors, path validation, state-preserving dispatch, and
  parameterized DuckDB Parquet copy;
- `crates/tabdat-runtime/tests/save_contract.rs` — eight success/failure and
  round-trip tests, including review-gap coverage; and
- `crates/tabdat-runtime/tests/use_contract.rs` — updated no-active save
  regression while retaining export deferral.

No dependency, unsafe code, FFI, backend selection, or root-binary surface
changed.

## Pinned oracle probes

Focused Python execution probe:

```sh
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_executor.py -k 'phase_9_save_writes_transformed_active_dataset'
```

Observed: `1 passed, 416 deselected in 1.98s`.

The previously recorded parser/script oracle was rerun to ensure the parser
contract remained intact:

```sh
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py tests/test_script.py
```

Observed: `516 passed in 0.48s`.

## Rust implementation evidence

The runtime validates the output extension, existing target, regular-file
condition, and parent directory before invoking the existing DuckDB backend.
The backend uses a bound path parameter in
`COPY (SELECT * FROM __tabdat_active) TO ? (FORMAT PARQUET)`. Successful saves
return output-oriented metadata whose source is the output path, while the
session's active metadata and relation remain unchanged. Failure paths return
typed errors before publication or relation mutation.

Focused command:

```sh
cargo test --locked -p tabdat-runtime --test save_contract -- --nocapture
```

Observed: `8 passed, 0 failed`.

Workspace checks:

```sh
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo test --locked --workspace --all-targets
cargo clippy --locked --workspace --all-targets -- -D warnings
git diff --check
```

Observed locally: formatting, check, full workspace tests, Clippy, and diff
checks passed. The full test run includes the new eight save tests and the
updated existing runtime regression suite.

Dependency and unsafe-code policy checks:

```sh
cargo deny check
cargo audit -D warnings
# metadata-driven cargo geiger loop from CONTRIBUTING.md
```

Observed locally: deny reported advisories/bans/licenses/sources `ok`; audit
reported no warnings; geiger reported exactly one first-party package per
manifest, `forbid(unsafe_code)` and zero first-party unsafe use. Transitive
dependency inventory warnings remain policy-reporting scope only.

## State and scope review inputs

The focused tests cover no-active backend non-initialization, parsed save,
schema/order/row-count/NULL round trips using deliberately non-sorted input,
transformed data, case-insensitive Parquet extension, parent creation,
overwrite gating and replacement, directory targets, parent failures, a valid-
parent backend-copy failure, empty relations, and unchanged active state.

Known bounded deviations are explicit: no `~` expansion or broader path
normalization, no temporary-file atomic replacement guarantee for a backend
write failure, and no `export`, CSV/Feather/Arrow, lazy/materialized, label or
panel persistence, CLI, JSON, or MCP surface.

## Independent review resolution

The independent review identified three bounded issues. The `~` expansion
difference was already recorded as an accepted scope deviation. The focused
suite now asserts row order without `ORDER BY` and adds a valid-parent,
deliberately failing backend-copy case that proves active state and rows remain
unchanged; a recovery save/read-back also verifies the private relation after
the failure.

## Hosted acceptance

PR #70 was marked ready and all three required PR-head workflows passed on
`8f40c2a`:

- [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35549742114)
  (job `106182183656`);
- [tabdat-runtime on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35549742038)
  (job `106182184129`); and
- [dependency and unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35549742114)
  (job `106182183788`).

PR [#70](https://github.com/SaehwanPark/tabdat-explore-rs/pull/70) was squash
merged as `9bbf804` at `2026-09-21T01:29:19Z`. Both merge-head workflows
passed on `9bbf804`:

- [main CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35551053158)
  (job `106185758109`); and
- [main tabdat-runtime boundary](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35551053257)
  (job `106185758619`).

Completion state is `accepted` with PR-head and post-merge verification
complete.

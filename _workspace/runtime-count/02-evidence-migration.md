# Bounded runtime `count` migration evidence

Status: implementation and independent review complete; draft PR #32 is open
while its hosted baseline, runtime, and policy workflows complete.

Producer: task owner, with pinned oracle evidence and independent runtime review

Consumers: reviewers and the next runtime maintainer

Boundary: pinned Python execution contract → Rust-owned eager-session row count

## Authority and contract inputs

The Python oracle is the clean sibling checkout `../tabdat-explore` at pinned
commit `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, package `0.25.0`, and Python
`3.13.3`. Its `uv.lock` SHA-256 is
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`; the
checkout was clean and no dependency synchronization or source edits were
performed. The bounded contract is in `01-contract.md`.

The accepted Rust prerequisites are the eager local-Parquet `use` session in
PR #22 and the read-only cached metadata `describe` boundary in PR #31. This
slice consumes the existing owned `DatasetInfo.row_count`; it does not add a
relation query, lazy plan, materialization path, or backend initialization for
`count`.

## Revisions and changed paths

- `0549bf0`: frozen contract and draft-PR handoff;
- `6e47ab0`: owned `CountResult`, count dispatch, and state/atomicity tests;
- PR #32: [Execute bounded runtime count](https://github.com/SaehwanPark/tabdat-explore-rs/pull/32), opened before implementation.

Changed implementation paths:

- `crates/tabdat-runtime/src/lib.rs`: owned count result and read-only
  `Command::Count` dispatch;
- `crates/tabdat-runtime/tests/use_contract.rs`: exact fresh-session,
  parser-to-session, repeated-read, and failed-replacement coverage;
- `_workspace/runtime-count/01-contract.md`: bounded authority, cached-count
  decision, and stop conditions.

No parser grammar, backend query, relation API, filesystem writer, dependency,
unsafe-code, serialization, CLI, or MCP surface changed.

## Pinned oracle probe

The focused command was run at the pinned revision:

```sh
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_executor.py -k \
  'test_count_returns_active_dataset_row_count or \
   test_phase_3_inspection_commands_require_active_dataset'
```

Observed result:

```text
7 passed, 410 deselected in 1.41s
```

The authoritative no-active error is exactly:
`count requires an active dataset; run use <path> first`. The success case
returns the eager fixture's three-row count. The pinned executor selection also
covers the neighboring inspection-command no-active contract.

## Rust checks

At implementation revision `6e47ab0`, the required local checks passed:

```text
cargo fmt --all -- --check                              passed
git diff --check                                        passed
cargo check --locked --workspace --all-targets          passed
cargo test --locked --workspace --all-targets           passed
  root scaffold: 1 test passed
  tabdat-language: 42 unit + 32 public integration tests passed
  tabdat-runtime: 3 unit + 19 integration tests passed
cargo clippy --locked --workspace --all-targets -- -D warnings
                                                         passed
```

The policy checks also passed locally:

```text
cargo deny check                                        advisories, bans, licenses, sources ok
cargo audit -D warnings                                 passed; no vulnerabilities reported
metadata-driven cargo geiger (all workspace packages,
  locked/all-targets/all-dependencies JSON assertions)  passed; first-party
  crates reported forbid(unsafe_code) and zero first-party unsafe usage
```

The geiger loop follows the all-package assertion in `CONTRIBUTING.md`.
Dependency inventory warnings from transitive DuckDB dependencies are not
first-party unsafe findings.

## State and parity evidence

The implementation has one read-only state transition:

| Before | Input | Result | After |
| --- | --- | --- | --- |
| no active dataset | `count` | typed `NoActiveDataset` with exact text | unchanged; backend remains uninitialized |
| active eager local-Parquet dataset | `count` | owned `CountResult { row_count }` from cached metadata | unchanged |
| repeated count | `count`, `count` | equal owned counts | unchanged |
| prior active dataset | failed replacement `use`, then `count` | prior row count | failed staging does not publish |

The Python eager backend obtains the value with `SELECT COUNT(*)`. The bounded
Rust session has already established and atomically published the same row count
while staging `use`, so reading that cache is an intentional implementation
choice with the same observable result for the supported eager-only state. It
avoids inventing a new query-failure mode. Lazy/materialized count behavior,
status tracking, transforms, labels, formatting, CLI, and MCP output remain
explicit deferrals.

## Hosted acceptance and completion state

PR #32 is currently draft and its current-head hosted checks are in progress:

- [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35336706832/job/105573165378);
- [tabdat-runtime on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35336706758/job/105573095981); and
- [dependency and unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35336706832/job/105573165210).

The independent review found no P0/P1/P2/P3 findings. The PR will be marked
ready only after its current-head jobs are green; post-merge `main` verification
and temporary-branch cleanup remain required before this evidence is accepted.

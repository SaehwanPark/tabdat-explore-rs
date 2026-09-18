# Bounded runtime `describe` migration evidence

Status: implementation is on draft PR #31; review, hosted checks, merge, and
post-merge verification are pending.

Producer: task owner, with pinned oracle evidence and independent runtime review

Consumers: reviewers and the next runtime maintainer

Boundary: pinned Python execution contract → Rust-owned eager-session metadata

## Authority and contract inputs

The Python oracle is the clean sibling checkout `../tabdat-explore` at pinned
commit `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, package `0.25.0`, and Python
`3.13.3`. Its `uv.lock` SHA-256 is
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`; the
checkout was clean and no dependency synchronization or source edits were
performed. The bounded contract is in `01-contract.md`.

The accepted Rust prerequisite is the eager local-Parquet `use` session in PR
#22. This slice deliberately consumes its owned `DatasetInfo` cache; it does
not add a query, relation, lazy plan, or backend initialization path.

## Revisions and changed paths

- `9eb9174`: frozen contract and draft-PR handoff;
- `cea90b1`: owned `DescribeResult`, typed no-active-dataset error, read-only
  session dispatch, and state/atomicity regressions;
- `6e40f0f`: direct private-unit assertion that a fresh-session `describe`
  leaves the DuckDB backend uninitialized;
- later evidence/review and merge revisions will be recorded here.

Changed implementation paths:

- `crates/tabdat-runtime/src/lib.rs`: owned describe result, typed error,
  display text, and read-only `Command::Describe` dispatch;
- `crates/tabdat-runtime/tests/use_contract.rs`: exact fresh-session,
  parser-to-session, repeated-read, and failed-replacement coverage;
- `_workspace/runtime-describe/01-contract.md`: bounded authority and stop
  conditions.

No parser grammar, backend schema query, relation API, filesystem writer,
dependency, unsafe-code, serialization, CLI, or MCP surface changed.

## Pinned oracle probe

The focused command was run at the pinned revision:

```sh
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_executor.py -k \
  'test_describe_requires_active_dataset or test_describe_returns_active_dataset'
```

Observed result:

```text
2 passed, 415 deselected in 0.89s
```

The broader `describe` selection produced the same two passing tests. The
authoritative no-active error is exactly:
`describe requires an active dataset; run use <path> first`. The success case
returns the active `DatasetInfo` unchanged; its three-row fixture has ordered
columns beginning with `age`.

## Rust checks

At implementation revision `cea90b1`, and again after the test-only tightening
at `6e40f0f`, the required local checks passed:

```text
cargo fmt --all -- --check                              passed
cargo check --locked --workspace --all-targets          passed
cargo test --locked --workspace --all-targets           passed
  root scaffold: 1 test passed
  tabdat-language: 42 unit + 32 public integration tests passed
  tabdat-runtime: 2 unit + 16 integration tests passed
cargo clippy --locked --workspace --all-targets -- -D warnings
                                                         passed
git diff --check                                        passed
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
| no active dataset | `describe` | typed `NoActiveDataset` with exact text | unchanged; backend remains uninitialized |
| active eager local-Parquet dataset | `describe` | owned equal `DescribeResult` | unchanged |
| prior active dataset | failed replacement `use`, then `describe` | prior dataset metadata | failed staging does not publish |

The result owns a clone of the existing `DatasetInfo`; no DuckDB handle,
statement, Arrow value, raw pointer, or foreign lifetime crosses the API.
Labels, panel metadata, lazy/materialized behavior, and other inspection
commands remain explicit deferrals.

## Hosted acceptance and completion state

Draft PR #31 is
[`Runtime: execute bounded eager-session describe`](https://github.com/SaehwanPark/tabdat-explore-rs/pull/31),
opened before implementation. Current-head Rust, policy, and runtime workflow
links, independent-review findings, ready/merge state, branch cleanup, and
post-merge `main` verification will be appended before this slice is marked
accepted.

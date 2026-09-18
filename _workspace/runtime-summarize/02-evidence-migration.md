# Bounded runtime `summarize` migration evidence

Status: implementation and local validation complete; acceptance pending
independent review, PR-head checks, merge, and post-merge hosted evidence.

Producer: task owner, with pinned oracle evidence and independent runtime review

Consumers: reviewers and the next runtime maintainer

Boundary: pinned Python execution contract → Rust-owned eager-session summaries

## Authority and contract inputs

The Python oracle is the clean sibling checkout `../tabdat-explore` at pinned
commit `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, package `0.25.0`, and Python
`3.13.3`. Its `uv.lock` SHA-256 is
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`; the
checkout was clean and no dependency synchronization or source edits were
performed. The bounded contract is in `01-contract.md`.

The accepted Rust prerequisites are the eager local-Parquet `use` session in
PR #22, read-only cached `describe` in PR #31, cached eager `count` in PR #32,
and owned eager `head`/`tail` previews in PRs #33/#34. This slice consumes their
owned active metadata and private DuckDB relation; it does not add lazy
materialization, transforms, or presentation surfaces.

## Revisions and changed paths

- `96b0eef`: recovered the summarize contract and opened draft PR #35;
- `862977a`: added owned summary results, eager DuckDB aggregates, typed
  diagnostics, and state/NULL/order/type tests; and
- documentation closeout and hosted evidence: pending acceptance.

PR #35 ([Execute bounded runtime summarize](https://github.com/SaehwanPark/tabdat-explore-rs/pull/35))
was opened as a draft before implementation.

Changed implementation and evidence paths:

- `crates/tabdat-runtime/src/lib.rs`: `ExecutionResult::Summarize`, owned
  `SummaryRow`/`SummarizeResult`, numeric type selection, exact diagnostics,
  and aggregate/value ownership;
- `crates/tabdat-runtime/tests/use_contract.rs`: fresh-session, parser-to-
  session, explicit/default/duplicate order, decimal/null/all-null,
  validation, repeated-read, and failed-replacement coverage; and
- `_workspace/runtime-summarize/`: contract, migration evidence, and review
  record.

No dependency, unsafe-code, relation-writer, filesystem-writer,
serialization, CLI, or MCP surface changed.

## Pinned oracle probe

The focused command was run at the pinned revision:

```sh
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_executor.py -k \
  'summarize or test_phase_3_inspection_commands_require_active_dataset'
```

Observed result:

```text
13 passed, 404 deselected in 1.01s
```

The authoritative no-active error is exactly:
`summarize requires an active dataset; run use <path> first`.

The fixture evidence covers requested and default schema-order selection,
explicit duplicate preservation, count/mean/sample-SD/minimum/maximum, exact
decimal ownership, ignored nulls, all-null nullable aggregates, unknown and
non-numeric validation, and read-only active metadata.

## Rust and policy checks

At implementation revision `862977a`, the required local checks passed:

```text
cargo fmt --all -- --check                              passed
git diff --check                                        passed
cargo check --locked --workspace --all-targets          passed
cargo test --locked --workspace --all-targets           passed
  root scaffold: 1 test passed
  tabdat-language: 42 unit + 32 public integration tests passed
  tabdat-runtime: 7 unit + 40 integration tests passed
cargo clippy --locked --workspace --all-targets -- -D warnings
                                                         passed
```

The policy checks passed locally:

```text
cargo deny check                                        advisories, bans, licenses, sources ok
cargo audit -D warnings                                 passed; no vulnerabilities reported
metadata-driven cargo geiger                            running/record pending
```

The geiger loop follows the all-package assertion in `CONTRIBUTING.md`: it
resolves every workspace package from locked metadata, checks
`forbid(unsafe_code)` and zero first-party unsafe usage, and treats transitive
dependency inventory warnings as non-first-party findings. Its final result
will be recorded before the PR is marked ready.

## State and parity evidence

The implementation is read-only after `use` publishes an eager relation:

| Before | Input | Result | After |
| --- | --- | --- | --- |
| no active dataset | `summarize` | typed `NoActiveDataset` with exact text | unchanged; backend remains uninitialized |
| active eager local-Parquet dataset | explicit numeric variables | owned rows in requested order, including duplicates | unchanged |
| active eager local-Parquet dataset | empty variable list | numeric columns in active schema order | unchanged |
| active eager local-Parquet dataset | null/all-null numeric values | non-null count and nullable aggregate fields | unchanged |
| unknown/non-numeric/default-without-numeric input | `summarize` | typed exact diagnostic | unchanged |
| aggregate or supported-value conversion fails | `summarize` | typed `SummaryFailed` displayed as `summarize failed for variable: <name>` | active metadata remains exactly as before |

Minimum and maximum reuse the owned `CellValue` boundary; mean and sample
standard deviation are nullable `f64`, and count is an owned `u64`.
Unsupported logical/container values remain an explicit summary failure rather
than leaking DuckDB or Arrow values. Lazy/materialized summaries, grouped
execution, transforms, labels, formatting, CLI, and MCP output remain explicit
deferrals.

## Hosted acceptance and completion state

PR-head baseline, runtime, and policy checks are pending at the time this
evidence was drafted. After independent review, PR #35 will be marked ready,
squash-merged, and its temporary branch deleted locally and remotely. The
roadmap checkbox and this document will be updated only after post-merge main
workflows and docs-inclusive validation are green.

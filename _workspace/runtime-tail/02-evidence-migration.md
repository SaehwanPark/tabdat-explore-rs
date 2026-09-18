# Bounded runtime `tail` migration evidence

Status: accepted after PR #34 squash merge `1c5affa`; post-merge main
verification and docs-inclusive hosted workflows are green.

Producer: task owner, with pinned oracle evidence and independent runtime review

Consumers: reviewers and the next runtime maintainer

Boundary: pinned Python execution contract → Rust-owned eager-session preview

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
and owned eager `head` in PR #33. This slice consumes their owned active
metadata and private DuckDB relation; it does not add lazy materialization,
transforms, or presentation surfaces.

## Revisions and changed paths

- `8b0e8c6`: frozen contract and draft-PR handoff;
- `477890c`: owned tail dispatch, descending bounded relation query with
  insertion-order restoration, and state/limit/value tests; and
- `8ef4932`: current-state documentation corrections for eager tail support.

PR #34 ([Execute bounded runtime tail](https://github.com/SaehwanPark/tabdat-explore-rs/pull/34))
was opened as a draft before implementation.

Changed implementation and evidence paths:

- `crates/tabdat-runtime/src/lib.rs`: `ExecutionResult::Tail`, shared owned
  preview execution, oracle-aligned tail ordering, and typed failures;
- `crates/tabdat-runtime/tests/use_contract.rs`: exact fresh-session,
  parser-to-session, default/zero/oversized, repeated-read, value-conversion,
  nontrivial-order, and failed-replacement coverage;
- `crates/tabdat-language/src/lib.rs`: current-state comment for executable
  bounded tail syntax; and
- `SPEC.md`, ADR 0007, and the eager-runtime handoff: stale bounded-inspection
  wording corrected to include tail.

No dependency, unsafe-code, relation-mutation, filesystem-writer,
serialization, CLI, or MCP surface changed.

## Pinned oracle probe

The focused command was run at the pinned revision:

```sh
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_executor.py -k \
  'test_tail_returns_last_rows or \
   test_active_row_order_is_consistent_for_previews_and_filters or \
   test_phase_3_inspection_commands_require_active_dataset'
```

Observed result:

```text
10 passed, 407 deselected in 1.01s
```

The authoritative no-active error is exactly:
`tail requires an active dataset; run use <path> first`. The success fixture
preserves schema order, source insertion order, exact decimals, text, and nulls.

## Rust checks

At current implementation revision `8ef4932`, the required local checks passed:

```text
cargo fmt --all -- --check                              passed
git diff --check                                        passed
cargo check --locked --workspace --all-targets          passed
cargo test --locked --workspace --all-targets           passed
  root scaffold: 1 test passed
  tabdat-language: 42 unit + 32 public integration tests passed
  tabdat-runtime: 6 unit + 33 integration tests passed
cargo clippy --locked --workspace --all-targets -- -D warnings
                                                         passed
```

The policy checks passed locally:

```text
cargo deny check                                        advisories, bans, licenses, sources ok
cargo audit -D warnings                                 passed; no vulnerabilities reported
metadata-driven cargo geiger                            passed; first-party packages
                                                         reported forbid(unsafe_code)
                                                         and zero first-party unsafe usage
```

The geiger loop follows the all-package assertion in `CONTRIBUTING.md`: it
resolves every workspace package from locked metadata, checks
`forbid(unsafe_code)` and zero first-party unsafe usage, and treats transitive
dependency inventory warnings as non-first-party findings.

## State and parity evidence

The implementation is read-only after `use` publishes an eager relation:

| Before | Input | Result | After |
| --- | --- | --- | --- |
| no active dataset | `tail` | typed `NoActiveDataset` with exact text | unchanged; backend remains uninitialized |
| active eager local-Parquet dataset | `tail`, finite supported limit | owned columns and at most that many final rows restored to insertion order | unchanged |
| active eager local-Parquet dataset | `tail 0` | schema-order columns and empty rows without a relation query | unchanged |
| active eager local-Parquet dataset | default or oversized finite limit | all available rows in insertion order | unchanged |
| limit conversion, query, or value conversion fails | `tail` | typed `PreviewFailed { command: "tail" }`, displayed as `tail failed` | active metadata remains exactly as before |

Signed/unsigned integer, floating-point, exact decimal, UTF-8 text, binary,
boolean, and null values are copied into the Rust-owned enum. Unsupported
logical/container values fail explicitly rather than leaking a DuckDB or Arrow
type. Lazy/materialized previews, transforms, labels, formatting, CLI, and MCP
output remain explicit deferrals.

## Hosted acceptance and completion state

PR #34 was opened as a draft before implementation, independently reviewed,
marked ready after all current-head checks passed, and squash-merged as
`1c5affac54c9450d732652ae303509a6bfc9cf63`. The temporary
`feat/runtime-tail` branch was deleted locally and remotely.

Current-head PR checks were green:

- [dependency and unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35355567633/job/105634186009);
- [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35355567633/job/105634185732);
- [tabdat-runtime on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35355567718/job/105634177177);
- [ReadStat feasibility](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35355567606),
  including its [Rust job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35355567606/job/105633965730);
- [libgretl feasibility](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35355567625/job/105634076038); and
- [libgretl OLS](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35355567676/job/105634084365).

Post-merge `main` workflows passed at `1c5affac54c9450d732652ae303509a6bfc9cf63`:

- [main dependency and unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35357992959/job/105641975529);
- [main Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35357992959/job/105641975794);
- [main tabdat-runtime](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35357992778/job/105641971830);
- [main ReadStat](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35357992853/job/105641971895);
- [main libgretl feasibility](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35357993037/job/105641973315); and
- [main libgretl OLS](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35357992793/job/105641971649).

The roadmap `tail` checkbox is accepted only with this merge and its post-merge
evidence; broader session, lazy/materialized, formatting, CLI, and MCP work
remain deferred.

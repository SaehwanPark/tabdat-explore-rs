# Bounded runtime `head` migration evidence

Status: accepted after PR #33 squash merge `b107251`; post-merge `main`
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
PR #22, read-only cached `describe` in PR #31, and cached eager `count` in
PR #32. This slice consumes their owned active metadata and private DuckDB
relation; it does not add lazy materialization, transforms, or presentation
surfaces.

## Revisions and changed paths

- `0173160`: frozen contract and draft-PR handoff;
- `414d8da`: owned preview/value surfaces, eager head dispatch, conversion and
  state/atomicity tests, and stale current-state documentation corrections;
- `e9a869c`: oracle-aligned row-number ordering, exact `i64::MAX` acceptance,
  nontrivial insertion-order regression, stale `Head` comment correction, and
  explicit rejection of `GEOMETRY` values;
- PR #33: [Execute bounded runtime head](https://github.com/SaehwanPark/tabdat-explore-rs/pull/33),
  opened before implementation.

Changed implementation paths:

- `crates/tabdat-runtime/src/lib.rs`: owned `CellValue`/`PreviewResult`, typed
  preview failure, `Command::Head` dispatch, oracle-aligned bounded relation
  query, and backend-value conversion;
- `crates/tabdat-runtime/tests/use_contract.rs`: exact fresh-session,
  parser-to-session, default/zero/oversized, repeated-read, value-conversion,
  and failed-replacement coverage;
- `_workspace/runtime-head/01-contract.md`: bounded authority and stop
  conditions.

No parser grammar, relation mutation, filesystem writer, dependency, unsafe-code,
serialization, CLI, or MCP surface changed.

## Pinned oracle probe

The focused command was run at the pinned revision:

```sh
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_executor.py -k \
  'test_head_returns_first_rows or \
   test_phase_3_inspection_commands_require_active_dataset'
```

Observed result:

```text
7 passed, 410 deselected in 1.52s
```

The authoritative no-active error is exactly:
`head requires an active dataset; run use <path> first`. The success fixture
preserves schema order, source insertion order, exact decimals, text, and nulls.

## Rust checks

At implementation revision `e9a869c`, the required local checks passed:

```text
cargo fmt --all -- --check                              passed
git diff --check                                        passed
cargo check --locked --workspace --all-targets          passed
cargo test --locked --workspace --all-targets           passed
  root scaffold: 1 test passed
  tabdat-language: 42 unit + 32 public integration tests passed
  tabdat-runtime: 5 unit + 25 integration tests passed
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

The implementation is read-only after `use` publishes an eager relation:

| Before | Input | Result | After |
| --- | --- | --- | --- |
| no active dataset | `head` | typed `NoActiveDataset` with exact text | unchanged; backend remains uninitialized |
| active eager local-Parquet dataset | `head`, finite supported limit | owned columns and at most that many owned rows in insertion order | unchanged |
| active eager local-Parquet dataset | `head 0` | schema-order columns and empty rows without a relation query | unchanged |
| active eager local-Parquet dataset | default or oversized finite limit | all available rows, bounded by the relation | unchanged |
| limit conversion, query, or value conversion fails | `head` | typed `PreviewFailed { command: "head" }`, displayed as `head failed` | active metadata remains exactly as before |

Signed/unsigned integer, floating-point, exact decimal, UTF-8 text, binary,
boolean, and null values are copied into the Rust-owned enum. Unsupported
logical/container values fail explicitly rather than leaking a DuckDB or Arrow
type. Lazy/materialized previews, `tail`, transforms, labels, formatting, CLI,
and MCP output remain explicit deferrals.

## Hosted acceptance and completion state

PR #33 was opened before implementation, reviewed independently, marked ready
after all current-head checks passed, and squash-merged as
`b107251d8cb80a228a4632eec41414c673c8e28f`. The temporary
`feat/runtime-head` branch was deleted locally and remotely. Current-head PR
checks were green:

- [dependency and unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35347629750/job/105607848665);
- [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35347629750/job/105607848905); and
- [tabdat-runtime on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35347629813/job/105607850297).

Post-merge `main` verification passed at `b107251d8cb80a228a4632eec41414c673c8e28f`:

- [main dependency and unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35349781739/job/105614878164);
- [main Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35349781739/job/105614877899); and
- [main tabdat-runtime](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35349781741/job/105614877774).

The same merge push passed the [ReadStat feasibility spike](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35349781792), [libgretl OLS Rust spike](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35349781724), and [libgretl feasibility spike](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35349781803).

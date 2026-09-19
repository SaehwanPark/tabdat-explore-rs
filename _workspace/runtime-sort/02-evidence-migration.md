# Runtime `sort` evidence and migration record

Status: implementation and local review complete; hosted acceptance is pending.

## Authority and oracle evidence

The behavior authority is the pinned Python checkout recorded in
[`01-contract.md`](01-contract.md): `tabdat-explore` revision
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, Python `3.13.3`, and the recorded
`uv.lock` SHA-256
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.

The recovered oracle was checked out in the isolated sibling clone
`C:\Users\saehwan\repos\tabdat-python-oracle` with the pinned revision and
clean working tree. The focused sort suite reported:

```text
uv run --no-sync pytest -q -p no:cacheprovider tests/test_sort.py
7 passed in 0.81s
```

The probe covers stable native ordering, NULLs last, parser behavior, metadata
preservation, unknown-variable atomicity, and the Python lazy-engine paths that
remain outside this Rust contract.

## Rust implementation evidence

The implementation is the library-only eager local-Parquet path in
`crates/tabdat-runtime`:

- `SortResult` and `ExecutionResult::Sort` own the post-transform metadata;
- source validation occurs before backend staging;
- the backend stages a source-order relation with a collision-free private row
  ordinal, quoted ascending `NULLS LAST` keys, and the ordinal as the final
  stable tie-breaker;
- staged schema and row count are checked before the shared transaction
  publishes the relation; and
- validation, stage, inspection, or publication failures leave the previously
  published metadata and private active relation unchanged.

Focused coverage is in
`crates/tabdat-runtime/tests/sort_contract.rs` (4 integration tests), the
private-backend no-active and mismatched-relation regressions in
`crates/tabdat-runtime/src/lib.rs`, and the updated no-active regression in
`crates/tabdat-runtime/tests/use_contract.rs`.

The contract checkpoint is commit `a2168be`; the implementation commit is
`11d2de3`; and the review-driven identifier-collision correction is `7e0abb6`.

## Local verification

The following checks passed locally on the final implementation head:

```text
cargo fmt --all -- --check                         passed
cargo check --locked --workspace --all-targets    passed
cargo test --locked --workspace --all-targets     passed
cargo clippy --locked --workspace --all-targets -- -D warnings
                                                     passed
git diff --check                                  passed
cargo deny check                                  passed
cargo audit -D warnings                           passed
```

The focused runtime suite reported 4 passed sort integration tests, 31 passed
runtime library tests, and 65 passed runtime `use_contract` tests. The focused
language suite also passed 42 unit tests and 42 parser-contract tests. The
metadata-driven geiger loop passed for `tabdat-explore-rs`, `tabdat-language`,
and `tabdat-runtime`, with `forbids_unsafe=true` and zero first-party unsafe
counts for each package. Dependency inventory warnings were not present in
this run.

## Hosted acceptance

Draft PR [#50](https://github.com/SaehwanPark/tabdat-explore-rs/pull/50) was
opened at the contract checkpoint and contains the bounded contract,
implementation, focused tests, and review evidence. It must be marked ready
only after the final PR-head CI and runtime-boundary jobs pass. Their direct
links and the later squash-merge commit will be recorded here before closeout.

## Deviations and deferrals

The Rust runtime intentionally does not claim the Python executor's panel or
label metadata updates, lazy or materialized execution, descending keys,
`gsort`, expression keys, `last_operation`, formatting, CLI/JSON/MCP surfaces,
or broad transform sequencing. These remain explicit future slices rather than
hidden behavior.

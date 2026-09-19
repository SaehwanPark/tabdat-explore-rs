# Runtime `gsort` evidence and migration record

Status: accepted and verified on `main` at merge commit
[`c06ed5a`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/c06ed5a7d913aaa5a4ddbdf58f79149a8b81a79e).

## Authority and oracle evidence

The behavior authority is the pinned Python checkout recorded in
[`01-contract.md`](01-contract.md): `tabdat-explore` revision
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, Python `3.13.3`, and the recorded
`uv.lock` SHA-256
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.

The recovered oracle was checked out in the isolated sibling clone
`C:\Users\saehwan\repos\tabdat-python-oracle` with the pinned revision and
clean working tree. The focused gsort suite reported:

```text
uv run --no-sync pytest -q -p no:cacheprovider tests/test_gsort.py
8 passed in 0.70s
```

The probe covers signed-key parsing, stable mixed-direction ordering, NULLs
last, metadata preservation, quoted identifiers, and unknown-variable
atomicity. The Python lazy-engine, panel/label, CLI, and JSON paths remain
outside this Rust contract.

## Rust implementation evidence

The implementation is the library-only eager local-Parquet path in
`crates/tabdat-runtime`:

- `GsortResult` and `ExecutionResult::Gsort` own the post-transform metadata;
- key variables and directions are extracted from the parser-owned `SortKey`
  values before backend staging;
- source validation occurs before staging;
- the backend shares the accepted `sort` ordinal/publication path while
  emitting quoted per-key `ASC` or `DESC` clauses with `NULLS LAST`; and
- staged schema and row count are inspected before publication, with the
  previously published metadata and private active relation preserved on
  failure.

Focused coverage is in
`crates/tabdat-runtime/tests/gsort_contract.rs` (4 integration tests), the
private-backend no-active and mismatched-relation regressions in
`crates/tabdat-runtime/src/lib.rs`, and the updated no-active regression in
`crates/tabdat-runtime/tests/use_contract.rs`.

The contract checkpoint is commit `0c71abd`; the implementation commit is
`013fa8a`.

## Local verification

The following checks passed locally on implementation head `013fa8a`:

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

Focused suites reported 4 passed gsort integration tests, 33 passed runtime
library tests, 65 passed runtime `use_contract` tests, 42 passed language unit
tests, and 42 passed parser-contract tests. The metadata-driven geiger loop
passed for `tabdat-explore-rs`, `tabdat-language`, and `tabdat-runtime`, with
`forbids_unsafe=true`, zero first-party unsafe counts, and status 0 for each
package.

## Hosted acceptance

Draft PR [#51](https://github.com/SaehwanPark/tabdat-explore-rs/pull/51) was
opened at the contract checkpoint, marked ready after local review and green
PR-head workflows, and squash-merged as
[`c06ed5a`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/c06ed5a7d913aaa5a4ddbdf58f79149a8b81a79e).

PR-head acceptance passed at final head
[`04b0d01`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/04b0d01e626cf002ab43403b295e650d46ba59be):

- [CI workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35464208959)
  ([Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35464208959/job/105953358264),
  [policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35464208959/job/105953358072));
- [runtime workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35464208962)
  ([Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35464208962/job/105953357646)).

Merge-head acceptance passed after the squash merge:

- [CI workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35465450690)
  ([Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35465450690/job/105956827450),
  [policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35465450690/job/105956827635));
- [runtime workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35465450673)
  ([Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35465450673/job/105956827373)).

The combined sort/gsort documentation closeout is recorded in the current
`main` documentation commit.

## Deviations and deferrals

The Rust runtime intentionally does not claim the Python executor's lazy or
materialized execution, panel or label metadata updates, `last_operation`,
formatting, CLI/JSON/MCP surfaces, or broad transform sequencing. These remain
explicit future slices rather than hidden behavior.

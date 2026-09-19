# Runtime recode evidence and migration record

Status: implementation checkpoint on branch feat/runtime-recode; the code
head passed the final PR-head workflows before the documentation closeout.

## Authority and oracle evidence

The behavior authority is the pinned Python checkout recorded in
[01-contract.md](01-contract.md): tabdat-explore revision
16b45d9b66b0d80f32d4d220e84d81bc5180bdbe, tree
601b236788872323af9277d2276a236154a0f129, Python 3.13.3, and the recorded
uv.lock SHA-256
0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239.

The recovered oracle was checked out in the isolated sibling clone
C:\Users\saehwan\repos\tabdat-python-oracle with the pinned revision and a
clean working tree. The focused recode probe reported:

    uv run --no-sync pytest -q -p no:cacheprovider tests/test_parser.py tests/test_executor.py -k recode
    3 passed, 903 deselected in 1.94s

The probe and source comparison cover parser-owned rule/value forms, ordered
CASE behavior, missingness and else fallback, generate/replace placement, and
executor validation. Lazy/materialized execution, panel and label metadata,
last-operation state, and CLI/JSON/MCP output remain outside this Rust slice.

## Rust implementation evidence

The implementation is the library-only eager local-Parquet path:

- crates/tabdat-language owns typed recode values, inclusive numeric ranges,
  rule order, and exactly one generate or replace target mode;
- crates/tabdat-runtime owns source/target/range validation, SQL identifier and
  literal quoting, ordered CASE compilation, staged projection, schema/count
  inspection, and publication;
- RecodeResult and ExecutionResult::Recode report the transformed dataset; and
- the backend publishes the staged relation only after all checks succeed,
  preserving the prior relation and metadata on validation or backend failure.

Focused Rust coverage is in
crates/tabdat-language/tests/parser_contract.rs and
crates/tabdat-runtime/tests/recode_contract.rs. It includes no-active
behavior without backend initialization, numeric ranges, missing/nonmissing and
else rules, text output, quoted identifiers including an embedded quote, empty
relations, validation atomicity, and staged/backend failure atomicity.

The contract checkpoint, implementation, formatting correction, and final
atomicity-test commits are respectively:

- afa3867 docs: define bounded recode contract;
- 2418642 runtime: execute bounded eager recode;
- e16d061 style: format recode runtime; and
- e846ea8 test: cover recode backend failure atomicity.

## Local verification

The locked Rust baseline and policy checks passed during implementation:

    cargo fmt --all -- --check
    cargo check --locked --workspace --all-targets
    cargo test --locked --workspace --all-targets
    cargo clippy --locked --workspace --all-targets -- -D warnings
    git diff --check
    cargo deny check
    cargo audit -D warnings

The final code head also passed the focused parser and runtime recode suites.
The metadata-driven geiger loop passed for tabdat-explore-rs,
tabdat-language, and tabdat-runtime with forbid-unsafe enabled, zero
first-party unsafe counts, and status 0 for each package.

## Hosted acceptance

Draft PR [#52](https://github.com/SaehwanPark/tabdat-explore-rs/pull/52) was
opened at the contract checkpoint. Final PR-head acceptance for code head
e846ea8 passed:

- [CI workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35468724919)
  with [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35468724919/job/105965715052)
  and [dependency/unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35468724919/job/105965714891);
 and
- [runtime workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35468724865)
  with [Linux runtime checks](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35468724865/job/105965713161).

The documentation commit that follows will trigger a fresh PR-head workflow;
its links and the squash merge/merge-head results will be added during the
main-branch closeout.

## Deviations and deferrals

This record does not claim a usable TabDat CLI, Python/R runtime dependency
parity, lazy or materialized execution, panel/label metadata, last-operation
state, formatting, output adapters, or broad transform sequencing. Those
remain separate roadmap slices.

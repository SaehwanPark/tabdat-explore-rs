# Bounded runtime `isid` migration evidence

Status: accepted bounded eager-runtime slice

Producer: task owner, with pinned oracle evidence and independent runtime review

Consumers: reviewers and the next runtime maintainer

Boundary: pinned Python execution contract → Rust-owned eager-session key
uniqueness result

## Authority and contract inputs

The pinned authority is `../tabdat-explore` at commit
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, package `0.25.0`, Python `3.13.3`,
and `uv.lock` SHA-256
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`. The
checkout was clean and no dependency synchronization or source edits were
performed. The bounded contract is in `01-contract.md`.

The accepted Rust prerequisites are the eager local-Parquet `use` session in
PR #22, read-only `describe` in PR #31, cached `count` in PR #32, owned
`head`/`tail` in PRs #33/#34, eager numeric `summarize` in PR #35, eager column
profiles in PR #36, SQL-NULL missingness in PR #37, and duplicate-key grouping
in PR #38. This slice consumes their owned active metadata and private DuckDB
relation.

## Recovery evidence

Focused Python validation at the pinned revision:

```text
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_isid.py
20 passed in 0.54s
```

## Implementation evidence

Commit `3a183ea` (`runtime: add bounded eager isid check`) implements the
bounded contract. It adds the owned `IsidResult`, typed diagnostics, session
dispatch, collision-safe grouped DuckDB aggregation, checked conversions, and
read-only/state-preservation coverage. The implementation remains eager and
local-Parquet only as scoped above.

Focused and workspace checks passed on the draft branch:

```text
cargo test --locked -p tabdat-runtime --test use_contract isid -- --nocapture
4 passed
cargo test --locked -p tabdat-runtime --all-targets
15 unit + 63 integration tests passed
cargo test --locked --workspace --all-targets
all workspace targets passed
cargo fmt --all -- --check
git diff --check
cargo check --locked --workspace --all-targets
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo deny check
cargo audit -D warnings
metadata-driven cargo geiger loop
all passed; first-party packages reported forbid(unsafe_code) and zero unsafe usage
```

The pinned Python focused module remains `20 passed in 0.54s`. The independent
review in `03-review.md` reported no actionable findings.

## Hosted acceptance and cleanup

PR #39 was marked ready only after all PR-head gates passed for commit `7feefdf`:

- [CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35401533278),
  including [dependency/unsafe policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35401533278/job/105782364157)
  and [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35401533278/job/105782364183);
- [TabDat runtime boundary](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35401533227)
  ([Linux job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35401533227/job/105782597582)).

PR #39 was squash-merged as
[`e04def0`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/e04def037a829db4cb4f969cd364aeb9aacae348)
and the temporary `feat/runtime-isid` branch was deleted locally and remotely.
All workflows triggered by that merge completed successfully:

- [CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35403193938),
  including [dependency/unsafe policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35403193938/job/105787356806)
  and [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35403193938/job/105787357091);
- [TabDat runtime boundary](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35403193942)
  ([Linux job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35403193942/job/105787356790)).
- [ReadStat feasibility](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35404774662)
  ([spike job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35404774662/job/105792139601),
   [Rust spike job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35404774662/job/105792139742));
- [libgretl feasibility](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35404774663)
  ([Linux job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35404774663/job/105792138501)); and
- [libgretl OLS](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35404774708)
  ([Linux job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35404774708/job/105792138553)).

This bounded migration is accepted. Lazy/materialized execution,
`last_operation`, labels, wildcard/range expansion, formatting, CLI/REPL,
JSON/MCP, and broader relation APIs remain explicit deferrals.

## Planned implementation and state checks

The bounded implementation will add an owned `IsidResult`, typed unknown,
semantic-failure, and backend-failure diagnostics, eager session dispatch, and
one quoted DuckDB grouped aggregate. It must preserve request order and
duplicates, count NULL-containing groups correctly, reject duplicates even
with `missok`, and leave active metadata unchanged on every failure.

Required local and hosted evidence will be recorded after implementation:

- `cargo fmt --all -- --check` and `git diff --check`;
- locked workspace check/test/Clippy and policy scans;
- metadata-driven first-party unsafe inventory;
- focused runtime/parser tests; and
- all required PR-head and post-merge workflows.

## Deferred scope

Lazy/materialized execution, `last_operation`, labels/panel metadata,
wildcard/range expansion, formatting, CLI, JSON, MCP, and the unchecked
`datasignature`/`assert`/other inspection surfaces remain explicit deferrals.

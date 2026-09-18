# Bounded runtime `duplicates` migration evidence

Status: accepted after PR #38 squash merge `6a10039`; independent review,
hosted checks, and temporary branch cleanup complete

Producer: task owner, with pinned oracle evidence and independent runtime review

Consumers: reviewers and the next runtime maintainer

Boundary: pinned Python execution contract → Rust-owned eager-session duplicate
reports

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
`head`/`tail` in PRs #33/#34, eager numeric `summarize` in PR #35, eager
column profiles in PR #36, and SQL-NULL missingness in PR #37. This slice
consumes their owned active metadata and private DuckDB relation.

## Recovery evidence

Focused Python validation at the pinned revision:

```text
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_duplicates.py
14 passed in 0.84s
```

The implementation evidence, local checks, review disposition, and hosted
acceptance are recorded below. PR #38 was opened as a draft before
implementation and marked ready only after all required PR-head workflows were
green.

## Implementation revision and local checks

- `535d37c`: added owned `DuplicatesResult`, typed unknown/failure diagnostics,
  eager session dispatch, a quoted grouped DuckDB aggregate with a
  collision-safe internal count alias, and focused unit/integration coverage.
- `74f8850`: recorded implementation evidence;
- `1fef73b`: recorded the independent review; and
- `69b4eb5`: recorded the first-party unsafe-inventory evidence.

PR #38 ([Add bounded eager duplicates report](https://github.com/SaehwanPark/tabdat-explore-rs/pull/38))
was squash-merged as [`6a10039`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/6a10039d6b3a062e7f98343e3e59489b32d6b977).
The temporary `feat/runtime-duplicates` branch was deleted locally and
remotely.

The pinned Python probe passed with `14 passed in 0.46s`. At implementation
head `535d37c`, the required Rust checks passed:

```text
cargo fmt --all -- --check                              passed
git diff --check                                        passed
cargo check --locked --workspace --all-targets          passed
cargo test --locked --workspace --all-targets           passed
  root scaffold: 1 test passed
  tabdat-language: 42 unit + 32 public integration tests passed
  tabdat-runtime: 13 unit + 60 integration tests passed
cargo clippy --locked --workspace --all-targets -- -D warnings
                                                         passed
```

Policy checks also passed locally: `cargo deny check` reported advisories,
bans, licenses, and sources ok; `cargo audit -D warnings` completed without
reported vulnerabilities. The metadata-driven `cargo geiger` loop completed
with first-party-safe reports for `tabdat-explore-rs`, `tabdat-language`, and
`tabdat-runtime`; hosted workflow evidence remains part of the acceptance gate.

The independent review of `535d37c` found no actionable issues. It confirmed
the report alias, NULL-equal grouping, checked aggregate arithmetic, collision-
safe aliases, exact diagnostics, and read-only failure preservation. The review
record is in `03-review.md`.

The required PR-head workflows passed before merge:

- [CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35394059810),
  including [policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35394059810/job/105758872778)
  and [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35394059810/job/105758873113);
- [runtime boundary](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35394059703),
  including the [Linux job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35394059703/job/105758908679).

The implementation is read-only after `use` publishes an eager relation:

| Before | Input | Result | After |
| --- | --- | --- | --- |
| no active dataset | `duplicates` | typed `NoActiveDataset` with exact text | unchanged; backend remains uninitialized |
| active eager local-Parquet dataset | selected keys | owned aggregate in requested/default order | unchanged |
| repeated NULL keys | any key list | NULL values group together; exact duplicate metrics | unchanged |
| unknown keys | `duplicates` | typed exact diagnostic before query | unchanged |
| missing/dropped active relation | `duplicates` | typed `DuplicatesFailed` displayed as `duplicates failed` | active metadata remains exactly as before |

## Post-merge verification

The documentation closeout commit [`f87130a`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/f87130ac66af5dccc17d27f78b7dcb4136cf53f1)
triggered the standard post-merge matrix, and every workflow completed
successfully:

- [CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35396216098),
  including [dependency/unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35396216098/job/105765906363)
  and [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35396216098/job/105765906370);
- [TabDat runtime boundary](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35396216172)
  ([Linux job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35396216172/job/105765909267));
- [ReadStat feasibility](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35396216212),
  with [native](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35396216212/job/105765524117)
  and [Rust](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35396216212/job/105765524411)
  jobs;
- [libgretl feasibility](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35396216133)
  ([Linux job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35396216133/job/105765534876)); and
- [libgretl OLS](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35396216126)
  ([Linux job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35396216126/job/105765523772)).

The superseded CI/runtime push runs for squash commit `6a10039` were cancelled
by the repository concurrency group when this closeout commit arrived; the
authoritative closeout matrix above is green.

## Deferred scope

Lazy/materialized execution, `last_operation`, labels, wildcard/range
expansion, formatting, CLI, JSON, MCP, and the unchecked `isid`/other
inspection surfaces remain explicit deferrals. No second data engine,
dependency, unsafe code, relation writer, serializer, or public value lifetime
is introduced by this bounded slice.

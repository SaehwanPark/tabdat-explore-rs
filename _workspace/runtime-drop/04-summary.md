# Bounded eager-runtime `drop` closeout

Status: accepted and verified on `main` at `50cf80c`.

Draft PR [#43](https://github.com/SaehwanPark/tabdat-explore-rs/pull/43) was
opened before implementation, independently reviewed, marked ready after the
required checks passed, and squash-merged as
[`50cf80c`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/50cf80c71d0c60f14bc8a96547cc93e3ed322bc2).
The temporary `feat/runtime-drop` branch was removed locally and remotely.

## Accepted scope

The slice adds `drop <explicit-varlist>` complement projection over the active
eager local-Parquet DuckDB relation. It validates all requested names before
mutation, preserves the surviving schema in source order plus row order, row
count, NULLs, and source metadata, supports quoted/backtick identifiers and
duplicate requests, and publishes through the staged atomic path. Unknown
variables, all-column removal, and backend/publication failures preserve the
published metadata and private relation. The typed result is `DropResult`.

Predicate-form `drop if <expression>`, lazy/materialized execution,
wildcard/range expansion, labels/panel metadata, `last_operation`, formatting,
CLI/REPL, JSON/MCP, and broader transformation sequencing remain deferred.

## Durable records

- [`01-contract.md`](01-contract.md) — pinned oracle contract and bounded scope;
- [`02-evidence-migration.md`](02-evidence-migration.md) — oracle, local,
  policy, PR-head, merge, branch-cleanup, and post-merge evidence;
- [`03-review.md`](03-review.md) — independent review and disposition;
- [SPEC verified-slice record](../../SPEC.md), [roadmap Phase 6.3](../../docs/TABDAT_RUST_PORT_ROADMAP.md),
  and [ADR 0007](../../docs/adr/0007-eager-parquet-duckdb-runtime-boundary.md).

## Post-merge verification

The merge-head code workflows passed:

- [CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35428243328),
  including [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35428243328/job/105857896234)
  and [dependency/unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35428243328/job/105857896085);
- [TabDat runtime boundary](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35428243325)
  ([Linux job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35428243325/job/105857896126)).

The documentation closeout commit `f764d89` passed the full native feasibility
matrix:

- [CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35429298624),
  [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35429298624/job/105860870955),
  and [dependency/unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35429298624/job/105860871020);
- [TabDat runtime boundary](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35429298645),
  [Linux job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35429298645/job/105860870889);
- [ReadStat feasibility](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35429298655),
  [ReadStat spike](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35429298655/job/105860871170),
  and [Rust spike](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35429298655/job/105860871190);
- [libgretl feasibility](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35429298639),
  [Linux job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35429298639/job/105860870926); and
- [libgretl OLS](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35429298638),
  [Linux job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35429298638/job/105860871137).

## Handoff

The next bounded roadmap candidate is `select`, with a separate contract and PR.
This slice does not establish predicate transforms, broad relation sequencing,
or general Phase 4 runtime/reporting completion.

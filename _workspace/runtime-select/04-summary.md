# Bounded eager-runtime `select` closeout

Status: accepted and verified on `main` at `228fa50`.

Draft PR [#44](https://github.com/SaehwanPark/tabdat-explore-rs/pull/44) was
opened before implementation, independently reviewed, marked ready after the
required checks passed, and squash-merged as
[`228fa50`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/228fa505dfeba3fb027590a2d430a28fa45e718f).
The temporary `feat/runtime-select` branch was removed locally and remotely.

## Accepted scope

The slice adds `select <explicit-varlist>` requested-order projection over the
active eager local-Parquet DuckDB relation. It validates all requested names
before mutation, rejects direct empty typed requests, preserves requested order,
deterministic duplicate projection names, row order, row count, NULLs, and
source metadata, supports quoted/backtick identifiers including embedded
quotes, and publishes through the shared staged atomic path. Unknown variables,
backend failures, and publication failures preserve the published metadata and
private relation. The typed result is `SelectResult`.

Predicate-form `select if <expression>`, lazy/materialized execution,
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

- [CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35432672560),
  including [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35432672560/job/105869917534)
  and [dependency/unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35432672560/job/105869917620);
- [TabDat runtime boundary](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35432672544)
  ([Linux job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35432672544/job/105869917478)).

The main docs closeout will trigger the full native feasibility matrix; its
final links will be appended to the evidence record before the next slice.

## Handoff

The next bounded roadmap candidate is `generate`, with a separate contract and
PR. This slice does not establish predicate transforms, broad relation
sequencing, or general Phase 4 runtime/reporting completion.

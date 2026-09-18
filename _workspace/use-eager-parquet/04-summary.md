# Eager local-Parquet runtime handoff

Status: accepted for bounded evaluation and merged on `main`.

PR [#22](https://github.com/SaehwanPark/tabdat-explore-rs/pull/22) was squash
merged as [`26dba2b`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/26dba2b6709e2a5bd16ea5e7f98463bf6d39d4e2),
after its implementation-head evidence and hosted matrix passed. The temporary
branch was removed locally and remotely. This artifact is the owner-to-maintainer
handoff for the bounded runtime slice; the contract, evidence, review, ADR, and
current-state files below are the durable record of what was accepted.

## Accepted scope

The slice implements one eager `Command::Use` of an existing local Parquet file
through a private DuckDB adapter. Broad `use` parity, lazy/remote/other-format
loading, general relation APIs, broad inspect execution beyond bounded
`describe`/`count`/`head`/`tail`, CLI/REPL, scripts,
reporting, JSON/MCP, and production DuckDB packaging remain deferred.

## Durable records

- [`01-contract.md`](01-contract.md) — pinned Python contract, Rust boundary,
  test contract, and explicit deviations.
- [`02-evidence-data.md`](02-evidence-data.md) — oracle, fixture, local/policy,
  native, platform, and hosted evidence.
- [`03-review.md`](03-review.md) — independent review findings and disposition.
- [`docs/adr/0007-eager-parquet-duckdb-runtime-boundary.md`](../../docs/adr/0007-eager-parquet-duckdb-runtime-boundary.md)
  — accepted bounded DuckDB decision and production deferrals.

## Post-merge verification

All workflows triggered by the squash commit completed successfully:

- [CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35239254970),
  including [Rust baseline job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35239254970/job/105263125043)
  and [dependency/unsafe-code policy job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35239254970/job/105263124673);
- [DuckDB feasibility](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35239254658)
  ([Linux job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35239254658/job/105263123598));
- [TabDat runtime boundary](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35239254673)
  ([Linux job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35239254673/job/105263122693));
- [ReadStat feasibility](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35239254720);
- [libgretl feasibility](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35239254529); and
- [libgretl OLS](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35239254679).

Local locked Cargo, dependency-policy, advisory, unsafe-inventory, pinned
Python, and exact eager-failure checks are recorded in `02-evidence-data.md`.

## Handoff

The runtime crate remains a library-only evaluation. It is not wired to the root
binary, does not provide a general data engine, and does not justify checking any
of the broad Phase 4 session, relation, load, inspect, transform, or reporting
gates. The next bounded roadmap slice should select a separate contract and PR;
`isid` syntax-only is the current candidate, with active-dataset execution and
key semantics deferred.

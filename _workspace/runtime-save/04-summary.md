# Bounded eager-runtime `save` closeout

Status: PR-head accepted and merged; merge-head verification pending.

## Delivered slice

PR [#70](https://github.com/SaehwanPark/tabdat-explore-rs/pull/70) adds
`save <path> [, replace]` execution for active eager local-Parquet relations.
The implementation validates targets, creates missing parents, gates overwrite,
writes through a parameterized DuckDB copy, returns owned output metadata, and
leaves the active session relation unchanged. Eight focused contract tests cover
success, round trips, transformed data, ordering, NULLs, empty relations, and
failure/recovery paths.

## Durable revisions

- contract: `a4bfbc2`;
- implementation: `8a51881`;
- review-gap closure: `e2ac065`, `e37a59f`;
- evidence inventory: `8f40c2a`; and
- squash merge: `9bbf804`.

## Validation

- Python save probe: `1 passed, 416 deselected`;
- Python parser/script oracle: `516 passed`;
- focused Rust save suite: `8 passed`;
- locked workspace format/check/test/Clippy and policy checks: passed locally;
- PR-head hosted workflows: all passed on `8f40c2a`; and
- main merge-head workflows: pending at the time of this closeout draft.

## Explicit deferrals

`export`, CSV/Feather/Arrow writers, lazy/materialized persistence, `~`
expansion and broader path normalization, atomic temporary-file replacement,
metadata/label/panel persistence, and CLI/JSON/MCP surfaces remain deferred.

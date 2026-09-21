# Bounded eager-runtime CSV `export` closeout

Status: accepted, merged, and verified on `main`.

## Delivered slice

PR [#72](https://github.com/SaehwanPark/tabdat-explore-rs/pull/72) adds
`export <path> [, replace]` execution for active eager local-Parquet relations
when the destination has a `.csv` extension. The implementation validates
targets, creates missing parents, gates overwrite, writes through a
parameterized DuckDB CSV copy with a header, returns owned output metadata, and
leaves the active session relation unchanged. Nine focused contract tests cover
exact output bytes, transformed data, ordering, quoting, NULLs, empty relations,
and failure/recovery paths.

## Durable revisions

- contract: `3b09266`;
- implementation and tests: `55c805b`.
- evidence/review: `cd3654c`;
- squash merge: `5cb5b34`.

## Validation

- Python export probe: `3 passed, 414 deselected`;
- Python parser/script oracle: `516 passed`;
- focused Rust export suite: `9 passed`;
- locked workspace format/check/test/Clippy and policy checks: passed locally;
- independent review: no actionable findings; and
- PR-head hosted workflows: all passed on `cd3654c`; and
- main merge-head workflows: [CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35556151175)
  and [tabdat-runtime](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35556151230)
  both passed on `5cb5b34`.

## Explicit deferrals

Parquet aliasing through `export`, Feather/Arrow writers, lazy/materialized
persistence, `~` expansion and broader path normalization, atomic temporary-file
replacement, metadata/label/panel persistence, and CLI/JSON/MCP surfaces remain
deferred.

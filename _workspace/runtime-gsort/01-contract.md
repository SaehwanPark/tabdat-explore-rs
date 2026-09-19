# Bounded eager-runtime `gsort` contract

Status: proposed at the contract checkpoint.

Producer: task owner, using `tabdat-migration` and `tabdat-data-semantics`.
Consumer: the bounded eager runtime implementation and its focused review.

## Scope

Execute the already-parsed form `gsort [+|-]varlist` against the active eager
local-Parquet DuckDB relation. This is the bounded directed-ordering extension
after the accepted eager `sort` path.

The runtime subset supports:

- one or more existing source columns in parser-provided key order;
- exact, case-sensitive identifier spelling, including quoted names and
  embedded double quotes;
- an optional ascending (`+` or no prefix) or descending (`-`) direction for
  each key;
- stable native DuckDB scalar ordering with SQL NULLs last in both directions,
  and prior active-row order preserved for complete ties; and
- preservation of all source columns, schema types and order, row count,
  source path, and eager execution metadata.

The relation is staged with a private row ordinal as the final ascending
tie-breaker. The ordinal is removed before publication, and publication uses
the existing `__tabdat_next` transaction path. Validation occurs before
staging. A missing key, empty direct key list, direction mismatch, SQL/schema/
row-count inspection failure, or publication failure leaves both the published
`DatasetInfo` and private `__tabdat_active` relation unchanged.

## Python contract

Pinned authority:

- repository: `https://github.com/SaehwanPark/tabdat-explore.git`;
- revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`;
- tree: `601b236788872323af9277d2276a236154a0f129`;
- Python: `3.13.3` in the recorded oracle environment;
- `uv.lock` SHA-256: `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.

Authoritative paths for this slice:

- `docs/commands/gsort.md` — per-key directions, NULL placement, stability,
  and quoted identifier semantics;
- `src/tabdat/models.py:275-289` — `SortKey` and `GsortCommand` fields;
- `src/tabdat/parser.py:2983-3001` — signed-key parser and diagnostics;
- `src/tabdat/executor.py:1008-1020` — active-state dispatch and transform
  recording;
- `src/tabdat/backend.py:1191-1246` — variable validation, private ordinal,
  direction-aware ordering, and stable tie handling; and
- `tests/test_gsort.py:68-137` — mixed directions, NULLs last, metadata, and
  unknown-variable atomicity.

The focused oracle probe recovered for this slice is:

```text
uv run --no-sync pytest -q -p no:cacheprovider tests/test_gsort.py
8 passed in 0.70s
```

The Python executor also supports DuckDB-lazy and Polars-lazy directed
sorting, labels, panel metadata, and CLI/JSON transform results. Those broader
behaviors are evidence for later slices, not claims of this Rust boundary.

## Bounded Rust contract

Add an owned `GsortResult { dataset: DatasetInfo }` and
`ExecutionResult::Gsort`. Add command-owned typed errors for:

- an empty direct key list (`gsort expects at least one variable`);
- one or more missing source columns (`gsort unknown variable: <names>`); and
- backend/staging/publication failure (`gsort failed`).

The implementation must:

1. return `NoActiveDataset { command: "gsort" }` before backend work;
2. reject an empty direct key list and validate every key variable against the
   published active schema before staging;
3. stage a source-order `SELECT` that adds a collision-free private row
   ordinal, orders each quoted requested column with its requested direction
   and `NULLS LAST`, uses the ordinal as the final ascending tie-breaker, and
   excludes the ordinal before publication;
4. inspect staged schema and row count, publish through the shared transaction,
   and update session metadata only after publication succeeds; and
5. preserve all schema names/types and values, directed row-order semantics,
   row count, source path, and eager execution metadata on success.

Exact Python panel/label metadata behavior, lazy or materialized execution,
`last_operation`, formatting, CLI, JSON, MCP, and broad transform sequencing
remain explicitly deferred.

## State-transition contract

| Situation | Active relation | Published metadata | Backend initialization |
| --- | --- | --- | --- |
| No active dataset | unchanged | unchanged | must not occur |
| Empty or unknown-key validation failure | unchanged | unchanged | already-loaded backend only |
| Stage, schema, row-count, SQL, or publication failure | unchanged | unchanged | already-loaded backend only |
| Successful directed sort | staged ordered relation becomes active | schema/count with preserved metadata | already-loaded backend |

## Test contract

Focused runtime coverage must include:

- no active dataset without initializing a backend;
- parsed eager directed sorting with mixed ascending/descending keys, NULLs
  last, stable complete ties, and preserved schema/data metadata;
- quoted identifiers, including an embedded double quote, and an empty
  relation;
- repeated keys with independent directions;
- empty direct key-list and unknown-variable failures with no state change;
- a dropped or mismatched active relation mapping to `GsortFailed` while the
  previously published metadata and relation remain available; and
- the existing parser contract remaining authoritative for syntax.

No new dependency, native backend, FFI, unsafe code, or ADR decision is
required.

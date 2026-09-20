# Bounded eager-runtime `decode` contract

Status: proposed at the WIP contract checkpoint on branch
`feat/runtime-decode`.

Producer: task owner, using `tabdat-migration` and `tabdat-data-semantics`.
Consumer: the bounded eager runtime implementation and its focused review.

## Scope

Execute the already-parsed form
`decode <numvar>, generate(<newvar>)` against an active eager local-Parquet
DuckDB relation when the numeric source was created by the bounded Rust
`encode` command in the same session.

The Rust slice supports:

- one numeric source column with an attached encode-produced code map;
- one new generated string target column;
- mapped integer codes converted to their original source strings;
- SQL NULL source values and codes without a map entry preserved as SQL NULL;
- source existence/type and target-collision validation;
- quoted identifiers, including embedded identifier quotes; and
- staged DuckDB publication preserving source path, row count, column order,
  row order, and eager execution metadata.

The session owns the decode map as private, Rust-owned provenance attached to
the generated encode column. It is not the general value-label metadata model:
the `label()` command, imported DTA labels, arbitrary user-created label sets,
variable labels, and public label metadata remain later roadmap work. The
already-parsed `encode ..., label(...)` option therefore remains explicitly
unsupported even though an ordinary encode creates the map needed for this
bounded round trip.

## Python contract

Pinned authority:

- repository: `https://github.com/SaehwanPark/tabdat-explore.git`;
- revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`;
- tree: `601b236788872323af9277d2276a236154a0f129`;
- Python: `3.13.3` in the recorded oracle environment; and
- `uv.lock` SHA-256: `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.

Authoritative paths for this slice:

- `docs/commands/decode.md` — syntax and conversion semantics;
- `src/tabdat/models.py:373-376` — `DecodeCommand` fields;
- `src/tabdat/parser.py:1340-1364` — source and `generate` parsing;
- `src/tabdat/executor.py:1795-1818` — attached-label lookup and metadata
  publication;
- `src/tabdat/backend.py:1344-1380` — numeric validation and CASE
  projection; and
- `tests/test_encode_decode.py` — focused parser, round-trip, and missing-label
  behavior.

The recovered focused oracle command was:

```text
uv run --no-sync pytest -q -p no:cacheprovider tests/test_encode_decode.py
6 passed in 0.96s
```

The oracle's broader label metadata behavior is evidence for later slices, not
a claim that this Rust branch supports arbitrary attached label sets.

## Bounded Rust contract

Add an owned `Decode { source, generate }` command value, `DecodeResult`, and
`ExecutionResult::Decode`. Add command-owned errors for:

- no attached encode-produced labels on the source;
- an unknown source variable;
- a non-numeric source variable;
- an existing target variable; and
- backend/staging/schema/count/publication failure (`decode failed`).

The implementation must:

1. return `NoActiveDataset { command: "decode" }` before backend work;
2. require an attached session-owned code map before staging;
3. validate source, target, and numeric domain before relation mutation;
4. compile a quoted `CASE` expression from integer codes to escaped string
   labels, with NULL for unmatched and NULL source values;
5. stage a full ordered projection, inspect schema and row count, publish
   through the shared transaction, and update active state only after
   publication succeeds; and
6. retain the source's map across harmless schema/order operations, update it
   across `rename` and projections, and invalidate it when a replacement or
   in-place recode changes the mapped source values.

The parser owns lexical values and exact quoted identifier spelling. The
runtime owns metadata lookup, domain validation, SQL quoting, relation
mutation, and atomic state publication.

## State-transition contract

| Situation | Active relation | Decode map | Backend initialization |
| --- | --- | --- | --- |
| No active dataset | unchanged | unchanged | must not occur |
| Missing map/source/type/target validation failure | unchanged | unchanged | already-loaded backend only |
| Lookup, staging, SQL, schema, row-count, or publication failure | unchanged | unchanged | already-loaded backend only |
| Successful eager decode | staged relation with appended string column becomes active | source map remains attached | already-loaded backend |
| Successful `use` | newly loaded relation becomes active | cleared | initialized as needed |

## Test contract

Focused Rust coverage must include:

- parser-produced source and `generate` fields plus exact missing/duplicate/
  unsupported-option diagnostics;
- no active dataset without initializing a backend;
- sorted `encode` followed by `decode`, including duplicate values and
  preserved NULLs;
- mapped codes decoded to strings, unmapped numeric codes decoded to NULL,
  generated-column placement/type, quoted names, and empty mappings;
- missing-label, unknown-source, non-numeric-source, and target-collision
  validation with active state unchanged; and
- a staged/backend failure mapped to `DecodeFailed` while the previous
  relation, metadata, and map remain available.

No new dependency, native backend, FFI, unsafe code, or ADR decision is
required. General value-label metadata, the `label` command, DTA ingestion,
lazy/materialized execution, panel metadata, last-operation state, formatting,
CLI/JSON/MCP, and broad transform sequencing remain deferred.

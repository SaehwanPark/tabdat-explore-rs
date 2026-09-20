# Bounded eager-runtime `encode` contract

Status: accepted on `main` at merge commit
[`af3e3b2`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/af3e3b2726778af5c5f3b5c13c4ba84e5291da61), via
[PR #53](https://github.com/SaehwanPark/tabdat-explore-rs/pull/53).

Producer: task owner, using `tabdat-migration` and `tabdat-data-semantics`.
Consumer: the bounded eager runtime implementation and its focused review.

## Scope

Execute the already-parsed form
`encode <strvar>, generate(<newvar>) [, label(<lblname>)]`
against the active eager local-Parquet DuckDB relation.

The Rust slice supports:

- one existing source column;
- one new generated target column;
- sorted unique nonmissing source-string values assigned 1-based integer codes;
- SQL NULL source values preserved as SQL NULL target values;
- source existence, source-string-domain, and target-collision validation;
- quoted identifiers, including embedded identifier quotes; and
- staged DuckDB publication preserving source path, row count, column order,
  and eager execution metadata.

The parser preserves the optional `label(<lblname>)` name in the owned command
value. The runtime rejects that option explicitly because this encode slice does
not yet own general variable/value-label metadata. Ordinary encode now retains
a private code-to-text map consumed by the separately accepted bounded decode
slice; the generic `label` command and arbitrary label sets remain later owners
of the public metadata contract.

The generated code column uses DuckDB integer semantics: nonempty mappings
produce `INTEGER` codes, while the empty-relation/null-only mapping follows the
oracle backend's typed-NULL behavior. Code assignment is based on the sorted
distinct nonmissing string values, not on first appearance in the active rows.

Validation, schema/count inspection, and publication failures leave the
published metadata and active relation unchanged.

## Python contract

Pinned authority:

- repository: `https://github.com/SaehwanPark/tabdat-explore.git`;
- revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`;
- tree: `601b236788872323af9277d2276a236154a0f129`;
- Python: `3.13.3` in the recorded oracle environment; and
- `uv.lock` SHA-256: `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.

Authoritative paths for this slice:

- `docs/commands/encode.md` — syntax, sorted 1-based coding, and value-label
  option;
- `src/tabdat/models.py:366-369` — `EncodeCommand` fields;
- `src/tabdat/parser.py:1320-1340` — source, `generate`, and `label` parsing;
- `src/tabdat/executor.py:1763-1794` — distinct-value mapping and label-state
  updates;
- `src/tabdat/backend.py:1316-1358` and `1380-1403` — source/type/target
  validation, ordered distinct lookup, and CASE projection; and
- `tests/test_encode_decode.py` — focused parser, coding, type, collision,
  and variable-label behavior.

The recovered focused oracle command was:

```text
uv run --no-sync pytest -q -p no:cacheprovider tests/test_encode_decode.py
6 passed in 0.96s
```

An additional isolated oracle probe confirmed that source rows `b, a, NULL, b`
produce codes `2, 1, NULL, 2`, with an `INTEGER` generated column. The Python
executor also supports value-label metadata, decode, lazy/materialized
execution, panel metadata, and CLI/JSON/MCP output. Those are evidence for
later slices, not claims of this Rust boundary.

## Bounded Rust contract

Add an owned `Encode` command value with `source`, `generate`, and optional
`label` fields. Add `EncodeResult { dataset: DatasetInfo }` and
`ExecutionResult::Encode`. Add command-owned errors for:

- an optional label name outside the current metadata boundary;
- an unknown source variable;
- a non-string source variable;
- an existing target variable; and
- backend/staging/schema/count/publication failure (`encode failed`).

The implementation must:

1. return `NoActiveDataset { command: "encode" }` before backend work;
2. validate the optional label boundary, source, and target before staging;
3. query sorted distinct nonmissing source values from the active relation;
4. compile a quoted `CASE` expression that maps those values to 1-based
   integer codes and leaves NULL/unmatched rows NULL;
5. stage a full ordered projection, inspect schema and row count, publish
   through the shared transaction, and update session metadata only after
   publication succeeds; and
6. preserve all existing data, schema position, row order, and active dataset
   metadata except for the appended generated code column.

The parser owns lexical values and exact quoted identifier spelling. The
runtime owns domain validation, SQL quoting, distinct-value discovery,
relation mutation, and atomic state publication.

## State-transition contract

| Situation | Active relation | Published metadata | Backend initialization |
| --- | --- | --- | --- |
| No active dataset | unchanged | unchanged | must not occur |
| Label/source/type/target validation failure | unchanged | unchanged | already-loaded backend only |
| Distinct lookup, staging, SQL, schema, row-count, or publication failure | unchanged | unchanged | already-loaded backend only |
| Successful eager encode | staged relation with appended code column becomes active | updated schema/count with source/execution metadata preserved | already-loaded backend |

## Test contract

Focused Rust coverage must include:

- no active dataset without initializing a backend;
- parser-produced source, `generate`, and optional `label` fields;
- sorted 1-based coding with duplicate values and preserved NULLs;
- generated-column placement, `INTEGER` typing, row/schema preservation, and
  repeated encode operations;
- quoted source and target identifiers, including an embedded double quote, and
  an empty relation;
- validation failures for label option, unknown source, non-string source, and
  target collision, with metadata and rows unchanged; and
- a staged/backend failure mapping to `EncodeFailed` while the previously
  published metadata and relation remain available.

No new dependency, native backend, FFI, unsafe code, or ADR decision is
required. General label metadata, the generic `label` command,
lazy/materialized execution, panel metadata, last-operation state, formatting,
CLI/JSON/MCP, and broad transform sequencing remain deferred; bounded decode is
recorded separately in `_workspace/runtime-decode/`.

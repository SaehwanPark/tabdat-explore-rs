# Contract: bounded eager-runtime `datasignature`

Status: accepted bounded eager-runtime slice

Selected skills: `tabdat-migration`, `tabdat-data-semantics`,
`simple-code-writer`

Boundary: pinned Python execution contract → Rust-owned eager-session
reproducibility fingerprint

## Scope

Execute the existing zero-argument `Command::Datasignature` against the active
dataset held by the bounded eager local-Parquet `tabdat-runtime` session. The
operation is read-only and returns an owned result containing the SHA-256
algorithm name, lowercase hexadecimal signature, row count, and public column
count. The signature covers public schema names and canonical types, active row
order, and every cell value. It excludes source paths, backend/execution mode,
labels, panel metadata, and internal columns.

This slice is bounded to eager local Parquet. It does not add lazy/materialized
execution, `last_operation`, formatting, CLI/REPL, JSON/MCP output, or a
general-purpose relation API. The current Rust session has no `last_operation`
field, so that Python side effect is an explicit deferral rather than an
unrecorded parity claim.

## Python authority

The Python oracle is the clean sibling checkout `../tabdat-explore` at pinned
commit `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, package `0.25.0`, and Python
`3.13.3`. Its `uv.lock` SHA-256 is
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`; the
checkout is clean and no dependency synchronization or source edits are
performed. Authority paths are:

- `src/tabdat/models.py:201-205,1236-1242` — command and result models;
- `src/tabdat/parser.py:594-604` — zero-argument syntax and diagnostics;
- `src/tabdat/executor.py:954-962` — active-dataset dispatch and failure policy;
- `src/tabdat/backend.py:1041-1079,2832-2952` — eager scan and signature
  encoding;
- `tests/test_datasignature.py` — focused parser, deterministic hash, schema,
  value, empty, and state behavior;
- `docs/commands/datasignature.md:1-80` and
  `docs/language-semantics.md:70-73` — public semantics.

Focused oracle validation at the pinned revision is expected to remain:

```sh
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_datasignature.py
```

The recovery run observed `11 passed in 0.94s`; the direct parse form ran as
`1 passed, 10 deselected in 0.28s`.

## Python behavior and exact protocol

- syntax is case-insensitive, accepts no arguments/options/`if`/assignment,
  and reports the exact parser diagnostic for rejected forms;
- no active data reports
  `datasignature requires an active dataset; run use <path> first`;
- the result uses `algorithm="sha256"`, a lowercase 64-character digest,
  `row_count`, and `column_count`;
- empty relations are valid and remain schema-dependent; and
- backend failure is reported as `datasignature failed` without changing active
  metadata.

The digest is SHA-256 seeded with `tabdat-datasignature/v1\0`. Every protocol
token is framed as an unsigned 8-byte big-endian byte length followed by bytes.
The schema emits `schema`, then one `column`, typed UTF-8 name (`T` prefix),
and canonical type (`T` prefix) per public column. Each row emits `row` and
typed cell values in schema order.

Canonical type aliases include integer widths, float widths, `BOOL`, `STRING`,
decimal precision/scale, timestamp units/time-zone markers, and recursive list
types. Value encodings include NULL, booleans, signed/unsigned integers,
Python-compatible IEEE float hex (including NaN, infinities, and signed zero),
fixed decimal text, UTC-normalized timestamps (including nanosecond precision),
dates, times, UTF-8 strings, bytes, recursively framed lists/arrays, intervals
as their three-integer month/day/nanosecond list representation, and
deterministically sorted mappings. Nested list/map/struct type hints propagate
timezone metadata so aware timestamps retain their UTC offset. Direct DuckDB
union values remain outside the bounded relation surface because local Parquet
round-trips them as ordinary structs.

The pinned fixture used by the oracle has an exact signature of
`0b61cef05ab04301df893652f666b6ed974668f0cd214ebae17cfff02a6b3aad` for
three rows and four columns. Complex date/timestamptz, NaN/infinity, decimal,
list, and NULL fixtures produce the same signature across the oracle's eager,
DuckDB-lazy, and Polars-lazy modes; this Rust slice claims only the eager path.

## Rust contract

Add an owned result beside the existing inspection results:

```rust
pub struct DatasignatureResult {
    pub algorithm: String,
    pub signature: String,
    pub row_count: u64,
    pub column_count: u64,
}
```

Expose `ExecutionResult::Datasignature` and a typed
`RuntimeError::DatasignatureFailed`. Reuse the existing no-active-dataset
diagnostic with `command: "datasignature"`; dispatch must initialize no
backend for that validation failure. Scan the private active DuckDB relation
in schema order and insertion order, preserve active metadata, and map query or
encoding failures to the typed backend error.

Use the pure-Rust `sha2` crate because SHA-256 is part of the cross-runtime
protocol; do not introduce native crypto or expose backend value types through
the runtime boundary. The dependency and protocol choice are recorded in
`docs/adr/0008-datasignature-sha256.md`.

## Test contract

Test the observable result and state independently:

- fresh-session exact no-active error and no backend initialization;
- the pinned three-row/four-column exact digest;
- deterministic repeated execution with unchanged metadata;
- schema/row-order sensitivity using separate Parquet fixtures;
- valid empty relation with a 64-character schema-dependent digest;
- complex date/timestamptz, NaN/infinity, decimal, list, and NULL values;
- nanosecond timestamps, nested timezone-bearing list/struct/map values, and
  interval values;
- quoted names and no alias assumptions; and
- a dropped/corrupt active relation returning `datasignature failed` without
  metadata mutation.

Lazy/materialized execution, `last_operation`, CLI/JSON/MCP, and the generic
`by` parser surface remain explicit deferrals. The Python documentation's
generic `by` wording is not promoted into this bounded Rust contract because
the pinned parser/executor path does not provide a useful direct zero-argument
runtime result for it.

Contract recovery is complete for the pinned Python behavior. The bounded Rust
implementation and hosted acceptance evidence will be recorded in
`02-evidence-migration.md` after implementation.

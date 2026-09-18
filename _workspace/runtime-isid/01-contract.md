# Contract: bounded eager-runtime `isid`

Status: contract recovered; implementation and hosted acceptance pending

Selected skills: `tabdat-migration`, `tabdat-data-semantics`,
`simple-code-writer`

Boundary: pinned Python execution contract → Rust-owned eager-session key
uniqueness check

## Scope

Execute the existing `Command::Isid { variables, missok }` against the active
dataset held by the bounded eager local-Parquet `tabdat-runtime` session. At
least one key variable is required; there is no default-all-columns behavior.
Explicit key order and repeated variables are preserved. The `missok` flag
allows incomplete key tuples only when their complete key combinations remain
unique.

A successful request returns an owned `IsidResult` containing the selected
variables, total rows, unique key groups, rows with a NULL key component, and
the requested `missok` flag. SQL NULL key values compare equal for grouping.
Duplicate groups fail regardless of `missok`; without `missok`, any row with a
NULL key component also fails. Empty relations pass with zero counts. The
operation is read-only: active metadata and the private relation remain
unchanged.

This slice is bounded to eager local Parquet. It does not add lazy/materialized
execution, transforms, labels/panel metadata, wildcard/range expansion,
`last_operation`, formatting, CLI/REPL, JSON/MCP output, or broader relation
behavior.

## Python authority

The Python oracle is the clean sibling checkout `../tabdat-explore` at pinned
commit `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, package `0.25.0`, and Python
`3.13.3`. Its `uv.lock` SHA-256 is
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`; the
checkout is clean and no dependency synchronization or source edits are
performed. Authority paths are:

- `src/tabdat/models.py:193-199,1228-1235` — command and result models;
- `src/tabdat/parser.py:591-592,3005-3015` — direct syntax and diagnostics;
- `src/tabdat/executor.py:931-952` — active-dataset dispatch and result/failure
  policy;
- `src/tabdat/backend.py:986-1039` — eager grouped aggregate;
- `tests/test_isid.py:83-255` — focused parser, unique/missing/duplicate,
  empty, alias-collision, unknown-variable, and state coverage;
- `docs/commands/isid.md:1-68` and `docs/language-semantics.md:66-68` —
  public semantics and deferred presentation.

Focused oracle validation at the pinned revision:

```sh
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_isid.py
```

Observed result: `20 passed in 0.54s` (Python `3.13.3`; no environment
synchronization).

## Python behavior

- no active dataset: `isid requires an active dataset; run use <path> first`;
- unknown keys are validated before backend execution and report
  `isid unknown variable: <comma-separated names>`, preserving request order
  and repeated unknown names;
- SQL NULL key values compare equal for grouping;
- `missing_key_rows` counts every row with at least one NULL key component;
- without `missok`, missing-key rows fail with
  `isid failed: <n> rows have missing key values (use , missok to permit them)`;
- duplicate groups fail regardless of `missok`, with
  `isid failed: <n> rows are in <g> duplicate key groups`;
- when both conditions fail, missing-key text precedes duplicate-group text,
  joined by `; `;
- empty relations pass with zero totals; and
- backend failures display `isid failed` while preserving active state.

For example, rows `(1,1),(1,2),(2,1),(NULL,1),(NULL,2)` produce
`total_rows=5`, `unique_groups=5`, and `missing_key_rows=2` for keys
`(patient_id, visit)`. They pass with `missok` and fail without it. Replacing
the final row with `(NULL,1)` creates one duplicate group and fails even with
`missok`.

## Rust contract

Add the following owned result beside the existing inspection results:

```rust
pub struct IsidResult {
    pub variables: Vec<String>,
    pub total_rows: u64,
    pub unique_groups: u64,
    pub missing_key_rows: u64,
    pub missok: bool,
}
```

Expose `ExecutionResult::Isid`, `IsidUnknownVariable`, a typed semantic
failure carrying missing-row/duplicate-row/group counts, and `IsidFailed` for
backend/query failure. Reuse `NoActiveDataset { command: "isid" }` and reject
an empty key list with the parser/runtime diagnostic
`isid expects at least one key variable`.

Use one generated DuckDB aggregate query: group by each quoted requested key,
count rows per group, then aggregate total rows, unique groups, duplicate
groups/rows, and rows in groups containing any NULL key. The internal count
alias must be collision-safe against user columns such as
`__tabdat_isid_count`. Convert aggregate values with checked integer
operations, and keep backend values and lifetimes private to the adapter.

## Test contract

Test the observable result and state independently:

- fresh-session exact no-active error and no backend initialization;
- parser-to-session `isid patient_id visit, missok`;
- unique composite keys with and without NULLs;
- missing-key failure without `missok` and success with it;
- duplicate non-NULL and duplicate NULL key groups fail even with `missok`;
- empty relation succeeds with zero counts;
- a real `__tabdat_isid_count` column cannot collide with the internal alias;
- unknown variables preserve order/repetition and prior active state;
- dropped/corrupt active relation returns `isid failed` without metadata
  mutation; and
- repeated successful execution is read-only.

## Implementation mapping and completion

- Native Rust: owned result, typed diagnostics, session validation, semantic
  failure formatting, and result dispatch;
- DuckDB: one quoted grouped aggregate with a collision-safe count alias and
  checked conversions;
- Existing parser/session: reuse typed `Command::Isid`, requested key order,
  and atomic eager relation lifecycle; and
- Deferred: lazy/materialized execution, labels, wildcard/range expansion,
  `last_operation`, formatting, JSON/MCP/CLI surfaces, and broader relation
  APIs.

Contract recovery is `complete` for the pinned Python behavior. Rust
implementation and hosted parity evidence remain `partial` until the bounded
slice is built, reviewed, merged, and documented.

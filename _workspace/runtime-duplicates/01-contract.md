# Contract: bounded eager-runtime `duplicates`

Status: accepted after PR #38 squash merge `6a10039`; hosted checks and branch cleanup complete

Selected skills: `tabdat-migration`, `tabdat-data-semantics`,
`simple-code-writer`

Boundary: pinned Python execution contract → Rust-owned eager-session duplicate
key aggregate

## Scope

Execute the existing `Command::Duplicates { variables }` against the active
dataset held by the bounded eager local-Parquet `tabdat-runtime` session. The
syntax-only parser already accepts `duplicates [report] [varlist]`; the leading
unquoted/string `report` alias is removed by the parser, while a quoted or
backtick-quoted `report` remains a key variable. An empty key list uses every
active column in schema order. An explicit list preserves request order and
duplicates.

A successful request returns an owned aggregate report with the selected key
variables and total rows, unique groups, duplicate groups, duplicate rows,
surplus rows, and maximum copies. SQL NULL key values compare equal and
participate in duplicate groups. The command is read-only: active metadata and
the private relation remain unchanged.

This slice is bounded to eager local Parquet. It does not add lazy/materialized
execution, transforms, labels, wildcard/range expansion, `last_operation`,
formatting, CLI/REPL, JSON/MCP output, or execution for `isid` and other
unchecked commands.

## Python authority

The Python oracle is the clean sibling checkout `../tabdat-explore` at pinned
commit `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, package `0.25.0`, and Python
`3.13.3`. Its `uv.lock` SHA-256 is
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`; the
checkout is clean and no dependency synchronization or source edits are
performed. Authority paths are:

- `src/tabdat/models.py:186-190,1216-1224` — command and result models;
- `src/tabdat/parser.py:581-589` — direct syntax and diagnostics;
- `src/tabdat/executor.py:918-929` — active-dataset dispatch and state;
- `src/tabdat/backend.py:931-984,2113-2156` — grouped aggregate and row
  construction;
- `tests/test_duplicates.py:67-278` — focused parser, report alias, ordering,
  NULL grouping, counts, failure, and no-active coverage;
- `docs/commands/duplicates.md:1-42` and
  `docs/language-semantics.md:61-65` — public semantics and deferred output.

Focused oracle validation at the pinned revision:

```sh
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_duplicates.py
```

Observed result: `14 passed in 0.84s` (Python `3.13.3`; no environment
synchronization).

## Python behavior

- no active dataset: `duplicates requires an active dataset; run use <path> first`;
- unknown keys are validated before backend execution and report
  `duplicates unknown variable: <comma-separated names>`, preserving request
  order and repeated unknown names;
- an empty key list returns schema-order variables;
- `total_rows` counts all rows;
- `unique_groups` counts grouped key combinations, including NULL groups;
- `duplicate_groups` counts groups with at least two rows;
- `duplicate_rows` counts every row in duplicate groups;
- `extra_rows` is the sum of each duplicate group size minus one;
- `max_copies` is the largest group size, or zero for no rows;
- backend failure is displayed as `duplicates failed`; and
- successful and failed execution preserve the active dataset and metadata.

For example, key rows `(1,a),(1,a),(2,b),(NULL,c),(NULL,c),(NULL,d)` produce
`total_rows=6`, `unique_groups=4`, `duplicate_groups=2`,
`duplicate_rows=4`, `extra_rows=2`, and `max_copies=2` for all columns. Empty
relations return zero aggregate counts while retaining the selected variable
names.

## Rust contract

Add the following owned result beside the existing inspection results:

```rust
pub struct DuplicatesResult {
    pub variables: Vec<String>,
    pub total_rows: u64,
    pub unique_groups: u64,
    pub duplicate_groups: u64,
    pub duplicate_rows: u64,
    pub extra_rows: u64,
    pub max_copies: u64,
}
```

Expose `ExecutionResult::Duplicates` and typed diagnostics for unknown keys,
an empty key requirement if the backend cannot represent one, and aggregate
failure. Reuse `NoActiveDataset { command: "duplicates" }` for the exact
no-active display. Validate all unknown variables before opening a query.

Use one generated DuckDB aggregate query: group by each quoted requested key,
count rows per group, then aggregate group sizes into the six report fields.
The internal count alias must be generated collision-safely against user
columns (the Python tests include `__tabdat_duplicate_count`). Convert counts
and arithmetic with checked integer operations. Keep backend values and
lifetimes private to the adapter.

## Test contract

Test the observable result and state independently:

- fresh-session exact no-active error and no backend initialization;
- parser-to-session `use` followed by `duplicates report id`;
- default schema-order keys, explicit order, duplicate keys, and quoted names;
- repeated NULL keys compare equal and contribute to duplicate groups;
- exact totals for duplicate and no-duplicate fixtures;
- empty relation returns zero counts and schema-derived variables;
- a real `__tabdat_duplicate_count` column cannot collide with an internal alias;
- unknown variables preserve order/repetition and prior active state;
- dropped/corrupt active relation returns `duplicates failed` without metadata
  mutation; and
- repeated successful execution is read-only.

## Implementation mapping and completion

- Native Rust: owned result, typed diagnostics, session validation, and result
  dispatch;
- DuckDB: one quoted group-size aggregate with collision-safe aliases and
  checked conversions;
- Existing parser/session: reuse the typed `Command::Duplicates`, active
  schema, and atomic eager relation lifecycle; and
- Deferred: lazy/materialized execution, labels, wildcard/range expansion,
  `last_operation`, formatting, JSON/MCP/CLI surfaces, and broader relation
  APIs.

Contract recovery and bounded Rust implementation are complete for the pinned
Python behavior. PR #38 was reviewed, squash-merged as `6a10039`, and its
temporary branch was deleted locally and remotely. Lazy/materialized,
presentation, and broader relation surfaces remain explicit deferrals.

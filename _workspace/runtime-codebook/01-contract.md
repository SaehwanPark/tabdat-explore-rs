# Contract: bounded eager-runtime `codebook`

Status: recovered contract; implementation pending.

Selected skills: `tabdat-migration`, `tabdat-data-semantics`,
`simple-code-writer`

Boundary: pinned Python execution contract → Rust-owned eager-session column profiles

## Scope

Execute the existing syntax-only `Command::Codebook { variables }` against the
active dataset held by the bounded eager local-Parquet `tabdat-runtime` session.
A successful request returns one owned profile row per requested variable. An
empty variable list profiles every active column in schema order. Explicit
variable order and duplicates are preserved. Each row reports the active schema
type, non-null count, null count, distinct non-null count, and up to three
non-null example values. The operation is read-only: active metadata and the
private relation remain unchanged.

This slice builds on the accepted eager local-Parquet `use` boundary in PR #22,
the read-only `describe` boundary in PR #31, cached eager `count` in PR #32,
owned eager `head`/`tail` previews in PRs #33/#34, and eager numeric
`summarize` in PR #35. It does not add lazy loading/materialization, label
metadata, wildcard/range expansion, transforms, formatting, JSON/MCP output,
CLI/REPL behavior, or execution for `missing`, `duplicates`, `isid`, or other
inspection commands.

The Rust result owns all values and exposes no DuckDB `Value`/`ValueRef`, Arrow
value, connection, statement, raw pointer, or foreign lifetime. Supported
scalar examples reuse the existing owned `CellValue` enum. If a profile's
non-null examples contain a DuckDB logical/container value that the current
owned boundary cannot represent (for example dates, lists, or structs), the
request returns a typed `CodebookFailed` error for that variable rather than
stringifying or leaking a backend value. This is an explicit bounded deviation
from Python's broader value support.

Variable labels are intentionally omitted from the Rust result because the
current eager session publishes no label metadata. Python label enrichment is a
recorded deferral, not an assertion that labels are empty.

## Python authority

The Python oracle is the clean sibling checkout `../tabdat-explore` at pinned
commit `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, with Python `3.13.3` and the
`uv.lock` digest recorded in `docs/migration/README.md`.

Authoritative paths:

- `src/tabdat/models.py:168-176` (`CodebookCommand`);
- `src/tabdat/models.py:1190-1204` (`CodebookRow`/`CodebookResult`);
- `src/tabdat/parser.py:567-572` (direct command parsing);
- `src/tabdat/executor.py:905-913,9696-9713` (active-dataset requirement,
  read-only result, and label enrichment);
- `src/tabdat/backend.py:890-898,2180-2212,3444-3452` (selection, counts,
  examples, and unknown-variable validation); and
- `tests/test_executor.py:8221-8288,9699-9715` and
  `tests/test_labels.py:105-127` (profile, all-null, validation, no-active,
  and label behavior).

The focused oracle command is:

```sh
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py tests/test_executor.py tests/test_labels.py \
  -k 'codebook or phase_3_inspection_commands_require_active_dataset'
```

The focused probe observed `12 passed, 900 deselected`. The exact no-active
diagnostic is `codebook requires an active dataset; run use <path> first`.

## Rust result and errors

The proposed backend-independent result is:

```text
CodebookRow {
  variable: String,
  data_type: String,
  nonmissing: u64,
  missing: u64,
  distinct: u64,
  examples: Vec<CellValue>,
}
CodebookResult { rows: Vec<CodebookRow> }
```

Add `ExecutionResult::Codebook`, `RuntimeError::CodebookUnknownVariable`, and
`RuntimeError::CodebookFailed`. Preserve exact Python display text:

- `codebook unknown variable: <comma-separated names>`;
- `codebook failed for variable: <name>`; and
- `codebook requires an active dataset; run use <path> first` through the
  existing `NoActiveDataset { command: "codebook" }` variant.

Unknown-variable validation occurs before backend access and preserves all
requested unknown names, including duplicates. An empty active relation returns
zero counts and empty examples for each schema column. Nulls are excluded from
nonmissing, distinct, and examples; duplicate non-null examples are retained.

## State and test contract

The operation must be read-only and failure-atomic:

| Before | Input | Result | After |
| --- | --- | --- | --- |
| no active dataset | `codebook` | typed `NoActiveDataset` with exact text | unchanged; backend remains uninitialized |
| active eager local-Parquet dataset | explicit variables | owned rows in requested order, including duplicates | unchanged |
| active eager local-Parquet dataset | empty variable list | one row per schema column in schema order | unchanged |
| active eager local-Parquet dataset | null/all-null values | exact counts and empty examples where appropriate | unchanged |
| unknown variables | `codebook` | typed exact diagnostic before query | unchanged |
| unsupported example conversion or backend failure | `codebook` | typed `CodebookFailed` for the variable | active metadata remains exactly as before |

Tests should cover:

- fresh-session no-active error and backend non-initialization;
- parser-to-session eager `use` followed by `codebook age cost`;
- exact schema types, counts, distinct values, and first-three non-null
  examples, including decimal, text, null, and repeated values;
- default schema order, explicit reordering, and duplicate variables;
- empty and all-null columns;
- exact unknown-variable diagnostics and failure-atomic state preservation;
- repeated read-only execution and failed/corrupt relation behavior; and
- explicit deferral/failure for unsupported logical/container example values,
  labels, lazy/materialized relations, and presentation surfaces.

## Documentation hygiene and stop conditions

After acceptance, correct stale current-state wording that still lists bounded
inspection only through `summarize`: `SPEC.md`, ADR 0007,
`_workspace/use-eager-parquet/04-summary.md`, and the `Codebook` command comment
in `crates/tabdat-language/src/lib.rs`. Keep historical runtime artifacts
unchanged. Create `_workspace/runtime-codebook/{01-contract,02-evidence-migration,03-review}.md`.
Do not mark the roadmap `codebook` checkbox until merge and post-merge hosted
evidence are green.

Stop and record a decision if example ordering, NULL/distinct semantics, or
backend-independent ownership cannot be established without broadening the
session or adding unsupported value coercions.

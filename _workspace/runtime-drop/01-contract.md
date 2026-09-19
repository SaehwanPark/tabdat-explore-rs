# Bounded eager-runtime `drop` contract

Status: contract recovered and bounded implementation accepted on `main` at
`50cf80c` (PR #43).

Producer: task owner, using `tabdat-migration`, `tabdat-data-semantics`, and
`simple-code-writer`, with independent Python-contract and scope review.
Consumer: bounded `drop` implementation and review, subsequently squash-merged
to `main` after hosted acceptance and temporary-branch cleanup.

## Scope

This loop targets the inverse of the accepted eager `keep` projection:

- `drop <explicit-varlist>` removes the named columns from the active eager
  local-Parquet DuckDB relation;
- remaining columns stay in original schema order, with row order, NULLs,
  source path, execution mode, and row count preserved;
- quoted and backtick identifiers are supported, and duplicate requests are
  harmless;
- validation, staging, schema/count inspection, and publication are atomic
  with respect to the published metadata and private active relation.

`drop if <expression>` is recovered in the Python contract but intentionally
deferred. The parser recognizes the form and returns an explicit bounded
runtime deferral diagnostic; this slice does not add a second expression or
row-filter contract.

## Python contract

Pinned authority:

- repository: `/Volumes/research/gitrepo/tabdat-explore`;
- revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`;
- tree: `601b236788872323af9277d2276a236154a0f129`;
- Python: `3.13.3`;
- `uv.lock` SHA-256:
  `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.

Authority:

- `src/tabdat/models.py:244-253,1262-1265`;
- `src/tabdat/parser.py:623-627,1026-1035,3036-3122`;
- `src/tabdat/executor.py:993-997,1350-1362,1829-1837`;
- `src/tabdat/backend.py:1170-1189,1247-1278`;
- `docs/commands/drop.md`, `src/tabdat/help/topics/drop.md`;
- `docs/language-semantics.md:40-47,91-122,149-160`;
- `tests/test_parser.py:254-270,1488-1494`;
- `tests/test_executor.py:8354-8400,8688-8703,9628-9639`.

The full Python command accepts `drop <varlist>` and `drop if <expression>`;
command names are case-insensitive, while variable spelling is exact. Quoted
and backtick identifiers preserve whitespace, punctuation, commas, and escaped
backticks. Exact parser diagnostics recovered for this slice include:

```text
drop expects a variable list or if clause
drop cannot combine a variable list with an if clause
drop does not accept options or assignment syntax
missing expression after if
duplicate if clause
incomplete expression after >=
unsupported token in command: -
comma must be followed by at least one option
```

For the variable form, Python validates every request before mutation, reports
unknown names as `drop unknown variable: ...` in request order, computes the
remaining schema in source order, and rejects removal of every column with
`drop would remove every column`. Duplicate requests are harmless. Successful
execution reports `Dropped selected columns`; row order, NULLs, source path,
and row count are preserved.

Focused oracle selections at the pinned revision passed:

```text
tests/test_parser.py -k drop:       2 passed, 487 deselected
tests/test_executor.py -k drop:    15 passed, 402 deselected
tests/test_cli.py -k drop_predicate: 2 passed, 181 deselected
```

The predicate form removes rows whose condition is true and retains false or
missing rows, with boolean/null validation, overflow-to-missing reporting, and
relative row-order guarantees. Those semantics are explicitly outside this
Rust slice.

## Bounded Rust contract

Add `Command::Drop { variables: Vec<String> }`, an owned `DropResult`, and an
`ExecutionResult::Drop` variant. The runtime must:

1. return `NoActiveDataset { command: "drop" }` before backend work;
2. validate all requested names against published metadata;
3. compute the complement in original schema order and reject zero columns;
4. quote every identifier in the staged DuckDB projection;
5. inspect staged schema/count, then transactionally publish;
6. update published metadata only after successful publication.

Use the existing eager `__tabdat_next` staging/publication boundary, sharing
the projection helper with `keep` where practical. Any validation, SQL,
schema/count, or publication failure must preserve the published metadata and
private active relation. No new backend, FFI, dependency, or unsafe code is
needed.

## Test contract

Focused Rust coverage must include:

- case-insensitive command names, ordered/repeated variables, and quoted/
  backtick identifiers;
- exact empty, missing-`if`, deferred-predicate, mixed-form, option,
  assignment, trailing-comma, and unsupported-token diagnostics;
- no active dataset without backend initialization;
- ordinary projection with source-order columns, row/NULL preservation, and
  an empty relation;
- duplicate requests and repeated drop operations;
- unknown-variable validation before query and all-column rejection;
- dropped/corrupt active-relation failure with metadata/relation preservation.

## Explicit deviations and deferrals

Predicate-form `drop if <expression>` remains deferred, including boolean/null
retention, expression functions, exact integral overflow, non-finite arithmetic,
mixed-domain validation, and row-level diagnostics. Lazy/materialized
execution, wildcard/range expansion, labels and panel metadata,
`last_operation`, formatting, CLI/REPL, JSON/MCP, and broad transformation
sequencing also remain outside this bounded slice.

Completion state: contract recovery, bounded implementation, hosted acceptance,
squash merge, and temporary-branch cleanup are complete. Predicate execution
and the other explicit deferrals above remain outside this accepted slice.

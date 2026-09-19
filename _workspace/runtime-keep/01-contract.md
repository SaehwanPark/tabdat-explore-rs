# Bounded eager-runtime `keep` contract

Status: accepted bounded slice implemented on `feat/runtime-keep`; merge and
main closeout pending.

Producer: task owner, using `tabdat-migration`, `tabdat-data-semantics`, and
`simple-code-writer`.
Consumer: runtime implementation and review on `feat/runtime-keep`.

## Scope

This loop targets the smallest useful eager `keep` mutation:

- `keep <explicit-varlist>` projects requested columns in requested order over
  the existing eager local-Parquet DuckDB relation.

Quoted and backtick identifiers are supported. `keep if <expression>` is
recovered in the Python contract but intentionally deferred so this slice does
not create a second expression/mutation contract; it will be a follow-up once
the transform boundary is established.

## Python contract

Pinned authority:

- repository: `/Volumes/research/gitrepo/tabdat-explore`
- revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`
- tree: `601b236788872323af9277d2276a236154a0f129`
- Python: `3.13.3`
- `uv.lock` SHA-256: `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`

Authority:

- `src/tabdat/models.py:244-247`, result `:1262-1265`;
- `src/tabdat/parser.py:623-624,1026-1035,3036-3101`;
- `src/tabdat/executor.py:993-1000,1336-1348,752-812`;
- `src/tabdat/backend.py:1152-1168,1247-1278,2518-2522,2303-2395`;
- `docs/commands/keep.md`;
- `docs/language-semantics.md:42-45,91-96,100-122,151-160`;
- `tests/test_parser.py:254-266,1334-1365`;
- `tests/test_executor.py:6818-7059,7061-7510,7661-7863,8688-8730,9695-9713`.

The full Python command accepts `keep varlist` and `keep if <expression>`;
command names are case-insensitive and quoted/backtick identifiers preserve
their exact spelling. Exact parser diagnostics include:

```text
keep expects a variable list or if clause
keep cannot combine a variable list with an if clause
keep does not accept options or assignment syntax
missing expression after if
```

For the selected projection form, unknown variables produce
`keep unknown variable: ...`; requested column order and row order are
preserved, and successful transforms report `Kept selected columns`.
When a requested variable is repeated, DuckDB preserves both projections and
renames the later result column deterministically (for example `age`, `age_1`).

## Bounded Rust contract

Add `Command::Keep { variables: Vec<String> }`, an owned transform result, and
an `ExecutionResult` variant. Require an active dataset before backend
initialization. Validate every requested identifier before staging; preserve
requested column order, row order, source path, eager mode, and row count;
publish updated schema and relation only after successful staging/inspection.
Failed validation, query, schema, count, or publication must leave both the
private active relation and published metadata unchanged. Repeated projection
operations are valid when the requested columns remain present.

Use quoted identifiers and the existing DuckDB staging/publication path. Do not
accept raw SQL, expand wildcards/ranges, or silently deduplicate requested
variables until duplicate-variable behavior is explicitly confirmed.

## Test contract

Focused coverage must include projection syntax and exact parser diagnostics,
no active dataset without backend initialization, ordinary and quoted column
projection order, row-order preservation, empty relations, unknown-column
validation before query, repeated transformations, duplicate-variable behavior,
dropped/corrupt active-relation failure, and failure-atomic metadata/relation
preservation.

Oracle recovery validation at the pinned revision reported `2 passed, 487 deselected`
for parser keep selection and `16 passed, 401 deselected` for the projection-focused
executor selection.

## Explicit deviations and deferrals

`keep if <expression>` and its boolean/null/overflow semantics remain deferred;
so do expression functions, arithmetic overflow reporting, lazy/materialized
execution, wildcard/range expansion, options, labels/panel metadata,
`last_operation`, CLI/JSON/MCP, and broader transformation sequencing.

Completion state: `complete` for contract recovery and bounded implementation;
hosted acceptance, merge, and main-documentation closeout remain pending.

# Bounded eager-runtime `assert` contract

Status: contract recovered; implementation pending (`partial`).

Producer: task owner, using `tabdat-migration` and `tabdat-data-semantics`.
Consumer: runtime implementation and review on `feat/runtime-assert`.

## Scope and selected skills

This loop targets one read-only `assert <boolean-expression>` command over the
existing eager local-Parquet DuckDB relation. It adds a typed expression boundary
and an owned result/error path without widening the runtime into a general
expression engine, lazy execution, CLI/JSON/MCP rendering, or row-level
diagnostics.

The migration contract comes from the pinned Python oracle. Data semantics cover
active-relation preconditions, SQL three-valued logic, ordering, and failure
atomicity. The Rust slice must quote identifiers and compile a typed AST rather
than interpolate user expression text.

## Python contract

Pinned oracle and environment:

- repository: `/Volumes/research/gitrepos/tabdat-explore`
- revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`
- tree: `601b236788872323af9277d2276a236154a0f129`
- Python: `3.13.3`
- `uv.lock` SHA-256: `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`

Authority:

- `src/tabdat/models.py:208-212,1244-1247`
- `src/tabdat/parser.py:277-278,837-867`
- `src/tabdat/executor.py:964-969`
- `src/tabdat/backend.py:1081-1110,2303-2627`
- `tests/test_assert.py`
- `docs/commands/assert.md`
- `docs/language-semantics.md:57-60`

The command is case-insensitive and has the form:

```text
assert <boolean-expression>
```

The oracle expression grammar covers identifiers (including quoted identifiers),
numeric and string literals, `null`, unary minus, parentheses, arithmetic
(`+`, `-`, `*`, `/`), comparisons (`==`, `!=`, `<`, `<=`, `>`, `>=`), and the
functions `abs`, `ceil`, `floor`, `ln`, `log`, `lower`, `round`, `sqrt`, `upper`,
and `e(sample)`. The first Rust slice intentionally supports the typed
identifier/literal/arithmetic/comparison subset; omitted functions and other
expression forms remain explicit deviations until separately implemented.

Exact parser diagnostics:

- `assert` → `assert expects a boolean expression`
- `assert age > 0, strict` → `assert does not accept options`
- `assert age > 0 if sex == 'F'` → `assert does not accept if clauses`
- `assert age = 0` → `assert does not accept assignment syntax`

Runtime result and semantics:

```text
AssertResult { checked: int, failed: int }
```

Every active row is checked. A false or SQL-NULL predicate fails; an empty
dataset passes with `(checked=0, failed=0)`. A successful check is read-only and
sets Python `last_operation` to `"assert"`; a failure preserves the active
dataset and prior operation. Explicit null comparisons use `IS NULL`/
`IS NOT NULL`; `null == null` is true and `null != null` is false. A non-boolean
predicate reports `predicate requires boolean expression`, and an unknown
identifier reports `expression unknown variable: <name>`.

The focused oracle module passes 10 tests, including parser, eager/lazy engine,
empty-dataset, state-preservation, CLI, and JSON cases. The parser-only
selection passes 1 test with 9 deselected; the runtime-focused selection passes
6 tests with 4 deselected.

## Rust contract

Add an owned expression AST to `tabdat-language` and a `Command::Assert {
expression }` variant. The bounded AST supports:

- identifiers with safe DuckDB quoting;
- validated numeric literals, quoted strings, and `null`;
- unary minus and parenthesized expressions;
- arithmetic and comparison operators listed above.

Runtime adds `AssertResult { checked: u64, failed: u64 }`,
`ExecutionResult::Assert`, and typed errors for unknown expressions, non-boolean
predicates, backend failure, and assertion failure. `NoActiveDataset {
command: "assert" }` must be returned before backend initialization.

The backend compiles the AST to generated SQL and evaluates one aggregate:

```sql
SELECT
  COUNT(*) AS checked,
  COUNT(*) FILTER (
    WHERE (<predicate>) IS NULL OR NOT (<predicate>)
  ) AS failed
FROM "__tabdat_active"
```

The expression is compiled twice in the failure filter only through the typed
compiler; no raw user SQL is accepted. The active relation and its published
metadata remain unchanged on both success and failure. This runtime currently
supports eager local Parquet only; lazy/materialized modes, CLI/JSON/MCP, full
function parity, and row-level diagnostics are deferred.

## Test contract

Use the existing synthetic fixture rows:

```text
age  bmi   sex  cost
30   22.5  F    100.0
42   25.0  M    150.0
54   27.5  F    NULL
```

Required focused cases:

1. Parser success for `assert age > 0`, quoted identifiers, null comparisons,
   arithmetic, and parentheses; exact rejection diagnostics above.
2. No active dataset returns `NoActiveDataset { command: "assert" }` without
   initializing a backend.
3. `assert age > 0` returns `(3, 0)` and preserves active metadata.
4. `assert cost > 0` returns the exact semantic failure
   `assertion failed: 1 of 3 rows failed`.
5. False and NULL predicates count as failures; empty `value == null` returns
   `(0, 0)`.
6. Unknown identifiers and non-boolean expressions fail before scanning and
   preserve active metadata.
7. Quoted column names are escaped safely; a dropped active relation maps to a
   typed backend failure without state mutation.

Oracle commands:

```sh
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider tests/test_assert.py
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_assert.py -k 'parse_assert_commands'
```

## Implementation mapping and gaps

The language layer owns parsing and AST types. The runtime session owns command
dispatch, active-dataset validation, typed results, and error atomicity. The
DuckDB adapter owns identifier quoting, type checking, expression-to-SQL
compilation, and the aggregate query. No native or foreign model types cross the
public boundary.

Known gaps are deliberate: Python lazy-engine parity, function calls, `e(sample)`,
full tokenizer/expression precedence parity, CLI/human/JSON/MCP surfaces, and
row-level failure diagnostics. They must not be claimed as implemented by this
slice.

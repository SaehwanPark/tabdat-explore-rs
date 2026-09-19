# Bounded eager-runtime `recode` contract

Status: accepted and verified on main at merge commit
`2e25cda63093621a5ded71eb774e35c78faa7ad1`.

Producer: task owner, using `tabdat-migration` and `tabdat-data-semantics`.
Consumer: the bounded eager runtime implementation and its focused review.

## Scope

Execute the already-parsed form
`recode <varlist> (<rule>) ... [, generate(<newvarlist>) | replace]`
against the active eager local-Parquet DuckDB relation. This is the next
bounded write-transform slice after the accepted eager `sort`/`gsort` paths.

The Rust slice supports:

- one or more existing source columns in parser-provided order;
- value-list rules whose inputs are numeric values or quoted/text values;
- numeric inclusive ranges with `min` and/or `max` endpoints;
- the `missing`, `nonmissing`, and `else` rule keywords;
- exactly one `generate(<newvarlist>)` or `replace` target mode;
- one generated output per source variable, appended after the existing
  columns, or in-place replacement of the selected source columns;
- first matching rule wins, with `else` as the unmatched fallback and the
  original value retained when no rule matches and no `else` rule exists; and
- staged DuckDB publication preserving source path, row count, column order,
  and eager execution metadata.

Rules are compiled as typed SQL `CASE` expressions. Numeric ranges are
validated against numeric source columns before staging. Text-producing rules
use text comparison/output semantics; numeric-only rules on numeric columns use
DuckDB's native numeric scalar semantics. Quoted identifiers and embedded
identifier quotes are passed through the shared identifier-quoting boundary.

Validation, schema/count inspection, and publication failures leave the
published metadata and active relation unchanged. Generated target names must
be unique, must match the number of source variables, and must not already
exist in the active schema.

## Python contract

Pinned authority:

- repository: `https://github.com/SaehwanPark/tabdat-explore.git`;
- revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`;
- tree: `601b236788872323af9277d2276a236154a0f129`;
- Python: `3.13.3` in the recorded oracle environment; and
- `uv.lock` SHA-256: `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.

Authoritative paths for this slice:

- `docs/commands/recode.md` — syntax and explicit `generate`/`replace`
  requirement;
- `src/tabdat/models.py:819-840` — `RecodeRange`, `RecodeRule`, and
  `RecodeCommand` fields;
- `src/tabdat/parser.py:3688-3909` — option, rule, range, keyword, and output
  parsing;
- `src/tabdat/executor.py:1376-1428` — active-state dispatch and validation;
- `src/tabdat/backend.py:1481-1586` — CASE compilation, fallback, generated
  column placement, and replacement behavior; and
- `tests/test_parser.py:1878-1917` plus
  `tests/test_executor.py:10021-10131` — focused parser/execution behavior.

The recovered focused oracle command was:

```text
uv run --no-sync pytest -q -p no:cacheprovider tests/test_parser.py tests/test_executor.py -k recode
3 passed, 903 deselected in 1.94s
```

The Python executor also supports lazy/materialized execution, panel metadata,
labels, broader type/coercion behavior, and CLI/JSON/MCP output. Those are
evidence for later slices, not claims of this Rust boundary.

## Bounded Rust contract

Add owned typed language values for recode inputs/ranges/rules and a typed
target mode. Add `RecodeResult { dataset: DatasetInfo }` and
`ExecutionResult::Recode`. Add command-owned errors for:

- no source variables or rules;
- unknown source variables;
- generated-output cardinality, duplicate-name, or existing-target
  validation failures;
- a range applied to a non-numeric source column; and
- backend/staging/schema/count/publication failure (`recode failed`).

The implementation must:

1. return `NoActiveDataset { command: "recode" }` before backend work;
2. validate all source variables, target mode/cardinality/collisions, and range
   source domains before staging;
3. compile each source-target pair into a quoted SQL `CASE` expression while
   retaining rule order, missingness keywords, inclusive ranges, else fallback,
   and unchanged-value fallback;
4. stage a full ordered projection, append generated targets only in generate
   mode, inspect schema and row count, publish through the shared transaction,
   and update session metadata only after publication succeeds; and
5. preserve existing data/schema metadata and row order except for the values
   explicitly transformed by the recode rules.

The parser owns lexical values and exact quoted identifier spelling. The
runtime owns domain validation, SQL quoting, relation mutation, and atomic
state publication.

## State-transition contract

| Situation | Active relation | Published metadata | Backend initialization |
| --- | --- | --- | --- |
| No active dataset | unchanged | unchanged | must not occur |
| Empty/rule/source/target validation failure | unchanged | unchanged | already-loaded backend only |
| Stage, SQL, schema, row-count, or publication failure | unchanged | unchanged | already-loaded backend only |
| Successful generate or replace recode | staged transformed relation becomes active | updated schema/count with source/execution metadata preserved | already-loaded backend |

## Test contract

Focused Rust coverage must include:

- no active dataset without initializing a backend;
- parser-produced numeric ranges, value lists, missingness keywords, else, and
  both target modes;
- generated recodes with one output per source, appended schema order, and
  numeric range results;
- replacement recodes with missing/nonmissing rules, unchanged fallback, and
  string source/output behavior;
- quoted source and generated identifiers, including an embedded double quote,
  and an empty relation;
- validation failures for unknown sources, target cardinality/collision,
  nonnumeric ranges, empty direct variables/rules, with metadata and rows
  unchanged; and
- a staged/backend failure mapping to `RecodeFailed` while the previously
  published metadata and relation remain available.

No new dependency, native backend, FFI, unsafe code, or ADR decision is
required. Lazy/materialized execution, panel/label metadata, last-operation
state, formatting, CLI/JSON/MCP, and broad transform sequencing remain
deferred.

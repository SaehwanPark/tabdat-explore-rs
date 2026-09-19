# Bounded eager-runtime `replace` contract

Status: draft at the contract checkpoint.

Producer: task owner, using `tabdat-migration` and `tabdat-data-semantics`.
Consumer: the bounded eager runtime implementation and its focused review.

## Scope

Execute the already-parsed form
`replace <target> = <expression> [if <condition>]` against the active eager
local-Parquet DuckDB relation. This is a deliberately bounded continuation of
the accepted eager `generate` staging path.

The runtime subset supports:

- an existing target column, preserving its schema position;
- numeric target assignments from numeric identifiers/literals, unary minus,
  `+`, `-`, `*`, and `/`, with the existing checked numeric SQL compiler;
- string target assignments from string identifiers/literals;
- an explicit `null` assignment for numeric or string targets, cast to the
  target's declared DuckDB type;
- optional boolean or missing predicates made from the existing typed
  identifier/literal/null/arithmetic/comparison expression nodes, including
  numeric and string comparisons and null-aware equality/inequality;
- quoted/backtick identifiers and targets, including embedded double quotes;
- true-condition replacement only; false and SQL-NULL conditions preserve the
  existing target value;
- preserved source metadata, row order, row count, SQL NULL values, and eager
  execution metadata; and
- staged publication through the existing `__tabdat_next` transaction path.

Validation occurs before staging. A failed validation, expression compilation,
schema or row-count inspection, SQL stage, or publication leaves both the
published `DatasetInfo` and private `__tabdat_active` relation unchanged.

The parser's richer `GenerateExpression` remains intact. Runtime rejects
function calls, unsupported target domains, cross-domain assignments, numeric
or string truthiness, and comparisons used as replacement values explicitly.

## Python contract

Pinned authority:

- repository: `https://github.com/SaehwanPark/tabdat-explore.git`;
- revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`;
- tree: `601b236788872323af9277d2276a236154a0f129`;
- Python: `3.13.3` in the recorded oracle environment;
- `uv.lock` SHA-256: `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.

Authoritative paths for this slice:

- `docs/commands/replace.md` — syntax, domain, missingness, and arithmetic
  behavior;
- `docs/language-semantics.md:28-38,77-117,222-233` — null-aware predicates,
  expression domains, replacement rules, arithmetic normalization, and write
  target/failure policy;
- `src/tabdat/models.py:289-300` — `ReplaceCommand`;
- `src/tabdat/parser.py:660-668,3073-3148` — command and expression parsing;
- `src/tabdat/executor.py:1437-1464,6550-6559` — validation, mutation, and
  active-state publication;
- `src/tabdat/backend.py:1451-1479,2303-2395,2524-2585` — replacement SQL,
  expression compilation, and domain validation; and
- `tests/test_executor.py:6876-6967,7287-7421,7626-7659,7944-8025,
  8134-8159,8740-8776,9628-9685` — eager replacement values, predicates,
  missing/non-finite arithmetic, domain errors, quoted names, sequencing, and
  atomic failure behavior.

The oracle has broader eager/lazy behavior, exact integral overflow counts,
non-finite normalization rules, panel/label metadata, `last_operation`, and
machine/reporting interfaces. Those are evidence for future slices, not claims
of this Rust boundary.

## Bounded Rust contract

Add an owned `ReplaceResult { dataset: DatasetInfo }` and
`ExecutionResult::Replace`. Add command-owned typed errors for:

- a missing target (`replace unknown variable: <name>`);
- unknown expression/predicate identifiers (`expression unknown variable: ...`);
- a non-boolean/non-missing predicate (`predicate requires boolean expression`);
- cross-domain or unsupported target assignments;
- function calls and unsupported replacement expression forms; and
- backend/staging/publication failure (`replace failed`).

The implementation must:

1. return `NoActiveDataset { command: "replace" }` before backend work;
2. validate the target, every referenced identifier, target/value domains, and
   predicate domain before staging;
3. compile only the accepted expression subset using quoted identifiers and the
   existing safe numeric SQL helpers;
4. stage one `SELECT` in source schema order, replacing the target expression
   with `CASE WHEN <condition> THEN <value> ELSE <target> END` when a predicate
   is present;
5. cast direct NULL replacement values to the target's declared type;
6. inspect staged schema and row count, publish through the shared transaction,
   and update session metadata only after publication succeeds; and
7. preserve the active source path, row order, row count, SQL NULL behavior,
   target position, and eager execution metadata on success.

Exact overflow-row reporting, lazy/materialized execution, function calls,
boolean/other backend target domains, labels/panel metadata, `last_operation`,
CLI/JSON/MCP, and broad transform sequencing remain explicitly deferred.

## State-transition contract

| Situation | Active relation | Published metadata | Backend initialization |
| --- | --- | --- | --- |
| No active dataset | unchanged | unchanged | must not occur |
| Target/expression/predicate validation failure | unchanged | unchanged | already-loaded backend only |
| Stage, schema, row-count, or publication failure | unchanged | unchanged | already-loaded backend only |
| Successful replacement | staged relation becomes active | replaced with staged schema/count and preserved source/mode | already-loaded backend |

## Test contract

Focused runtime coverage must include:

- no active dataset without initializing a backend;
- parsed numeric replacement with and without predicates, preserving schema
  position and row order;
- string replacement and explicit NULL replacement with target-type
  preservation;
- null-aware and false predicates preserving nonmatching values;
- arithmetic expression precedence, quoted/embedded-quote identifiers, and an
  empty relation;
- target, unknown-identifier, domain, predicate, and function-call failures
  preserving metadata and rows; and
- a dropped or mismatched active relation mapping to `ReplaceFailed` while the
  previously published metadata and relation remain available.

The existing parser contract tests remain authoritative for syntax. No new
dependency, native backend, FFI, unsafe code, or ADR decision is required.

Completion state: draft; oracle recovery and contract definition are complete,
while implementation, review, hosted acceptance, merge, and post-merge
verification remain pending.

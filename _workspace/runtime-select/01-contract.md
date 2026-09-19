# Bounded eager-runtime `select` contract

Status: contract recovered for the bounded eager slice; implementation is in
progress on `feat/runtime-select`.

Producer: task owner, using `tabdat-migration` and `tabdat-data-semantics`, with
independent Python-contract and Rust-boundary reconnaissance.
Consumer: bounded `select` implementation, review, and hosted acceptance.

## Scope

This loop targets the explicit-varlist projection already recognized by the
language layer:

- `select <explicit-varlist>` projects the requested columns from the active
  eager local-Parquet DuckDB relation;
- requested column order and duplicate requests are preserved (DuckDB gives a
  repeated projection a deterministic suffixed name such as `age_1`);
- quoted and backtick identifiers preserve exact spelling, punctuation, commas,
  whitespace, and embedded quotes;
- row order, row count, NULL values, source path, and eager mode are preserved;
- validation, staging, schema/count inspection, and publication are atomic with
  respect to published metadata and the private active relation.

`select if <expression>` and all lazy/materialized parity are outside this
bounded Rust slice. The parser’s existing syntax-only contract remains shared;
runtime execution is the new boundary under test.

## Python contract

Pinned authority:

- repository: `/Volumes/research/gitrepo/tabdat-explore`;
- revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`;
- tree: `601b236788872323af9277d2276a236154a0f129`;
- Python: `3.13.3`;
- `uv.lock` SHA-256:
  `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.

Authority paths:

- `src/tabdat/models.py:256-259` (`SelectCommand`);
- `src/tabdat/parser.py:629-634` (syntax branch);
- `src/tabdat/executor.py:999-1000,1364-1368` (dispatch/execution);
- `src/tabdat/backend.py:1152-1168,2638-2661` (projection and publication);
- `docs/commands/select.md:1-18`;
- `tests/test_parser.py` select syntax cases;
- `tests/test_executor.py:8134-8159,8377-8476,8706-8737`.

The pinned parser accepts case-insensitive command names, exact identifier
spelling, ordered/repeated names, and quoted/backtick identifiers. Recovered
diagnostics include:

```text
select expects at least one variable
select only accepts a variable list
select assignment requires a target before =
select assignment requires an expression after =
comma must be followed by at least one option
missing expression after if
duplicate if clause
incomplete expression after >=
unsupported token in command: ==
unsupported token in command: -
unsupported token in command: +
unsupported token in command: !
unsupported token in command: @
unsupported token in command: :
```

Python delegates execution to the existing `keep_columns` backend operation,
so its observable unknown-variable failure currently says `keep unknown
variable: ...` and a direct empty typed request says `keep failed`. Those are
implementation leakage in the oracle; the Rust boundary will use command-owned
`select` diagnostics and record that intentional improvement in the evidence.

## Bounded Rust contract

Add `SelectResult`, `ExecutionResult::Select`, and typed select errors. The
runtime must:

1. return `NoActiveDataset { command: "select" }` before backend work;
2. reject an empty direct typed request deterministically;
3. validate every requested name before querying;
4. reuse `DuckDbBackend::project_columns` for quoted SQL, staging, schema/count
   inspection, and transactional publication;
5. preserve requested order and duplicate projection behavior;
6. publish `Session.active_dataset` only after successful publication;
7. preserve both published metadata and the private active relation on every
   validation, SQL, inspection, or publication failure.

No new backend, FFI, dependency, unsafe code, or ADR decision is required.

## Test contract

Focused Rust coverage must include:

- parser case-insensitivity, quoted/backtick names, duplicate-if and incomplete
  operator diagnostics;
- no active dataset and backend non-initialization;
- requested order, duplicate names, quoted names including an embedded double
  quote, row/NULL/count/source preservation, and empty relations;
- repeated projections and direct empty typed input;
- unknown-variable validation with metadata/private-relation preservation;
- corrupt or mismatched active relation mapping to `SelectFailed` without
  changing the previously published relation.

## Explicit deviations and deferrals

Predicate filtering, expression execution, lazy/materialized behavior,
wildcard/range expansion, label/panel metadata, `last_operation`, formatting,
CLI/REPL, JSON/MCP, and broad transform sequencing remain deferred. This
bounded contract does not claim full Python `select` parity or broad Phase 4
completion.

Completion state: contract recovery complete; implementation, independent
review, hosted acceptance, merge, and branch cleanup pending.

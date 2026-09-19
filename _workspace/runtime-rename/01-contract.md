# Bounded eager-runtime `rename` contract

Status: proposed at the contract checkpoint.

Producer: task owner, using `tabdat-migration` and `tabdat-data-semantics`.
Consumer: the bounded eager runtime implementation and its focused review.

## Scope

Execute the already-parsed form `rename <old> <new>` against the active eager
local-Parquet DuckDB relation. This is the next bounded schema-transform slice
after the accepted eager `select`, `generate`, and `replace` paths.

The runtime subset supports:

- one existing source column and one new target name;
- exact, case-sensitive identifier spelling, including names supplied through
  the parser's quoted/backtick identifier forms and embedded double quotes;
- preservation of source schema position, DuckDB logical type, SQL NULL values,
  row order, row count, source path, and eager execution metadata; and
- staged publication through the existing `__tabdat_next` transaction path.

Validation occurs before staging. A missing source, an already-used target,
SQL/schema/row-count inspection failure, or publication failure leaves both the
published `DatasetInfo` and private `__tabdat_active` relation unchanged.
The same-name request is rejected as an already-used target, matching the
oracle's target-collision rule.

## Python contract

Pinned authority:

- repository: `https://github.com/SaehwanPark/tabdat-explore.git`;
- revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`;
- tree: `601b236788872323af9277d2276a236154a0f129`;
- Python: `3.13.3` in the recorded oracle environment;
- `uv.lock` SHA-256: `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.

Authoritative paths for this slice:

- `docs/commands/rename.md` — syntax and single-column rename intent;
- `docs/language-semantics.md:1-18,28-38,162-170` — exact identifier
  spelling, quoted names, and row-preserving transforms;
- `src/tabdat/models.py:262-264` — `RenameCommand` fields;
- `src/tabdat/executor.py:1022-1023,1370-1374,6544-6549` — dispatch,
  active-state publication, panel-metadata adaptation, and validation;
- `src/tabdat/backend.py:1280-1300` — source-order SQL projection and
  target/source validation; and
- `tests/test_executor.py:8390-8404,8740-8776,9628-9685` — row-order,
  sequencing, and atomic target-validation evidence.

The focused oracle probe recovered for this slice is:

```text
uv run --no-sync pytest -q -p no:cacheprovider tests/test_executor.py \
  -k "rename or quoted_identifiers_execute_with_exact_spelling or \
      rename_generate_and_replace_update_active_dataset or \
      phase_3_transformations_report_user_facing_errors"
3 passed, 414 deselected
```

The Python executor also preserves panel metadata and supports eager, DuckDB-
lazy, and Polars-lazy sessions. Those broader behaviors are evidence for later
slices, not claims of this Rust boundary.

## Bounded Rust contract

Add an owned `RenameResult { dataset: DatasetInfo }` and
`ExecutionResult::Rename`. Add command-owned typed errors for:

- a missing source (`rename unknown variable: <name>`);
- an already-used target (`rename target already exists: <name>`); and
- backend/staging/publication failure (`rename failed`).

The implementation must:

1. return `NoActiveDataset { command: "rename" }` before backend work;
2. validate the source and target against the published active schema before
   staging;
3. stage one source-order `SELECT`, aliasing only the source column to the
   quoted target name;
4. inspect staged schema and row count, publish through the shared transaction,
   and update session metadata only after publication succeeds; and
5. preserve the active source path, row order, row count, SQL NULL behavior,
   source-column position, logical type, and eager execution metadata on
   success.

Exact Python panel/label metadata behavior, lazy or materialized execution,
wildcards or multi-column rename forms, `last_operation`, formatting, CLI,
JSON, MCP, and broad transform sequencing remain explicitly deferred.

## State-transition contract

| Situation | Active relation | Published metadata | Backend initialization |
| --- | --- | --- | --- |
| No active dataset | unchanged | unchanged | must not occur |
| Source/target validation failure | unchanged | unchanged | already-loaded backend only |
| Stage, schema, row-count, SQL, or publication failure | unchanged | unchanged | already-loaded backend only |
| Successful rename | staged relation becomes active | renamed schema/count with preserved metadata | already-loaded backend |

## Test contract

Focused runtime coverage must include:

- no active dataset without initializing a backend;
- parsed rename preserving schema position, logical type, row order, values,
  NULLs, row count, source, and eager metadata;
- quoted identifiers, including an embedded double quote, and an empty
  relation;
- target collision, same-name collision, and unknown-source failures;
- failure atomicity for all validation failures; and
- a dropped or mismatched active relation mapping to `RenameFailed` while the
  previously published metadata and relation remain available.

The existing parser contract tests remain authoritative for syntax. No new
dependency, native backend, FFI, unsafe code, or ADR decision is required.

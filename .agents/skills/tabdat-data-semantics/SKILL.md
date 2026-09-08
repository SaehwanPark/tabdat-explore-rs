---
name: tabdat-data-semantics
description: Use when designing, implementing, or reviewing TabDat DuckDB data commands, relation lifecycles, lazy execution, missingness, ordering, labels, or atomic session mutations.
---

# TabDat data semantics

## When to use

Use for data loading, inspection, transformations, combination, SQL, and export.
This skill owns relation/session correctness, not numerical estimator validation
or low-level ReadStat ownership. Pair with those specialists at the boundary.

## Required inputs

- Command contract, representative dataset, expected schema/results/errors.
- Existing session and data implementation, if present, plus Python fixtures and
  revision when parity is requested.
- Relevant proposal §6 and roadmap Phase 4; start from
  [AGENTS.md](../../../AGENTS.md). DuckDB is planned, not installed in the scaffold.

## Workflow

1. Recover observable semantics with `tabdat-migration` when necessary. Specify
   missingness, coercion, overflow, ordering, identifier rules, and eager/lazy
   behavior before expressing the command as SQL. Do not substitute DuckDB defaults
   for TabDat's contract or infer Stata behavior solely from familiar syntax.
2. Define a state-transition table: active relation, named tables, materialization
   status, schema cache, row count known/unknown, labels, panel metadata, and any
   affected estimation state. For each field state whether it is preserved,
   replaced, invalidated, or unchanged on failure, citing the contract.
3. Map work onto the canonical DuckDB execution path. Use safe parameter binding
   for values and deliberate identifier quoting/validation; do not interpolate
   untrusted identifiers or expressions directly into generated SQL. Keep the
   explicit user `sql` command distinct from generated-query boundaries.
4. Preserve lazy scanning until the command requires materialization. Metadata,
   status, and completion should not force scans unnecessarily. Unknown row count
   is not zero; cached metadata needs explicit invalidation on relevant changes.
5. Design fallible mutations so failed validation, conversion, or backend execution
   cannot leave partially updated relation/session metadata. Use the appropriate
   transactional/staging boundary and verify both data and metadata rollback.
6. Keep data/label conversions explicit at I/O boundaries, including DTA missing
   values and value labels. Measure copies/materialization where Arrow/DuckDB or
   native adapters exchange large data. Use `tabdat-native-backends` for ReadStat
   memory/callback safety rather than letting native types enter session state.
7. Test the implemented contract with tiny inspectable fixtures and run root
   checks. Differential-test matching eager/lazy `.td` workflows using the same
   input and semantic JSON comparator when the oracle is available.

## Validation matrix

Choose the applicable cases and explain omitted ones:

| Surface | Evidence to require |
| --- | --- |
| Load/inspect | Empty input, all-missing columns, schema/types, known/unknown count, repeated inspection without unintended materialization |
| Transform | Missing propagation, coercion/overflow, quoted identifiers, ordering where promised, failed mutation leaves prior state intact |
| Join/append/reshape | Duplicate/missing keys, schema mismatch, row ordering and resulting metadata according to the recovered contract |
| Labels/panel | Preservation or clearing after selection, rename, row/column changes, and failure |
| SQL/export | Generated-query escaping, result schema, contract-defined ordering, round-trip behavior, deterministic errors |
| Lazy/cache | Eager/lazy equivalence, cache invalidation, scan/materialization evidence, cancellation if implemented |

Do not sort away ordering regressions in comparisons. Do not equate SQL NULL,
NaN, or extended missing values without an explicit conversion contract. Use
small local fixtures by default; remote/S3 tests require an available authorized
endpoint and should report unavailable infrastructure separately from failures.

## Expected outputs and stop conditions

Return the command contract, state-transition table, execution/materialization
mapping, changed paths, fixture/comparison evidence, and measured costs if claimed.
Use the [shared handoff format](../../../docs/harness/tabdat/team-spec.md) when
results must survive the session. Mark missing oracle or remote infrastructure
as a validation gap, not a pass. Stop for a semantics decision when missingness,
ordering, or metadata behavior cannot be established; never add a second data
engine just to sidestep an unexplained mismatch.

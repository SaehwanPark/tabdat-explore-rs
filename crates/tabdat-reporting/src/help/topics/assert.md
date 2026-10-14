# assert

Check every active row against a bounded predicate before analysis or persistence.

```text
assert <comparison-expression>
assert age >= 0
assert bmi == null
assert sex != "unknown"
```

Comparisons, arithmetic, column references, numeric/string literals and explicit SQL NULL
are supported. Function calls, logical-expression breadth, Boolean literal keywords, options,
`if` clauses, assignment and `by:` execution are not installed. Numeric columns alone are not
truth values. SQL NULL equality/inequality is an explicit missingness test; otherwise FALSE or
SQL-NULL predicate results fail. Stored empty strings and nonfinite values retain the existing
native expression semantics; this is not a finite-value check.

```text
assertion passed: 3 rows
Failed rows: 0
```

An empty dataset passes with zero checked and failed rows. Failure reports
`assertion failed: F of N rows failed` and stops later commands in batch/scripts; the interactive
session can recover. The command is read-only: data, order, labels, named-table ownership and
cached model/sample are unchanged. It checks current rows, not the retained estimation sample.
There is no row-level diagnostic, filtering or repair operation.

CLI-v1 JSON reports successful `AssertResult` with exactly `data.checked` and `data.failed`.
A semantic failure is an execution error, not a successful assertion envelope. Script errors
retain source locations. A noninteractive output write/flush failure also stops later commands;
already completed effects/results are not rolled back. Local eager CSV/Parquet loading is
installed; lazy execution is not advertised.

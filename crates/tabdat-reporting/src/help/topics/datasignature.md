# datasignature

Compute a deterministic SHA-256 fingerprint of the active relation for reproducibility checks.

```text
use survey.parquet
datasignature
```

```text
Data signature
Algorithm: sha256
Rows: 3
Columns: 4
Signature: <64 lowercase hexadecimal characters>
```

The existing TabDat-native versioned protocol covers public column names and canonical logical
types, active row order and every cell value. SQL NULL and nonfinite values have explicit
encodings. Labels, panel metadata, source paths and execution-engine identifiers are excluded.
The digest describes current data, not the cached regression estimation sample. Repeating the
request without data/schema/order changes returns the same digest; changing labels alone does
not. Serialization uses the existing producer, not an additional reporting scan.

The command is read-only and accepts no varlist, options, `if` clause, assignment or `by:`
execution. It leaves rows, schema, labels, named ownership and cached model/sample unchanged.
An empty relation still has a schema-dependent digest and zero rows. Local eager CSV/Parquet
loading is installed; lazy/Polars loading is not installed. Parquet roundtrips can preserve the
active signature when values/schema/order are preserved; CSV may re-infer types or NULLs and
is not a general signature-preserving archive.

CLI-v1 JSON reports `DatasignatureResult` with exactly `data.algorithm`, `data.signature`,
`data.row_count` and `data.column_count`. Success streams as an ordinary nested-script leaf.
Execution failures have no successful fingerprint envelope; noninteractive write/flush failures
stop later commands while retaining completed effects/results.

This is not a checksum of file bytes, cryptographic authentication, a persistent saved-signature
comparison/verification command or a promise of compatibility with other statistical packages.

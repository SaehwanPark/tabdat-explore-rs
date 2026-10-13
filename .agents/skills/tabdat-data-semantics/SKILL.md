---
name: tabdat-data-semantics
description: Check observable DuckDB/data/IO behavior, missingness, ordering, metadata, and failure state in supported workflows.
---

# Data and session semantics

Use for loading, inspection, transformations, SQL, joins, persistence and session
state. Start from the current code and a small representative fixture; the Python
oracle is optional unless supported behavior is being compared or migrated.

- Define the supported input/output schema, type conversions, missingness and
  row-order promises. Do not assume DuckDB defaults are automatically TabDat
  semantics.
- Identify affected relation, labels, panel metadata, cached counts and model
  state. Specify what must remain unchanged on failure.
- Use one canonical DuckDB path with deliberate SQL value binding and identifier
  validation/quoting; distinguish explicit user SQL from generated SQL.
- Avoid eager scans just to gather metadata; unknown counts are not zero.
- Test empty/missing data, representative type failures, successful mutation,
  rollback/failure atomicity, and appropriate export/output recovery.
- Keep backend-native rows/handles inside the data adapter. Call in native/FFI
  expertise only for native ownership changes.

Prefer the smallest meaningful end-to-end fixture over exhaustive combinations
of every proposed data mode. Document unsupported lazy/remote/DTA behavior
honestly. See [architecture](../../../ARCHITECTURE.md) and
[current status](../../../SPEC.md).

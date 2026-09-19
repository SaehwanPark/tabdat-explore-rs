# Independent review: bounded eager-runtime `drop`

Status: review complete at `9a90db8`; no blocking correctness findings.

The review examined parser diagnostics and predicate deferral, active-schema
validation, complement ordering, identifier quoting, all-column rejection,
staging/publication atomicity, metadata/private-relation preservation, and the
backend-initialization boundary.

The following low-severity evidence gaps were closed before hosted acceptance:

- malformed tokens inside a deferred predicate now preserve tokenizer errors
  instead of being swallowed by the deferral helper;
- an end-to-end fixture uses an identifier containing an embedded double quote,
  exercising DuckDB identifier escaping through `drop`;
- the corrupt-active-relation unit test uses a mismatched private table and
  verifies the original relation remains queryable after `DropFailed`.

The implementation validates all requested names before backend work, computes
the complement in published schema order, rejects removing every column before
staging, quotes every projected identifier, and updates `Session.active_dataset`
only after transactional publication. Focused parser/runtime tests, full locked
Cargo gates, and diff checks pass. No new dependency or unsafe boundary was
introduced.

Predicate-form execution and its boolean/null/overflow semantics remain
explicitly deferred, along with lazy/materialized execution, wildcard/range
expansion, labels/panel metadata, `last_operation`, formatting, CLI/JSON/MCP,
and broader transform sequencing.

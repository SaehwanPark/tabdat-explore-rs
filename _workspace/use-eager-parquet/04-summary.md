# Eager local-Parquet runtime handoff

Status: pending post-merge handoff; PR #22's accepted implementation is ready,
but the squash SHA and post-merge workflow links are recorded only after merge.

This artifact is the owner-to-maintainer summary for the bounded runtime slice.
It will be finalized on `main` with the squash-merge SHA, post-merge checks, and
the exact files that carry the accepted contract, evidence, review, ADR, and
current-state updates. Until then, the implementation-head evidence and hosted
matrix are recorded in [`02-evidence-data.md`](02-evidence-data.md), and the
review disposition is recorded in [`03-review.md`](03-review.md).

The accepted scope is one eager `Command::Use` of an existing local Parquet file
through a private DuckDB adapter. Broad `use` parity, lazy/remote/other-format
loading, general relation APIs, inspect/count execution, CLI/REPL, scripts,
reporting, JSON/MCP, and production DuckDB packaging remain deferred.

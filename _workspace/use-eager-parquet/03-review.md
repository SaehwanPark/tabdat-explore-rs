# Eager local-Parquet runtime review

Status: accepted for bounded evaluation and merged on `main`; broad runtime
parity and production DuckDB integration remain deferred.

Reviewer set: independent runtime/parser, contract, and workspace/native review
agents

Contract reviewed: `01-contract.md`

Evidence reviewed: `02-evidence-data.md`

## Acceptance to review

- `Session::new()` is backend-free; a supported eager local-Parquet `Use`
  request initializes DuckDB only when execution is attempted.
- The public result boundary owns source path, ordered column names/types, row
  count, eager mode, and absent lazy engine; no DuckDB or Arrow handle escapes.
- Existing regular local `.parquet` files are staged and inspected before an
  atomic active-relation replacement; failed reads do not publish metadata or
  destroy the prior active relation.
- URI, lazy, non-Parquet, missing, directory, and CSV-option forms are rejected
  deterministically within the bounded runtime contract.
- Existing parser/scaffold behavior remains unchanged, and the root binary is
  still a scaffold rather than a claimed CLI/runtime surface.
- CSV/DTA/Feather/Arrow, remote sources, lazy plans, named tables, general
  relation APIs, transformations, statistics, labels, `describe`/`count`,
  reporting, CLI, script, and MCP behavior remain explicitly deferred.

## Review passes and findings

The independent parser/runtime pass found a P2 parity gap: Rust originally
checked existence before suffix, unlike Python's suffix-first resolver. Commit
`aea6728` reorders validation and adds missing/directory unsupported-suffix
regressions. The same pass identified Python `~` expansion; Rust leaves it to the
caller and records the deferral in the contract/evidence rather than silently
claiming parity.

The workspace/native pass found a high-severity CI policy defect at the earlier
head `57753b5`: the unchanged plain `cargo geiger` step failed the required
hosted security job on dependency asset warnings even though first-party crates
had zero unsafe usage ([failed run](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35225665696)).
Commit `cb0e8c3` changes the CI step to parse JSON, assert `forbid(unsafe_code)`
and zero first-party unsafe counts, and surface transitive inventory as a warning.
The same pass found no runtime-specific hosted build workflow;
`.github/workflows/runtime.yml` now adds a Linux x86_64 runtime check path.

The final review found no additional implementation defect beyond the documented
`~` and diagnostic/scope deviations. The suffix-order gap was fixed in `aea6728`,
the hosted geiger package-mapping/policy-visibility defect was fixed in `559f293`,
and the temporary-path isolation and reproducible eager-failure probe were added
in the same follow-up. The private adapter's ownership, transactional cleanup,
exclusive `&mut self` access, no-`Send`/`Sync` promise, and unpublished
license/notice disposition are recorded in ADR 0007 and the evidence artifact.
No finding is treated as closed merely because local Cargo checks pass.

## Verification reviewed

The evidence artifact records the focused/full pinned Python checks, Rust
baseline, dependency/advisory scans, metadata-driven geiger inventory, and
`git diff --check`. All eight hosted checks passed on implementation head
`559f293`; links and job IDs are recorded in `02-evidence-data.md`.

## Disposition

PR #22's bounded contract was squash merged as
[`26dba2b`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/26dba2b6709e2a5bd16ea5e7f98463bf6d39d4e2).
All post-merge workflows passed; their run and job links are recorded in
[`04-summary.md`](04-summary.md). Do not claim broad Phase 4 `use` support or
blanket DuckDB production adoption from this slice.

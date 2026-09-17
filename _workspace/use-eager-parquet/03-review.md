# Eager local-Parquet runtime review

Status: partial; the implementation and local evidence are ready on draft PR
#22. Review findings have fixes in the current worktree, while hosted checks and
final ADR disposition remain open.

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

The remaining review pass will record final severity-ranked findings, fixes and
commit references, and the disposition for any unresolved ownership,
transaction, platform, license, or scope concern. No finding is treated as
closed merely because local Cargo checks pass.

## Verification reviewed

The evidence artifact records the focused/full pinned Python checks, Rust
baseline, dependency/advisory scans, metadata-driven geiger inventory, and
`git diff --check`. Before promotion, rerun the final head and attach links for
the Rust baseline, dependency/unsafe policy, Linux runtime-boundary workflow,
and every path-scoped native workflow triggered by the final diff.

## Disposition

Keep PR #22 draft until the independent reviews and every required hosted job
are green. If the bounded contract is accepted, mark it ready, squash-merge it,
delete the temporary branch, and then update this artifact and the ADR with the
merge SHA and final hosted links. Do not claim broad Phase 4 `use` support or
blanket DuckDB production adoption from this slice.

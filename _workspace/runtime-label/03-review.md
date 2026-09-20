# Runtime label review

Status: accepted after PR [#55](https://github.com/SaehwanPark/tabdat-explore-rs/pull/55)
and merge-head hosted verification.

Review basis: [01-contract.md](01-contract.md), the pinned oracle evidence in
[02-evidence-migration.md](02-evidence-migration.md), and the implementation
checkpoint [`aa6f952`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/aa6f952f859630a8c77b24887f7b96a012f9d24f).

## Findings

No blocking correctness, security, performance, or maintainability findings
remain for the bounded contract.

### Correctness and state

- Parser values are owned and preserve quoted text, identifier spelling, signs,
  numeric text, duplicate-value checks, and bounded option diagnostics.
- Label mutations validate active variables and set references before replacing
  the session metadata; failed validation leaves the prior relation and labels
  unchanged.
- Encode publishes the new relation before publishing its generated set and
  attachment; decode consumes only an attached integer mapping and publishes
  variable-label propagation after backend success.
- `use` clears labels; rename and surviving-column projections reconcile
  variable labels and attachments; value-changing replace and in-place recode
  invalidate affected attachments while retaining named sets.

### Security and boundary safety

- No unsafe Rust, FFI, new dependency, or native backend surface was added.
- Label metadata is copied into owned result values and does not expose DuckDB
  handles or backend lifetimes.
- The implementation remains library-only and does not claim CLI, JSON, MCP,
  inspection, or persistence support.

### Scope and residual risk

The implementation intentionally does not implement JSON persistence, DTA label
ingestion, label rendering in `describe`/`codebook`/`tabulate`, lazy execution,
panel metadata, or broad transform sequencing. Those remain separate slices.

## Disposition

The bounded eager local-Parquet session-local label contract is accepted. PR
head, merge-head, and policy workflows passed; documentation-closeout workflow
links are recorded in the companion summary.


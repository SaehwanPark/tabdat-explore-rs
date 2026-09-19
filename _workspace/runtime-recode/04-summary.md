# Bounded eager-runtime recode closeout

Status: ready for documentation closeout and merge after final PR-head
acceptance.

## Accepted scope

The runtime executes the parsed eager local-Parquet subset of the recode
VARLIST (RULE) ... [, generate(NEWVARLIST) | replace] form for active
DuckDB sessions. It supports scalar values, inclusive numeric ranges,
missing/nonmissing and else rules, ordered first-match behavior, unchanged
fallback, one generated output per source, and in-place replacement.

The implementation validates all sources and generated targets before staging,
quotes identifiers and literals, stages a full ordered projection, checks the
result schema and row count, and publishes atomically through the existing
DuckDB session boundary. Failure leaves the prior relation and published
metadata unchanged.

## Durable records

- [01-contract.md](01-contract.md) — pinned oracle contract and bounded scope;
- [02-evidence-migration.md](02-evidence-migration.md) — oracle, local, and
  hosted evidence;
- [03-review.md](03-review.md) — three-pass review disposition;
- [SPEC.md](../../SPEC.md) — verified implementation-state record; and
- [roadmap Phase 6.3](../../docs/TABDAT_RUST_PORT_ROADMAP.md).

PR [#52](https://github.com/SaehwanPark/tabdat-explore-rs/pull/52) was opened
as a draft at the contract checkpoint. It will be marked ready and squash
merged after the documentation commit passes its PR-head workflows.

Panel/label metadata, lazy or materialized execution, last-operation state,
formatting, CLI/JSON/MCP, and broad transform sequencing remain deferred.

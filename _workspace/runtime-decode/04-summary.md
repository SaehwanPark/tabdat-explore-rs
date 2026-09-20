# Bounded eager-runtime `decode` closeout

Status: accepted and verified on `main` at merge commit
[`0845e6c`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/0845e6cf37a50c317d7ee30acee2d85474c7bcd2), via
[PR #54](https://github.com/SaehwanPark/tabdat-explore-rs/pull/54).

## Accepted scope

The Rust runtime now executes the bounded eager local-Parquet form
`decode <numvar>, generate(<newvar>)` when the numeric source was produced by
ordinary `encode` in the same session. Sorted one-based encode mappings are
retained as private session provenance; decode maps known integer codes back to
strings, preserves NULL and unmapped values as NULL, supports quoted names and
empty mappings, and publishes the full projection atomically. Rename,
projection, replacement, recode, `use`, retry, and failure paths reconcile or
preserve that provenance according to the accepted contract.

The slice does not claim arbitrary value-label metadata, the generic `label`
command, DTA-imported labels, label persistence, lazy/materialized execution,
panel metadata, output adapters, or broad transform sequencing.

## Durable records

- [01-contract.md](01-contract.md) — pinned oracle contract and bounded scope;
- [02-evidence-migration.md](02-evidence-migration.md) — oracle, local, PR-head,
  and merge-head evidence;
- [03-review.md](03-review.md) — correctness, state, security, and scope review;
- `crates/tabdat-language/tests/parser_contract.rs` and
  `crates/tabdat-runtime/tests/decode_contract.rs` — focused contract coverage;
- [roadmap Phase 6.3](../../docs/TABDAT_RUST_PORT_ROADMAP.md); and
- [SPEC.md](../../SPEC.md) — current accepted behavior.

## Hosted acceptance

The final PR head [`1ffb845`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/1ffb84553f0b098f0b9b25ac589cdfaf2baccba7) passed
[CI run 35479575291](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35479575291)
and [runtime run 35479575205](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35479575205).
The squash merge head `0845e6c` was then verified by
[main CI run 35480547109](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35480547109)
and [main runtime run 35480547104](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35480547104).
The remote feature branch was deleted after merge and pruned locally.

Documentation closeout commit
[`08dba48`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/08dba4868d70f44f50cce6a11392dc8118357cda) then passed:

- [final main CI run 35481514850](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35481514850);
- [final ReadStat workflow 35481514778](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35481514778);
- [final libgretl feasibility workflow 35481514780](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35481514780); and
- [final libgretl OLS workflow 35481514788](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35481514788).

The final evidence-only commit will record the hosted links for this update.

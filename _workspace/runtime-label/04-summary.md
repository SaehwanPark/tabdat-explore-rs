# Bounded eager-runtime `label` closeout

Status: accepted and verified on `main` at merge commit
[`70b9745`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/70b9745ae7e22855c763bcb9e3ed40332646723c),
via [PR #55](https://github.com/SaehwanPark/tabdat-explore-rs/pull/55).

## Accepted scope

The Rust runtime now executes the bounded eager local-Parquet session-local
forms:

```text
label variable <varname> "text"
label variable <varname>, clear
label define <lblname> <value> "text" ... [, replace]
label values <varname> <lblname>
label values <varname>, clear
label list [<lblname> ...]
label drop <lblname> ...
```

The session owns normalized variable labels, named value-label sets, and
attachments. Encode publishes default or named sets, decode consumes attached
integer sets, and schema/value-changing operations reconcile metadata according
to the contract.

`label save/use`, DTA ingestion, inspection/reporting rendering, lazy execution,
output adapters, CLI, JSON/MCP surfaces, and broad transform parity remain
deferred.

## Durable records

- [01-contract.md](01-contract.md) — pinned oracle contract and bounded scope;
- [02-evidence-migration.md](02-evidence-migration.md) — oracle, local, PR-head,
  and merge-head evidence;
- [03-review.md](03-review.md) — correctness, state, security, and scope review;
- `crates/tabdat-language/tests/parser_contract.rs` and
  `crates/tabdat-runtime/tests/label_contract.rs` — focused contract coverage;
- [roadmap Phase 6.3](../../docs/TABDAT_RUST_PORT_ROADMAP.md); and
- [SPEC.md](../../SPEC.md) — current accepted behavior.

## Hosted acceptance

PR-head CI/runtime and policy workflows passed for `aa6f952`. The squash merge
head `70b9745` was then verified by [main CI run 35484575373](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35484575373)
and [main runtime run 35484575369](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35484575369).
The documentation-closeout commit `3a0ab00` then passed [final main CI run
35485577830](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35485577830),
[final ReadStat workflow 35485577864](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35485577864),
[final libgretl feasibility workflow 35485577816](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35485577816),
and [final libgretl OLS workflow
35485577786](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35485577786).

The remote feature branch was deleted after merge and pruned locally.

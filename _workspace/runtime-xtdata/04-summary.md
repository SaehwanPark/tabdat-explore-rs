# `xtdata` syntax slice summary

## Outcome

Accepted parser-only slice. PR [#63](https://github.com/SaehwanPark/tabdat-explore-rs/pull/63)
was merged to `main` as
[74eea071d118f3c97c931a9c9345c17293e56440](https://github.com/SaehwanPark/tabdat-explore-rs/commit/74eea071d118f3c97c931a9c9345c17293e56440).

The language layer now recognizes
`xtdata <varlist>, within|between` and returns an owned typed within or
between action. It preserves the recovered flag-only option boundary and
variable quoting. The runtime continues to return explicit
`UnsupportedCommand { name: "xtdata" }`; no panel metadata, transform
formula, relation publication, or generated-column path was added.

Contract, migration evidence, and review are recorded in:

- [01-contract.md](01-contract.md)
- [02-evidence-migration.md](02-evidence-migration.md)
- [03-review.md](03-review.md)

The focused oracle parser test, local locked/policy checks, PR-head workflows,
and merge-head workflows all passed. The next dependency for runtime `xtdata`
is panel metadata and relation-publication semantics; Phase 11.1 runtime work
remains unchecked until that contract is implemented and validated.

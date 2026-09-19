# Bounded syntax-only `replace` closeout

Status: accepted; implementation, hosted checks, merge, branch cleanup, and
merge-head workflow verification are complete.

This loop adds the backend-independent
`replace <target> = <expression> [if <condition>]` command. It retains the
target, replacement expression, and optional predicate as owned typed nodes,
while leaving relation mutation and all data/runtime semantics deferred.

Implementation head: `1f0b945`; [PR #47](https://github.com/SaehwanPark/tabdat-explore-rs/pull/47)
was marked ready and squash-merged as
[`87ec017`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/87ec0173aae4c3e2cb8b27d1b26295aaf17cd18f).

The contract, oracle provenance, focused checks, independent review, hosted
run links, merge SHA, temporary-branch cleanup, and post-merge verification
are recorded in the companion artifacts:

- [`01-contract.md`](01-contract.md) — pinned behavior and bounded scope;
- [`02-evidence-migration.md`](02-evidence-migration.md) — oracle, local,
  hosted, merge, and workflow evidence;
- [`03-review.md`](03-review.md) — review disposition and residual gaps.

Execution, schema/type validation, predicate truthiness, missing/non-finite
arithmetic, overflow accounting, metadata, lazy/materialized behavior,
transform sequencing, CLI/JSON/MCP, and full parser parity remain deferred.

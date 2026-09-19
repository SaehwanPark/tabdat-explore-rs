# `generate` syntax slice closeout

Status: closeout is pending hosted acceptance and merge.

This loop adds the backend-independent, syntax-only
`generate <target> = <expression>` command. It retains identifiers, literals,
unary/binary operators, comparisons, parentheses, and function-call syntax in
owned typed nodes, while leaving all execution and data semantics deferred.

Implementation head: `772eb58`; draft PR: [#45](https://github.com/SaehwanPark/tabdat-explore-rs/pull/45).

The contract, oracle provenance, focused checks, independent review, hosted
run links, merge SHA, temporary-branch cleanup, and post-merge verification
belong in the companion artifacts. The next bounded handoff is an eager
runtime `generate` contract for a deliberately limited expression subset; the
roadmap's full transform-command item remains unchecked until that evidence
exists.

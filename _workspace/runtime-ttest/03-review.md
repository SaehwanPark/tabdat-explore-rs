# Runtime `ttest` review

## Scope

Reviewed the complete slice diff from the Rust main base through the runtime
implementation, tests, validation record, and user-facing documentation. The
review covered typed command dispatch and result/error shapes, SQL construction
and identifier quoting, finite/listwise sample selection, group filtering and
ordering, all three numerical paths, state preservation, and the documented
group-key boundary.

## Findings

No actionable correctness or compatibility findings remain in the bounded
slice. The review caught two Clippy issues before final verification: the new
`TtestResult` made `ExecutionResult` disproportionately large, and the hex
decoder used a manual even-length check. The result variant is now boxed and the
decoder uses `is_multiple_of`; focused and workspace checks passed on that final
shape.

The review verified that one-sample, paired, pooled, and Welch calculations use
sample SD with `n - 1`, the expected degrees of freedom and confidence interval,
and separate SciPy-compatible CDF calls for left, two-sided, and right tails.
Undefined inference and constant samples preserve the pinned oracle's NaN and
point-interval behavior. Runtime reads use quoted identifiers, exclude NULL and
non-finite tested values, require complete finite pairs, apply the exactly-two
group rule after filtering, and leave active dataset and estimation state
unchanged.

Nested list/struct/map, interval, and timezone-aware temporal group values remain
typed limitations. The known DuckDB timezone conversion discrepancy is recorded
in the contract and validation evidence. These constraints are reflected in
`SPEC.md`, `ARCHITECTURE.md`, and the roadmap.

## Verification

The pinned Python tests, SciPy numerical references, focused Rust contracts, and
all four workspace Cargo gates passed locally. See
[`02-validation.md`](02-validation.md) for commands, versions, and the full
comparison record. GitHub Actions were disabled under ADR 0009 before the hosted
workspace checks completed; their partial/cancelled status is recorded in the
validation evidence.

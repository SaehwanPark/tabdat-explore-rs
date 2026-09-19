# `replace` syntax slice review

Status: accepted; independent review, hosted acceptance, merge, temporary
branch cleanup, and merge-head verification are complete.

## Scope reviewed

- `Command::Replace` and `parse_replace_command` in
  `crates/tabdat-language/src/lib.rs`;
- typed parser coverage in `crates/tabdat-language/tests/parser_contract.rs`;
- runtime command mapping and deferred-execution regression in
  `crates/tabdat-runtime/src/lib.rs` and `tests/use_contract.rs`;
- the pinned contract and migration evidence in this workspace slice;
- the explicit runtime, data-semantics, and output deferrals.

## Review questions

1. Does direct `replace <target> = <expression> [if <condition>]` parsing
   preserve the pinned token, quote, expression, nested-boundary, and
   diagnostic behavior without introducing a second expression grammar?
2. Are target and expression values owned and backend-independent, with the
   runtime boundary exhaustive and execution visibly deferred?
3. Do tests prove that parsing and attempted execution have no backend or
   active-dataset side effects?
4. Are mutation, predicate, type, missingness, lazy/materialized, and output
   claims explicitly deferred rather than implied by parser coverage?

## Findings and disposition

The complete implementation diff, callers, pinned contract, focused tests, and
existing generate/runtime boundary were reviewed. No actionable correctness,
safety, ownership, compatibility, or scope finding remains. The parser reuses
`GenerateExpressionParser`, tracks parenthesis depth for top-level `if` and
comma boundaries, preserves quoted identifiers and function calls, and emits
the bounded diagnostics recovered from the pinned oracle. Runtime execution
returns `UnsupportedCommand { name: "replace" }` before any backend or dataset
state can be initialized.

The focused parser/runtime tests, complete local checks, PR-head workflows, and
merge-head workflows are recorded in
[`02-evidence-migration.md`](02-evidence-migration.md). Full eager `replace`
parity is not approved by this review; it requires a separate data-semantics
and runtime contract.

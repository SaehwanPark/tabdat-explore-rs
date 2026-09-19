# `generate` syntax slice review

Status: independent review accepted; hosted acceptance and merge closeout are
pending.

## Scope reviewed

- `Command::Generate` and the dedicated parser in
  `crates/tabdat-language/src/lib.rs`;
- typed parser coverage in `crates/tabdat-language/tests/parser_contract.rs`;
- runtime command mapping and deferred-execution regression in
  `crates/tabdat-runtime/src/lib.rs` and `tests/use_contract.rs`;
- the pinned contract and migration evidence in this workspace slice;
- current-state SPEC/roadmap wording and the final hosted/merge evidence.

## Review questions

1. Does direct `generate <target> = <expression>` parsing preserve the pinned
   token, quote, precedence, call, and diagnostic behavior without changing
   `assert` or broadening tokenizer scope?
2. Are names and expression nodes owned, backend-independent, and exhaustive at
   the runtime boundary, with execution visibly deferred?
3. Do tests prove that parsing and attempted execution have no backend/session
   side effects?
4. Are all runtime/type/function/lazy/output claims explicitly deferred rather
   than implied by parser coverage?

## Findings and disposition

The first independent review identified three actionable parser issues:

- an unquoted `if` could be accepted as a target;
- comparison chains were rejected even though the pinned parser accepts them;
- an empty expression before a comma produced the wrong diagnostic.

The fixes are present at `d272735`, with the parse→execute regression
strengthened at `772eb58`. Re-review of that head found no remaining actionable
findings. Focused parser/runtime tests, formatting, and `git diff --check`
passed.

The bounded slice is therefore approved for hosted acceptance and merge once
the current-head workflows are green. Full eager `generate` parity is not
approved by this review; it requires a separate data-semantics contract.

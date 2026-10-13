# ADR 0023: syntax resource budgets and isolated fuzz tooling

- Status: accepted
- Scope: syntax-only parser safety and external validation; not native runtime integration

## Evidence

Phase 3's cargo-fuzz target reproduces AddressSanitizer stack overflow on
`generate x = ` followed by 20,000 opening parentheses, `1`, and matching closing
parentheses. Crash SHA-1: `4e73007b38f158d84777ad73f74e985bd9181274`.
The ordinary short-input smoke campaign alone did not expose this; targeted
adversarial seeds remain necessary. Recursive unary/function syntax and recursive
command prefixes have the same structural risk. Iteratively parsed binary chains
also produce recursively owned ASTs traversed by equality/drop/visitors.

## Decision

Before building recursive expression ASTs, reject more than **256 tokens per
expression** with `expression exceeds limit of 256 tokens`. Reject recursive
command parsing at depth 32 (outer command depth zero) with
`command nesting exceeds limit of 32`. Limits are deterministic and local, not
thread-local mutable state. Raw SQL text, file paths and whole script length are
not subject to the expression token budget. This is an intentional safety
restriction, not Python parity for arbitrarily large expressions; no existing
pinned corpus fixture exceeds these budgets. The Python oracle itself has
implementation recursion constraints, not these stable diagnostics.

Keep libFuzzer in a separate `fuzz/` validation workspace and lock its dependencies.
Use cargo-fuzz 0.13.2 and libfuzzer-sys 0.4.13; the wrapper is MIT/Apache-2.0 and
LLVM's bundled fuzz engine is Apache-2.0 with LLVM exception. This tooling is not
linked into production, adds no native ownership to domain APIs and does not
change accepted runtime backends or product redistribution decisions. Safe target
source forbids unsafe; native instrumentation is an external validation facility.

## Consequences

Very large legitimate formulas must be simplified or split. The limits are
explicitly tested; raising them requires stack/AST lifecycle evidence, not merely
removing a failing test. These budgets prevent the reproduced recursion failure,
not all resource exhaustion: tokenization/script bytes and backend work still
need later whole-product resource policies. Fuzz smoke is bounded evidence, not
proof of parser correctness or unrestricted hostile-input acceptance.

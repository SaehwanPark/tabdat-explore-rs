# bounded tokenizer migration evidence

## Status and boundary

Status: `complete`.

This is the accepted language-layer tokenizer slice. It ports the recovered
lexical boundary as an owned Rust API; it does not claim command-specific
`parse_simple_body` grammar, complete parser integration, varlists, options,
expressions, scripts, runtime execution, data behavior, or statistical
behavior.

## Authority and recovered behavior

The Python oracle is the isolated checkout at
`C:\Users\saehwan\repos\tabdat-python-oracle`, pinned to revision
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` (tree
`601b236788872323af9277d2276a236154a0f129`). Its `uv.lock` SHA-256 is
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.
The relevant source is `src/tabdat/parser.py` (`_Token` and `_tokenize`).

The focused oracle command was:

    uv run --no-sync pytest -q -p no:cacheprovider tests/test_parser.py

It passed with `489 passed`. Direct probes recovered the following behavior:

- Unicode identifiers and digits are accepted; command whitespace includes the
  oracle's extended control separators.
- Backtick identifiers decode doubled backticks, retain a `quoted` marker, and
  reject empty or unterminated forms with exact diagnostics.
- Single- and double-quoted strings have no escape processing and reject an
  unterminated quote.
- Numbers consume digits and dots and reject a second dot as
  `malformed number: <text>`.
- The two-character symbols are `==`, `!=`, `<=`, and `>=`; the one-character
  symbols are `, = < > + - * / ( ) : .`.
- Other punctuation preserves `unsupported token in command: <char>`.

The oracle records token offsets as Unicode-scalar positions rather than Rust
byte offsets. Its quoted-string `start` is immediately after the opening quote
(for example, the empty string `''` records `start = 1`, `end = 2`); the Rust
implementation preserves this recovered quirk and documents why.

## Rust implementation

PR [#68](https://github.com/SaehwanPark/tabdat-explore-rs/pull/68) adds the
owned `TokenKind`, `Token`, and public `tokenize` API in
[`crates/tabdat-language/src/lib.rs`](../../crates/tabdat-language/src/lib.rs).
The tokenizer uses Unicode-scalar iteration, owned token text, exact recovered
diagnostics, and the recovered offsets. Existing option/expression token
consumers delegate through the shared tokenizer; command-specific simple-body
parsing remains out of scope.

Focused coverage is in
[`crates/tabdat-language/tests/tokenizer_contract.rs`](../../crates/tabdat-language/tests/tokenizer_contract.rs):
four tokenizer contract tests passed, and the existing parser contract suite
continued to pass with 72 tests.

The contract checkpoint is
[`fc63507`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/fc63507);
the implementation checkpoint is
[`2a3d271`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/2a3d271);
the offset documentation clarification is
[`18274a3`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/18274a35c0a8475e2a4e202b3f5111d752eb67fc);
and the squash merge is
[`45da1ac`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/45da1ac86b5a7194bf020417fd11d2a540e78e1e).

## Verification evidence

The following repository checks passed locally:

    cargo fmt --all -- --check
    cargo test --locked -p tabdat-language --test tokenizer_contract
    cargo test --locked -p tabdat-language --test parser_contract
    cargo check --locked --workspace --all-targets -j 1
    cargo test --locked --workspace --all-targets -j 1
    cargo clippy --locked --workspace --all-targets -j 1 -- -D warnings
    cargo deny check
    cargo audit -D warnings
    metadata-driven cargo geiger checks for the workspace packages
    git diff --check

The full Rust test run covered the scaffold, 44 language unit tests, four
tokenizer contract tests, 72 parser-contract tests, 33 runtime unit tests, and
the existing runtime integration contracts. The metadata-driven geiger reports
found no first-party unsafe code in `tabdat-explore-rs`, `tabdat-language`, or
`tabdat-runtime`.

Final PR-head acceptance passed:

- [CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35538903636),
  including [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35538903636/job/106152838481)
  and [dependency/unsafe policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35538903636/job/106152838733).
- [Runtime boundary workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35538903646),
  including [Linux runtime](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35538903646/job/106152838654).

Post-merge `main` acceptance passed:

- [merge-head CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35540026708),
  including [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35540026708/job/106155889869)
  and [dependency/unsafe policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35540026708/job/106155889798).
- [merge-head runtime boundary workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35540026769),
  including [Linux runtime](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35540026769/job/106155890097).

## Deviations and deferrals

This slice intentionally leaves command parsing, varlists, option parsing,
`if` clauses, expression AST/precedence, function calls, missing literals,
quoted/unquoted command-boundary integration, prefixed commands, scripts,
runtime execution, reporting, CLI/JSON/MCP surfaces, and broad Python parser
parity deferred. The original broad roadmap item `Port tokenizer behavior`
remains unchecked; this evidence records only the bounded shared tokenizer API
and its current consumers.

# Bounded tokenizer contract

Status: draft checkpoint before implementation
Producer: task owner
Consumer: implementer/reviewer
Rust base: `main` at `8184c1e`
Python oracle: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` (tree
`601b236788872323af9277d2276a236154a0f129`)

## Scope

Port the pinned Python parser's reusable lexical tokenizer as a pure,
backend-independent Rust language API. The bounded API is:

```rust
pub fn tokenize(input: &str) -> Result<Vec<Token>, ParseError>;
```

`TokenKind` has the finite variants `Identifier { quoted: bool }`, `String`,
`Number`, and `Symbol`. Each owned `Token` carries its kind, decoded text, and
Unicode-scalar `start`/`end` offsets. The offsets intentionally follow the
Python oracle's character-index convention rather than Rust UTF-8 byte
offsets.

The tokenizer must:

- skip command whitespace, including the Python information-separator
  characters U+001C through U+001F;
- recognize Unicode alphabetic/underscore-start identifiers and
  alphanumeric/underscore continuations;
- decode backtick-quoted identifiers, including doubled-backtick escapes,
  while rejecting empty or unterminated identifiers;
- recognize decimal number spellings beginning with a digit or `.` followed by
  a digit, rejecting more than one dot;
- decode single- and double-quoted strings without adding escape semantics;
- emit the two-character symbols `==`, `!=`, `<=`, and `>=`, then the bounded
  one-character symbol set `, = < > + - * / ( ) : .`;
- return the exact recovered diagnostics for malformed quotes, malformed
  numbers, and unsupported characters.

`tokenize_use_options` will delegate to this tokenizer so option and
expression parsers consume the recovered lexical contract. The existing
command-specific `parse_simple_body` paths are not refactored in this slice;
their command grammar, condition splitting, assignment handling, and broader
tokenizer integration remain separate work.

## Python authority and direct observations

The authoritative implementation is `_Token` and `_tokenize` in
`src/tabdat/parser.py` at the pinned oracle revision. Direct probes in the
isolated oracle confirmed, among other cases:

| Input | Recovered result |
| --- | --- |
| `wage exposure, robust lags(2) >= .5` | identifiers, comma, identifiers, parentheses, number, `>=`, and `.5` number tokens |
| `` `a``b` `` | quoted identifier text `a`b` |
| `"hello world" ''` | string text `hello world`, then an empty string token |
| `α_1 ١٢` | identifier `α_1`, number `١٢`, with character offsets |
| `1..2` | `malformed number: 1..2` |
| `` ` `` | `unterminated quoted identifier` |
| `''unterminated` | `unterminated quoted string` |
| `` `` `` | `quoted identifier cannot be empty` |
| `;` or `@` | `unsupported token in command: ;` / `unsupported token in command: @` |

The oracle source has no comment, semicolon, escape-string, or additional
operator behavior in this contract. Higher-level command validation remains
outside the tokenizer.

## Rust contract

- `TokenKind`, `Token`, and `tokenize` are owned language-layer types/functions;
  they perform no I/O, execution, global-state access, or backend
  initialization.
- `Token` text is decoded for quoted forms and preserves unquoted spelling.
- `start` is inclusive and `end` is exclusive in Unicode-scalar offsets.
- `ParseError::message()` and `Display` expose the stable diagnostics listed
  above.
- Existing option/expression parser behavior must remain green after the
  internal delegation change.

## Test contract and acceptance

Add focused integration coverage for token kinds, quoted flags, decoded text,
Unicode-scalar offsets, whitespace, numbers, operators, and every listed error.
Run the pinned oracle probes and the full locked Rust/policy checks. Acceptance
requires format, check, test, clippy, dependency policy, advisory audit,
first-party unsafe-code scans, and hosted PR-head and merge-head workflows to
pass. Do not claim the unchecked roadmap items for command parsing, varlists,
options, conditions, expressions, prefixed commands, or simple-body integration
from this tokenizer slice.

No DuckDB, ReadStat, libgretl, statistics, runtime session, filesystem, or
native dependency is in scope.

# Review: syntax-only `save` and `export`

Status: partial — bounded implementation review is complete; hosted acceptance,
merge, and branch-cleanup gates remain pending.

Reviewer: independent read-only parser reviewer, reconciled by task owner

## Original acceptance checks

- [x] Add owned `Command::Save` and `Command::Export` values with lexical path
  text and a `replace` flag.
- [x] Parse case/whitespace-normalized names, quoted/symbolic paths, attached
  symbolic suffixes, and flag-only repeated `replace` options.
- [x] Preserve exact arity, option, condition, assignment, punctuation, comma,
  and quote diagnostics in the bounded contract.
- [x] Keep filesystem validation, active-dataset access, writers, formats,
  overwrite behavior, and persistence effects deferred.
- [x] Add explicit runtime unsupported-command regressions for both commands.
- [x] Run the pinned focused and parser/script oracle suites.
- [x] Run local locked Rust, formatting, Clippy, diff, advisory, license, and
  first-party unsafe-code checks.
- [ ] Confirm all required PR-head hosted jobs pass.
- [ ] Mark PR ready, merge, delete the temporary branch, and record post-merge
  main checks.

## Evidence reviewed

- `01-contract.md` and the pinned source/test/doc paths;
- implementation revision `e014705` and documentation revision `0afb8c5`;
- `crates/tabdat-language/src/lib.rs` and
  `crates/tabdat-language/tests/parser_contract.rs`;
- `crates/tabdat-runtime/src/lib.rs` and
  `crates/tabdat-runtime/tests/use_contract.rs`;
- local Rust/policy checks and the two pinned Python oracle runs recorded in
  `02-evidence-migration.md`.

## Findings

No P0/P1 correctness, security, dependency, state-mutation, or scope findings.
The parser helper validates path arity and command-specific options after the
shared tokenizer, preserving the oracle's generic malformed-option diagnostics.
The runtime mappings only produce `UnsupportedCommand` and do not initialize or
mutate a session backend.

The following P2/P3 tokenizer differences are intentional bounded deferrals,
not acceptance failures:

- malformed numeric-looking paths such as `save 1.2.3` are retained as lexical
  path text rather than rejected by a future number tokenizer;
- attached reserved-`if` paths such as `save if/foo` are not routed through a
  full expression parser yet.

Both are named in the contract and evidence ledger and require a later tokenizer
/ condition milestone before any broader parity claim.

## Required follow-up

Complete the hosted PR checks, promote PR #29 from draft after the bounded review
remains green, squash-merge it, remove `feat/parser-save-export-syntax` locally
and remotely, then update this ledger and `02-evidence-migration.md` with the
merge commit, hosted job links, and post-merge `main` verification. Keep Phase
6.5 persistence/output unchecked.

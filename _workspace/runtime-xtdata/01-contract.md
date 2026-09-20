# Bounded `xtdata` syntax contract

## Status and scope

- Status: `accepted` after PR #63 and its PR-head and merge-head workflows
  passed.
- Owner: the parent task owner; this slice is intentionally parser-only.
- Target: recover and port the direct `xtdata` command boundary without
  claiming panel-index transforms, relation mutation, or active-session
  execution.
- Rust paths in scope: `crates/tabdat-language/src/lib.rs`, its parser tests,
  the runtime unsupported-command contract, and this evidence directory.
- Runtime paths are otherwise out of scope. `tabdat-runtime` will return an
  explicit typed unsupported-command error for `Command::XtData` until panel
  metadata and within/between relation semantics are separately implemented.

The accepted syntax target is:

    xtdata <varlist>, within|between

The parser produces an owned typed command containing the requested variables
and exactly one transform, `within` or `between`. The option names are the
oracle's flag tokens: they do not accept values, unsupported options are
rejected, and repeated copies of the same accepted flag retain the oracle's
set-membership behavior.

## Authority and recovered behavior

- Python oracle checkout: `C:\Users\saehwan\repos\tabdat-python-oracle`.
- Pinned revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`.
- Pinned tree: `601b236788872323af9277d2276a236154a0f129`.
- Lockfile SHA-256: `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.
- Oracle sources: `src/tabdat/models.py` (`XtDataCommand`),
  `src/tabdat/parser.py` (`_parse_xtdata`), and `docs/commands/xtdata.md`.
- Focused oracle selection:
  `uv run --no-sync pytest -q -p no:cacheprovider tests/test_parser.py -k xtdata`
  produced `7 passed, 482 deselected` at the pinned revision.

The recovered parser accepts one or more variable arguments before the comma.
It accepts the unquoted flag `within` or `between`, requires exactly one of
those memberships, and rejects conditions, expressions, unsupported options,
and valued forms such as `within=true`. Option matching preserves the oracle's
case-sensitive option names; the command name remains case-insensitive.

## Rust contract

1. `parse_command` recognizes `xtdata` case-insensitively and returns
   `Command::XtData` with an owned `XtDataCommand`.
2. The command requires at least one variable argument and a comma option
   section containing exactly one accepted transform membership.
3. `within` maps to `XtDataTransform::Within`; `between` maps to
   `XtDataTransform::Between`.
4. Accepted transform options are flags only. Unsupported options, valued
   options, both transforms, neither transform, conditions, assignment syntax,
   and malformed variable boundaries preserve bounded diagnostics.
5. Quoted variable names retain decoded text. Quoted option names are not
   accepted as option identifiers.
6. Parsing does not inspect files, require panel metadata, initialize DuckDB,
   or mutate session state.
7. Runtime execution is explicitly deferred; no within/between columns,
   panel grouping, ordering, or relation publication path is introduced by
   this parser slice.

## Acceptance checks

- Oracle: focused `tests/test_parser.py -k xtdata` passes at the pinned
  revision.
- Rust: focused language parser tests cover within/between, quoted variable
  names, valued/unsupported options, duplicate transform membership, missing
  transform, and condition/arity diagnostics.
- Runtime: a focused contract test proves parsed `xtdata` returns the typed
  unsupported-command error without backend work.
- Repository baseline: format, locked workspace check/test, and clippy with
  warnings denied pass before the PR is marked ready.
- Documentation and `git diff --check` pass; only this slice's files are
  staged.

## Known gaps and next dependency

This slice does not implement panel metadata lookup, numeric-variable
validation, within/between formulas, generated-column naming, collision
handling, row/order preservation, missingness, lazy/materialized behavior,
publication, labels, formatting, CLI, JSON, MCP, or broad Python `xtdata`
parity. Those behaviors remain future runtime evidence, gated by a panel
metadata/session contract and a dedicated transform execution contract. The
parser can be merged without implying that a parsed `xtdata` command executes
today.

The contract checkpoint is
[`32307dd`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/32307dd073cfc71150378fc0b2e2363fb0b3db5b);
the implementation checkpoint is
[`29b3d6d`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/29b3d6d2421bc69e32ebc20a3986b77e65d695d7); and PR
[#63](https://github.com/SaehwanPark/tabdat-explore-rs/pull/63) was squash-merged
to `main` as
[`74eea071d118f3c97c931a9c9345c17293e56440`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/74eea071d118f3c97c931a9c9345c17293e56440).

# Bounded panel syntax contract

## Status and scope

- Status: `accepted` after PR #62 and its PR-head and merge-head workflows
  passed.
- Owner: the parent task owner; this slice is intentionally parser-only.
- Target: recover and port the direct `panel` command boundary without
  claiming panel metadata or structural-summary execution.
- Rust paths in scope: `crates/tabdat-language/src/lib.rs`, its parser tests,
  the runtime unsupported-command contract, and this evidence directory.
- Runtime paths are otherwise out of scope. `tabdat-runtime` will continue to
  return an explicit typed unsupported-command error for `Command::Panel` until
  active-relation panel metadata and summary semantics are separately
  implemented.

The accepted forms are:

    panel
    panel <id_var> <time_var>
    panel clear

The parser produces an owned typed action: report, clear, or set with distinct
identifier and time variables. An unquoted or string-quoted `clear` is the
control keyword; backtick-quoted `clear` remains a variable name, matching the
recovered parser boundary.

## Authority and evidence inputs

- Python oracle checkout: `C:\Users\saehwan\repos\tabdat-python-oracle`.
- Pinned revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`.
- Pinned tree: `601b236788872323af9277d2276a236154a0f129`.
- Lockfile SHA-256: `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.
- Oracle sources: `src/tabdat/models.py` (`PanelCommand`),
  `src/tabdat/parser.py` (`_parse_panel`), and `docs/commands/panel.md`.
- Focused oracle selection:
  `uv run --no-sync pytest -q -p no:cacheprovider tests/test_parser.py -k panel`
  produced `7 passed, 482 deselected` at the pinned revision.

The parser-focused panel selection is the differential checkpoint. Panel
execution tests are recorded as deferred evidence because they require active
relation metadata, panel grouping/ordering, structural summaries, and session
publication semantics that do not yet exist in this Rust workspace.

## Rust contract

1. `parse_command` recognizes `panel` case-insensitively and returns
   `Command::Panel` with a typed report, clear, or set action.
2. Empty `panel` reports; exactly two variable arguments set panel identifiers;
   exactly one unquoted or string-quoted `clear` argument clears the panel
   declaration.
3. Quoted/backtick identifiers retain decoded text. Backtick-quoted `clear` is
   not treated as the clear keyword; string-quoted `clear` follows the oracle's
   clear-action tokenization.
4. Identifier and time-variable names must be distinct. Conditions, options,
   assignment syntax, extra arguments, and malformed boundaries preserve the
   bounded panel syntax diagnostic.
5. Parsing does not inspect files, initialize DuckDB, or mutate session state.
6. Runtime execution is explicitly deferred; no panel metadata, structural
   summary, relation ordering, or publication path is introduced by this
   parser slice.

## Acceptance checks

- Oracle: focused `tests/test_parser.py -k panel` passes at the pinned
  revision.
- Rust: focused language parser tests cover report, set, clear, quoted names,
  control-keyword boundaries, duplicate names, and malformed syntax.
- Runtime: a focused contract test proves parsed panel returns the typed
  unsupported-command error without backend work.
- Repository baseline: format, locked workspace check/test, and clippy with
  warnings denied pass before the PR is marked ready.
- Documentation and `git diff --check` pass; only this slice's files are
  staged.

## Known gaps and next dependency

This slice does not implement panel metadata ownership, active-schema lookup,
numeric/time validation, duplicate entity-time checks, panel structural
summaries, ordering, missingness, persistence, lazy/materialized behavior,
labels, formatting, CLI, JSON, MCP, or broad Python `panel` parity. Those
behaviors remain future runtime evidence, gated by relation/session semantics
and a dedicated panel execution contract. The parser can be merged without
implying that a parsed panel command executes today.

The contract checkpoint is
[`331b5fb4fb0a59c91a886c3e07f47fab68592cd5`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/331b5fb4fb0a59c91a886c3e07f47fab68592cd5);
the implementation checkpoint is
[`67de5b835489d196dc800607054b227871b8b6d4`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/67de5b835489d196dc800607054b227871b8b6d4);
the keyword-boundary test checkpoint is
[`d1eeb2a321c80ec13e6f5daa57129cbecbe8d7c4`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/d1eeb2a321c80ec13e6f5daa57129cbecbe8d7c4);
and PR [#62](https://github.com/SaehwanPark/tabdat-explore-rs/pull/62) was
squash-merged to `main` as
[`92e5d5e5be9aaededbfe3f44ded2d4319cdcac87`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/92e5d5e5be9aaededbfe3f44ded2d4319cdcac87).

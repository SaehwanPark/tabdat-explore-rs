# Bounded reshape syntax contract

## Status and scope

- Status: `accepted` after PR #61 and its PR-head and merge-head workflows
  passed.
- Owner: the parent task owner; this slice is intentionally parser-only.
- Target: recover and port the direct `reshape` command boundary without
  claiming relation execution.
- Rust paths in scope: `crates/tabdat-language/src/lib.rs`, its parser tests,
  the runtime unsupported-command contract, and this evidence directory.
- Runtime paths are otherwise out of scope. `tabdat-runtime` will continue to
  return an explicit typed unsupported-command error for `Command::Reshape`
  until relation/session reshape semantics are separately implemented.

The accepted form is:

    reshape long|wide <varlist>, i(<id_vars>) j(<name>)

The parser produces an owned typed command containing the direction, ordered
reshape variables, ordered identifier variables, and the single `j()` output
name. Direction is case-insensitive; the recovered `i` and `j` option names
use their lowercase spellings. Quoted identifiers retain their decoded text,
while a backtick-quoted direction token is not treated as a direction keyword.

## Authority and evidence inputs

- Python oracle checkout: `C:\Users\saehwan\repos\tabdat-python-oracle`.
- Pinned revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`.
- Pinned tree: `601b236788872323af9277d2276a236154a0f129`.
- Lockfile SHA-256: `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.
- Oracle sources: `src/tabdat/models.py` (`ReshapeCommand`),
  `src/tabdat/parser.py` (`_parse_reshape` and option helpers), and
  `docs/commands/reshape.md`.
- Focused oracle selection:
  `uv run --no-sync pytest -q -p no:cacheprovider tests/test_parser.py -k reshape`
  produced `16 passed, 473 deselected` at the pinned revision.

The parser-focused reshape selection is the differential checkpoint. Reshape
execution tests are recorded as deferred evidence because they require the
Python relation/session model and the Rust relation publication semantics that
do not yet exist in this workspace.

## Rust contract

1. `parse_command` recognizes `reshape` case-insensitively and returns
   `Command::Reshape` with a typed direction and owned variable names.
2. The direction is exactly `long` or `wide` when unquoted. At least one
   reshape variable is required, and variable names must be unique.
3. Exactly one `i(<id_vars>)` option and one `j(<name>)` option are required.
   Identifier lists are ordered and unique; `j()` must contain exactly one
   name.
4. The variable list, identifier list, and `j()` name must be pairwise
   distinct. Options other than `i` and `j` are rejected in sorted diagnostic
   order.
5. Conditions, assignment syntax, missing required options, duplicate options,
   malformed option values, and invalid direction/arity preserve the bounded
   diagnostics recovered from the pinned parser tests.
6. Parsing does not inspect files, initialize DuckDB, or mutate session state.
7. Runtime execution is explicitly deferred; no relation registry, reshape SQL,
   hidden backend relation, or publication path is introduced by this parser
   slice.

## Acceptance checks

- Oracle: focused `tests/test_parser.py -k reshape` passes at the pinned
  revision.
- Rust: focused language unit/integration parser tests cover long and wide
  forms, quoted identifiers, duplicate/distinctness rules, option validation,
  malformed boundaries, and stable diagnostics.
- Runtime: a focused contract test proves parsed reshape returns the typed
  unsupported-command error without backend work.
- Repository baseline: format, locked workspace check/test, and clippy with
  warnings denied pass before the PR is marked ready.
- Documentation and `git diff --check` pass; only this slice's files are
  staged.

## Known gaps and next dependency

This slice does not implement long/wide relation reshaping, identifier
uniqueness and missingness semantics, wide-column naming/collision rules,
ordering, row counts, type coercion, labels, lazy/materialized behavior,
failure-atomic publication, persistence, formatting, CLI, JSON, MCP, or broad
Python `reshape` parity. Those behaviors remain future runtime evidence, gated
by relation/session semantics and a dedicated reshape execution contract. The
parser can be merged without implying that a parsed reshape command executes
today.

The contract checkpoint is
[`48323f75423773e3b0501cd1eb7d2a8bcb0a3b5a`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/48323f75423773e3b0501cd1eb7d2a8bcb0a3b5a);
the casing correction is
[`e3e9897abb8fba93e912ee853885f7a1d18e7665`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/e3e9897abb8fba93e912ee853885f7a1d18e7665);
the implementation checkpoint is
[`0d1a059ab5c03c72fc78cb855ce4c9ea122dc0b9`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/0d1a059ab5c03c72fc78cb855ce4c9ea122dc0b9);
and PR [#61](https://github.com/SaehwanPark/tabdat-explore-rs/pull/61) was
squash-merged to `main` as
[`e6cc4f1768c9b55b8ead702a08a36283f2a27bee`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/e6cc4f1768c9b55b8ead702a08a36283f2a27bee).

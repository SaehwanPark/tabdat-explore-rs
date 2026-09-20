# Bounded `estat` diagnostics syntax contract

## Status and scope

- Status: `draft` at the contract checkpoint; acceptance follows the PR-head
  implementation and merge-head workflow evidence.
- Owner: the parent task owner; this slice is intentionally parser-only.
- Target: recover and port the direct no-option `estat` subcommand boundary for
  `firststage`, `overid`, `endogenous`, and `hausman` without claiming any
  post-estimation calculation, model-state lookup, or active-session execution.
- Rust paths in scope: `crates/tabdat-language/src/lib.rs`, its parser tests,
  the runtime unsupported-command contract, and this evidence directory.
- Runtime paths are otherwise out of scope. `tabdat-runtime` will return an
  explicit typed unsupported-command error for `Command::Estat` until IV/panel
  model-state and post-estimation contracts are separately implemented.

The accepted syntax target is:

    estat firststage|overid|endogenous|hausman

The parser produces an owned typed command containing one of the four selected
diagnostic subcommands. The subcommand is case-insensitive; single- and
double-quoted text is accepted by the pinned oracle, while backtick quoting is
not. Selected forms do not accept options.

## Authority and recovered behavior

- Python oracle checkout: `C:\Users\saehwan\repos\tabdat-python-oracle`.
- Pinned revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`.
- Pinned tree: `601b236788872323af9277d2276a236154a0f129`.
- Lockfile SHA-256: `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.
- Oracle sources: `src/tabdat/models.py` (`EstatCommand`),
  `src/tabdat/parser.py` (`_parse_estat`), and `docs/commands/estat.md`.
- Focused oracle selection:
  `uv run --no-sync pytest -q -p no:cacheprovider tests/test_parser.py -k estat`
  produced `19 passed, 470 deselected` at the pinned revision.

The recovered selected forms accept exactly one subcommand argument, reject
conditions and extra arguments with the `estat` syntax diagnostic, and reject
options with `estat <subcommand> does not support options`. The oracle accepts
command and subcommand case variants and preserves the selected normalized
subcommand in the typed model. Other `estat` subcommands, structured spatial
options, and `report` options are outside this bounded slice.

## Rust contract

1. `parse_command` recognizes `estat` case-insensitively and returns
   `Command::Estat` with an owned `EstatCommand` for the four selected
   subcommands.
2. `EstatSubcommand::{FirstStage, Overid, Endogenous, Hausman}` represents the
   normalized oracle names.
3. The selected command requires exactly one subcommand argument; conditions,
   expressions, and extra arguments preserve the bounded syntax diagnostic.
4. Selected subcommands do not accept options and preserve the recovered
   option diagnostic.
5. Parsing does not inspect model state or files, run a diagnostic, initialize
   DuckDB, or mutate session state. Runtime execution is explicitly deferred.

## Acceptance checks

- Oracle: focused `tests/test_parser.py -k estat` passes at the pinned
  revision.
- Rust: focused language parser tests cover all four selected subcommands,
  command/subcommand case handling, quoted subcommands, syntax boundaries,
  and option rejection with exact diagnostics.
- Runtime: a focused contract test proves parsed `estat` returns the typed
  unsupported-command error without model or backend work.
- Repository baseline: format, locked workspace check/test, and clippy with
  warnings denied pass before the PR is marked ready.
- Documentation and `git diff --check` pass; only this slice's files are
  staged.

## Known gaps and next dependency

This slice does not implement first-stage, overidentification, endogeneity, or
Hausman calculations; model-state routing; IV/panel validation; covariance;
missingness; result formatting; labels; CLI; JSON; MCP; or broad Python
`estat` parity. The remaining `estat` subcommands and structured spatial/report
syntax are also future parser work. These behaviors remain gated by typed
statistical model-state and post-estimation contracts. The parser can be merged
without implying that a parsed `estat` command executes today.

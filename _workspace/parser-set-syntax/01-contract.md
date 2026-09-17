# Contract: syntax-only `set`

Status: accepted and merged in PR #15 (`bdc1433`).

## Python contract

The migration oracle is the clean sibling checkout `../tabdat-explore` at
commit `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` (tree
`601b236788872323af9277d2276a236154a0f129`). The pinned source and tests define
`SetCommand(name, value)` with the setting name limited to `graph_format`,
`artifact_dir`, or `graph_open`:

- `src/tabdat/models.py:447-450`
- `src/tabdat/parser.py:701-702,1012-1025`
- `tests/test_parser.py:374-384,1542-1544`
- `docs/commands/set.md:1-25`

The direct syntax is:

```text
set graph_format <value>
set artifact_dir <path>
set graph_open on|off
```

Command names and setting names are case-insensitive. Values remain strings and
preserve their spelling; quoted values are unquoted, so a path or value may
contain spaces. The parser accepts values that a later configuration layer may
reject (for example, `set graph_format pdf` or `set graph_open maybe`). A
backtick-quoted setting name is not an unquoted setting identifier and therefore
produces an unknown-setting diagnostic; single- and double-quoted names follow
the pinned tokenizer's string behavior.

The exact direct-command diagnostics are:

| Input shape | Diagnostic |
| --- | --- |
| missing or extra arguments, conditions, options, or assignment syntax | `set expects syntax: set name value` |
| unknown setting name | `unknown setting: <spelled name>` |
| `set = value` or `set= value` | `set assignment requires a target before =` |
| trailing comma | `comma must be followed by at least one option` |
| `if` without an expression | `missing expression after if` |
| unsupported tokenizer punctuation | `unsupported token in command: <token>` |

## Rust contract

The language crate will expose:

```rust
Command::Set {
  name: SettingName,
  value: String,
}
```

`SettingName` is a finite Rust enum for the three recognized names. Parsing is
pure and backend-independent: it does not validate setting values, mutate
configuration or session state, inspect the filesystem, initialize plotting,
or return a runtime result. The existing generic parser remains unchanged for
other commands; symbol-bearing setting values are handled only by the `set`
syntax path.

The Python parser's `by ...: set ...` behavior and configuration execution are
deferred until prefixed-command and session/state contracts are recovered.

## Test contract

Rust tests will cover the accepted forms with case/whitespace normalization,
quoted values including spaces, preserved value spelling, the three setting
names, unknown and backtick-quoted names, missing/extra arguments, options,
conditions, assignments, trailing commas, and exact diagnostics. The pinned
configuration parser test is:

```sh
cd ../tabdat-explore
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py -k phase_9_configuration_and_persistence_commands
```

The unchanged pinned parser/script suite remains the broader regression oracle:

```sh
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py tests/test_script.py
```

## Implementation mapping and deferrals

- Native Rust: `tabdat-language` command and parser types/tests only.
- Deferred: typed configuration state, value validation, session mutation,
  `SetResult`, CLI/JSON/MCP rendering, persistence, plotting, and backend use.
- No DuckDB, ReadStat, libgretl, filesystem, or environment dependency is added.

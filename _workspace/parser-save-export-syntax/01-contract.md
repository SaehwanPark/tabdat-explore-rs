# Contract: syntax-only `save` and `export`

Status: frozen bounded contract; implementation and acceptance evidence are
pending on the feature branch.

Selected skills: `tabdat-migration`, `simple-code-writer`

Rust base: `a437011` (`main` after the accepted syntax-only `gsort` slice and
its post-merge documentation closeout).

## Scope

Add the direct `save <path> [, replace]` and
`export <path> [, replace]` forms to the backend-independent language layer.
Each command returns one owned path and a replacement flag. Parsing does not
inspect the filesystem or active dataset, write output, validate a format,
mutate session state, initialize a backend, or expose CLI/JSON/MCP behavior.
Runtime execution remains an explicit unsupported capability.

## Python contract

The authority is the clean sibling checkout `../tabdat-explore` at pinned commit
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, package `0.25.0`, and Python
`3.13.3`. Its `uv.lock` SHA-256 is
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`; the
checkout is clean and no dependency synchronization or source edits were
performed.

Authoritative paths:

- `src/tabdat/models.py:453-463`: `SaveCommand` and `ExportCommand` with a
  `Path` and a default-false `replace` flag;
- `src/tabdat/parser.py:148-149,704-708,1612-1624`: command inventory,
  dispatch, and the shared `_parse_save_or_export` validator;
- `src/tabdat/parser.py:3043-3123`: generic argument, quote, symbol, and option
  handling used by these commands;
- `tests/test_parser.py:374-391,1829-1834`: positive phase-9 forms and invalid
  command cases;
- `docs/commands/save.md:11-17` and `docs/commands/export.md:11-17`:
  published syntax and examples.

The direct forms are:

```text
save <path> [, replace]
export <path> [, replace]
```

Accepted syntax preserves path spelling after quote removal and allows exactly
one path argument. Command names are case-insensitive; option names are not.
Single- and double-quoted strings and backtick identifiers may contain spaces;
backticks support doubled-backtick escaping. Symbolic path text such as
`a/b:c`, `a+b`, `a==b`, commas inside quotes, and attached command suffixes
(`save:out.parquet`, `export/out.csv`) is accepted. `replace` is a flag and
may repeat as adjacent option tokens (`save out.parquet, replace replace`),
but comma-separated repeated options are not part of this contract.

Observed values include:

```text
save output.parquet                    -> Save(path="output.parquet", replace=false)
export "my output.parquet"             -> Export(path="my output.parquet", replace=false)
export output.csv, replace             -> Export(path="output.csv", replace=true)
save a==b                              -> Save(path="a==b", replace=false)
save "a,b", replace                    -> Save(path="a,b", replace=true)
```

The Python model uses `Path`; `Path("")` displays as `Path(".")`. Rust will
retain the unwrapped lexical path string in this syntax-only boundary (including
an empty quoted path), leaving platform normalization and filesystem policy to a
future output boundary rather than pretending that writes are supported now.

Exact observed diagnostics are:

| Input shape | Diagnostic |
| --- | --- |
| missing or multiple paths (`save`, `save a b`) | `save expects exactly one path` |
| missing or multiple paths for `export` | `export expects exactly one path` |
| unsupported flag (`save out, force`) | `save unsupported option: force` |
| unsupported export flag | `export unsupported option: force` |
| value on `replace` (`replace=true`, `replace(foo)`) | `{name} option replace does not accept a value` (after the generic option tokenizer's malformed-value diagnostics where applicable) |
| non-empty condition or assignment (`save out if x > 0`, `save out = x`) | `{name} does not accept if clauses or assignment syntax` |
| bare `if` | `missing expression after if` |
| assignment with no target (`save = x`) | `save assignment requires a target before =` |
| assignment ending at `=` (`save out =`) | `save assignment requires an expression after =` |
| trailing comma (`save out,`) | `comma must be followed by at least one option` |
| unsupported punctuation (`save out@x`) | `unsupported token in command: @` |
| comma-separated repeated option (`save out, replace, replace`) | `option names must be identifiers` |

`replace` is case-sensitive (`REPLACE` is unsupported). Generic option parsing
retains its own diagnostics for malformed parentheses, missing `=`, quoted
tokens, and unterminated quotes; those are not replaced with a command-level
message.

## Rust contract

Expose two owned command variants:

```rust
Command::Save { path: String, replace: bool }
Command::Export { path: String, replace: bool }
```

The parser splits only at the first unquoted comma, reuses the existing
symbol-aware simple-body parser for the path, and reuses the existing option
tokenizer for the flag tail. It must preserve quote removal, path symbols,
ordered repeated flags, and the diagnostics above. Attached symbolic suffixes
are routed to the corresponding command before the generic command-name scan.

The runtime adds exhaustive command-name mappings only. Executing either
variant returns `RuntimeError::UnsupportedCommand { name }` without creating a
backend or touching session state.

## Test contract

Rust unit and public integration tests will cover:

- case/whitespace normalization and both typed command variants;
- quoted-space and quoted-comma paths, backtick escaping, symbolic paths, and
  attached symbolic command suffixes;
- default and repeated `replace` flags, including case-sensitive names;
- exact missing/multiple-path, condition, assignment, option, punctuation,
  trailing-comma, and quote diagnostics;
- runtime deferral for `save` and `export`;
- unchanged parser/runtime behavior for all existing commands.

The focused pinned-oracle check is:

```sh
cd ../tabdat-explore
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py -k 'phase_9_configuration_and_persistence or invalid_commands'
```

The observed result is `419 passed, 70 deselected` at the pinned revision. The
broader parser/script oracle and the required locked Rust/policy checks must be
rerun for acceptance evidence.

## Implementation mapping and deferrals

- Native Rust: owned command variants, direct parser dispatch, option validation,
  deterministic diagnostics, and unit/public tests.
- Runtime: command-name mapping plus explicit unsupported-command regressions.
- Deferred: filesystem checks, active-dataset preconditions, overwrite policy,
  Parquet/CSV/Feather/Arrow writers, format selection, result/reporting and
  serialization, CLI/JSON/MCP surfaces, path normalization, and all persistence
  effects. Full tokenizer parity for Unicode classification, attached `if`
  condition parsing, and malformed-number edge cases remains outside this
  bounded slice.

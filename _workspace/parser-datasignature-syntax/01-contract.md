# Contract: syntax-only `datasignature`

Status: proposed for the next bounded parser slice.

Selected skills: `tabdat-migration`, `simple-code-writer`

Rust base: `main` at `e1e4215c740a931cdb75ef589563c9496f03d3b3` (set syntax
merged and post-merge CI green).

## Scope

Add only the direct, zero-argument `datasignature` command to the pure
`tabdat-language` parser. The slice must not access an active relation, hash
data, mutate session state, execute a command, serialize a result, initialize
DuckDB or another backend, or add a runtime dependency. Existing command forms
and diagnostics remain unchanged, including the unresolved `status -/+`
deviation recorded in `docs/migration/decisions.md`.

## Python contract

The pinned clean oracle is `../tabdat-explore` at commit
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` (tree
`601b236788872323af9277d2276a236154a0f129`). Authoritative paths are:

- `src/tabdat/models.py:202-205`: empty `DatasignatureCommand`;
- `src/tabdat/parser.py:108-123,594-604`: registration and validation;
- `tests/test_datasignature.py:54-65`: parser cases;
- `docs/commands/datasignature.md:7-14`: direct syntax and exclusions.

Accepted forms are case-insensitive `datasignature` with surrounding and
Python-compatible separator whitespace. The parser returns an empty typed
command and does not compute a signature. The direct command diagnostics are:

| Input shape | Diagnostic |
| --- | --- |
| arguments, `if` with an expression, options, or assignment syntax | `datasignature does not accept arguments, if clauses, options, or assignment syntax` |
| `if` without an expression | `missing expression after if` |
| trailing comma | `comma must be followed by at least one option` |
| `= value` or `=value` | `datasignature assignment requires a target before =` |
| `==`, `-`, or `+` tokens | `unsupported token in command: <token>` |

The Python parser currently accepts `by id: datasignature` even though the
command documentation excludes `by:`; prefixed-command parsing and that
execution boundary remain outside this direct slice.

## Rust contract

Add a fieldless public variant:

```rust
Command::Datasignature
```

Route only the direct command through the existing zero-argument validation
pattern. Reuse the owned `ParseError`, preserve exact diagnostics, keep
`#![forbid(unsafe_code)]`, and use no new dependencies.

## Test contract

Add unit and public integration coverage for the bare command, case/whitespace
normalization, U+001C separator compatibility, every diagnostic in the table,
and the unchanged existing parser matrix. Run the focused oracle check:

```sh
cd ../tabdat-explore
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_datasignature.py -k test_parse_datasignature_form
```

The broader pinned parser/script regression remains:

```sh
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py tests/test_script.py
```

## Implementation mapping and deferrals

- Native Rust: one typed command variant, dispatch, validation, and tests.
- Deferred: SHA-256/signature semantics, active-dataset preconditions, schema
  and row-order rules, missing/non-finite handling, session mutation, result
  types, terminal/JSON/MCP rendering, `by:` wrappers, and backend use.
- No DuckDB, ReadStat, libgretl, filesystem, environment, or statistics code.

Acceptance requires the focused/full oracle results, Rust fmt/check/test/Clippy,
policy scans, `git diff --check`, and hosted CI. Stop if direct syntax parity
requires general prefixed-command or execution work; record the conflict
instead of broadening this slice.

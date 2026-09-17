# Contract: syntax-only `use`

Status: proposed.

Selected skills: `tabdat-migration`, `simple-code-writer`

Rust base: `main` at `7df99da` (the merged `datasignature` slice with its
post-merge documentation update and green main CI).

## Scope

Add only direct `use` command parsing to the pure `tabdat-language` crate. The
slice must return an owned typed command without opening a file, probing a URI,
reactivating a named table, creating a relation, mutating session state,
initializing DuckDB/Polars/another backend, or adding a runtime dependency.
Existing commands and diagnostics remain unchanged. Script parsing, `by:`
wrappers, macro expansion, and execution are outside this slice.

## Python contract

The pinned clean oracle is the sibling checkout `../tabdat-explore` at commit
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` (tree
`601b236788872323af9277d2276a236154a0f129`). Authoritative paths are:

- `src/tabdat/models.py:118-133`: `UseCommand` fields and defaults;
- `src/tabdat/parser.py:271-276,360-364,870-936`: command routing, parser,
  option validation, and `://` path classification;
- `src/tabdat/parser.py:3159-3325`: generic option values and diagnostics;
- `src/tabdat/parser.py:3327-3386`: tokenizer behavior used by options;
- `tests/test_parser.py:90-110,1414-1425,1898-1904`: accepted forms and
  malformed-command coverage;
- `docs/commands/use.md:1-27` and `src/tabdat/help/topics/use.md:1-24`:
  public syntax and execution notes.

The direct command is case-insensitive (`use`/`USE`) and accepts surrounding
Python-compatible separator whitespace. After the three-character command
prefix, Python strips whitespace, splits at the first comma, and requires the
pre-comma portion to contain exactly one whitespace-delimited path token. Path
quotes are not removed: `use "my file.csv"` has two path tokens and is rejected,
while `use ""` is a literal local path token. A path containing `://` anywhere
is represented as a raw URI string; all other paths are represented as local
path text. No URI validation or filesystem check occurs.

Accepted command forms and defaults:

```text
use <path>
use <path>, lazy
use <path>, lazy engine=duckdb|polars
use <path>, delimiter(<string-or-identifier>) has_header(true|false)
```

The result has `execution_mode = eager` and no lazy engine by default. A bare
`lazy` flag selects lazy mode and defaults `lazy_engine` to `duckdb`; an
explicit `engine=duckdb` or `engine=polars` is permitted only with `lazy` and
is compared case-insensitively. `delimiter` accepts either a parenthesized
single string/identifier or an `option=value` string and preserves its value,
including an empty string. `has_header` accepts only one unquoted,
case-insensitive `true`/`false` identifier inside parentheses. Option names are
lowercase-sensitive: `LAZY`, `Engine`, and `HAS_HEADER` are unknown options.
Each recognized option may occur at most once. Options may appear in any order.

The exact direct-command diagnostics are:

| Input shape | Diagnostic |
| --- | --- |
| missing path or more than one pre-comma path token | `use expects exactly one path: use <path>` |
| a comma with no following option | `comma must be followed by at least one option` |
| an option name/value that is malformed before use-specific validation | the generic tokenizer/option diagnostic, such as `option names must be identifiers`, `option <name> expects at least one value`, `option <name> requires a value after =`, or `option <name> values must be identifiers` |
| duplicate option names | `use option specified more than once` |
| unrecognized option | `unknown use option: <name>` |
| `lazy` with any value, including `lazy=true` or `lazy(...)` | `use lazy option does not accept a value` (unless generic option parsing fails first) |
| `engine` not yielding a string | `use engine option expects a string value` |
| `engine` string other than `duckdb`/`polars` | `use engine must be duckdb or polars` |
| `engine` supplied without `lazy` | `use engine option requires lazy mode` |
| `delimiter` not yielding a string | `use delimiter option expects a string value` |
| `has_header` not yielding a boolean | `use has_header option expects a boolean value` |
| unsupported punctuation while tokenizing options | `unsupported token in command: <token>` |

Command-boundary oddities such as `use, lazy`, `use:data`, and `use==x` are
preserved as explicit parser-boundary cases rather than broadening the slice;
the Python dispatcher reports `unknown command: use` for the first, and its
structured parser reports the command-boundary diagnostics for the latter two.
Full general tokenizer parity, quoted/escaped path syntax, option expressions,
and `by ...: use ...` are deferred.

## Rust contract

Expose the following syntax-only public types:

```rust
pub enum DataSource {
    LocalPath(String),
    Uri(String),
}

pub enum ExecutionMode {
    Eager,
    Lazy,
}

pub enum LazyEngine {
    DuckDb,
    Polars,
}

pub enum Command {
    Use {
        source: DataSource,
        execution_mode: ExecutionMode,
        lazy_engine: Option<LazyEngine>,
        delimiter: Option<String>,
        has_header: Option<bool>,
    },
    // existing variants unchanged
}
```

Parsing is pure and backend-independent. The parser may add a small
use-specific option tokenizer/helper, but it must not introduce `PathBuf`, URI
or filesystem APIs, DuckDB/Polars types, environment access, or unsafe code to
the public/application boundary.

## Test contract

Rust tests will cover eager/local and URI sources, case and separator
normalization, lazy default/explicit engines, CSV options and value spelling,
option order, duplicate/unknown/type/constraint errors, missing and extra path
tokens, trailing commas, and the unchanged existing parser matrix. Public
integration tests must pattern-match every new typed field rather than relying
on debug formatting.

The focused pinned oracle checks are:

```sh
cd ../tabdat-explore
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py -k 'test_parse_use_command or test_parse_invalid_commands'
```

The broader pinned parser/script regression remains:

```sh
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py tests/test_script.py
```

## Implementation mapping and deferrals

- Native Rust: typed source/mode/engine enums, direct dispatch, option parsing,
  deterministic diagnostics, and unit/public tests.
- Deferred: file or URI loading, format inference, CSV/Parquet/Arrow/Stata
  decoding, named-table lookup, relation ownership, lazy planning, session
  mutation, `UseResult`, CLI/JSON/MCP rendering, script and prefix wrappers,
  and backend capability initialization.

Acceptance requires the focused/full oracle results, Rust fmt/check/test/Clippy,
policy scans, `git diff --check`, independent parser/contract/workspace review,
and all hosted CI checks. Stop if parity requires execution or broad command
grammar work; record the conflict instead of silently expanding this slice.

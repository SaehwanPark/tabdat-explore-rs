# Contract: syntax-only `run <script-path>`

Status: draft contract; implementation and acceptance evidence are pending.

Selected skills: `tabdat-migration`, `simple-code-writer`

Rust base: `81812e0` (`main` after the accepted `isid` syntax slice and its
post-merge documentation-only CI run).

## Scope

Add only the direct `run <script-path>` command to the pure
`tabdat-language` parser. The slice returns an owned typed command without
opening a path, reading a script, executing commands, mutating session state,
initializing a backend, or adding a runtime dependency. Existing commands and
diagnostics remain unchanged. Line-oriented scripts, comments, nested or
recursive `run`, macros, control flow, file/line diagnostics, CLI/JSON/MCP
surfaces, and `by:` wrappers remain deferred.

## Python contract

The pinned clean oracle is the sibling checkout `../tabdat-explore` at commit
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` (tree
`601b236788872323af9277d2276a236154a0f129`, package `0.25.0`, Python
`3.13.3`). Its `uv.lock` SHA-256 is
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239` and the
worktree is clean. Authoritative paths are:

- `src/tabdat/models.py:438-440`: `RunCommand(path: Path)`;
- `src/tabdat/parser.py:146-147,252-306,303-304,518-523,1002-1009,3036-3123,3327-3395`:
  executable command inventory, generic command-name dispatch/tokenization,
  direct routing, the specialized parser, and the generic argument/assignment
  diagnostic paths used at the command boundary;
- `tests/test_parser.py:370-371,1539-1544`: the positive form and invalid
  command matrix;
- `docs/commands/run.md:1-25` and `src/tabdat/help/topics/run.md:1-25`:
  public syntax and execution description.

The Python dispatcher strips surrounding whitespace, takes the first
whitespace-delimited token as a case-insensitive command name, and routes
`run` to `_parse_run`. That parser removes the first three characters (the
`run` prefix), strips whitespace, splits the remainder with ordinary
`str.split()`, and requires exactly one token. It returns
`RunCommand(path=Path(path_token))`.

Accepted direct forms and observed values are:

```text
run analysis.td       -> RunCommand(path=Path("analysis.td"))
RUN   analysis.td     -> the same command
run\tanalysis.td      -> the same command
run\x1canalysis.td    -> the same command (Python command-separator control whitespace)
run "analysis.td"    -> Path('"analysis.td"') (quotes are retained)
run analysis.td,      -> Path('analysis.td,') (comma is part of the token)
```

The path token is not quote-unescaped or filesystem-validated. A quoted or
backtick-delimited token containing whitespace therefore counts as multiple
tokens and is rejected. Python's `Path(...)` object performs its own lexical
path representation (for example, `Path("./a.td")` displays as `a.td`); this
slice does not claim script-path normalization or execution semantics. The
Rust syntax command will own the exact one-token path text so those effects
remain at a future script/file boundary.

Exact diagnostics frozen for direct and command-boundary cases are:

| Input shape | Diagnostic |
| --- | --- |
| `run`, `run   `, `run a.td b.td`, `run "a b.td"`, or `run a.td if x > 0` | `run expects exactly one path: run <script>` |
| `run,` | `comma must be followed by at least one option` |
| `run,foo` | `unknown command: run` |
| `run=foo` | `run assignment requires a target before =` |
| `run:foo` | `unsupported token in command: :` |

`run == foo` is routed to the specialized parser and produces the same
`run expects exactly one path: run <script>` arity diagnostic because the
remainder has two whitespace-delimited tokens. The attached-boundary rows are
also grounded in the generic dispatcher and tokenizer: the dispatcher takes
the first whitespace-delimited token as the command name, while the generic
argument parser recognizes comma/equal boundaries and the tokenizer recognizes
`==` and `:` as symbols (see `parser.py:252-306,3036-3123,3327-3395`). A
reproducible pinned-oracle probe is:

```sh
cd ../tabdat-explore
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync python -c 'from tabdat.parser import parse_command, ParseError
for text in ("run,foo", "run=foo", "run==foo", "run:foo"):
    try:
        print(f"{text!r} -> {parse_command(text)!r}")
    except ParseError as exc:
        print(f"{text!r} -> {exc}")'
```

It prints `unknown command: run`, `run assignment requires a target before =`,
`unsupported token in command: ==`, and `unsupported token in command: :`,
respectively. Attached punctuation outside these cases, generalized tokenizer
parity, and prefixed command behavior are not broadened by this slice.

## Rust contract

Expose one additional owned syntax command:

```rust
pub enum Command {
    Run { path: String },
    // existing variants unchanged
}
```

`path` is the exact non-empty whitespace token after the `run` prefix. Parsing
is pure and backend-independent: no `PathBuf` normalization, filesystem
access, environment lookup, script loading, or command execution occurs.
The runtime only needs an exhaustive command-name arm; executing `Run` remains
`RuntimeError::UnsupportedCommand { name: "run" }`.

## Test contract

Rust unit and public integration tests will cover:

- direct path capture, case normalization, and command-separator whitespace;
- retained quotes/backticks and punctuation in a one-token path;
- the exact missing/multiple-token diagnostic;
- the frozen comma, assignment, and colon command-boundary diagnostics;
- runtime deferral through `UnsupportedCommand { name: "run" }`;
- the unchanged parser/runtime test matrix.

The focused pinned oracle check is:

```sh
cd ../tabdat-explore
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py -k 'test_parse_phase_8_run_command or test_parse_invalid_commands'
```

The observed result is `419 passed, 70 deselected in 0.44s`. The broader
regression remains:

```sh
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py tests/test_script.py
```

It previously passed `516 passed` at the same pinned revision and must be
rerun for acceptance. No trusted statistical reference or native-backend check
is relevant to this syntax-only slice.

## Implementation mapping and deferrals

- Native Rust: one owned `Command::Run` variant, direct dispatch, exact
  one-token validation, deterministic diagnostics, and unit/public tests.
- Runtime: only the command-name mapping and an explicit unsupported-command
  regression; no script result or session effect.
- Deferred: script file reads, line-oriented execution, comments, multiline
  SQL, `seed`, `let`, macro expansion, `if`/`else`/`end`, nested `run`,
  recursion rejection, file/line diagnostics, CLI/JSON/MCP output, and all
  filesystem/path normalization semantics.

Acceptance requires the focused/full oracle results, Rust fmt/check/test/
Clippy, policy scans, `git diff --check`, independent parser/contract/workspace
review, and all hosted CI checks. The completion state is `partial` until
those artifacts and checks are recorded.

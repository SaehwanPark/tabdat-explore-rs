# Contract: syntax-only `rename <old> <new>`

Status: draft bounded contract; implementation and acceptance evidence are
pending.

Selected skills: `tabdat-migration`, `simple-code-writer`

Rust base: `dad2636` (`main` after the accepted `run` syntax slice and its
post-merge documentation-only CI run).

## Scope

Add only the direct `rename <old-name> <new-name>` command to the pure
`tabdat-language` parser. The slice returns an owned typed command without
looking up columns, checking collisions, transforming a relation, mutating
session state, initializing a backend, or adding a runtime dependency.
Existing commands and diagnostics remain unchanged. Conditions, options,
assignments, by-wrappers, CLI/JSON/MCP surfaces, and execution semantics remain
deferred.

## Python contract

The pinned clean oracle is the sibling checkout `../tabdat-explore` at commit
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` (tree
`601b236788872323af9277d2276a236154a0f129`, package `0.25.0`, Python
`3.13.3`). Its `uv.lock` SHA-256 is
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239` and the
worktree is clean.

Authoritative paths are:

- `src/tabdat/models.py:262-265`: `RenameCommand(old_name, new_name)`;
- `src/tabdat/parser.py:146-147,252-306,3036-3123,3327-3395,646-651`:
  executable-command inventory, generic dispatch/tokenization, and the
  specialized `rename` branch;
- `tests/test_parser.py:254-273,1490-1497`: positive transformation coverage
  and invalid-command cases;
- `docs/commands/rename.md:1-18` and
  `src/tabdat/help/topics/rename.md`: public syntax and execution description;
- `src/tabdat/cli.py:35,126,329`: command effect/catalog metadata.

The Python dispatcher strips surrounding whitespace, takes a
case-insensitive command name, and routes `rename` through the generic parsed
parts. The specialized branch rejects conditions, options, and expressions,
requires exactly two arguments, and returns the two parsed argument values.
Generic tokenization removes surrounding single/double/backtick quoting and
preserves the resulting argument spelling.

Accepted direct forms and observed values are:

```text
rename sex gender       -> RenameCommand(old_name="sex", new_name="gender")
RENAME   sex   gender   -> the same command
rename `old-name` `new-name` -> RenameCommand("old-name", "new-name")
rename "old" "new"    -> RenameCommand("old", "new")
rename old old         -> RenameCommand("old", "old")
```

Names are syntactically captured only. Existence, collision, type, and
same-name validation belong to the future relation/session transformation.

Exact diagnostics frozen for direct and command-boundary cases are:

| Input shape | Diagnostic |
| --- | --- |
| `rename`, `rename old`, or `rename old new now` | `rename expects exactly two variables: rename old new` |
| `rename if`, `rename old if`, or `rename old new if` | `missing expression after if` |
| `rename old if x > 0`, `rename old new if x > 0`, or `rename old new, replace` | `rename expects exactly two variables: rename old new` |
| `rename old new,` | `comma must be followed by at least one option` |
| `rename=old new` or `rename = old` | `rename assignment requires a target before =` |
| `rename==old new` | `unsupported token in command: ==` |
| `rename:old new` | `unsupported token in command: :` |
| `rename old-new new` | `unsupported token in command: -` |

The generic dispatcher and tokenizer provide the command-boundary behavior
(see `parser.py:252-306,3036-3123,3327-3395`). Attached-symbol variants not
listed above, generalized tokenizer parity, and prefixed command behavior are
not broadened by this slice.

A reproducible pinned-oracle probe is:

```sh
cd ../tabdat-explore
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync python -c 'from tabdat.parser import parse_command, ParseError
cases = ("rename sex gender", "RENAME   sex   gender", "rename `old-name` `new-name`", "rename \\\"old\\\" \\\"new\\\"", "rename old old", "rename", "rename old", "rename old new now", "rename old if x > 0", "rename old new if x > 0", "rename old new, replace", "rename old new,", "rename=old new", "rename==old new", "rename:old new", "rename old-new new")
for text in cases:
    try:
        print(f"{text!r} -> {parse_command(text)!r}")
    except ParseError as exc:
        print(f"{text!r} -> {exc}")'
```

At the pinned revision it prints the accepted commands and the exact
diagnostics in the table above.

## Rust contract

Expose one additional owned syntax command:

```rust
pub enum Command {
    Rename { old_name: String, new_name: String },
    // existing variants unchanged
}
```

Parsing is pure and backend-independent. Reuse the existing simple-command
argument and quote handling, require exactly two arguments, and preserve the
argument values after generic quote removal. Do not add `PathBuf`, filesystem
access, schema lookup, relation mutation, environment lookup, or execution.
The runtime needs only an exhaustive command-name arm; executing `Rename`
remains `RuntimeError::UnsupportedCommand { name: "rename" }`.

## Test contract

Rust unit and public integration tests will cover:

- canonical, uppercase, and extra-whitespace forms;
- quoted/backtick names and duplicate names;
- wrong arity, conditions/options, assignment, and punctuation diagnostics;
- runtime deferral through `UnsupportedCommand { name: "rename" }`;
- the unchanged parser/runtime test matrix.

The focused pinned oracle check is:

```sh
cd ../tabdat-explore
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py -k 'test_parse_phase_3_transformation_commands or test_parse_invalid_commands'
```

The oracle previously passed `419 passed, 70 deselected` for this selection and
`516 passed` for `tests/test_parser.py tests/test_script.py`; both are rerun for
acceptance. No trusted statistical reference or native-backend check is
relevant to this syntax-only slice.

## Implementation mapping and deferrals

- Native Rust: one owned `Command::Rename` variant, direct dispatch, exact
  two-argument validation, deterministic diagnostics, and unit/public tests.
- Runtime: only command-name mapping and an explicit unsupported-command
  regression; no session effect.
- Deferred: active-schema lookup, collision/same-name validation, relation
  transformation, panel metadata, by-groups, CLI/JSON/MCP output, and all
  backend/session semantics.

Acceptance requires the focused/full oracle results, Rust fmt/check/test/
Clippy, policy scans, `git diff --check`, independent parser/contract/workspace
approval, all hosted PR-head checks, the squash merge, deleted-branch
verification, and post-merge `main` checks.

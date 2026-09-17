# Set-command syntax evidence

Status: partial; implementation is pushed in PR #15 and awaits review and
hosted verification.

Producer: task owner

Consumer: reviewer and next maintainer

Boundary: Python parser contract → Rust syntax-only parser

Rust implementation revisions: `47a58c8` (contract), `627591b`, `58426aa`,
`a54e55f`, and `8fcd522` (parser, parity fixes, quote boundaries, and tests)

Python oracle revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` (tree
`601b236788872323af9277d2276a236154a0f129`)

## Inputs and authority

The contract was recovered from the pinned clean sibling checkout
`../tabdat-explore`:

- `src/tabdat/models.py:447-450` (`SetCommand` and its finite names);
- `src/tabdat/parser.py:701-702,1012-1025` (routing and parser validation);
- `tests/test_parser.py:374-384,1542-1544` (accepted and invalid forms);
- `docs/commands/set.md:1-25` (public syntax and examples).

The focused oracle check passed without changing the sibling checkout:

```text
uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py -k phase_9_configuration_and_persistence_commands
1 passed, 488 deselected in 0.28s
```

The full pinned parser/script regression suite also passed:

```text
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py tests/test_script.py
516 passed in 0.43s
```

Targeted probes confirmed case-insensitive command and setting names,
case-preserving values, local paths containing `/`, quoted values containing
spaces, preserved symbol-bearing values including `<=`/`>=`, the
accepted-but-not-yet-validated value domain, backtick-quoted-name rejection,
unsupported non-tokenizer punctuation, adjacent quoted fragments, missing/extra
arguments, conditions, options, assignments, and trailing-comma handling. The
Rust contract intentionally covers only the direct command; configuration
execution and prefixed-command behavior remain deferred.

## Changed paths

- `crates/tabdat-language/src/lib.rs`: `SettingName`, `Command::Set`, dedicated
  dispatch, symbol-aware setting-value parsing, and exact diagnostics;
- `crates/tabdat-language/tests/parser_contract.rs`: public typed-command tests;
- `_workspace/parser-set-syntax/01-contract.md`: bounded migration contract;
- `_workspace/parser-set-syntax/02-evidence-migration.md`: this evidence record.

No session/configuration state, filesystem access, plotting, data relation,
execution, result, serialization, CLI, or runtime dependency changed.

## Rust verification

The implementation currently passes locally:

```text
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo test --locked --workspace --all-targets
  root smoke: 1 passed
  tabdat-language unit tests: 16 passed
  tabdat-language integration tests: 6 passed
cargo clippy --locked --workspace --all-targets -- -D warnings
git diff --check
cargo deny check
  advisories ok, bans ok, licenses ok, sources ok
cargo audit -D warnings
  exit 0; Cargo.lock scanned with the pinned advisory database
per-package cargo geiger metadata loop
  root and tabdat-language: 0 unsafe usage, `#![forbid(unsafe_code)]`
```

PR #15 is the hosted-check authority; native ReadStat/libgretl workflows remain
isolated from this language-only change.

## Supported and deferred behavior

Supported here is only parsing: `set graph_format <value>`,
`set artifact_dir <path>`, and `set graph_open <value>` produce owned typed
commands with preserved value text and deterministic diagnostics. The parser does
not validate image formats, booleans, paths, mutate configuration/session state,
inspect the filesystem, open graphs, or initialize a backend.

Typed configuration state, value validation, persistence, runtime effects, JSON/
MCP/terminal rendering, prefixed commands, and the full tokenizer/option grammar
remain deferred. Differences outside the contract table are inputs for future
parser work, not silently accepted runtime behavior.

# Parser syntax foundation evidence

Status: partial until hosted CI and review/merge gates complete
Producer: task owner
Consumer: reviewer/next maintainer
Rust revision: `2d4ef57` on `feat/parser-syntax-foundation`
Python oracle revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`

## Python contract

The clean sibling checkout at the pinned revision passed the bounded parser/script
suite under Python 3.13.3:

```text
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py tests/test_script.py
516 passed in 0.44s
```

Targeted probes covered `help`/`?`, lowercased topics, `status`, `exit`/`quit`,
empty/unknown/quoted commands, comma/equal delimiters, `==`, trailing commas,
Unicode command normalization, U+001C–U+001F whitespace, and doubled/unterminated
quotes. The expected results and source/test paths are recorded in `01-contract.md`.

## Rust contract

`tabdat-language` exposes `parse_command(&str) -> Result<Command, ParseError>`.
`Command::Help { topic }`, `Command::Status`, and `Command::Exit` are owned and
backend-independent. `ParseError` owns deterministic display text and implements
`Error`. The crate forbids unsafe code and performs no I/O, execution, or capability
initialization. The root `Hello, world!` binary and its smoke test remain unchanged.

## Test contract

Local results at `2d4ef57`:

```text
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo test --locked --workspace --all-targets
cargo clippy --locked --workspace --all-targets -- -D warnings
git diff --check
cargo deny check
cargo audit -D warnings
workspace-package cargo geiger scans
RUSTDOCFLAGS='-D warnings' cargo doc --locked --workspace --no-deps
```

All commands passed locally. Rust tests cover positive aliases/case/whitespace,
stable error messages, unknown and quoted commands, malformed quotes,
information-separator whitespace, argument, option, assignment, and unsupported
`==` forms. Hosted CI for PR #11 is still the authoritative merge gate and must be
green on this revision.

## Implementation mapping

- Native Rust: first-command boundary, quote diagnostics, typed syntax commands,
  and parse errors.
- DuckDB/native/statistics: not used.
- Deferred: full tokenizer/expression AST, remaining commands, scripts, execution,
  reporting, JSON/MCP surfaces, and differential harness automation.

No intentional Rust/Python behavior deviation is accepted for the bounded forms.
Diagnostics for malformed quoted tokens after a first token and punctuation outside
this contract (for example `status:` or `status/foo`) remain deferred with the
general tokenizer work; they do not affect the accepted scoped commands.

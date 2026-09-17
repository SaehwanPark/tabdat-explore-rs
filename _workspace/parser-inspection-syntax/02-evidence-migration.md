# Inspection-command syntax evidence

Status: partial pending review and hosted CI  
Producer: task owner  
Consumer: reviewer and next maintainer  
Boundary: Python parser contract → Rust syntax-only parser  
Rust implementation revision: `7264dae` (working tree also contains the
documentation updates for this slice)  
Python oracle revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` (tree
`601b236788872323af9277d2276a236154a0f129`)

## Inputs and authority

The contract was recovered from the pinned sibling checkout
`../tabdat-explore`:

- `src/tabdat/models.py:215-241` (`CountCommand`, `HeadCommand`, `TailCommand`);
- `src/tabdat/parser.py:606-621` (dispatch) and `3018-3033` (limit validation);
- `tests/test_parser.py:205-215` (positive forms) and invalid-command cases near
  `1478-1489`;
- `docs/commands/count.md`, `docs/commands/head.md`, and
  `docs/commands/tail.md`.

The pinned parser/script oracle was run without changing the sibling checkout:

```text
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py tests/test_script.py
516 passed in 0.45s
```

Targeted probes confirmed `count`, default and canonicalized `head`/`tail`
limits, quoted numeric arguments, `u64::MAX + 1`, a 100-digit value, and the
exact invalid-limit, unsupported-token, option, condition, assignment, and
trailing-comma diagnostics recorded in `01-contract.md`.

## Changed paths

- `crates/tabdat-language/src/lib.rs`: `Command::Count`, `Command::Head`,
  `Command::Tail`, validated `RowLimit`, syntax parser, and diagnostics;
- `crates/tabdat-language/tests/parser_contract.rs`: public command and limit
  assertions;
- `_workspace/parser-inspection-syntax/01-contract.md`: bounded contract;
- `README.md`, `SPEC.md`, `ARCHITECTURE.md`, and
  `docs/TABDAT_RUST_PORT_ROADMAP.md`: current-state and evidence wording only.

No backend, session, relation, execution, result, serialization, or runtime
dependency changed.

## Rust verification

The following commands passed locally on the implementation revision:

```text
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo test --locked --workspace --all-targets
  root smoke: 1 passed
  tabdat-language unit tests: 10 passed
  tabdat-language integration tests: 3 passed
cargo clippy --locked --workspace --all-targets -- -D warnings
git diff --check
cargo deny check
cargo audit -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --locked --workspace --no-deps
```

The CI Geiger metadata loop scanned both workspace packages and reported no
unsafe usage. Existing isolated ReadStat and libgretl spike tests also passed;
the DuckDB spike has no changed paths in this slice and remains covered by the
merged PR #11 workflow. Hosted PR #12 checks are still pending at this evidence
revision.

## Supported and deferred behavior

Supported here is parsing only: case-insensitive command names, surrounding and
Python-compatible separator whitespace, defaults, zero/leading-zero limits,
quoted numeric limits, arbitrary decimal length, owned typed commands, and the
listed deterministic diagnostics. The parser does not inspect an active dataset,
execute `count`/`head`/`tail`, preserve backend row order, handle missing values,
convert limits for a backend, or produce results/terminal/JSON/MCP output.

The full tokenizer, expression grammar, command inventory, session lifecycle,
and Phase 4 data-runtime work remain unchecked in the roadmap. Backend overflow
policy for decimal limits must be decided explicitly in the later execution
contract; this parser does not silently clamp or reject large syntax values.

## Known deferred diagnostic gaps

Because this slice deliberately does not add the Python tokenizer or expression
grammar, it does not claim parity for malformed forms outside the contract table.
For example, attached punctuation (`head?1`, `count:`), compound symbols such as
`!=`, malformed-number diagnostics with multiple decimal points, expression
errors after an `if`, missing assignment expressions, duplicate `if` clauses, and
malformed option lists still use the bounded parser's simpler diagnostics (or
remain at the command-boundary error). These cases are inputs for the future
tokenizer/parser slice, not silently accepted execution behavior.

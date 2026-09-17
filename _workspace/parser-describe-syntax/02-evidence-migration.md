# Describe-command syntax evidence

Status: partial pending hosted CI
Producer: task owner  
Consumer: reviewer and next maintainer  
Boundary: Python parser contract → Rust syntax-only parser  
Rust implementation revision: `a1d2276`  
Python oracle revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` (tree
`601b236788872323af9277d2276a236154a0f129`)

## Inputs and authority

The contract was recovered from the pinned sibling checkout
`../tabdat-explore`:

- `src/tabdat/models.py:136-140` (`DescribeCommand`);
- `src/tabdat/parser.py:525-529` (dispatch and no-argument validation);
- `tests/test_parser.py:187-188` (positive form) and `1464-1465` (invalid
  arguments/conditions);
- `docs/commands/describe.md:5-9` (public syntax).

Oracle checks ran without changing the sibling checkout:

```text
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py -k 'parse_describe_command or parse_invalid_commands'
419 passed, 70 deselected in 0.40s

PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py tests/test_script.py
516 passed in 0.44s
```

Targeted probes confirmed case/whitespace normalization, the bare command,
argument/condition/option rejection, assignment and `==` diagnostics, the
unsupported-minus diagnostic, and trailing-comma handling. A follow-up
regression probe also confirms that the pre-existing `status -1` and `status +1`
diagnostics remain unchanged by this slice.

## Changed paths

- `crates/tabdat-language/src/lib.rs`: `Command::Describe`, dispatch, and
  exact no-argument diagnostics;
- `crates/tabdat-language/tests/parser_contract.rs`: public describe assertions;
- `_workspace/parser-describe-syntax/01-contract.md`: bounded contract;
- `README.md`, `SPEC.md`, `ARCHITECTURE.md`, and
  `docs/TABDAT_RUST_PORT_ROADMAP.md`: current-state and evidence wording only.

No backend, session, relation, execution, result, serialization, or runtime
dependency changed.

## Rust verification

The implementation passed locally:

```text
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo test --locked --workspace --all-targets
  root smoke: 1 passed
  tabdat-language unit tests: 12 passed
  tabdat-language integration tests: 4 passed
cargo clippy --locked --workspace --all-targets -- -D warnings
git diff --check
```

The follow-up local run after the review fix passed the same locked checks;
Rust totals remain one root smoke test, 12 language unit tests, and four
language integration tests.

The merged `main` revision `79ae1d9` had green dependency/unsafe-policy,
ReadStat, DuckDB, and libgretl hosted checks immediately before this branch.
PR #13 hosted checks for the current implementation/docs revision remain
pending; the PR will not be merged until every current-head check is green.

## Supported and deferred behavior

Supported here is only parsing: case-insensitive `describe`, surrounding and
Python-compatible separator whitespace, the deterministic no-argument command,
and the listed exact diagnostics. The parser does not inspect schema metadata,
require an active dataset, execute a command, or produce results/terminal/JSON/MCP
output.

Full tokenizer, expression, option, varlist, command-boundary, session, and
Phase 4 data-runtime behavior remain deferred. Any malformed-token differences
outside the contract table are inputs for the future tokenizer/parser slice, not
silently accepted execution behavior.

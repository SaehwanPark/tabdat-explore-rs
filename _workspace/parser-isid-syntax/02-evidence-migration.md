# `isid` syntax evidence

Status: draft pending independent review and hosted checks. The bounded slice
is accepted only when PR #23 is squash-merged; the Phase 4 `isid` execution
item remains unchecked.

Producer: task owner

Consumer: reviewers and the next maintainer

Boundary: pinned Python parser contract → Rust syntax-only parser

Rust implementation revisions: `5b7b2ee` (contract), `e0bcde1` (typed command,
bounded parser, runtime rejection, and tests), and `abfa95f` (isid-specific
duplicate-key and doubled-backtick regression coverage). The documentation/
evidence revisions complete this record. The final squash commit will be recorded
here after PR #23 merges.

Python oracle revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` (tree
`601b236788872323af9277d2276a236154a0f129`), Python 3.13.3. The clean sibling
checkout and lock digest remain those recorded in `docs/migration/README.md`.

## Inputs and authority

The contract was recovered from the clean pinned sibling checkout
`../tabdat-explore`:

- `src/tabdat/models.py:193-199` (`IsidCommand` with ordered `variables` and
  `missok`);
- `src/tabdat/parser.py:591-592,3005-3015` (routing and direct command
  construction);
- `src/tabdat/parser.py:3036-3123,3159-3200` (generic argument, option,
  condition, assignment, and diagnostic behavior);
- `tests/test_isid.py:83-98` (focused parser contract);
- `docs/commands/isid.md:1-68` (public syntax and execution semantics); and
- `src/tabdat/help/topics/isid.md:1-25` (interactive syntax/help text).

The focused oracle check passed without modifying the pinned checkout:

```text
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_isid.py -k 'test_parse_isid_forms'
1 passed, 19 deselected in 0.28s
```

The full pinned parser/script regression also passed without modifying the
oracle:

```text
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py tests/test_script.py
516 passed in 0.50s
```

Additional read-only probes confirmed the exact bounded behavior for ordered
and duplicate keys, quoted and backtick identifiers (including a comma and
doubled backticks), duplicate `missok`, exact lowercase option matching,
trailing commas, unsupported options, option values, missing `if` expressions,
assignments, and unsupported punctuation. Representative results include:

```text
'isid patient_id visit' -> IsidCommand(variables=('patient_id', 'visit'), missok=False)
'ISID `patient_id` visit, missok' -> IsidCommand(variables=('patient_id', 'visit'), missok=True)
'isid `a,b`, missok' -> IsidCommand(variables=('a,b',), missok=True)
'isid' -> ParseError: isid expects at least one key variable
'isid patient_id if visit > 0' -> ParseError: isid only accepts a variable list and missok option
'isid patient_id =' -> ParseError: isid assignment requires an expression after =
'isid patient_id, report' -> ParseError: isid unsupported option: report
'isid patient_id, missok(true)' -> ParseError: isid option missok does not accept a value
'isid patient_id, MISSOK' -> ParseError: isid unsupported option: MISSOK
```

The Python `IsidCommand` execution path and language-semantics documentation
remain authority for a future runtime slice; this parser slice makes no claim
about active-data behavior.

## Changed paths

- `crates/tabdat-language/src/lib.rs`: owned `Command::Isid`, direct dispatch,
  bounded top-level comma handling, exact option/argument diagnostics, and
  unit tests;
- `crates/tabdat-language/tests/parser_contract.rs`: public typed-command and
  exact-diagnostic coverage;
- `crates/tabdat-runtime/src/lib.rs` and
  `crates/tabdat-runtime/tests/use_contract.rs`: exhaustive command-name
  mapping and explicit deferred-execution rejection;
- `_workspace/parser-isid-syntax/01-contract.md`: recovered migration
  boundary;
- this file and `_workspace/parser-isid-syntax/03-review.md`: evidence and
  independent-review records; and
- `README.md`, `ARCHITECTURE.md`, `SPEC.md`, and
  `docs/TABDAT_RUST_PORT_ROADMAP.md`: current verified-slice references.

No active relation, schema lookup, variable validation, missingness or
duplicate scan, session mutation, execution, result serialization, CLI,
script engine, statistical calculation, or backend capability changed.

## Rust verification

The implementation and documentation passed the required local gates:

```text
cargo fmt --all -- --check                                  passed
cargo check --locked --workspace --all-targets              passed
cargo test --locked --workspace --all-targets               passed
  root scaffold: 1; tabdat-language: 30 unit + 19 integration;
  tabdat-runtime: 2 unit + 6 integration
cargo clippy --locked --workspace --all-targets -- -D warnings passed
git diff --check                                            passed
cargo deny check                                            passed
cargo audit -D warnings                                     passed
```

The metadata-driven `cargo geiger` policy loop from `CONTRIBUTING.md` was run
for every workspace package with locked metadata and JSON output. All first-
party packages forbid unsafe code and report zero first-party unsafe functions
or expressions; dependency inventory warnings are retained as policy output:

```text
tabdat-explore-rs  forbids_unsafe=true  unsafe_functions=0  unsafe_exprs=0  unscanned=0
tabdat-language    forbids_unsafe=true  unsafe_functions=0  unsafe_exprs=0  unscanned=0
tabdat-runtime     forbids_unsafe=true  unsafe_functions=0  unsafe_exprs=0  unscanned=33
```

The runtime's 33 unscanned dependency assets are the expected bundled-DuckDB
inventory warning described by ADR 0007; the geiger process completed
successfully and did not identify first-party unsafe usage. These scans do not
prove native FFI safety.

Hosted checks for the final PR head and the post-merge main commit will be
recorded after GitHub completes them. Local evidence is not a substitute for
those hosted checks.

## Supported and deferred behavior

Supported here is direct syntax only: an owned `Command::Isid` records the
ordered key-variable text and whether exact lowercase `missok` was supplied,
with the recorded bounded diagnostics. Runtime execution remains a typed
`UnsupportedCommand` error.

Deferred are wildcard/range expansion, unknown-variable validation, null-key
and duplicate-key semantics, active-relation/schema behavior, eager/lazy
scans, materialization, labels/panel metadata, `by:` wrappers, full
tokenizer/varlist/option and expression grammar, scripts, CLI/JSON/MCP
surfaces, result/reporting/serialization, and backend capability
initialization. No Rust/Python differential suite beyond the focused oracle
checks is claimed.

# Doctor-command syntax evidence

Status: accepted and merged in PR #14
Producer: task owner
Consumer: reviewer and next maintainer
Boundary: Python parser contract → Rust syntax-only parser
Rust implementation revisions: `f93b88e`, evidence/review `22a4909` and
`15a1abe`; merged as `0b918c7`
Python oracle revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` (tree
`601b236788872323af9277d2276a236154a0f129`)

## Inputs and authority

The contract was recovered from the pinned sibling checkout
`../tabdat-explore`:

- `src/tabdat/models.py:150-154` (`DoctorCommand`);
- `src/tabdat/parser.py:543-553` (dispatch and no-argument validation);
- `tests/test_doctor.py:51-68` (positive and invalid parser cases);
- `docs/commands/doctor.md:8-12` (public syntax).

Oracle checks ran without changing the sibling checkout:

```text
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_doctor.py -k 'parser_doctor_valid or parser_doctor_invalid'
6 passed, 8 deselected in 0.27s

PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py tests/test_script.py
516 passed in 0.45s
```

Targeted probes confirmed case/whitespace normalization, the bare command,
argument/condition/option rejection, the missing-`if` diagnostic, assignment,
`==`, `-`, `+`, and trailing-comma handling. The `by:` prefixed-command guard
remains outside this no-general-tokenizer slice.

## Changed paths

- `crates/tabdat-language/src/lib.rs`: `Command::Doctor`, dispatch, and exact
  no-argument diagnostics;
- `crates/tabdat-language/tests/parser_contract.rs`: public doctor assertion;
- `_workspace/parser-doctor-syntax/01-contract.md`: bounded contract;
- `README.md`, `SPEC.md`, `ARCHITECTURE.md`, and
  `docs/TABDAT_RUST_PORT_ROADMAP.md`: current-state and evidence wording only;
- `_workspace/parser-describe-syntax/{01-contract,02-evidence,03-review}.md`:
  post-merge status corrections;
- `docs/migration/decisions.md`: explicit unresolved `status -/+` deviation.

No environment probe, backend, session, relation, execution, result,
serialization, or runtime dependency changed.

## Rust verification

The implementation passed locally:

```text
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo test --locked --workspace --all-targets
  root smoke: 1 passed
  tabdat-language unit tests: 14 passed
  tabdat-language integration tests: 5 passed
cargo clippy --locked --workspace --all-targets -- -D warnings
git diff --check
```

PR #14 current-head checks all passed: Rust baseline, dependency/unsafe policy,
ReadStat, and both libgretl workflows. The preceding `main` revision `f6525b3`
also had green post-merge CI, policy, ReadStat, and libgretl workflows. The
feature branch was deleted locally and remotely after the merge; this status
correction is documentation-only.

## Supported and deferred behavior

Supported here is only parsing: case-insensitive `doctor`, surrounding and
Python-compatible separator whitespace, the deterministic no-argument command,
and the listed exact diagnostics. The parser does not inspect host capabilities,
require an active dataset, execute a command, or produce results/terminal/JSON/MCP
output.

Full tokenizer, expression, option, varlist, prefixed-command, session, and
environment/reporting behavior remain deferred. Any malformed-token differences
outside the contract table are inputs for future tokenizer/parser work, not
silently accepted runtime behavior.

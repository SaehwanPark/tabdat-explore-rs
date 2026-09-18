# Migration decision log

This log tracks authority changes and intentional deviations, not feature completion.

| ID | Status | Scope | Decision / evidence |
| --- | --- | --- | --- |
| MIG-0001 | Accepted | Repository and baseline | [ADR 0001](../adr/0001-isolated-rust-port-and-migration-authority.md); Python commit `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` is the initial external oracle. |
| MIG-0002 | Unresolved | `status` tokenizer diagnostics beyond the bounded sign forms | At the pinned Python revision, `status -1` and `status +1` return `unsupported token in command: -` and `unsupported token in command: +`; the bounded PR #30 correction now matches those forms, including attached signs and a bare `status if`. The broader status lexical matrix (other punctuation, malformed numbers, and malformed condition expressions) remains unresolved until a future tokenizer slice. See the detailed reproduction, paths, impact, and resolution criteria below. Do not claim whole-parser/status parity from the bounded correction. |
| MIG-0003 | Accepted for bounded evaluation | Eager local-Parquet `use` path normalization | Python expands `Path(path).expanduser()` before suffix/existence checks; the bounded Rust runtime accepts an already-resolved local path and deliberately leaves `~` expansion to its caller. This is the documented scope deviation in [ADR 0007](../adr/0007-eager-parquet-duckdb-runtime-boundary.md), `_workspace/use-eager-parquet/01-contract.md`, and `_workspace/use-eager-parquet/02-evidence-data.md`, reviewed in PR #22. |

No other behavioral deviations have been accepted. MIG-0003 is limited to the
bounded runtime evaluation; the recorded syntax ports establish Rust command
behavior only for their recorded forms, not whole-language parity.

For each conflict/deviation, append an entry with a stable ID and details:

- Status: unresolved, accepted, rejected, or superseded.
- Affected contract and Python/Rust revisions; source/doc/test paths.
- Minimal reproduction, expected/observed results, and environment.
- Alternatives, user impact, and rationale (link an ADR for accepted deviations).
- Exact tests/reference evidence and resolution criteria; blocked dependent slices.
- Reviewing PR and superseding entry, if any.

Never delete unresolved evidence or silently rewrite tolerances/fixtures. Baseline
updates also get an entry identifying old/new pins and affected contract checks.

### MIG-0002 details

- **Affected paths and revisions:** Python `src/tabdat/parser.py` at the pinned
  oracle revision; Rust `crates/tabdat-language/src/lib.rs` and its
  `rejects_arguments_for_read_only_and_exit_commands` regression assertions at
  PR #13's follow-up revision `622e100`, merged as `f6525b3`, with the bounded
  sign/empty-condition correction in PR #30. The surrounding migration evidence
  is `_workspace/parser-describe-syntax/02-evidence-migration.md` and the
  review disposition is `_workspace/parser-describe-syntax/03-review.md`.
- **Minimal reproduction:** in the clean Python oracle, run
  `PYTHONDONTWRITEBYTECODE=1 uv run --no-sync python` and call
  `parse_command("status -1")` and `parse_command("status +1")`; the expected
  errors are `unsupported token in command: -` and `unsupported token in command:
  +`. PR #30 adds exact Rust regressions for those forms, `status-1`,
  `status+1`, signed forms before a comma, and bare `status if`.
- **Impact and rationale:** only malformed punctuation/empty-condition syntax
  after the syntax-only `status` command is affected; no data, session, or
  runtime behavior changes. The correction is intentionally limited to the
  observed contract and leaves the general tokenizer boundary explicit.
- **Resolution criteria and blocked work:** a future tokenizer/parity slice must
  compare the remaining status error matrix, choose Python parity or an accepted
  Rust deviation for other punctuation, malformed numbers, and malformed
  condition expressions, add exact regression tests, and supersede this entry.
  No current bounded syntax slice is blocked; claims of whole-parser/status
  parity remain blocked until that decision is resolved.
- **Review tracking:** discovered during the PR #13 contract review, rechecked
  before PR #14 implementation, and partially addressed by PR #30. Keep this
  entry unresolved until the broader matrix is decided.

### MIG-0003 details

- **Affected paths and revisions:** Python `src/tabdat/backend.py:3945-3966` at
  oracle commit `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`; Rust path handling in
  `crates/tabdat-runtime/src/lib.rs` at PR #22. The contract and review are
  `_workspace/use-eager-parquet/01-contract.md` and
  `_workspace/use-eager-parquet/03-review.md`.
- **Minimal reproduction:** Python resolves `use ~/patients.parquet` through
  `Path(...).expanduser()` before local-file validation. Rust receives the
  caller's `PathBuf` unchanged; an unresolved `~` therefore does not name the
  same file unless the caller expands it first.
- **Impact and rationale:** only shell-style home expansion differs. Keeping
  environment-dependent expansion outside the library avoids hidden process
  environment reads at this first runtime boundary; ordinary absolute and
  relative caller-resolved local paths retain the bounded eager-load contract.
- **Resolution criteria and blocked work:** a future path-normalization slice
  must choose caller-only normalization or explicit Rust expansion, add parity
  fixtures for `~` and platform home semantics, and supersede this entry. Broad
  `use` parity remains blocked on that decision and on the deferred formats,
  URI, lazy, and option paths.
- **Review tracking:** accepted for the bounded evaluation in PR #22 and linked
  from ADR 0007; supersede this entry if the runtime API later owns expansion.

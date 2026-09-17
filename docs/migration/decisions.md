# Migration decision log

This log tracks authority changes and intentional deviations, not feature completion.

| ID | Status | Scope | Decision / evidence |
| --- | --- | --- | --- |
| MIG-0001 | Accepted | Repository and baseline | [ADR 0001](../adr/0001-isolated-rust-port-and-migration-authority.md); Python commit `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` is the initial external oracle. |
| MIG-0002 | Unresolved | `status -1` and `status +1` parser diagnostics | At the pinned Python revision, both inputs return `unsupported token in command: -` and `unsupported token in command: +`. Rust `tabdat-language` preserves the pre-existing generic `status does not accept arguments, if clauses, options, or assignment syntax` diagnostic for both inputs. The difference was reproduced on Python 3.13.3 and Rust after PR #13's regression fix (`622e100`, merged as `f6525b3`). See the detailed reproduction, paths, impact, and resolution criteria below. It is outside the `doctor` slice; do not silently treat the Rust result as Python parity. |
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
  PR #13's follow-up revision `622e100`, merged as `f6525b3`. The surrounding
  migration evidence is `_workspace/parser-describe-syntax/02-evidence-migration.md`
  and the review disposition is `_workspace/parser-describe-syntax/03-review.md`.
- **Minimal reproduction:** in the clean Python oracle, run
  `PYTHONDONTWRITEBYTECODE=1 uv run --no-sync python` and call
  `parse_command("status -1")` and `parse_command("status +1")`; the expected
  errors are `unsupported token in command: -` and `unsupported token in command:
  +`. In Rust, `cargo test --locked --workspace --all-targets` exercises the
  preserved generic diagnostic for both inputs.
- **Impact and rationale:** only malformed punctuation after the syntax-only
  `status` command is affected; no data, session, or runtime behavior changes.
  The generic Rust result was retained to preserve the existing public Rust
  behavior while the bounded `describe` slice was kept free of unrelated parity
  changes.
- **Resolution criteria and blocked work:** a dedicated parser-parity slice must
  compare the complete status error matrix, choose Python parity or an accepted
  Rust deviation, add exact regression tests, and update this entry. No current
  bounded syntax slice is blocked; claims of whole-parser/status parity remain
  blocked until that decision is resolved.
- **Review tracking:** discovered during the PR #13 contract review and recorded
  before PR #14 implementation; PR #14's contract review rechecked the entry.
  There is no superseding entry yet.

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

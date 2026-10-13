# Migration decision log

This log tracks authority changes and intentional deviations, not feature completion.

| ID | Status | Scope | Decision / evidence |
| --- | --- | --- | --- |
| MIG-0001 | Accepted | Repository and baseline | [ADR 0001](../adr/0001-isolated-rust-port-and-migration-authority.md); Python commit `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` is the initial external oracle. |
| MIG-0002 | Unresolved | `status` tokenizer diagnostics beyond the bounded sign forms | At the pinned Python revision, `status -1` and `status +1` return `unsupported token in command: -` and `unsupported token in command: +`; the bounded PR #30 correction now matches those forms, including attached signs and a bare `status if`. The broader status lexical matrix (other punctuation, malformed numbers, and malformed condition expressions) remains unresolved until a future tokenizer slice. See the detailed reproduction, paths, impact, and resolution criteria below. Do not claim whole-parser/status parity from the bounded correction. |
| MIG-0003 | Accepted for bounded evaluation | Eager local-Parquet `use` path normalization | Python expands `Path(path).expanduser()` before suffix/existence checks; the bounded Rust runtime accepts an already-resolved local path and deliberately leaves `~` expansion to its caller. This is the documented scope deviation in [ADR 0007](../adr/0007-eager-parquet-duckdb-runtime-boundary.md), `_workspace/use-eager-parquet/01-contract.md`, and `_workspace/use-eager-parquet/02-evidence-data.md`, reviewed in PR #22. |

| MIG-0004 | Accepted | Syntax resource budgets | [ADR 0023](../adr/0023-syntax-resource-budgets-and-fuzz-tooling.md), PR #194: reject expressions above 256 tokens and command parsing at depth 32 to prevent reproduced stack exhaustion. |

| MIG-0005 | Accepted | Script seed syntax/range | Signed 64-bit ASCII decimal seeds; preserve pinned oracle outputs and assert the five explicit Rust rejections. PR #195; see details below. |

MIG-0003 is limited to the
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

### MIG-0004 details

- **Affected contract:** pinned Python parser accepts/rejects large syntax according
  to interpreter recursion behavior; Rust PR #194 adds stable resource diagnostics
  before recursive parsing/AST construction. Existing 672 captured inputs agree.
- **Reproduction:** `generate x = ` plus 20,000 opening parentheses, `1`, and
  20,000 closing parentheses previously triggered sanitizer stack overflow. Rust
  now returns `expression exceeds limit of 256 tokens`. Deep `by`/`bayes:` input
  returns `command nesting exceeds limit of 32`. See ADR 0023 for crash identity.
- **Impact/alternatives:** large formulas must be split; unrestricted recursion or
  simply moving it to a larger stack does not bound recursive AST lifecycle work.
  Limits do not cap raw SQL, script bytes, tokenization memory or backend resources.
- **Evidence/review:** exact-limit and adversarial property regressions, unchanged
  pinned corpus replay, two successful bounded sanitizer campaigns; three local
  owner review passes in PR #194. Future budget changes require measured stack/AST
  lifecycle evidence. Unrestricted input acceptance is explicitly not claimed.

### MIG-0005 details

- **Contract/reproduction:** pinned Python `src/tabdat/script.py` accepts
  `seed 1_000`, `seed ٠١٢`, and integers outside signed 64-bit range. The 91-call
  captured oracle includes those two forms and three overflow examples. Rust
  `parse_script_directive` rejects them with `seed expects an integer` and retains
  the script path/line. Python fixture results remain unchanged.
- **Decision/impact:** support optional sign plus ASCII decimal digits in `i64`
  range. Ordinary decimal scripts retain their behavior; scripts using Python's
  expanded integer grammar must use a supported seed. No new arbitrary-precision
  representation or Unicode numeric dependency is warranted for a value currently
  recorded only in ephemeral script context, not consumed by an execution RNG.
  Supporting underscore/Unicode forms alone would not resolve the range issue.
- **Evidence:** `script_seed_contract_is_signed_ascii_decimal_i64` tests signs,
  leading zeros, endpoints and invalid inputs without context mutation.
  `python_script_oracle` checks all 91 captured calls with exactly five explicit
  exceptions and checks their exact Rust errors. Canonical/nested CLI tests verify
  sequential execution and source diagnostics; see [script evidence](phase3-scripts.md).
- **Scope/review:** bounded decision in PR #195 under the refreshed AGENTS.md
  policy; no architecture or public API expansion, so no new ADR. Full Python
  seed parity is not claimed. Specify and validate deterministic RNG semantics
  separately before adding a command that consumes this context value.

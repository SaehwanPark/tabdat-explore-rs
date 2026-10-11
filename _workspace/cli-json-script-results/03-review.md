# Review: CLI JSONL results from `.td` scripts

**Producer:** task owner applying the `code-reviewer` workflow.
**Consumer:** PR reviewer and next maintainer.
**State:** complete. **Rust revision reviewed:**
`e9747e7893f1191dfd217598d18330e7bccc3f05`, plus the documentation update in
the current branch.

## Scope reviewed

Reviewed the complete branch diff against `origin/main`, the runtime script
executor and recursive call path, CLI script error formatting, JSON command and
result helpers, the pinned Python CLI tests, focused Rust integration tests,
and the workspace validation results recorded in
[`02-evidence-migration.md`](02-evidence-migration.md).

## Findings

**No actionable issues found.**

The preflight callback runs after parsing and before command execution. Nested
scripts receive both callbacks, while `run` itself does not emit a result. The
result observer runs immediately after each successful leaf command, preserving
depth-first script order. `execute_run` uses no-op callbacks, retaining the
existing non-JSON behavior. Source locations use the runtime's already-resolved
script path and 1-based command line.

The supported command set and serializers are aligned: `use` and `count` return
the only observed result variants; `exit` has no result; `run` recursively
validates its children. The preflight regression confirms an unsupported export
cannot write a file. No dependency or unsafe-code change was introduced.

## Residual scope

The Python oracle supports additional JSON result types, including
`PreviewResult`; the Rust slice reports its narrower capability explicitly and
does not claim full JSON script parity. GitHub Actions are disabled, so review
and validation evidence is local.

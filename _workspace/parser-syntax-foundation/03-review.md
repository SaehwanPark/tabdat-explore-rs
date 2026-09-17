# Parser syntax foundation review

Status: partial until hosted checks pass on the final revision
Reviewer: task owner with three independent read-only review passes
Revision reviewed: `f9f225e`

## Original acceptance checks

- Backend-independent `tabdat-language` parser for `help`/`?`, `status`, and
  `exit`/`quit`.
- Owned commands and deterministic errors, with `#![forbid(unsafe_code)]`.
- Root scaffold behavior unchanged and no runtime/backend dependency.
- Pinned Python parser/script evidence and focused Rust coverage.
- Root and isolated-spike checks green locally; hosted CI green before merge.

## Review passes and findings

1. Contract/scope pass found that adding a root workspace caused the previously
   independent spike manifests to fail path-scoped CI. Fixed by excluding all
   three spike directories from the root workspace and adding empty workspace
   declarations to their manifests. Local DuckDB formatting, ReadStat tests, and
   libgretl tests then passed.
2. Parser parity pass found `==`, Python information-separator whitespace, malformed
   leading quote, help-delimiter, Unicode normalization, and trailing-comma edge
   diagnostics. The bounded implementation now covers these cases with tests.
3. Workspace/CI pass found root Geiger scanned only the root package after the new
   workspace member was added. Fixed the CI and contributor command to iterate every
   package manifest from `cargo metadata` with `set -euo pipefail`.
4. Follow-up parser pass found only later malformed quote tokens and punctuation
   diagnostics outside the intentionally deferred full-tokenizer boundary. The
   contract and evidence explicitly defer those cases; they do not affect accepted
   scoped commands. A final follow-up review is pending on the latest documentation
   and workspace-isolation commit.

## Verification reviewed

Local root formatting/check/test/Clippy/diff checks, policy checks, rustdoc, and the
three isolated spike checks passed after the fixes. The latest hosted PR #11 checks
remain the merge authority; superseded failed/cancelled runs are not evidence against
the current head.

## Disposition

No unresolved blocking finding remains for the bounded contract. Keep the PR draft
until all checks for the final pushed revision are successful, then mark it ready,
merge it, and remove the temporary branch as authorized by the roadmap loop.

# CLI JSONL `use` and `count` review

## Scope

Reviewed the contract, result-to-envelope mapping, CLI dispatch changes, help
text, and integration coverage. The slice is limited to JSON batch execution of
local eager `use` and `count`; it does not change runtime state or reporting
behavior for non-JSON commands.

## Findings

No actionable correctness or compatibility findings remain in the bounded
slice. The CLI checks command support after parsing and before execution, so an
unsupported result command cannot mutate session state or write an output file.
The supported success envelopes preserve the Python `LoadResult` and
`CountResult` labels and field names, with explicit null metadata fields for
capabilities not represented in the current Rust dataset type. Existing runtime
errors continue to use the established error envelope and stderr path.

## Known limits

- Successful JSON results for runtime types other than load/count are rejected
  before execution.
- JSON script failures still produce error envelopes; successful script commands
  do not yet stream result envelopes.
- Terminal result rendering remains deferred.

These are documented capability limits, not findings against this slice. The
focused and full local Rust checks passed; hosted checks are unavailable because
repository GitHub Actions are disabled under ADR 0009.

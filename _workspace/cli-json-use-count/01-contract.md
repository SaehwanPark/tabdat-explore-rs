# CLI JSONL `use` and `count` contract

## Python contract

- Authority: Python TabDat commit `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`,
  tree `601b236788872323af9277d2276a236154a0f129`, `uv.lock` SHA-256
  `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.
- Verified checkout: clean sibling `../tabdat-explore`; Python 3.13.3, DuckDB
  1.5.2. No dependency synchronization was performed.
- Relevant sources are `src/tabdat/cli.py` (`_run_commands`, `_run_one`),
  `src/tabdat/formatter.py` (`format_result_json`), and
  `src/tabdat/models.py` (`LoadResult`, `DatasetInfo`, `CountResult`). CLI
  behavior is covered by `tests/test_cli.py`.
- A successful `--json -c <command>` emits one compact JSONL result envelope for
  each command that returns a result, in command order. The envelope fields are
  `data`, `result_type`, and `schema_version`; version is integer `1`, and
  result labels are `LoadResult` and `CountResult`.
- For `use <absolute-local-csv-path>` on a two-row CSV with columns `x,y`,
  `LoadResult.data.dataset` contains `columns`, `execution_mode`,
  `label_metadata`, `lazy_engine`, `panel_metadata`, `path`, and `row_count`.
  The two metadata fields and lazy engine are JSON null for this eager load;
  `path` is the supplied path, and each column contains `data_type` and `name`.
- `count` emits `CountResult.data.row_count`. A successful `use` then `count`
  workflow emitted two JSON lines, with no stderr output. The observed example
  used a temporary two-row CSV; DuckDB inferred `BIGINT` for both columns.
- The pinned guide `docs/user-guide/scripting-and-json.md` specifies the same
  JSONL result envelope for batch and scripts. Script output order, nested
  scripts, and partial-result-before-error behavior are tested in
  `tests/test_cli.py`; successful script streaming is outside this Rust slice.

## Rust contract

- Scope this implementation to repeated `-c` batch commands that successfully
  return `use` or `count` results. `exit` has no result and remains accepted.
- Emit one versioned `ResultEnvelope` JSON line for each successful supported
  result, sharing the runtime's typed `ExecutionResult` rather than executing a
  separate reporting query.
- Map Rust `DatasetInfo.source` to the Python JSON field `path`; serialize the
  eager execution mode as `eager`, lazy engine as a lowercase value or null,
  and include `panel_metadata: null` and `label_metadata: null` because those
  metadata families are not present in this runtime result type.
- Before executing any other parsed command in a JSON batch, emit a versioned
  `JsonResultUnsupported` error envelope and return CLI usage status 2. Preserve
  prior result lines from earlier supported commands. This makes the partial
  result-serialization boundary explicit and prevents an unsupported command
  from mutating state or writing files without a result line. This is a Rust
  capability limit, not a claim that Python rejects those commands.
- Existing parse/runtime error envelopes and terminal output behavior remain
  unchanged. JSON success streaming for script files and result types other
  than `LoadResult`/`CountResult` is explicitly deferred.

## Test contract

- Use a temporary two-row CSV and invoke the actual binary with
  `--json -c "use <path>" -c count`; parse each stdout line independently and
  compare the complete semantic envelopes, including null metadata fields,
  result labels, row counts, and column metadata.
- Assert success writes only JSONL to stdout and leaves stderr empty.
- Assert a later unsupported JSON command emits the prior supported results,
  followed by a typed error envelope, exits 2, and does not create an artifact
  for a blocked `export` command.
- Preserve existing parse/execution error envelopes, JSON discovery output, and
  non-JSON batch behavior.
- Use the pinned Python command and docs above as the oracle. Full JSONL parity
  for other result types and scripts is not claimed.

## Implementation mapping

- Native Rust: map `ExecutionResult::Load` and `ExecutionResult::Count` into
  versioned envelopes in the root CLI reporting boundary; reject unsupported
  JSON batch result types before calling `Session::execute`.
- DuckDB: reuse the metadata and count already returned by `Session`; no new
  query or backend path.
- FFI/native optional capabilities: none. No dependency or unsafe-code change.
- Deferred: success-result JSON for other execution variants, JSON script result
  streaming, terminal result rendering, HTML, and MCP rendering.

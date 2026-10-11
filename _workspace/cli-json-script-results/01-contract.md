# CLI JSONL results from `.td` scripts

## Scope and acceptance

Extend the existing bounded JSON result surface so `--json -f <script.td>`
streams successful `use` and `count` results as one compact versioned JSON line
per command. Nested `run` commands preserve depth-first execution order. Script
metadata and echoed commands stay off stdout. If a command cannot be serialized
by the current JSON result boundary, reject it before executing it and emit one
`ScriptError` envelope with the resolved file path and 1-based source line;
successful result lines emitted before that point remain on stdout. Non-JSON
script execution and existing script error exit codes remain unchanged.

The slice does not add new result types, support JSON success results for every
Python script command, or add special handling for `--json -c "run ..."`.

## Python contract

- Authority: Python TabDat commit
  `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
  `601b236788872323af9277d2276a236154a0f129`, `uv.lock` SHA-256
  `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.
  Verified sibling checkout: `../tabdat-explore`; Python 3.13.3, DuckDB 1.5.2.
- `src/tabdat/cli.py` streams result JSON while executing each script command.
  `src/tabdat/script.py` supplies command order and nested-run execution;
  `src/tabdat/formatter.py` defines the envelope. `tests/test_cli.py` covers
  metadata/echo suppression, nested result ordering, source locations, and one
  error envelope for nested failure.
- Successful commands emit their result envelopes immediately in execution
  order. Nested script results appear inline between the enclosing script's
  preceding and following results. JSON mode suppresses script headers and
  command echoes.
- On failure, prior successful JSON lines remain. The final error envelope
  identifies the failing script path and line, stderr retains the human-readable
  diagnostic, and no later command runs. Nested failure reports the nested
  file's location and produces one error envelope.
- The Python script JSON surface supports more result types than this Rust
  slice. In particular, the pinned nested-order test also executes `head 1` and
  expects `PreviewResult`; the Rust slice deliberately remains bounded to its
  already-supported `LoadResult` and `CountResult` types.

## Rust contract

- `Session` remains the owner of script parsing, macro/control-flow handling,
  nested path resolution, command execution, and `ScriptError` source locations.
- Add a callback-enabled script execution entry point while preserving
  `execute_run` as a no-op-callback wrapper. The pre-execution callback sees
  each parsed executable command before effects; it can reject a command with a
  source-located `ScriptError`. The result callback receives each successful
  leaf `ExecutionResult` immediately after execution. Both callbacks propagate
  through nested scripts. Directives, inactive branches, and `exit` do not
  produce results; `run` itself is a container and does not produce a JSON line.
- The CLI preflight accepts only `use`, `count`, `exit`, and nested `run` in
  JSON scripts. The first two use the existing JSON serializers. Unsupported
  commands are stopped before execution, so they cannot mutate session state or
  perform file output without a matching success envelope.
- Output and error envelope shapes remain the same as the prior JSON batch
  slice. This bounded Rust capability is not a claim that Python rejects other
  script commands.

## Test contract

- Use temporary local CSV input with deterministic schema and row count.
- Verify a successful script streams `LoadResult` then `CountResult`, with
  schema version 1, no metadata or echoed commands, and empty stderr.
- Verify nested scripts preserve result order for supported `use`/`count`
  commands, including a result after the nested script returns.
- Verify an unsupported command after a supported result is rejected before
  execution, retains the earlier line, reports the exact resolved path and
  1-based line as one `ScriptError`, and prevents a later supported command.
- Verify nested unsupported-command errors point to the child script and emit
  only one error envelope. Verify ordinary non-JSON script execution remains
  unchanged.
- Run the applicable pinned Python CLI tests for the authoritative script
  behavior, then run focused Rust integration coverage and the locked workspace
  gates. Record which broader Python script result forms are outside this
  Rust slice rather than claiming full JSON script parity.

## Implementation mapping

- Native Rust: add result observation and pre-execution validation hooks at the
  existing runtime script command boundary; serialize the existing typed leaf
  results in `src/json_results.rs`; preserve CLI error formatting and status
  mapping.
- DuckDB: reuse `LoadResult` and `CountResult` produced by the session; no
  additional backend query is introduced.
- FFI/native optional capabilities: none.
- Deferred: preview and other successful result envelopes in scripts, full
  Python JSON script parity, JSON success streaming for `-c "run ..."`, and
  broader JSON result coverage.

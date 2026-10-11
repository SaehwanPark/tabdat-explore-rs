# CLI JSONL `head` and `tail` preview results

## Scope and acceptance

Extend the bounded CLI JSON success surface with typed `PreviewResult` output
for `head` and `tail`, both in repeated `-c` batches and in `.td` scripts,
including nested `run`. Emit one `PreviewResult` envelope per command with the
owned column list and selected rows, preserving column and row order. Keep exact
integer values, fixed-scale decimal strings, null/non-finite handling, and
base64-prefixed binary cells consistent with the pinned Python formatter.

Reject other successful result kinds before execution. Preserve current runtime
preview errors and non-JSON behavior. This does not add serializers for the
remaining `ExecutionResult` variants or new runtime `CellValue` variants.

## Python contract

- Authority: Python TabDat commit
  `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
  `601b236788872323af9277d2276a236154a0f129`, `uv.lock` SHA-256
  `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.
  Verified sibling checkout: `../tabdat-explore`; Python 3.13.3, DuckDB 1.5.2.
- `src/tabdat/models.py` defines `PreviewResult` as ordered columns and rows.
  `src/tabdat/formatter.py::_json_safe_value` recursively converts tuples and
  lists, leaves strings/integers/booleans unchanged, maps non-finite floats to
  null, formats `Decimal` with fixed notation as a string, and represents bytes
  as `base64:<standard-base64>` strings. `format_result_json` emits the result
  type `PreviewResult`.
- `tests/test_cli.py::test_cli_json_nested_scripts_preserve_result_order`
  expects `LoadResult`, nested `CountResult`, then parent `PreviewResult` for
  `use`, nested `run`, and `head 1`. Python JSON output also confirms that
  `head` and `tail` each expose `data.columns` and `data.rows` in result order.
- Numeric values remain JSON numbers where representable in Python (including
  arbitrary-size integers); exact decimals are strings. Missing cells and
  non-finite floats become JSON null. Binary data is base64-prefixed text.

## Rust contract

- The `Session` already returns owned `PreviewResult { columns, rows }` for
  `head` and `tail`; this slice does not issue another query or change session
  state. `tail` uses the row order already selected by the runtime.
- Add `Head` and `Tail` to the JSON batch and script preflight allow-lists.
  Serialize either execution variant with result type `PreviewResult` and data
  fields `columns` and `rows`. `run` remains a container and `exit` emits no
  result. Other commands retain pre-execution rejection.
- Convert the existing `CellValue` variants as follows: `Null` to JSON null;
  booleans, signed/unsigned integers, and finite floats to JSON scalars;
  non-finite floats to null; decimals to fixed-scale strings; text to JSON
  strings; and bytes to `base64:<standard-base64>` strings. Enable
  `serde_json` arbitrary precision so `i128`/`u128` cells are not rounded or
  rejected. Use the already-locked base64 crate as a direct dependency.
- This serializer covers values the current runtime can return through
  `CellValue`; adding temporal and other unsupported cell types is deferred.

## State-transition table

| Path | Active relation / metadata | Result/output | Failure behavior |
|---|---|---|---|
| Successful `head` / `tail` | Preserved by existing read-only preview execution | One `PreviewResult` JSON line | N/A |
| Preview command with no active dataset or a runtime preview error | Unchanged | Prior result lines remain | Existing execution error envelope or source-located `ScriptError` |
| Unsupported successful command in JSON mode | Not executed | Prior result lines remain, then unsupported error | Batch `JsonResultUnsupported`; script `ScriptError` with path/line |

## Test contract

- Verify batch `use`, `head`, and `tail` output `LoadResult`, `PreviewResult`,
  and `PreviewResult`, including exact `columns` and row selection/order.
- Verify a nested script with `use`, child `count`, and parent `head 1` emits
  `LoadResult`, `CountResult`, and `PreviewResult` in order with no metadata or
  command echoes. Compare to the pinned nested-script test.
- Unit-test cell conversion for null, booleans, Unicode text, signed and
  unsigned values beyond 64-bit JSON integer ranges, finite and non-finite
  floats, fixed-scale positive/negative/zero decimals, empty and non-empty
  binary payloads, and empty preview rows.
- Keep unsupported-command preflight coverage and verify other batch/script
  behavior remains unchanged.
- Run the pinned Python CLI preview tests, focused Rust tests, all four locked
  workspace gates, documentation link checks, and `git diff --check`.

## Implementation mapping

- Native Rust: extend `src/json_results.rs` over the typed preview result and
  convert owned `CellValue`s at the CLI JSON boundary; update the CLI allow-list
  messages and help text.
- DuckDB: reuse the preview rows already owned by `Session`; no new query,
  relation, or backend capability.
- Dependency surface: enable `serde_json` arbitrary precision and directly use
  the already-locked base64 crate for exact machine output.
- FFI/native optional capabilities: none. Full JSON serialization, terminal
  table rendering, and other result types remain deferred.

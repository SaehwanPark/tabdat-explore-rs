# CLI JSONL `use` and `count` validation

## Provenance and oracle evidence

- Python oracle: commit `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
  `601b236788872323af9277d2276a236154a0f129`; `uv.lock` SHA-256
  `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.
- The clean sibling checkout and manifest were reverified before the slice.
  Environment: Python 3.13.3, DuckDB 1.5.2.
- Ran the pinned oracle with a temporary two-row CSV through
  `--json -c "use <path>" -c count`. It emitted two compact JSONL lines. The
  first was `LoadResult` with the source path, two `BIGINT` columns, eager mode,
  null label/panel/lazy metadata, and row count 2; the second was
  `CountResult` with `row_count: 2`.
- Focused pinned tests:
  `PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider
  tests/test_cli.py::test_cli_json_script_suppresses_metadata_and_echoes
  tests/test_cli.py::test_cli_json_errors_keep_stderr_and_nonzero_exit` —
  **2 passed**.

## Rust checks

- `cargo test --locked --test cli_json_use_count_contract` — **4 passed**.
- `cargo fmt --all -- --check` — passed.
- `cargo check --locked --workspace --all-targets` — passed.
- `cargo test --locked --workspace --all-targets` — all targets passed.
- `cargo clippy --locked --workspace --all-targets -- -D warnings` — passed.
- `git diff --check` — passed after documentation closeout.
- Relative Markdown links — passed for SPEC, roadmap, and the three slice artifacts.
- GitHub Actions are disabled under ADR 0009. No hosted checks ran for this PR.

The focused test verifies the complete load/count envelope data, clean stdout and
stderr separation, the typed unsupported-result envelope, rejection before an
`export` can create a file, explicit help text, and the existing count execution
error envelope. Full JSON result parity for other runtime results and successful
JSON script streams remain unvalidated and deferred.

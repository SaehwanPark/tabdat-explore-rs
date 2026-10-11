# Migration and validation evidence: JSON preview results

## Python authority

- Repository: sibling checkout `../tabdat-explore`.
- Commit: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`.
- Tree: `601b236788872323af9277d2276a236154a0f129`.
- `uv.lock` SHA-256: `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.
- Runtime: Python 3.13.3 and DuckDB 1.5.2.
- The oracle checkout was clean when checked.

The contract comes from `src/tabdat/models.py` (`PreviewResult`),
`src/tabdat/formatter.py` (`_json_safe_value` and `format_result_json`), and
the JSON CLI tests. The formatter leaves integers, booleans, and strings as
scalars; maps non-finite floats to null; formats `Decimal` in fixed notation;
and emits bytes as `base64:<standard-base64>`. The nested script test verifies
that results are streamed in execution order.

## Oracle checks

Command:

```sh
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_cli.py::test_cli_json_nested_scripts_preserve_result_order \
  tests/test_cli.py::test_cli_json_serializes_binary_table_cells \
  tests/test_cli.py::test_cli_json_preserves_exact_decimal_and_nonfinite_policies
```

Result: **3 passed**. The checks cover nested result ordering, binary cells,
exact decimal formatting, and non-finite float handling. Rust unit tests
separately exercise the complete currently supported `CellValue` mapping,
including `i128::MIN` and `u128::MAX`.

## Rust checks

All four repository baseline checks passed on this slice:

```sh
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo test --locked --workspace --all-targets
cargo clippy --locked --workspace --all-targets -- -D warnings
```

The full workspace test run passed, including two serializer unit tests,
`tests/cli_json_preview_contract.rs` (2 tests), and
`tests/cli_json_use_count_contract.rs` (11 tests). Earlier focused runs of the
same Rust targets also passed. `git diff --check` passed after the documentation
updates.

GitHub Actions were disabled for this repository by PR #158. No hosted status
or GitHub CI result is claimed for PR #161.

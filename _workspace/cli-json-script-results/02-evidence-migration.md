# Migration evidence: CLI JSONL results from `.td` scripts

**Producer:** task owner. **Consumer:** code reviewer and next maintainer.
**State:** complete for the bounded contract. **Rust implementation revision:**
`e9747e7893f1191dfd217598d18330e7bccc3f05` (branch
`feat/cli-json-script-results`); documentation artifacts are added in this
follow-up commit. The implementation tree was clean at the start of evidence
recording.

## Boundary and changed paths

- `crates/tabdat-runtime/src/lib.rs`: add callback-enabled script execution,
  invoke validation before each parsed command, observe successful leaf results,
  and propagate both callbacks through nested scripts. `execute_run` remains a
  no-op-callback wrapper.
- `src/json_results.rs`: accept `use`, `count`, `exit`, and nested `run` in
  scripts while retaining the existing `LoadResult` and `CountResult` serializers.
- `src/cli.rs`: stream JSON lines during script execution and preserve the
  existing source-located error envelope and exit mapping.
- `tests/cli_json_use_count_contract.rs`: add end-to-end success, nested order,
  unsupported-command preflight, source-location, inactive-branch, and non-JSON
  compatibility cases.
- `SPEC.md` and `docs/TABDAT_RUST_PORT_ROADMAP.md`: record the accepted bounded
  slice while leaving full result serialization unchecked.

## Inputs and method

The Python oracle was verified at commit
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, with `uv.lock` SHA-256
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`. The clean
sibling checkout reported Python 3.13.3 and DuckDB 1.5.2. Tests used the existing
environment; no dependency synchronization was performed.

Pinned oracle command, from `../tabdat-explore`:

```sh
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_cli.py::test_cli_json_script_suppresses_metadata_and_echoes \
  tests/test_cli.py::test_cli_json_nested_scripts_preserve_result_order \
  tests/test_cli.py::test_cli_json_script_error_includes_source_location \
  tests/test_cli.py::test_cli_json_nested_script_failure_emits_one_error_envelope
```

Observed: **4 passed**. The Python nested-order test includes `head 1` and
expects `PreviewResult`, confirming that the pinned Python JSON script surface is
broader than this Rust slice.

Focused Rust command, from the Rust repository root:

```sh
cargo test --locked --test cli_json_use_count_contract
```

Observed: **11 passed**. The cases verify compact result order and no extra
stdout/stderr text, nested `use`/`count` result order, source-located failures,
preservation of earlier lines, blocked export side effects, inactive branch
handling, and unchanged non-JSON script output.

Required workspace gates, all passed on the implementation tree:

```sh
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo test --locked --workspace --all-targets
cargo clippy --locked --workspace --all-targets -- -D warnings
```

The final full workspace test command exited 0. Documentation checks passed:
relative Markdown links in `SPEC.md`, the roadmap, and the slice contract all
resolved; `git diff --check origin/main` passed. No skill frontmatter changed.

GitHub Actions remain disabled: `gh api
repos/SaehwanPark/tabdat-explore-rs/actions/permissions` returned
`{"enabled":false,"sha_pinning_required":false}`. No hosted checks are claimed.

## Expected and observed behavior

- A supported script containing `use` then `count` emits `LoadResult` followed
  by `CountResult`, each in schema version 1, with no script header or command
  echo.
- Nested `run` commands stream child results at their execution position. A
  later parent result follows child results.
- Unsupported script commands are rejected before runtime execution. Earlier
  successful result lines remain, and the sole error envelope reports the
  resolved script path and 1-based line. Nested failures report the child path.
- Errors from supported runtime commands also preserve earlier results and use
  the failing script path and line. Commands in inactive branches do not reach
  preflight.

## Scope limit

The Rust success surface is intentionally limited to existing `LoadResult` and
`CountResult` serializers. Commands such as `head` are stopped before effects
and returned as `ScriptError`; this is an explicit capability boundary, not a
claim that the Python oracle rejects them. Full Python JSON script parity and
all other result types remain unimplemented.

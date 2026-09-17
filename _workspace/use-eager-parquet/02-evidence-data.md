# Eager local-Parquet runtime evidence

Status: partial; implementation and local checks are complete on draft PR #22,
while independent review and hosted acceptance remain in progress.

Producer: task owner

Consumer: reviewers and the next maintainer

Boundary: pinned Python `use` execution contract → Rust-owned eager local-Parquet
runtime boundary

Rust implementation revisions: `d2e0b7c` (contract and ADR), `bddb87d` (test
contract and fixture), `57753b5` (runtime implementation), `8ba52f3` (lazy-init,
relation-atomicity, and repeated-load tests), and `aea6728` (Python-compatible
suffix validation order, parser-to-session wiring, and rejection coverage).

Python oracle revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` (tree
`601b236788872323af9277d2276a236154a0f129`), Python 3.13.3. The clean sibling
checkout and `uv.lock` digest remain those recorded in
[`docs/migration/README.md`](../../docs/migration/README.md).

## Inputs and authority

The contract was recovered from the clean pinned sibling checkout
`../tabdat-explore`:

- `src/tabdat/models.py:937-950` (`ColumnInfo`);
- `src/tabdat/models.py:1004-1028` (`DatasetInfo`);
- `src/tabdat/models.py:1119-1126` (`LoadResult`);
- `src/tabdat/executor.py:873-876,1310-1334` (`use` dispatch and session update);
- `src/tabdat/backend.py:165-392` (source loading and eager Parquet staging);
- `src/tabdat/backend.py:394-426` (schema and row-count metadata);
- `src/tabdat/backend.py:3914-3966` (local path expansion, suffix precedence,
  and existence checks);
- `tests/conftest.py:15-34` (synthetic three-row, four-column fixture);
- `tests/test_executor.py:750-763` (eager local-Parquet success);
- `tests/test_executor.py:1043-1059` (failed load preserves active state); and
- `docs/commands/use.md:1-27` (public source and mode syntax).

The focused runtime oracle passed:

```text
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_executor.py -k \
  'test_use_loads_active_dataset or test_failing_lazy_use_preserves_existing_active_dataset'
2 passed, 415 deselected in 1.92s
```

The broader `use` executor subset also passed (`42 passed, 375 deselected`),
covering the surrounding Python source/mode/error matrix. Because the pinned
preservation test is lazy, an explicit eager probe was run separately against a
synthetic three-row fixture:

```text
ERROR ExecutionError use could not read Parquet file: <corrupt.parquet>
BEFORE 3 eager None
AFTER 3
```

The probe confirms that a corrupt eager load preserves the prior active dataset
and that a subsequent count still returns three rows.

The successful fixture reports three rows and four columns ordered `age`,
`bmi`, `sex`, `cost`, with eager mode and no lazy engine. A corrupt Parquet load
raises `use could not read Parquet file: <path>` and leaves the prior active
dataset metadata unchanged. The full pinned parser/script regression also
passed without modifying the oracle:

```text
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py tests/test_script.py
516 passed in 0.69s
```

## Rust implementation and behavior

`crates/tabdat-runtime` is a safe library boundary over a private
`duckdb::Connection`:

- `Session::new()` stores no backend; a supported load initializes DuckDB lazily;
- only `Command::Use` with eager mode, a local existing regular file, and a
  case-insensitive `.parquet` extension is accepted;
- URI sources, lazy mode, `lazy_engine`, delimiter, and header options return a
  typed deterministic error before backend work;
- the adapter sets `preserve_insertion_order`, stages `read_parquet(?)` into a
  temporary table, reads ordered `DESCRIBE` metadata and `COUNT(*)`, then
  replaces the active relation in a transaction;
- session metadata is published only after the relation publish succeeds; and
- no DuckDB, Arrow, raw pointer, or foreign-lifetime type is exposed publicly.

Suffix validation intentionally precedes existence/type checks, matching the
pinned Python resolver: an absent or directory path with an unsupported suffix
reports the unsupported-format error first. Rust does not expand `~`; this is an
explicit caller-resolves-path deferral from Python's `expanduser()` behavior.

The runtime integration tests generate the same three-row fixture through the
pinned DuckDB dependency, assert owned schema/order/count/mode metadata, cover
invalid and out-of-scope requests, exercise a second successful replacement,
and verify corrupt staged reads preserve the prior metadata. Private unit tests
also verify a fresh session has no backend and a failed staged read leaves the
prior active relation queryable.

Deferred are CSV/DTA/Feather/Arrow, URI/network access, lazy plans, named tables,
general relation/query APIs, schema-cache invalidation, transformations,
statistics, labels, `describe`/`count`, CLI/REPL, script execution, and
JSON/MCP/reporting surfaces. This evidence does not establish broad `use`
parity or full DuckDB product adoption.

## Local Rust and policy verification

The targeted runtime checks passed on `aea6728`; the full workspace forms of the
baseline and policy gates must be rerun on the final evidence commit before
promotion. Record the final exact output for all of these gates:

```text
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo test --locked --workspace --all-targets
cargo clippy --locked --workspace --all-targets -- -D warnings
git diff --check
cargo deny check
cargo audit -D warnings
```

The metadata-driven geiger policy loop was run with JSON output for the root
package, `tabdat-language`, and `tabdat-runtime`; all three local scans completed
successfully. The runtime JSON scan contained 169 packages, no packages without
metrics, and no first-party unsafe usage. The first hosted attempt at
`Report unsafe code` failed because plain `cargo geiger` returned nonzero for 33
unscanned dependency assets even though first-party code was clean; this is
tracked as a workflow policy bug, not hidden as a passing gate. CI now captures
the JSON inventory and fails only on first-party unsafe usage.

```text
tabdat-explore-rs  forbids_unsafe=true  unsafe_functions=0  unsafe_exprs=0
tabdat-language    forbids_unsafe=true  unsafe_functions=0  unsafe_exprs=0
tabdat-runtime     forbids_unsafe=true  unsafe_functions=0  unsafe_exprs=0
duckdb             forbids_unsafe=false unsafe_functions=4  unsafe_exprs=983
libduckdb-sys      forbids_unsafe=false unsafe_functions=0  unsafe_exprs=18
```

The transitive DuckDB unsafe inventory is expected and contained behind the
private adapter; it is not treated as proof of FFI safety. The scan reported 33
non-scanned asset/generated files, with no package lacking metrics. `cargo deny`
and `cargo audit` passed locally; the allowed `CDLA-Permissive-2.0` entry remains
an explicit redistribution obligation rather than blanket native-license
approval.

## Native and platform review

The existing
[`docs/feasibility/duckdb.md`](../../docs/feasibility/duckdb.md) and
[`ADR 0003`](../../docs/adr/0003-duckdb-feasibility-spike.md) record the
`duckdb-rs` `1.10505.0` / `libduckdb-sys` `1.10505.0` candidate, bundled linkage,
DuckDB `v1.5.5`, native unsafe inventory, and the CDLA license review. This PR
adds the domain-owned adapter and transaction/state tests but does not broaden
the candidate's support claims. Hosted Linux and local macOS-relevant build
evidence, ownership/threading review, and final license/notice disposition are
required before the ADR is accepted and the PR is merged.

## Hosted acceptance

Draft PR: [#22](https://github.com/SaehwanPark/tabdat-explore-rs/pull/22).
The final head must have the Rust baseline, dependency/unsafe policy, and the
new Linux runtime-boundary workflow green. Existing ReadStat/libgretl workflows
remain path-scoped feasibility checks and are not triggered by this runtime-only
change. Their final run links and the squash-merge SHA will be added here before
this artifact changes to `Status: accepted`.

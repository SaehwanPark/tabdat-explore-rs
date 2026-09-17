# Eager local-Parquet runtime evidence

Status: accepted for bounded evaluation; broad `use` parity and production
DuckDB integration remain deferred after PR #22's green hosted matrix.

Producer: task owner

Consumer: reviewers and the next maintainer

Boundary: pinned Python `use` execution contract → Rust-owned eager local-Parquet
runtime boundary

Rust implementation revisions: `d2e0b7c` (contract and ADR), `bddb87d` (test
contract and fixture), `57753b5` (runtime implementation), `8ba52f3` (lazy-init,
relation-atomicity, and repeated-load tests), and `aea6728` (Python-compatible
suffix validation order, parser-to-session wiring, and rejection coverage).
Commit `cb0e8c3` adds the first-party unsafe-code CI gate, Linux runtime workflow,
and current-state/evidence corrections. Commit `559f293` fixes the workspace
manifest/package mapping in that gate, retains geiger reports for review, and
adds the reproducible eager-failure probe and migration-decision entry.

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

```sh
cd ../tabdat-explore
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync python - <<'PY'
from pathlib import Path
from tempfile import TemporaryDirectory

import duckdb

from tabdat.executor import Executor
from tabdat.models import CountCommand, CountResult, LoadResult, UseCommand

with TemporaryDirectory() as temp:
  root = Path(temp)
  valid = root / "patients.parquet"
  corrupt = root / "corrupt.parquet"
  connection = duckdb.connect(database=":memory:")
  try:
    connection.execute(
      """
      copy (
        select * from (
          values
            (30, 22.5, 'F', 100.0),
            (42, 25.0, 'M', 150.0),
            (54, 27.5, 'F', null)
        ) as patients(age, bmi, sex, cost)
      ) to ? (format parquet)
      """,
      [str(valid)],
    )
  finally:
    connection.close()

  executor = Executor()
  try:
    before = executor.execute(UseCommand(valid))
    assert isinstance(before, LoadResult)
    print("BEFORE", before.dataset.row_count, before.dataset.execution_mode, before.dataset.lazy_engine)
    corrupt.write_text("not parquet")
    try:
      executor.execute(UseCommand(corrupt))
    except Exception as exc:
      print("ERROR", type(exc).__name__, str(exc))
    after = executor.execute(CountCommand())
    assert isinstance(after, CountResult)
    print("AFTER", after.row_count)
  finally:
    executor.close()
PY
```

```text
BEFORE 3 eager None
ERROR ExecutionError use could not read Parquet file: <temporary-directory>/corrupt.parquet
AFTER 3
```

The command is pinned by the surrounding oracle checkout and uses its existing
DuckDB fixture shape. It confirms that a corrupt eager load preserves the prior
active dataset and that a subsequent count still returns three rows. The
temporary-directory component is intentionally normalized in this artifact.

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
Rust's unsupported-suffix and unsupported-configuration diagnostics are
deliberately narrower than Python's broad-format message because those formats,
lazy modes, and options are outside this runtime slice.

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

The targeted runtime checks passed on `aea6728`, and the full workspace gates
passed on the accepted implementation head `559f293`:

```text
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo test --locked --workspace --all-targets
cargo clippy --locked --workspace --all-targets -- -D warnings
git diff --check
cargo deny check
cargo audit -D warnings
```

Observed results:

```text
cargo fmt --all -- --check                                  passed
cargo check --locked --workspace --all-targets              passed
cargo test --locked --workspace --all-targets                passed
  root scaffold: 1; tabdat-language: 28 unit + 17 integration;
  tabdat-runtime: 2 unit + 5 integration
cargo clippy --locked --workspace --all-targets -- -D warnings passed
git diff --check                                            passed
cargo deny check                                            passed
cargo audit -D warnings                                     passed
```

The metadata-driven geiger policy loop was run with JSON output for the root
package, `tabdat-language`, and `tabdat-runtime`; all three local scans completed
successfully. The hosted policy job on `559f293` also completed the same loop and
published per-package totals in its step summary. The runtime JSON scan contained
169 packages, no packages without metrics, and no first-party unsafe usage. The
first hosted attempt at
`Report unsafe code` failed because plain `cargo geiger` returned nonzero for 33
unscanned dependency assets even though first-party code was clean; this is
tracked as a workflow policy bug, not hidden as a passing gate. CI now retains
the JSON reports for the job and fails only on first-party unsafe usage.

```text
tabdat-explore-rs  forbids_unsafe=true  unsafe_functions=0  unsafe_exprs=0
tabdat-language    forbids_unsafe=true  unsafe_functions=0  unsafe_exprs=0
tabdat-runtime     forbids_unsafe=true  unsafe_functions=0  unsafe_exprs=0
duckdb             forbids_unsafe=false unsafe_functions=4  unsafe_exprs=983
libduckdb-sys      forbids_unsafe=false unsafe_functions=0  unsafe_exprs=18
```

Hosted runtime geiger summary (`559f293`):

```text
tabdat-runtime     forbids_unsafe=true  unsafe_functions=0  unsafe_exprs=0
transitive runtime inventory: unsafe_functions=401  unsafe_exprs=23335
unscanned dependency files: 33
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
DuckDB `v1.5.5`, native unsafe inventory, and the CDLA license review. PR #22
adds the domain-owned adapter and transaction/state tests without broadening the
candidate's support claims. Local macOS Apple Silicon evidence is `Darwin 27.0
arm64` with Rust 1.97.1; hosted Linux x86_64 evidence is the passing runtime
workflow linked below. The adapter review records private connection ownership,
no `Send`/`Sync` or panic-catching promises, staged cleanup, transactional
publish, and RAII teardown. The repository remains unpublished, so the bundled
license/notice review is accepted only as an evaluation disposition; production
redistribution still needs an AGPL-compatible notice/package decision.

## Hosted acceptance

PR: [#22](https://github.com/SaehwanPark/tabdat-explore-rs/pull/22), accepted after
all eight required checks passed on implementation head `559f293`:

- [Rust baseline and dependency/unsafe policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35232676933)
  (jobs `105240552352` and `105240552463`);
- [Linux runtime boundary](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35232676906)
  (job `105240552009`);
- [DuckDB feasibility](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35232676753)
  (job `105240552575`);
- [ReadStat feasibility](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35232677233)
  (jobs `105240552909` and `105240553260`); and
- [libgretl feasibility](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35232676886)
  plus [libgretl OLS](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35232676817)
  (jobs `105240552368` and `105240551819`).

The documentation-only acceptance follow-up is kept separate from this
implementation evidence; merge and post-merge checks are recorded in
`04-summary.md`.

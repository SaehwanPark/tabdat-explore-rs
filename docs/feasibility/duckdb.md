# DuckDB Rust feasibility spike

The hosted Linux workflow result cited below is historical. GitHub Actions are
disabled and workflow definitions are removed under
[ADR 0009](../adr/0009-disable-github-actions.md).

- Status: **partial**; prototype accepted for continued evaluation, not production
  integration.
- Candidate: `duckdb-rs` crate `1.10505.0` with `bundled` and `parquet` features.
- Native package observed through the crate: `libduckdb-sys` `1.10505.0`; SQL
  reports DuckDB `v1.5.5`.
- Spike paths: `spikes/duckdb-prototype/` and its committed `Cargo.lock`.
- Decision record: [ADR 0003](../adr/0003-duckdb-feasibility-spike.md).

## Contract and scope

The smallest tested capability is an in-memory connection owning an `active` view:

1. load a local CSV with `read_csv_auto(..., header = true)`;
2. load a local Parquet file with `read_parquet(...)`;
3. count rows repeatedly without replacing the relation;
4. query Arrow `RecordBatch` results and count their owned rows; and
5. copy a tiny CSV relation to Parquet to create the inspectable fixture.

This is a feasibility prototype, not a session API. It does not promise TabDat
commands, ordering/missingness semantics, remote HTTP/S3 behavior, labels, lazy
state metadata, cancellation, or public backend types. Generated SQL literals escape
single quotes in paths; production identifier/query boundaries need a separate
contract and parameter/quoting review.

## API/version evidence

`cargo info` identified the candidate at crates.io with MIT license and Rust 1.85.1
minimum toolchain. The exact dependency and transitive versions are in the spike
lockfile. The wrapper exposes safe `Connection`, `Statement::query_arrow`, and
owned Arrow batches; its implementation uses `libduckdb-sys` internally. We did not
select this version for the product runtime; the spike only makes the candidate
measurable.

## Ownership and safety ledger

| Boundary | Observed evidence | Decision/risk |
| --- | --- | --- |
| Spike facade | `src/lib.rs` has `#![forbid(unsafe_code)]`; `ActiveRelation` owns `Connection`; no raw pointer or foreign type escapes the spike API. | Safe prototype boundary is viable for the tested operations. It is not yet a domain/application facade. |
| DuckDB wrapper | `duckdb-rs` and `libduckdb-sys` are safe-facing crates over native code; Arrow batches are tied to the statement iterator and consumed before it is dropped. | Review upstream ownership/thread guarantees before adding `Send`/`Sync` or long-lived session sharing. |
| Unsafe inventory | `cargo geiger --all-dependencies --all-targets --locked` completed scanning. It reported `0/0` unsafe usage for `tabdat-duckdb-spike`, and `4/12` functions, `983/1374` expressions, `8/8` impls, and `35/40` methods with unsafe usage in `duckdb 1.10505.0`'s dependency subtree. The command exits nonzero because dependencies contain unsafe code; this is an inventory finding, not a clean-safety pass. | Keep native code behind an approved adapter; do not treat a safe wrapper or geiger report as proof of FFI safety. |
| Cleanup | Temporary CSV/Parquet fixtures are removed by the test guard and benchmark; `Connection` owns teardown through its Rust wrapper. | Partial-construction, panic, cancellation, and concurrent access need dedicated adapter tests. |

The geiger scan also warned that some generated/native files were not scanned. That
limitation is preserved rather than hidden.

## Semantic results

On macOS Apple Silicon with the repository-pinned Rust 1.97.1 toolchain:

```text
csv_relation_supports_repeated_count_and_arrow_results ... ok
parquet_relation_supports_scan_and_arrow_results ... ok
2 passed; 0 failed
```

The tests observed two rows for CSV and Parquet, repeated `COUNT(*) = 2`, and two
Arrow result rows. `cargo audit -D warnings` passed for the 168-package spike lock;
`cargo deny check` passed advisories, licenses, bans, and sources with duplicate
versions reported as warnings (`base64`, `getrandom`, `hashbrown`, `linux-raw-sys`,
`r-efi`, `rustix`, `syn`, and `windows-sys`). The bundled dependency's
`CDLA-Permissive-2.0` license is explicitly recorded in `deny.toml`; this is not a
blanket native redistribution approval.

## Platform/build matrix

| Platform | Toolchain | Result | Scope |
| --- | --- | --- | --- |
| macOS Apple Silicon | Rust 1.97.1, DuckDB candidate above | Pass | `cargo fmt`, locked test, locked release build, benchmark; local only |
| Linux x86_64 | Repository-pinned toolchain | Pass ([PR #6 workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/34194987287)) | Debug test/build path passed; release performance and audit/unsafe inventory remain local-only in this slice |
| Other targets | — | Not tested | Out of scope for this slice |

The original run deferred remote HTTP/S3 and system-vs-bundled linkage. The
later HTTP/unsigned-S3 section closes only the bounded remote-read spike.

## Performance measurements

Release build, compilation excluded, local macOS Apple Silicon, one run, two-row
CSV fixture, one in-memory connection:

```text
duckdb_version=v1.5.5
rows=2
cold_open_us=16146
first_query_us=2824
repeated_100_queries_us=95325
```

These are orientation measurements, not release gates: they include local filesystem
and process conditions, do not establish cold process startup, and are not compared
against a baseline or representative dataset. Repeat on named hardware and realistic
fixtures before adopting latency targets.

## Decision and remaining risks

**Recommendation: continue as an isolated feasibility prototype; defer production
adoption.** The candidate can load local CSV/Parquet and produce Arrow batches through
a safe-facing API with small owned results. Adoption remains blocked on:

- a supported target/linkage decision beyond the tested Linux/macOS configurations;
- upstream thread/concurrency and allocator/teardown review;
- representative schema, null/missingness, ordering, and large-result semantics;
- remote/HTTPFS/S3 requirements if initial parity needs them;
- measured dependency/build/RSS costs and system-vs-bundled packaging choice;
- AGPL-compatible redistribution/notices review for the bundled native artifact; and
- a domain-owned adapter and failure-atomic session lifecycle.

No DuckDB runtime dependency was added to the root package. The next backend slice
must either close these gaps or record a reasoned defer/reject decision; it must not
turn this prototype into implicit product support.

## HTTP and unsigned S3 spike (2026-10-11)

Status: **pass for the bounded remote-read experiment**, not product remote
support. `src/http_tests.rs` hosts synthetic Parquet on an agent-owned loopback
server with HEAD, GET, byte ranges, and 404 responses. No third-party data,
cloud bucket, access keys, or public fixture endpoint is used.
`ActiveRelation::from_http_parquet` explicitly installs/loads signed `httpfs`;
installation can contact DuckDB's extension service. The remote test is ignored
by default, so ordinary spike tests do not silently request that capability.

Observed locally with pinned DuckDB `v1.5.5`, macOS 27.0.1 arm64:

```text
duckdb=v1.5.5 rows=4096 ranges=21 missing_object=recovered s3_unsigned=passed
```

Exact checks: repeated count 4,096, id 2 has SQL NULL label, final label
`row-4096`, Arrow rows 4,096, at least one byte-range response, failure reading
an absent object, and successful count after that failure. A separate connection
reads `s3://bucket/fixture.parquet` using an explicit loopback endpoint,
path-style URLs, unsigned access, and disabled TLS; count, id sum, and non-NULL
label count agree exactly (4,096 / 8,390,656 / 4,095). This answers whether the
pinned extension's S3 route works, not whether AWS signing, credentials/IAM,
virtual-host addressing, HTTPS/certificates, proxies, or cancellation work.
The pinned Python contract includes S3 Parquet, so those broader requirements
cannot be silently declared out of scope for eventual parity.

Reproduce (extension service/cache access may be needed):

```sh
CARGO_TARGET_DIR=target/duckdb-spike cargo test \
  --manifest-path spikes/duckdb-prototype/Cargo.toml --locked \
  http_parquet -- --ignored --nocapture
```

Normal local spike format/check/test/Clippy also pass (two local tests; the
remote test is opt-in). Root locked checks pass separately. Three explicit
owner review passes covered query quoting/network scope, server lifecycle/range
handling, and values/error assertions/documentation scope; no actionable blocker
remains in this experiment. No independent reviewer runtime was available.
Changed-document links and whitespace pass. No native unsafety was added and
no dependency/lockfile changed. No new Linux remote run, TLS fixture, public AWS
run, or cold network performance measurement is claimed. Production extension
bundling/offline installation remains a distribution decision.

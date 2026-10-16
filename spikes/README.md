# Native feasibility prototypes

These isolated Cargo projects are excluded from the production workspace. A
prototype build or reference result is not evidence of an installed TabDat
capability or general platform/distribution support.

| Directory | Evidence |
| --- | --- |
| `duckdb-prototype/` | [DuckDB feasibility](../docs/feasibility/duckdb.md) |
| `readstat-prototype/` | [ReadStat feasibility](../docs/feasibility/readstat.md) |
| `gretl-prototype/` | [libgretl feasibility](../docs/feasibility/libgretl.md) and related estimator investigations |
| `native-svg-prototype/` | [Native SVG feasibility](../docs/feasibility/native-svg.md) |

Follow the relevant investigation's explicit manifest, features, native dependency
and licensing instructions when reproducing a spike. Ordinary production checks
are in [CONTRIBUTING.md](../CONTRIBUTING.md); supported runtime behavior is in
[SPEC.md](../SPEC.md).

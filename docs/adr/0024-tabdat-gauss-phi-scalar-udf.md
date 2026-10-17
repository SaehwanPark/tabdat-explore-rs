# ADR 0024: tabdat_gauss_phi scalar UDF for bit-consistent SQL probabilities

- Status: Accepted
- Date: 2026-10-17
- Scope: DuckDB data layer; not a claim of builtin-function parity

## Cause and decision

Probit prediction columns probabilize a linear index inside SQL, like the
logistic prediction columns already do. DuckDB's builtin `normal_cdf` cannot
serve directly: the capability contract for binary results pins probabilities
to `tabdat_stats::normal_cdf` — R-validated to machine precision, exactly
mapping saturated tails onto 0/1, and keeping `p + q` exactly one through
complementary half-tails. DuckDB's builtin may differ in the last ulp and does
not document our saturation convention, so a `predict` column could disagree
with the same fit's published probabilities elsewhere.

`tabdat-data` therefore registers `tabdat_gauss_phi(double) -> double` on every
backend connection, delegating each call to `tabdat_stats::normal_cdf`. The
data crate gains a dependency on the pure statistics crate, which stays acyclic
because tabdat-stats has no data-layer dependency.

## Safety and implementation choices

The duckdb crate's data-chunk scalar interface reaches raw vectors through
`unsafe` accessors, which would end the workspace's all-crates
`unsafe_code`-forbidden guarantee. The registered function uses the
duckdb-crate arrow scalar interface (`vscalar` + `vscalar-arrow` features)
instead: typed arrow arrays make the entire first-party implementation safe,
and `duckdb`'s own re-exported arrow keeps the array version aligned with the
bundled engine. No first-party `unsafe` is added. Panics are contained by the
duckdb crate's registration helpers, but the implementation returns explicit
errors rather than panicking on unexpected types.

Null maps to null. Integers coerce per DuckDB's usual exact-signature rules.
The function is non-volatile, so constant folding is allowed.

## Consequences and validation

Unit tests assert bit-equality with the engine function across a
multi-chunk `range()` and the exact `p + q = 1` sum, saturation to exact
0/1, null passthrough, and integer coercion. The UDF exists only on backend
connections; other engines remain unsupported as before, and builtin function
availability elsewhere is unchanged. Future prediction-link variants may add
signatures, but any change to the delegated function is a statistics-crate
validation change, not a data-layer convention.

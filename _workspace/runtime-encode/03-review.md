# Runtime encode review

Status: accepted after green PR-head workflows and squash merge as
[`af3e3b2`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/af3e3b2726778af5c5f3b5c13c4ba84e5291da61).

Review basis: [01-contract.md](01-contract.md), the pinned oracle evidence in
[02-evidence-migration.md](02-evidence-migration.md), and the branch diff from
`main` through [`8c80894`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/8c80894b21042479be1b250fbc84c066579fc0d1).

## Findings

No blocking correctness, security, performance, or maintainability findings
remain for the bounded contract.

### Correctness

- Parser ownership is typed and preserves source/target spelling plus the
  optional label name.
- Source existence, target collision, and string-domain validation happen
  before value discovery or relation staging.
- Distinct nonmissing values are ordered by DuckDB, mapped from one, and
  compiled into an ordered quoted CASE expression; NULL rows remain NULL.
- The backend stages a complete projection, checks ordered schema names and row
  count, and publishes only after those checks succeed.
- The validation-order hardening in `1f67205` preserves the oracle's collision
  precedence and makes code-index conversion non-panicking.

### Security and boundary safety

- No unsafe Rust, FFI, new dependency, or native backend surface was added.
- Identifiers use the shared double-quote escaping boundary and source values
  use SQL-literal escaping; the generated SQL does not interpolate raw names or
  values.
- The optional label request returns a typed runtime error rather than silently
  dropping metadata semantics.

### State and failure behavior

- No-active requests return before backend initialization.
- Validation, distinct lookup, SQL staging, schema/count inspection, and
  publication failures leave the active relation and published metadata
  unchanged.
- Focused tests exercise a malformed target that reaches the backend failure
  mapping and verify the prior relation remains readable.

### Performance and scope

The implementation intentionally follows the bounded oracle shape: one ordered
distinct lookup followed by one staged CASE projection. It makes no claim about
large-cardinality encode performance, lazy plans, label metadata, or broad
Python parity; those need separate evidence.

## Disposition

The implementation is accepted for the bounded eager local-Parquet `encode`
slice. The label option remains parsed but explicitly unsupported in this
runtime until the label metadata contract is implemented.

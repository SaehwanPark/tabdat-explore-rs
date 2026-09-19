# Runtime `gsort` review record

Status: complete at final PR head `04b0d01`; no actionable findings remain.
Hosted acceptance passed before squash merge as PR #51 (`c06ed5a`).

The review used three independent passes over the contract, pinned oracle
evidence, implementation diff, focused tests, and policy results.

## Pass 1: contract and parity

- Compared the Rust command dispatch, key extraction, validation errors,
  per-key direction, NULL placement, and stable ordinal tie-breaker with the
  pinned Python parser, executor, backend, and test paths listed in
  `01-contract.md`.
- Confirmed optional `+`/no-prefix ascending and `-` descending semantics are
  already owned by the parser and are passed as typed booleans, not interpolated
  user strings.
- Confirmed duplicate keys remain legal, exact identifier spelling is retained,
  and quoted identifiers can include embedded double quotes.
- No parity finding remains within the bounded eager scope.

## Pass 2: data, SQL, and state transitions

- Traced `Session::execute` through no-active validation, empty-key and
  unknown-variable validation, directed SQL staging, staged schema/count
  inspection, transaction publication, and delayed `active_dataset` replacement.
- Confirmed `ASC`/`DESC` and `NULLS LAST` are selected only from typed direction
  booleans, while identifiers pass through the existing quote helper.
- Confirmed the private ordinal is collision-safe under DuckDB's case-insensitive
  identifier matching, remains the final ascending tie-breaker, and is excluded
  before publication.
- Confirmed direction-count mismatch fails before staging and stage/inspection/
  publication failures preserve the prior metadata and relation.
- No actionable data-semantics, atomicity, or SQL-injection finding remains.

## Pass 3: regression, compatibility, and policy

- Confirmed `gsort` no longer falls through to `UnsupportedCommand`, while lazy,
  materialized, panel/label, output, and broad transform surfaces remain
  explicitly deferred.
- Confirmed no dependency, lockfile, native backend, FFI, or unsafe-code
  changes were introduced.
- Confirmed focused tests, the full locked workspace checks, advisory/license
  checks, and metadata-driven unsafe inventory pass locally.
- No actionable regression, compatibility, security, or maintainability
  finding remains.

## Disposition

No severity-ranked findings remain. The PR passed final-head checks and was
squash-merged after the review disposition was recorded.

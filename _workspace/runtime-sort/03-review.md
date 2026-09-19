# Runtime `sort` review record

Status: complete at final PR head `55e1fc6`; no actionable findings remain.
Hosted acceptance passed before squash merge as PR #50 (`f33987a`).

The review used three independent passes over the contract, pinned oracle
evidence, implementation diff, focused tests, and policy results.

## Pass 1: contract and parity

- Compared the Rust dispatch, validation errors, stable ordering, and NULL
  placement with the pinned Python executor/backend paths listed in
  `01-contract.md`.
- Confirmed requested keys are validated in input order, duplicate keys remain
  legal, and the parser remains authoritative for syntax.
- Found one low-severity collision risk: a user column whose spelling differs
  only by ASCII case from the private ordinal could collide in DuckDB. The
  backend now checks ordinal candidates case-insensitively, and the quoted
  regression fixture covers the uppercase collision (`7e0abb6`).

## Pass 2: data, SQL, and state transitions

- Traced `Session::execute` through pre-stage validation, private ordinal
  staging, quoted `ASC NULLS LAST` ordering, staged schema/count inspection,
  transaction publication, and delayed `active_dataset` replacement.
- Confirmed the final ordinal is the source insertion order and therefore
  preserves complete ties; it is excluded before publication.
- Confirmed schema and row-count checks reject mismatched private relations and
  preserve the prior metadata and relation on failure.
- Confirmed user identifiers are quoted through the existing escaping helper,
  including embedded double quotes.
- No actionable data-semantics, atomicity, or SQL-injection finding remains.

## Pass 3: regression, compatibility, and policy

- Confirmed `sort` no longer falls through to `UnsupportedCommand`, while
  `gsort` remains explicitly deferred.
- Confirmed no dependency, lockfile, native backend, FFI, or unsafe-code
  changes were introduced.
- Confirmed the focused suite, full locked workspace checks, advisory/license
  checks, and metadata-driven unsafe inventory pass locally.
- No actionable regression, compatibility, security, or maintainability
  finding remains.

## Disposition

The one review finding was fixed and regression-tested in `7e0abb6`. No
severity-ranked findings remain. The PR passed final-head checks and was
squash-merged after the review disposition was recorded.

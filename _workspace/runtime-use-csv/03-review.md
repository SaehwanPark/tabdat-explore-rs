# Bounded eager-runtime CSV `use` review

Reviewer: independent read-only review agent
Scope: PR #74 implementation and focused contract tests through the
compatibility/error-boundary follow-up.

## Findings and resolution

The first review identified one existing-fixture failure and three lower-risk
compatibility/coverage issues:

- the old text fixture (`"not csv"`) is valid one-column CSV to DuckDB; it was
  replaced with invalid UTF-8 and a dedicated `CsvRead` assertion;
- the shared Parquet transaction diagnostic was restored, with a separate
  `CsvTransaction` error for CSV publication failures;
- Parquet delimiter/header options are rejected before filesystem validation,
  preserving the previous error precedence; and
- focused coverage now includes an unsupported suffix, empty input, all prior
  replacement rows, and label metadata preservation.

The reviewer found the core CSV implementation structurally sound: path,
delimiter, and header values are bound; staging is cleaned on every failure;
publication remains transactional; and active metadata/labels are changed only
after successful publication. No security, ownership, or semantic finding
remains after the follow-up.

## Review conclusion

No P0/P1 or lower-severity implementation findings remain. The focused CSV
suite passes `6 passed`; the existing `use_contract` suite passes `65 passed`.
The reviewer did not authorize claims beyond the bounded local eager CSV
contract.

## Evidence reviewed

- pinned Python authority and focused execution probe;
- `01-contract.md` and `02-evidence-migration.md`;
- `crates/tabdat-runtime/src/lib.rs`;
- `crates/tabdat-runtime/tests/use_csv_contract.rs` and the updated
  `use_contract.rs`; and
- local locked workspace, policy, and diff checks.

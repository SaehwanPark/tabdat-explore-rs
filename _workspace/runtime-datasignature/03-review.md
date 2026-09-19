# Independent review: bounded eager-runtime `datasignature`

Status: accepted after correction

Reviewer: independent runtime reviewer

The independent review initially identified three medium parity risks:

- nanosecond timestamps were truncated to microseconds;
- nested list/map/struct timestamps lost timezone metadata; and
- interval values returned a backend failure instead of the oracle's recursive
  three-integer list representation.

The implementation corrected all three. Direct `Value::Union` remains an
explicit bounded-scope deferral because DuckDB local-Parquet round-trips union
columns as structs. The focused contract test now compares independent oracle
digests for nanosecond, nested list/struct/map timestamp, and interval fixtures;
the standard, complex, and empty fixtures remain covered as well. No actionable
correctness, state-preservation, safety, or maintainability findings remain for
the bounded eager slice.

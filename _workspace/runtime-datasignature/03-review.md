# Independent review: bounded eager-runtime `datasignature`

Status: accepted after final correction

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
the standard, complex, empty, temporal-map-key, bare-field, and escaped-field
fixtures remain covered as well. A final low-frequency finding about doubled
quotes in nested STRUCT names was fixed in `22d6b36` by making the quote-aware
splitter and field parser consume escaped quotes and adding the exact oracle
digest `5b331a8cd85c3a489eb42e9b69d9937f930ff05b0875873ea5c691a28870fd54`.
No actionable correctness, state-preservation, safety, or maintainability
findings remain for the bounded eager slice.

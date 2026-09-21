# Bounded eager-runtime CSV `use` closeout

Status: accepted, merged, and verified on `main`.

## Delivered slice

PR [#74](https://github.com/SaehwanPark/tabdat-explore-rs/pull/74) adds eager
local `.csv` input to the existing library-only `Command::Use` boundary. The
case-insensitive suffix is validated before backend initialization; optional
delimiter and header values are bound through DuckDB's `read_csv_auto`; rows
are staged and inspected before atomic publication; and typed read/schema/count/
publication errors preserve the prior active relation and metadata. Six focused
CSV tests cover defaults, options, quoting and NULLs, uppercase extensions,
empty/header-only input, and failed replacement state preservation.

## Durable revisions

- contract checkpoint: `f0aa129`;
- initial implementation: `de98fef`;
- compatibility/error-boundary fixes: `dcd33cf`;
- final compatibility coverage and evidence: `2dcc9ea`;
- squash merge: `7a5b8d4`.

## Validation

- pinned Python ingestion probe: `1 passed, 416 deselected`;
- focused Rust CSV suite: `6 passed`;
- existing Rust `use_contract` suite: `65 passed`;
- locked workspace format/check/test/Clippy and `git diff --check`: passed;
- `cargo deny`, `cargo audit -D warnings`, and the metadata-driven unsafe-code
  inventory: passed locally;
- independent review: no actionable findings;
- exact-head PR workflows on `2dcc9ea`: [CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35562235760)
  and [tabdat-runtime](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35562235734)
  passed; and
- merge-head workflows on `7a5b8d4`: [CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35563640913)
  and [tabdat-runtime](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35563640908)
  passed.

## Explicit deferrals

Remote or URI sources, lazy/materialized execution, Feather/Arrow/DTA input,
`~` expansion and broader path normalization, metadata serialization, atomic
temporary-file replacement, named-table/session generalization, and CLI,
JSON, or MCP surfaces remain deferred.

# Internal domain JSON v1 prototype

This is an **internal typed migration format**, not the Python-compatible CLI JSON.
[ADR 0022](../adr/0022-domain-state-and-workspace-boundaries.md) records the separation.
The machine-readable [envelope schema prototype](domain-json-v1.schema.json)
validates version/shape and includes canonical emitted limits, panel fields and
count results; typed Rust decoding additionally enforces payload invariants.
`tabdat_domain::DomainEnvelope<T>` encodes `{ "schema_version": 1, "data": T }`;
the version is mandatory and a different version or unknown envelope field fails
decoding. Use the typed Rust decoder for payload validation, not a generic map.
Changing a tag/field/value encoding requires an explicitly reviewed migration.

| Contract | v1 shape | Validation/evidence |
| --- | --- | --- |
| Command / expression / recode / result unions | `{"kind":"snake_case_variant","payload":...}`; unit variants omit payload | serde typed decode; representative direct/nested command roundtrips |
| Finite unit choices | snake_case string | enum decoding rejects unknown variants |
| RowLimit | canonical ASCII decimal string | constructor and decoder reject signs, whitespace, empty/nondecimal strings; no bounded-integer conversion |
| PanelMetadata | `{"id_variable":"id","time_variable":"time"}` | nonempty, distinct keys, including during decode |
| CellValue | tagged scalar union | variant distinguishes NULL, boolean, integer, float, decimal, text and bytes |
| Signed/unsigned 128-bit scalar integers | decimal string payload | exact range-checked parse; extrema roundtrip without JSON-number loss |
| Decimal | width, scale, decimal string scaled value | exact payload roundtrip; backend validates width/scale semantics |
| Scalar float | text payload | preserves negative zero, infinity and NaN as values rather than NULL (NaN bit payload is not preserved) |
| Bytes | array of u8 | typed range checks; not the CLI's base64 encoding |
| DatasetMetadata | source union; optional row count, panel and labels | unknown lazy counts represented by null; no lazy engine execution implied |
| ExecutionState | optional dataset/family, ordered named-table map, labels/panel and operation/materialization metadata | backend-free default and runtime snapshot tests |
| EstimationState | `{"family":"linear", "state":...}` or another finite family tag | one optional most-recent model, not nullable per-family slots |
| Diagnostic | finite category, message, optional path/line | located typed-error conversion and envelope roundtrip |

The NLS slice adds `EstimationState::NonlinearFit` (`family: nonlinear_fit`) and
`ExecutionResult::Nl` (`kind: nl`) with the owned finite `NlsResult` payload. The
legacy `Nonlinear(ModelState)` retains its unchanged `nonlinear` tag and payload;
these are not aliases and both map to the nonlinear family without granting
linear-model post-estimation. The distinct encoding follows the reviewed additive
Poisson/NB2 precedent; the high-risk NLS plan review specifically required retaining
the old tag. `state_wire::legacy_nonlinear_and_owned_nls_keep_distinct_roundtrippable_tags`
checks old/new roundtrips and the new result tag. Other v1 encodings are unchanged.

Sample command:

```json
{"schema_version":1,"data":{"kind":"head","payload":{"limit":"5"}}}
```

Sample owned result:

```json
{"schema_version":1,"data":{"kind":"count","payload":{"row_count":7}}}
```

`DatasetInfo` is the existing eager compatibility record (`source`, measured
`row_count`, ordered `columns`, execution mode and lazy engine), not the general
Python `DatasetInfo` JSON shape. The CLI still explicitly maps it to Python's
`path`/panel/label field set and uses numeric previews, string decimals and base64.
Stats records and ordinary optional/result f64 fields retain their existing serde
contracts; their nonfinite values are not guaranteed to roundtrip. The scalar cell
contract is lossless except NaN bit payloads. Filesystem PathBuf serialization can
reject non-UTF8 paths, rather than silently normalizing their bytes.

Decoding does not establish all semantic invariants or grant execution authority:
backend availability, schema/model compatibility, complete samples, statistical
validity, SQL safety policy and filesystem permissions remain execution checks.
This prototype is stable for the committed migration fixtures; it is not a complete
JSON Schema for unimplemented commands or a public transport protocol promise.

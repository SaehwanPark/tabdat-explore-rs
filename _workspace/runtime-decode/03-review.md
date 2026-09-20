# Runtime decode review

Status: local review checkpoint; hosted acceptance is pending for PR [#54](https://github.com/SaehwanPark/tabdat-explore-rs/pull/54).

Review basis: [01-contract.md](01-contract.md), the pinned oracle evidence in
[02-evidence-migration.md](02-evidence-migration.md), and the branch diff from
`main` through [`73be75e`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/73be75e).

## Findings

No blocking correctness, security, performance, or maintainability findings
remain for the bounded contract at this checkpoint.

### Correctness

- Parser ownership is typed and preserves source/target spelling, quoted
  identifiers, required `generate`, duplicate-option, and unsupported-option
  diagnostics.
- Encode records its code-to-text map only after the staged relation publishes;
  decode reads a cloned map before backend work and leaves it available for a
  retry after a failed publication.
- Source/target/type validation precedes staging, and the backend checks ordered
  schema and row count before publishing.
- NULL and unmapped code paths compile to the SQL NULL branch; empty mappings
  use a typed `VARCHAR` NULL expression.
- Rename and projection update the private map, while replacement and in-place
  recode invalidate mappings whose numeric values may have changed.

### Security and boundary safety

- No unsafe Rust, FFI, new dependency, or native backend surface was added.
- Identifiers use the shared double-quote escaping boundary and decoded labels
  use the existing SQL string-literal escaping helper.
- The runtime does not claim arbitrary value-label metadata: the `label()`
  option remains an explicit typed encode error and the decode map is private
  session provenance.

### State and failure behavior

- No-active decode returns before backend initialization.
- Missing-map, source/type/target validation, staging, schema/count, and
  publication failures leave the active relation and decode map unchanged.
- Successful `use` clears prior encode provenance; successful projection,
  rename, and value-changing transforms reconcile it with the new relation.

### Scope and residual risk

The implementation deliberately does not add the general label metadata model
needed for arbitrary attached sets, DTA import, variable labels, or the
`label()` command. It therefore does not claim full Python decode parity or
public label metadata serialization. Those are the next label/backend slices.

## Disposition

The implementation is locally accepted for the bounded eager local-Parquet
`encode` → `decode` session round trip, subject to green PR-head and merge-head
hosted workflows.

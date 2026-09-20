# Bounded eager-runtime session-local `label` contract

Status: proposed at the WIP contract checkpoint on branch
`feat/runtime-label`.

Producer: task owner, using `tabdat-migration` and `tabdat-data-semantics`.
Consumer: the bounded eager language/runtime implementation and focused review.

## Scope

Implement the non-I/O session-local forms of the Python label family against an
active eager local-Parquet session:

```text
label variable <varname> "text"
label variable <varname>, clear
label define <lblname> <value> "text" ... [, replace]
label values <varname> <lblname>
label values <varname>, clear
label list [<lblname> ...]
label drop <lblname> ...
```

The slice deliberately excludes `label save` and `label use`; deterministic
JSON persistence is a later I/O boundary. It also does not claim that labels
are rendered by `describe`, `codebook`, `tabulate`, CLI, JSON, or MCP surfaces.

The Rust session will own normalized `LabelMetadata` containing:

- variable-name to display-text labels;
- named value-label sets whose values are integers, numeric text, or strings;
  and
- variable-name to value-label-set attachments.

`label list` returns the current metadata without changing it. `label define`
is atomic and requires `, replace` when replacing an existing set. `label drop`
removes the named sets and their attachments. All mutating label commands update
session state only after validation succeeds.

The existing encode/decode boundary is extended in the same slice: ordinary
encode continues to create a generated value-label set, `encode ..., label(...)`
selects its set name, and decode consumes an attached integer value-label set.
Source variable labels are copied to generated encode/decode targets where the
oracle does so. `use` clears label metadata; rename and surviving-column
projections reconcile variable labels and attachments; value-changing replace
and in-place recode invalidate affected value-label attachments.

## Python contract

Pinned authority:

- repository: `https://github.com/SaehwanPark/tabdat-explore.git`;
- revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`;
- tree: `601b236788872323af9277d2276a236154a0f129`;
- Python: `3.13.3` in the recorded oracle environment; and
- `uv.lock` SHA-256: `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.

Authoritative paths for this slice:

- `docs/commands/label.md` — public syntax and persistence deferral;
- `src/tabdat/models.py:350-363` — `LabelCommand`, value-label, and metadata
  models;
- `src/tabdat/parser.py:1359-1450` — label action and value parsing;
- `src/tabdat/executor.py:1555-1761` — label state transitions;
- `src/tabdat/executor.py:10299-10405` — metadata normalization and
  transform reconciliation; and
- `tests/test_labels.py` and `tests/test_encode_decode.py` — focused parser,
  metadata, transform, and encode/decode behavior.

The isolated oracle checkout at
`C:\Users\saehwan\repos\tabdat-python-oracle` is clean at the pinned
revision. Focused recovery reported:

```text
uv run --no-sync pytest -q -p no:cacheprovider tests/test_labels.py
6 passed in 0.55s

uv run --no-sync pytest -q -p no:cacheprovider tests/test_encode_decode.py
6 passed in 0.86s
```

The oracle's JSON dictionary persistence, imported DTA labels, inspection
formatting, and CLI/JSON/MCP result surfaces are evidence for later slices, not
claims of this bounded Rust contract.

## Bounded Rust contract

Add an owned typed label command family to `tabdat-language`, preserving quoted
strings, quoted identifiers, numeric signs, duplicate-value validation, and
bounded option diagnostics. Add owned runtime metadata and `LabelResult` data;
all result data must remain independent of DuckDB handles and lifetimes.

The runtime must:

1. return `NoActiveDataset { command: "label" }` before backend work;
2. validate variables and label-set references before changing session state;
3. preserve the prior active relation and metadata on every validation or
   backend failure;
4. normalize metadata deterministically by variable/set name;
5. publish encode relation changes before publishing their generated label set;
6. use the attached integer mapping for decode and reject non-integer label
   values for that conversion; and
7. reconcile metadata across successful `use`, rename, projections, replace,
   recode, encode, and decode transitions without exposing general backend
   metadata.

The parser owns lexical values and exact quoted spelling. The runtime owns
metadata invariants, relation/type checks, SQL quoting, and atomic state
publication. No new dependency, native backend, FFI, unsafe code, or ADR
decision is expected.

## State-transition contract

| Situation | Active relation | Session label metadata | Backend initialization |
| --- | --- | --- | --- |
| No active dataset | unchanged | unchanged | must not occur |
| Label validation failure | unchanged | unchanged | already-loaded backend only |
| Successful metadata-only label mutation | unchanged | replaced atomically | already-loaded backend only |
| Encode relation/publish failure | unchanged | unchanged | already-loaded backend only |
| Successful encode/decode or schema projection | updated atomically | reconciled with surviving columns | already-loaded backend |
| Successful `use` | newly loaded relation | cleared | initialized as needed |

## Test contract

Focused Rust coverage must include:

- parser-produced variable, define, values, list, and drop forms plus exact
  bounded diagnostics;
- variable-label set/clear, named value-label definition/replacement,
  attachment/clear, list filtering, and drop behavior;
- unknown-variable, unknown-set, duplicate-value, and duplicate-definition
  failures that preserve metadata atomically;
- encode with default and explicit label names, decode through an attached
  integer set, and source variable-label propagation;
- use/rename/keep/drop/select/replace/recode metadata reconciliation;
- quoted identifiers, quoted labels, NULLs, and empty relations where the
  existing eager runtime permits them; and
- a retry after a relation publication failure with both relation and metadata
  unchanged.

The contract intentionally leaves JSON label dictionaries, DTA ingestion,
inspection/reporting label rendering, lazy/materialized execution, panel
metadata, last-operation state, formatting, CLI, JSON/MCP surfaces, and broad
transform sequencing deferred.

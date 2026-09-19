# Bounded eager-runtime `generate` contract

Status: contract recovered; bounded implementation is in progress on the
`feat/runtime-generate` draft branch.

Producer: task owner, using `tabdat-migration`, `tabdat-data-semantics`, and
`simple-code-writer`, with independent Python-contract and Rust-boundary
reconnaissance.
Consumer: the bounded eager runtime implementation and its focused review.

## Scope

This loop executes the already-parsed form
`generate <target> = <expression>` against the active eager local-Parquet
DuckDB relation. The first runtime boundary intentionally supports only:

- numeric identifiers and validated numeric literals;
- unary numeric negation;
- `+`, `-`, `*`, and `/`, with the existing checked numeric SQL compiler;
- quoted/backtick identifiers and targets, including embedded double quotes;
- appending the generated column after the existing schema while preserving
  source order, row order, row count, NULL propagation, source metadata, and
  eager execution metadata;
- staged publication through the existing `__tabdat_next` transaction path.

Validation happens before staging. A failed validation, compilation, schema or
row-count inspection, SQL stage, or publication leaves both the published
`DatasetInfo` and private `__tabdat_active` relation unchanged.

The parser’s richer `GenerateExpression` remains intact. Runtime rejects the
parsed forms outside this slice explicitly rather than treating syntax support
as execution parity.

## Python contract

Pinned authority:

- repository: `/Volumes/research/gitrepo/tabdat-explore`;
- revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`;
- tree: `601b236788872323af9277d2276a236154a0f129`;
- Python: `3.13.3`;
- `uv.lock` SHA-256:
  `0f0e1dedbff49b4c77b470a75510b8809436c7d3dc77d1eac372ab9c7264d239`.

Authority paths:

- `src/tabdat/models.py:290-292` (`GenerateCommand`);
- `src/tabdat/parser.py:653-658,3327-3484` (command and expression parsing);
- `src/tabdat/executor.py:1427-1437` (validation and mutation boundary);
- `src/tabdat/backend.py:1397-1408,2303-2395,2506-2629` (target checks,
  expression compilation, and generated-column staging);
- `tests/test_executor.py:7551-7623,7784-7941,9628-9685` (arithmetic,
  missing/non-finite behavior, and atomic failures).

The oracle supports a broader expression language and eager/lazy fallbacks.
Its full behavior includes strings, booleans, NULL comparisons, supported
function calls, exact integral overflow accounting, row-level normalization of
division-by-zero and non-finite arithmetic, labels/panel metadata, and
`last_operation` updates. Those behaviors are evidence for future slices, not
claims of this Rust boundary.

## Bounded Rust contract

Add an owned `GenerateResult { dataset: DatasetInfo }` and
`ExecutionResult::Generate`. Add command-owned errors for:

- an existing target (`generate target already exists: <name>`);
- unknown referenced variables (`expression unknown variable: ...`);
- non-numeric operands (`expression type mismatch: arithmetic requires numeric operands`);
- unsupported expression forms (strings, comparisons, NULL literals, and calls);
- backend/staging/publication failure (`generate failed`).

The implementation must:

1. return `NoActiveDataset { command: "generate" }` before backend work;
2. validate target collision, every identifier, and every numeric operand before
   staging;
3. compile only the supported expression subset using quoted identifiers and
   existing safe numeric SQL helpers;
4. stage `SELECT *, <expression> AS <quoted target> FROM __tabdat_active`;
5. inspect staged schema and row count, publish through the shared transaction,
   and update session metadata only after publication succeeds;
6. preserve source, row count, schema order, row order, NULLs, eager mode, and
   lazy-engine metadata (`None` for this eager slice).

Exact integer-overflow counts, division/non-finite normalization parity,
function calls, string/boolean/NULL/comparison results, lazy/materialized
execution, labels/panel metadata, `last_operation`, CLI/JSON/MCP, and broad
transform sequencing remain explicitly deferred. The result therefore carries
dataset metadata only; it does not claim full Python `generate` parity.

## Test contract

Focused runtime coverage must include:

- no active dataset with backend still uninitialized;
- parsed-command execution for `generate age2 = age + 1`;
- target appended after existing columns with expected values and row order;
- unary minus, operator precedence, parentheses, direct numeric literals, and
  quoted source/target identifiers;
- empty active relation preserving zero rows while adding the target schema;
- target collision, unknown variable, non-numeric operand, string/comparison/
  NULL/function-call rejection preserving metadata and preview;
- a dropped or mismatched active relation mapping to `GenerateFailed` without
  changing previously published metadata;
- repeated successful generation and normal arithmetic NULL propagation.

The parser contract remains covered by the existing language tests, including
function-call AST retention and quoted identifiers. No new dependency, native
backend, FFI, unsafe code, or ADR decision is required.

Completion state: contract recovery and draft-PR setup are complete;
implementation, independent review, hosted acceptance, merge, and temporary
branch cleanup remain pending.

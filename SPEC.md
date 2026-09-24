# TabDat Rust implementation state

## Current behavior

The Rust 2024 binary remains a scaffold: it prints `Hello, world!` and exits
successfully. The workspace contains a backend-independent `tabdat-language`
crate with a syntax-only parser, a library-only `tabdat-runtime` crate for eager
DuckDB tabular relations and data commands, and a pure, safe `tabdat-stats` crate
for statistical problem specifications, explicit sample tracking, parameter inference,
covariance representation, post-estimation state, and Householder QR least squares baseline
fitting. These paths are not wired into the
binary and do not provide a usable TabDat CLI, general data runtime, or
statistical model implementation.
The [proposal](docs/TABDAT_RUST_PORT_PROJECT_PROPOSAL.md) and
[roadmap](docs/TABDAT_RUST_PORT_ROADMAP.md) describe planned work, not support.

Merged PR #27 (`7cf21ae`) adds the verified syntax-only `sort <varlist>` form.
Merged PR #50 (`f33987a`) adds the bounded eager runtime subset documented below;
broader sort behavior remains deferred.

Merged PR #28 (`fd94133`) adds the verified syntax-only `gsort [+|-]varlist`
form. Merged PR #51 (`c06ed5a`) adds the bounded eager runtime subset
documented below; broader directed-sort behavior remains deferred.

Merged PR #52 (2e25cda) adds the bounded eager runtime subset for recode
documented below; broader recode behavior remains deferred.

Merged PR #53 (af3e3b2) adds the bounded eager runtime subset for encode
documented below; broader encode/decode and label-dictionary behavior remains
deferred until the later bounded slices.

Merged PR #29 (`8b16223`) adds the bounded syntax-only `save <path> [, replace]` and
`export <path> [, replace]` forms. The language layer owns the lexical path and
replacement flag; filesystem validation, active-dataset access, output formats,
and persistence remain deferred.

Merged PR #45 (`63e65ec`) adds the bounded syntax-only
`generate <target> = <expression>` form. The language layer preserves an owned
expression tree, including function-call syntax, while expression evaluation,
schema/type validation, target mutation, and all runtime/output surfaces remain
deferred.

Merged PR #47 (`87ec017`) adds the bounded syntax-only
`replace <target> = <expression> [if <condition>]` form. The language layer
preserves the existing typed expression tree for the replacement and optional
condition, while relation mutation, schema/type validation, predicate
truthiness, and all runtime/output surfaces remain deferred.

Merged PR #46 (`98979bc`) adds a separate bounded eager runtime slice for
`generate`: numeric identifiers/literals, unary minus, and `+`, `-`, `*`, `/`
append a generated column to an active local-Parquet DuckDB relation through
staged failure-atomic publication. Target collisions, unknown variables,
non-numeric operands, unsupported expression forms, quoted identifiers, empty
relations, row order, and metadata preservation are covered by typed runtime
results/errors and focused tests. This library-only path is not wired into the
binary CLI and does not claim function-call, string/boolean/NULL/comparison,
exact overflow-count, lazy/materialized, label/panel, `last_operation`,
CLI/JSON/MCP, or broad transform parity.

## Verified slice: syntax-only `save` and `export` commands

Add direct, backend-independent `save <path> [, replace]` and
`export <path> [, replace]` commands. Case-insensitive command names, quoted and
symbolic paths, repeated flag-only `replace` options, exact arity/option/
condition/assignment diagnostics, and explicit runtime deferral are covered by
focused unit and public integration tests.

Evidence: `_workspace/parser-save-export-syntax/01-contract.md`,
`crates/tabdat-language/src/lib.rs`,
`crates/tabdat-language/tests/parser_contract.rs`,
`crates/tabdat-runtime/src/lib.rs`, and
`crates/tabdat-runtime/tests/use_contract.rs`. The pinned configuration and
persistence parser subset passed with `419 passed, 70 deselected` at revision
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`; the full parser/script oracle and
hosted acceptance checks remain part of the PR evidence.

This slice does not inspect paths or active data, write files, validate output
formats, mutate session state, initialize a backend, or claim persistence/output
parity.

## Verified slice: bounded eager runtime `save` command

Merged PR #70 (`9bbf804`) adds the library-only runtime implementation of the
already-parsed `save <path> [, replace]` form for an active eager local-Parquet
DuckDB relation. It accepts a case-insensitive `.parquet` destination, creates
missing parent directories, rejects existing targets unless `replace` is
requested, and writes the currently published relation through a parameterized
DuckDB copy. The owned `SaveResult` identifies the output path and output
metadata; the active relation, metadata, and session-local labels remain
unchanged. Schema order, row order, row count, and SQL NULLs are covered by
round-trip tests, including backend-copy failure recovery.

Evidence: `_workspace/runtime-save/01-contract.md`,
`_workspace/runtime-save/02-evidence-migration.md`,
`_workspace/runtime-save/03-review.md`,
`crates/tabdat-runtime/src/lib.rs`, and
`crates/tabdat-runtime/tests/save_contract.rs`. The pinned Python save probe,
the parser/script oracle, locked workspace checks, policy checks, PR-head
workflows, and merge-head workflows all passed for this bounded slice.

This remains a library-only eager local-Parquet boundary, not a usable CLI.
CSV-only `export`, Feather/Arrow writers, lazy/materialized output, path `~`
expansion, atomic temporary-file replacement, metadata/label/panel persistence,
and CLI/JSON/MCP surfaces remain separate bounded or deferred work.

## Verified slice: bounded eager runtime CSV `export`

Merged PR #72 (`5cb5b34`) adds the library-only runtime implementation of the
already-parsed `export <path> [, replace]` form for an active eager local-Parquet
DuckDB relation. It accepts a case-insensitive `.csv` destination, creates
missing parent directories, rejects existing targets unless `replace` is
requested, and writes a header-bearing CSV through a parameterized DuckDB copy.
The owned `ExportResult` identifies the output path and output metadata; the
active relation, metadata, and session-local labels remain unchanged. Exact
schema/row order, decimal formatting, quoted text, SQL NULL fields, transformed
data, empty relations, and backend-failure recovery are covered by focused
contract tests.

Evidence: `_workspace/runtime-export-csv/01-contract.md`,
`_workspace/runtime-export-csv/02-evidence-migration.md`,
`_workspace/runtime-export-csv/03-review.md`,
`_workspace/runtime-export-csv/04-summary.md`,
`crates/tabdat-runtime/src/lib.rs`, and
`crates/tabdat-runtime/tests/export_contract.rs`. The pinned Python export
probe, parser/script oracle, locked workspace checks, policy checks, PR-head
workflows, and merge-head workflows all passed for this bounded slice.

This remains a library-only eager local-Parquet boundary, not a usable CLI.
Parquet aliasing through `export`, Feather/Arrow writers, lazy/materialized
output, path `~` expansion, atomic temporary-file replacement,
metadata/label/panel persistence, and CLI/JSON/MCP surfaces remain deferred.

## Verified slice: bounded eager runtime CSV `use`

Merged PR #74 (`7a5b8d4`) extends the separate library-only runtime boundary
with eager local `.csv` input for the already-parsed `use <path>` form. The
case-insensitive extension is validated before backend initialization;
optional `delimiter` and `has_header` values are bound through DuckDB's
`read_csv_auto`; and rows are staged, inspected, and published atomically.
Successful loads return the existing owned `LoadResult` with source path,
ordered schema, and row count, while failed reads and publication preserve the
active relation, metadata, labels, and backend usability.

Evidence: `_workspace/runtime-use-csv/01-contract.md`,
`_workspace/runtime-use-csv/02-evidence-migration.md`,
`_workspace/runtime-use-csv/03-review.md`,
`_workspace/runtime-use-csv/04-summary.md`,
`crates/tabdat-runtime/src/lib.rs`,
`crates/tabdat-runtime/tests/use_contract.rs`, and
`crates/tabdat-runtime/tests/use_csv_contract.rs`. Six focused CSV tests and
the existing 65-test `use_contract` suite pass; locked workspace, policy, and
hosted PR/merge-head checks are recorded in the evidence artifacts.

This remains a library-only eager local Parquet/CSV boundary, not a usable CLI.
Remote or URI sources, lazy/materialized execution, Feather/Arrow/DTA input,
`~` expansion, broader path normalization, metadata serialization, atomic
temporary-file replacement, and CLI/JSON/MCP surfaces remain deferred.

## Verified slice: syntax-only `generate` command

Add direct, backend-independent `generate <target> = <expression>` syntax. The
parser preserves exact quoted identifier spelling, arithmetic/comparison
precedence, null/string/numeric literals, nested function-call nodes, and the
pinned malformed-input diagnostics. A quoted identifier named `if` remains
distinct from the unsupported clause marker, and the pinned trailing-comma
quirk is retained without introducing option parsing.

Evidence: `_workspace/parser-generate-syntax/`,
`crates/tabdat-language/src/lib.rs`,
`crates/tabdat-language/tests/parser_contract.rs`,
`crates/tabdat-runtime/src/lib.rs`, and
`crates/tabdat-runtime/tests/use_contract.rs`. The pinned parser probe passed
`5 passed, 484 deselected`; focused Rust parser/runtime checks passed, the
locked workspace baseline and policy checks passed locally, and the final PR
head passed the policy, baseline, and runtime workflows before squash merge as
PR #45 (`63e65ec`).

This slice does not evaluate expressions, inspect an active dataset or schema,
validate types or target collisions, mutate relations/session state, initialize
a backend, or claim CLI/JSON/MCP/runtime parity. Those remain a separate eager
runtime contract.

## Verified slice: syntax-only `replace` command

Add direct, backend-independent `replace <target> = <expression> [if
<condition>]` syntax. The parser preserves owned target and expression nodes,
reuses the generate expression grammar, distinguishes nested `if` identifiers
from the top-level clause marker, and retains the pinned bounded diagnostics.
Runtime execution remains an explicit unsupported-command boundary.

Evidence: `_workspace/parser-replace-syntax/`,
`crates/tabdat-language/src/lib.rs`,
`crates/tabdat-language/tests/parser_contract.rs`,
`crates/tabdat-runtime/src/lib.rs`, and
`crates/tabdat-runtime/tests/use_contract.rs`. The pinned parser probe passed
`419 passed, 70 deselected`; focused Rust parser/runtime checks and the locked
workspace/policy checks passed locally. PR #47 (`87ec017`) passed the policy,
baseline, and runtime workflows before squash merge, and the merge-head
workflows passed as recorded in the companion evidence.

This slice does not inspect an active dataset or schema, evaluate expressions,
validate target types or collisions, mutate relations/session state, initialize
a backend, or claim CLI/JSON/MCP/runtime parity. Those remain a separate
data-semantics and eager-runtime contract.

## Verified slice: bounded eager runtime `generate` command

Execute the parsed numeric `generate <target> = <expression>` subset against an
active eager local-Parquet DuckDB relation. The runtime validates before staging,
uses quoted SQL identifiers, appends the generated column in schema order, and
publishes through the shared transactional `__tabdat_next` path. Failures leave
the published metadata and private active relation unchanged.

Evidence: `_workspace/runtime-generate/`, `crates/tabdat-runtime/src/lib.rs`,
`crates/tabdat-runtime/tests/generate_contract.rs`, and the legacy deferred
runtime regression in `crates/tabdat-runtime/tests/use_contract.rs`. Local
format/check/test/Clippy, dependency-policy, audit, and metadata-driven geiger
checks passed; PR #46 (`98979bc`) and its post-merge CI/runtime workflows
passed.

The accepted runtime subset is library-only and does not establish binary CLI,
JSON/MCP, lazy/materialized, function-call, string/boolean/NULL/comparison,
exact overflow-count, label/panel, `last_operation`, or broad transform parity.

## Verified slice: bounded eager runtime `replace` command

Execute the parsed `replace <target> = <expression> [if <condition>]` subset
against an active eager local-Parquet DuckDB relation. The runtime validates the
target, identifiers, domains, predicate, and supported expression forms before
staging a source-order projection. It preserves the target's schema position,
row order, row count, NULL behavior, source and eager metadata, and publishes
through the shared transactional `__tabdat_next` path before changing session
metadata.

Evidence: `_workspace/runtime-replace/`,
`crates/tabdat-runtime/src/lib.rs`,
`crates/tabdat-runtime/tests/replace_contract.rs`, and the updated deferred
runtime regression in `crates/tabdat-runtime/tests/use_contract.rs`. Local
format/check/test/Clippy, dependency-policy, audit, and metadata-driven geiger
checks passed. Draft PR [#48](https://github.com/SaehwanPark/tabdat-explore-rs/pull/48)
was opened at the contract checkpoint, passed its PR-head workflows, and was
squash-merged as
[`df2cad9`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/df2cad9e8f61479ad67f118441acbbeb0704c408).
Merge-head workflow evidence is recorded in the companion workspace artifact.

This accepted runtime subset is library-only. Function calls, unsupported
boolean/other target domains, exact overflow-count diagnostics, lazy/materialized
execution, labels/panel metadata, `last_operation`, formatting, CLI, JSON, MCP,
and broad transform parity remain deferred.

## Verified slice: bounded eager runtime `rename` command

Execute the parsed `rename <old> <new>` subset against an active eager
local-Parquet DuckDB relation. The runtime validates the source and target
names before staging, rejects unknown sources and target collisions (including
same-name requests), and stages a quoted source-order projection that aliases
only the renamed column. Schema position and logical type, row order, row
count, SQL NULL values, source path, and eager execution metadata are
preserved; publication is failure-atomic through the shared transactional
`__tabdat_next` path.

Evidence: `_workspace/runtime-rename/`,
`crates/tabdat-language/src/lib.rs`,
`crates/tabdat-runtime/src/lib.rs`,
`crates/tabdat-runtime/tests/rename_contract.rs`, and the updated no-active
runtime regression in `crates/tabdat-runtime/tests/use_contract.rs`. The
pinned oracle probe, focused Rust tests, locked workspace baseline, dependency
policy, audit, and metadata-driven geiger checks passed locally. Draft PR
[#49](https://github.com/SaehwanPark/tabdat-explore-rs/pull/49) passed its
PR-head workflows and was squash-merged as
[`0bd547f`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/0bd547f15db7fef01e2a60554fe52a8eb4a4f129); merge-head workflow evidence
is recorded in the companion artifact.

This accepted runtime subset is library-only. Panel/label metadata, lazy or
materialized execution, wildcard or multi-column forms, `last_operation`,
formatting, CLI, JSON, MCP, and broad transform sequencing remain deferred.

## Verified slice: bounded eager runtime `sort` command

Execute the parsed `sort <varlist>` subset against an active eager local-Parquet
DuckDB relation. The runtime validates every requested source column before
staging, orders by one or more quoted native DuckDB scalar keys ascending with
SQL `NULL` values last, and preserves prior row order for complete ties through
a private collision-free row ordinal. All source columns, schema types and
order, row count, source path, and eager execution metadata are preserved;
staging, inspection, and publication failures leave the prior state unchanged.

Evidence: `_workspace/runtime-sort/`,
`crates/tabdat-runtime/src/lib.rs`,
`crates/tabdat-runtime/tests/sort_contract.rs`, and the updated no-active
runtime regression in `crates/tabdat-runtime/tests/use_contract.rs`. The
pinned oracle probe reported 7 passed tests. Local format/check/test/Clippy,
dependency-policy, audit, and metadata-driven geiger checks passed. Draft PR
[#50](https://github.com/SaehwanPark/tabdat-explore-rs/pull/50) was opened at
the contract checkpoint; final PR head
[`55e1fc6`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/55e1fc6b327f42505531ba6da0dee89640f25a1f)
passed the PR-head CI and runtime workflows before squash merge as
[`f33987a`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/f33987ada50beb4030075a7eb388ec8855b64421).
Merge-head and documentation-closeout workflow evidence is recorded in the
companion artifact.

This accepted runtime subset is library-only. Panel/label metadata, lazy or
materialized execution, descending keys, `gsort`, expression keys,
`last_operation`, formatting, CLI, JSON, MCP, and broad transform sequencing
remain deferred.

## Verified slice: bounded eager runtime `gsort` command

Execute the parsed `gsort [+|-]varlist` subset against an active eager
local-Parquet DuckDB relation. The runtime validates every requested key before
staging, orders each quoted native DuckDB scalar key in its requested
ascending or descending direction with SQL `NULL` values last, and preserves
prior row order for complete ties through a private collision-free row
ordinal. All source columns, schema types and order, row count, source path,
and eager execution metadata are preserved; staging, inspection, and
publication failures leave the prior state unchanged.

Evidence: `_workspace/runtime-gsort/`,
`crates/tabdat-runtime/src/lib.rs`,
`crates/tabdat-runtime/tests/gsort_contract.rs`, and the updated no-active
runtime regression in `crates/tabdat-runtime/tests/use_contract.rs`. The
pinned oracle probe reported 8 passed tests. Local format/check/test/Clippy,
dependency-policy, audit, and metadata-driven geiger checks passed. Draft PR
[#51](https://github.com/SaehwanPark/tabdat-explore-rs/pull/51) was opened at
the contract checkpoint; final PR head
[`04b0d01`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/04b0d01e626cf002ab43403b295e650d46ba59be)
passed the PR-head CI and runtime workflows before squash merge as
[`c06ed5a`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/c06ed5a7d913aaa5a4ddbdf58f79149a8b81a79e).
Merge-head and documentation-closeout workflow evidence is recorded in the
companion artifact.

This accepted runtime subset is library-only. Panel/label metadata, lazy or
materialized execution, `last_operation`, formatting, CLI, JSON, MCP, and
broad transform sequencing remain deferred.

## Verified slice: bounded eager runtime recode command

Execute the parsed recode VARLIST (RULE) ... [, generate(NEWVARLIST) | replace]
subset against an active eager local-Parquet DuckDB relation. The runtime
supports numeric and quoted/text scalar inputs, inclusive numeric ranges,
missing/nonmissing and else rules, ordered first-match behavior, unchanged
fallback, one generated output per source, and in-place replacement.
Validation occurs before staging. A full ordered projection is staged, its
schema and row count are inspected, and it becomes active only after
publication succeeds; failures preserve the previous relation and metadata.

Evidence: _workspace/runtime-recode/, crates/tabdat-language/src/lib.rs,
crates/tabdat-runtime/src/lib.rs, and
crates/tabdat-runtime/tests/recode_contract.rs. The pinned oracle probe
reported 3 passed recode tests. Local format/check/test/Clippy,
dependency-policy, audit, and metadata-driven geiger checks passed. Draft PR
52 was opened at the contract checkpoint, and its final documentation-head
workflows passed before squash merge as 2e25cda. PR-head, merge-head, and
documentation-closeout workflow links are recorded in the companion evidence
artifact.

This accepted runtime subset is library-only. Lazy/materialized execution,
panel metadata, last_operation, formatting, CLI, JSON, MCP, and broad
transform sequencing remain deferred.

## Verified slice: bounded eager runtime encode command

Execute the parsed `encode <strvar>, generate(<newvar>)` subset against an
active eager local-Parquet DuckDB relation. The runtime discovers sorted unique
nonmissing source strings, assigns one-based integer codes, preserves source
NULLs as target NULLs, appends the generated column, and publishes the staged
projection atomically. Quoted identifiers, embedded identifier quotes, empty
relations, source existence/type validation, target collisions, and backend
failure preservation are covered. Ordinary encode also retains session-owned
label metadata for the bounded decode and label slices; the optional
`label(<lblname>)` name selects the generated value-label set.

Evidence: `_workspace/runtime-encode/`, `crates/tabdat-language/src/lib.rs`,
`crates/tabdat-language/tests/parser_contract.rs`,
`crates/tabdat-runtime/src/lib.rs`, and
`crates/tabdat-runtime/tests/encode_contract.rs`. The pinned oracle focused
suite reported 6 passed tests and an isolated probe confirmed `b, a, NULL, b`
maps to `2, 1, NULL, 2` with an integer generated column. Local format,
check, test, Clippy, dependency-policy, audit, and metadata-driven geiger
checks passed. PR [#53](https://github.com/SaehwanPark/tabdat-explore-rs/pull/53)
passed its [PR-head CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35474913800)
and [runtime workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35474913819)
before squash merge as
[`af3e3b2`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/af3e3b2726778af5c5f3b5c13c4ba84e5291da61).
Merge-head workflows `35484575373`/`35484575369` passed; documentation-closeout
workflow links are recorded in the companion evidence artifact.

This accepted runtime subset is library-only. Label persistence and rendering,
lazy/materialized execution, panel metadata, `last_operation`, formatting, CLI,
JSON, MCP, and broad transform sequencing remain deferred.

## Verified slice: bounded eager runtime `decode` command

Merged PR [#54](https://github.com/SaehwanPark/tabdat-explore-rs/pull/54) adds
the bounded eager local-Parquet `decode <numvar>, generate(<newvar>)` path to
`tabdat-runtime`. It consumes only the private code-to-text map produced by an
ordinary same-session `encode`, maps known integer codes back to strings, and
returns SQL NULL for source NULLs and unmapped codes. Quoted identifiers,
empty mappings, source/type/target validation, rename/projection provenance,
retry after failed publication, and atomic staged publication are covered.

Evidence: `_workspace/runtime-decode/`, the implementation, and focused parser
and runtime contract tests. The pinned oracle focused suite reported 6 passed
tests. Local locked Rust checks, dependency-policy, advisory, and
metadata-driven geiger checks passed; PR-head workflows
`35479575291`/`35479575205` and merge-head workflows
`35480547109`/`35480547104` passed before and after squash merge as
[`0845e6c`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/0845e6cf37a50c317d7ee30acee2d85474c7bcd2).

This slice does not claim arbitrary imported value-label metadata, label
persistence, lazy/materialized execution, panel metadata, output adapters, or
broad transform sequencing.

## Verified slice: bounded eager runtime session-local `label` metadata

Merged PR [#55](https://github.com/SaehwanPark/tabdat-explore-rs/pull/55) adds
the bounded eager local-Parquet session-local `label` forms for variable
labels, named value-label definitions/replacement, variable attachments,
listing, and dropping. `LabelMetadata`, `ValueLabelSet`, and `LabelResult` are
owned Rust values; validation and metadata publication are atomic with respect
to the active relation. Encode publishes default or explicit named sets,
decode consumes attached integer sets, and successful `use`, rename,
keep/drop/select, value-changing replace, and in-place recode reconcile the
metadata that survives the new schema or values.

Evidence: `_workspace/runtime-label/`, the typed parser and runtime
implementations, and focused parser/runtime contract tests. The pinned oracle
focused suites reported 6 label tests and 6 encode/decode tests passing. Local
locked format/check/test/Clippy, dependency-policy, advisory, and
metadata-driven geiger checks passed. PR-head workflows
`35483588143`/`35483588144` passed before squash merge as
[`70b9745`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/70b9745ae7e22855c763bcb9e3ed40332646723c).
Merge-head workflows `35484575373`/`35484575369` and documentation-closeout
workflows `35485577830`, `35485577864`, `35485577816`, and `35485577786` passed;
the detailed links are recorded in the companion evidence artifact.

This accepted slice remains library-only. `label save/use`, DTA-imported
labels, inspection/reporting rendering, lazy/materialized execution, panel
metadata, output adapters, CLI, JSON, MCP, and broad transform sequencing
remain deferred.

## Verified slice: bounded eager runtime tabulate command

Merged PR [#56](https://github.com/SaehwanPark/tabdat-explore-rs/pull/56)
(24405a6) adds the bounded eager local-Parquet one- and two-way frequency
table path to tabdat-runtime. Direct row and optional column variables are
typed at the language boundary. One-way results return category, count, and
percentage cells; two-way results return count cells with optional row and
column percentages. The missing flag includes SQL NULL dimensions, nolabel
suppresses attached session-local value labels, and failed validation or reads
leave active state unchanged.

Evidence: _workspace/runtime-tabulate/, the implementation, and focused
parser/runtime contract tests. The pinned oracle focused checks, locked Rust
baseline, dependency-policy, advisory, metadata-driven geiger checks,
independent review, PR-head workflows, squash merge, and merge-head workflows
all passed. The accepted merge commit is
[24405a6](https://github.com/SaehwanPark/tabdat-explore-rs/commit/24405a6878034506e18086d5092d665b4dd25159).

This accepted slice remains library-only. values/stat aggregation, if
predicates, by-prefixes, multiple dimensions, named-table execution,
lazy/materialized execution, persistence, formatting, CLI, JSON, MCP, and
broad Python tabulate parity remain deferred.

## Verified slice: bounded eager runtime `collapse` command

Merged PR [#57](https://github.com/SaehwanPark/tabdat-explore-rs/pull/57)
([94391af](https://github.com/SaehwanPark/tabdat-explore-rs/commit/94391afe84ab1b1c8c3c5006a55911426d6767b1))
adds the bounded eager local-Parquet grouped-aggregate path to tabdat-runtime.
The typed language boundary accepts direct `collapse <statistic> <variables>,
by(<groups>)` forms for `count`, `mean`, `sum`, `min`, and `max`, with exact
bounded rejection for conditions, assignment syntax, unsupported statistics,
and malformed grouping options. Runtime execution validates active state,
schema names, and numeric requirements before staging a quoted DuckDB query;
it groups SQL NULL values explicitly, orders groups ascending with NULL last,
counts non-NULL values, and atomically publishes an owned `CollapseResult`.
Source/eager metadata is preserved, and session-local variable labels and
value-label attachments are pruned to surviving group columns.

Evidence: [_workspace/runtime-collapse/](_workspace/runtime-collapse/),
the implementation, and focused parser/runtime contract tests. The pinned
oracle checks, locked Rust baseline, dependency-policy, advisory,
metadata-driven geiger checks, review, PR-head workflows, squash merge, and
merge-head workflows all passed. Documentation-closeout workflow links are
recorded in the companion evidence artifact after their completion.

This accepted slice remains library-only. Conditions, weights, named-table
execution, lazy/materialized execution, panel propagation, persistence,
formatting, CLI, JSON, MCP, and broad Python `collapse` parity remain deferred.

## Verified slice: bounded eager runtime by command

Merged PR [#58](https://github.com/SaehwanPark/tabdat-explore-rs/pull/58)
([cc818e5](https://github.com/SaehwanPark/tabdat-explore-rs/commit/cc818e5e5e395d9b0f8c86a0dee451417ca98002))
adds the bounded eager local-Parquet grouped read-only path to tabdat-runtime.
The typed language boundary accepts by group-list prefixes over summarize and
count, including quoted identifiers and exact rejection of missing delimiters,
empty grouping lists, nested by commands, and help/status/doctor children.
Grouped summarize returns an owned ByResult with group values followed by
mean-variable columns; an omitted variable list selects numeric non-group
columns in schema order. Grouped count returns group values and Count using
COUNT(*). Both forms treat SQL NULL group values as explicit groups, order
groups ascending with NULL last, validate schema and numeric requirements before
querying, and preserve the active relation and session metadata.

Evidence: [_workspace/runtime-by/](_workspace/runtime-by/), the implementation,
and focused parser/runtime contract tests. The pinned oracle checks, locked Rust
baseline, dependency-policy, advisory, metadata-driven geiger checks,
independent review, PR-head workflows, squash merge, and merge-head workflows
all passed. Documentation-closeout workflow links are recorded in the companion
evidence artifact; the closeout commit and its main CI, ReadStat feasibility,
libgretl feasibility, and libgretl OLS Rust spike workflows also passed.

This accepted slice remains library-only. Grouped tabulate and other child
commands, conditions, weights, named-table execution, lazy/materialized
execution, panel propagation, persistence, formatting, CLI, JSON, MCP, and
broad Python by parity remain deferred.

## Verified slice: syntax-only `join` command

Merged [PR #59](https://github.com/SaehwanPark/tabdat-explore-rs/pull/59)
([585c53f](https://github.com/SaehwanPark/tabdat-explore-rs/commit/585c53fcdce135456abded9b27df75fbcbcda8cf))
adds a bounded, backend-independent `join <table> on <keylist> [, how=inner|left suffix(_right)]`
syntax boundary to `tabdat-language`. The parser returns owned table/key data,
typed `inner`/`left` mode, and a non-empty suffix, applies the recovered
`inner` and `_right` defaults, preserves key order, and covers bounded
diagnostics for quoted separators, duplicate keys, reserved tables, and
unsupported or duplicate options. Runtime deliberately returns the typed
unsupported-command result; it does not initialize DuckDB, inspect files, or
mutate session state.

Evidence: [_workspace/runtime-join/](_workspace/runtime-join/), including the
[migration evidence](_workspace/runtime-join/02-evidence-migration.md),
[review](_workspace/runtime-join/03-review.md), and
[summary](_workspace/runtime-join/04-summary.md). The pinned oracle parser
selection, locked Rust baseline, dependency/advisory/unsafe-code policy,
metadata-driven geiger checks, PR-head workflows, squash merge, and merge-head
workflows all passed.

This accepted syntax slice leaves named-table registry and activation, SQL
creation, DuckDB join execution, key type/null semantics, collision/order and
publication rules, labels, lazy/materialized behavior, persistence, formatting,
CLI, JSON, MCP, and broad Python `join` parity deferred. Phase 6.4 runtime
`join` was subsequently implemented in PR #139 (squash merge `56babda`).

## Verified slice: syntax-only `append` command

Merged [PR #60](https://github.com/SaehwanPark/tabdat-explore-rs/pull/60)
([8ab016f](https://github.com/SaehwanPark/tabdat-explore-rs/commit/8ab016f1e014445288d2411cf7d058e2bd523b49))
adds a bounded, backend-independent `append <table>` syntax boundary to
`tabdat-language`. The parser returns an owned table name, applies the recovered
identifier and reserved-name checks, accepts the established quoted/backtick
forms, and preserves bounded diagnostics for malformed arity, options,
conditions, and assignment syntax. Runtime deliberately returns the typed
unsupported-command result; it does not initialize DuckDB, inspect files, or
mutate session state.

Evidence: [_workspace/runtime-append/](_workspace/runtime-append/), including the
[migration evidence](_workspace/runtime-append/02-evidence-migration.md),
[review](_workspace/runtime-append/03-review.md), and
[summary](_workspace/runtime-append/04-summary.md). The pinned oracle parser
selection, locked Rust baseline, dependency/advisory/unsafe-code policy,
metadata-driven geiger checks, PR-head workflows, squash merge, and merge-head
workflows all passed.

This accepted syntax slice leaves named-table registry and activation, SQL
creation, DuckDB append execution, schema compatibility, column union/type and
missingness semantics, row ordering/publication, labels, lazy/materialized
behavior, persistence, formatting, CLI, JSON, MCP, and broad Python `append`
parity deferred. Phase 6.4 runtime `append` was subsequently implemented in PR #141 (squash merge `800a231`).

## Verified slice: syntax-only `reshape` command

Merged [PR #61](https://github.com/SaehwanPark/tabdat-explore-rs/pull/61)
([e6cc4f1](https://github.com/SaehwanPark/tabdat-explore-rs/commit/e6cc4f1768c9b55b8ead702a08a36283f2a27bee))
adds a bounded, backend-independent
`reshape long|wide <varlist>, i(<id_vars>) j(<name>)` syntax boundary to
`tabdat-language`. The parser returns an owned `ReshapeCommand` with typed long
or wide direction, ordered variables and identifiers, and the single `j()` name.
It preserves the recovered quoted/unquoted direction behavior, requires unique
and pairwise-distinct names, requires exactly one lowercase `i()` and `j()`
option, and preserves bounded diagnostics for malformed or unsupported forms.
Runtime deliberately returns the typed unsupported-command result; it does not
initialize DuckDB, inspect files, or mutate session state.

Evidence: [_workspace/runtime-reshape/](_workspace/runtime-reshape/), including
the [migration evidence](_workspace/runtime-reshape/02-evidence-migration.md),
[review](_workspace/runtime-reshape/03-review.md), and
[summary](_workspace/runtime-reshape/04-summary.md). The pinned oracle parser
selection, locked Rust baseline, dependency/advisory/unsafe-code policy,
metadata-driven geiger checks, PR-head workflows, squash merge, and merge-head
workflows all passed.

This accepted syntax slice leaves long/wide relation execution, identifier-group
and missingness semantics, wide-column naming and collision rules, row ordering,
row counts, type coercion, relation publication, labels, lazy/materialized
behavior, persistence, formatting, CLI, JSON, MCP, and broad Python `reshape`
parity deferred. Phase 6.4 runtime `reshape` was subsequently implemented in PR #143 (squash merge `4d90584`).

## Verified slice: syntax-only `panel` command

Merged [PR #62](https://github.com/SaehwanPark/tabdat-explore-rs/pull/62)
([92e5d5e](https://github.com/SaehwanPark/tabdat-explore-rs/commit/92e5d5e5be9aaededbfe3f44ded2d4319cdcac87))
adds a bounded, backend-independent `panel` syntax boundary to
`tabdat-language`. The parser returns an owned typed report, clear, or set
action for `panel`, `panel <id_var> <time_var>`, and `panel clear`,
preserves the recovered string/backtick `clear` keyword boundary, and
requires distinct entity and time names. Runtime deliberately returns the typed
unsupported-command result; it does not inspect an active relation, initialize
DuckDB, or mutate session state.

Evidence: [_workspace/runtime-panel/](_workspace/runtime-panel/), including the
[migration evidence](_workspace/runtime-panel/02-evidence-migration.md),
[review](_workspace/runtime-panel/03-review.md), and
[summary](_workspace/runtime-panel/04-summary.md). The pinned oracle parser
selection, focused Rust parser/runtime tests, locked Rust baseline, dependency/
advisory/unsafe-code policy, PR-head workflows, squash merge, and merge-head
workflows all passed.

This accepted syntax slice leaves panel metadata ownership, active-schema and
numeric/time validation, duplicate entity-time checks, structural summaries,
ordering, missingness, relation publication, labels, lazy/materialized behavior,
persistence, formatting, CLI, JSON, MCP, and broad Python `panel` parity
deferred. Phase 11.1 runtime `panel` remains unchecked.

## Verified slice: syntax-only `xtdata` command

Merged [PR #63](https://github.com/SaehwanPark/tabdat-explore-rs/pull/63)
([74eea07](https://github.com/SaehwanPark/tabdat-explore-rs/commit/74eea071d118f3c97c931a9c9345c17293e56440))
adds a bounded, backend-independent `xtdata` syntax boundary to
`tabdat-language`. The parser returns an owned typed within or between action
for `xtdata <varlist>, within|between`, preserves quoted variable names and
flag-only transform options, and requires exactly one transform. Runtime
deliberately returns the typed unsupported-command result; it does not inspect
panel metadata, initialize DuckDB, or mutate session state.

Evidence: [_workspace/runtime-xtdata/](_workspace/runtime-xtdata/), including
the [migration evidence](_workspace/runtime-xtdata/02-evidence-migration.md),
[review](_workspace/runtime-xtdata/03-review.md), and
[summary](_workspace/runtime-xtdata/04-summary.md). The pinned oracle parser
selection, focused Rust parser/runtime tests, locked Rust baseline, dependency/
advisory/unsafe-code policy, PR-head workflows, squash merge, and merge-head
workflows all passed.

This accepted syntax slice leaves panel metadata ownership, active-schema and
numeric validation, within/between formulas, generated-column naming and
collisions, missingness, ordering, relation publication, labels,
lazy/materialized behavior, persistence, formatting, CLI, JSON, MCP, and broad
Python `xtdata` parity deferred. Phase 11.1 runtime `xtdata` remains unchecked.

## Verified slice: syntax-only `ivregress` command

Merged [PR #64](https://github.com/SaehwanPark/tabdat-explore-rs/pull/64)
([d355551](https://github.com/SaehwanPark/tabdat-explore-rs/commit/d35555111800b4655d30d1bc1448da8ba41d2f21))
adds a bounded, backend-independent `ivregress` syntax boundary to
`tabdat-language`. The parser returns an owned typed 2SLS or GMM command for
`ivregress 2sls|gmm <y> [exog_vars], endog(<var>) iv(<vars>)`, preserves
ordered and quoted variables, and supports the bounded robust, cluster, and
noconstant options with exact required-option and mutual-exclusion validation.
Runtime deliberately returns the typed unsupported-command result; it does not
fit a model, initialize DuckDB, or mutate session state.

Evidence: [_workspace/runtime-ivregress/](_workspace/runtime-ivregress/),
including the [migration evidence](_workspace/runtime-ivregress/02-evidence-migration.md),
[review](_workspace/runtime-ivregress/03-review.md), and
[summary](_workspace/runtime-ivregress/04-summary.md). The pinned oracle parser
selection, focused Rust parser/runtime tests, locked Rust baseline, dependency/
advisory/unsafe-code policy, PR-head workflows, squash merge, and merge-head
workflows all passed.

This accepted syntax slice leaves IV identification, first-stage and second-
stage fitting, GMM weighting, robust/cluster covariance, degrees of freedom,
missingness, prediction, post-estimation state, labels, formatting, CLI, JSON,
MCP, and broad Python `ivregress` parity deferred. Phase 11.1 runtime
`ivregress 2sls` and `ivregress gmm` remain unchecked.

## Verified slice: syntax-only `xtreg` command

Merged [PR #65](https://github.com/SaehwanPark/tabdat-explore-rs/pull/65)
([c88d002](https://github.com/SaehwanPark/tabdat-explore-rs/commit/c88d0028395633d9353fbac7afc4b485231042e3))
adds a bounded, backend-independent `xtreg` syntax boundary to
`tabdat-language`. The parser returns an owned typed fixed- or random-effects
command for `xtreg <y> <xvars>, fe|re`, preserves ordered and quoted
predictors, and supports the bounded `robust` and one-variable `cluster`
options with exact estimator-exclusivity and option-conflict validation.
Runtime deliberately returns the typed unsupported-command result; it does not
inspect panel metadata, fit a model, initialize DuckDB, or mutate session
state.

Evidence: [_workspace/runtime-xtreg/](_workspace/runtime-xtreg/), including
the [migration evidence](_workspace/runtime-xtreg/02-evidence-migration.md),
[review](_workspace/runtime-xtreg/03-review.md), and
[summary](_workspace/runtime-xtreg/04-summary.md). The pinned oracle parser
selection, focused Rust parser/runtime tests, locked Rust baseline, dependency/
advisory/unsafe-code policy, PR-head, squash merge, and merge-head workflows
all passed.

This accepted syntax slice leaves panel metadata ownership, fixed/random-effects
transformation and estimation, Hausman comparisons, covariance and clustered
degrees-of-freedom semantics, missingness, prediction, post-estimation state,
labels, formatting, CLI, JSON, MCP, and broad Python `xtreg` parity deferred.
Phase 11.1 runtime `xtreg, fe` and `xtreg, re` remain unchecked.

## Verified slice: syntax-only selected `estat` diagnostics

Merged [PR #66](https://github.com/SaehwanPark/tabdat-explore-rs/pull/66)
([484148e](https://github.com/SaehwanPark/tabdat-explore-rs/commit/484148ee6a25dfe5f245fd2f7be01eec5efd27b3))
adds a bounded, backend-independent `estat` syntax boundary to
`tabdat-language` for `firststage`, `overid`, `endogenous`, and `hausman`.
The parser returns an owned typed diagnostic subcommand, preserves command and
subcommand case normalization plus single/double-quoted subcommands, and
rejects options on the selected no-option forms with the recovered diagnostic.
Runtime deliberately returns the typed unsupported-command result; it does not
inspect model state, calculate a post-estimation result, initialize DuckDB, or
mutate session state.

Evidence: [_workspace/runtime-estat/](_workspace/runtime-estat/), including
the [migration evidence](_workspace/runtime-estat/02-evidence-migration.md),
[review](_workspace/runtime-estat/03-review.md), and
[summary](_workspace/runtime-estat/04-summary.md). The pinned oracle parser
selection, focused Rust parser/runtime tests, locked Rust baseline, dependency/
advisory/unsafe-code policy, PR-head, squash merge, and merge-head workflows
all passed.

This accepted syntax slice leaves first-stage, overidentification, endogeneity,
and Hausman calculations, statistical model-state routing, IV/panel validation,
covariance, missingness, output/reporting, labels, formatting, CLI, JSON, MCP,
remaining `estat` subcommands, and broad Python `estat` parity deferred. Phase
11.1 runtime `estat firststage`, `estat overid`, `estat endogenous`, and
`estat hausman` remain unchecked.

## Verified slice: syntax-only `xtabond` command

Merged [PR #67](https://github.com/SaehwanPark/tabdat-explore-rs/pull/67)
([820376d](https://github.com/SaehwanPark/tabdat-explore-rs/commit/820376d6c7ce3c4a4ff8abf24fe9a9b3ad84cea6))
adds a bounded, backend-independent `xtabond` syntax boundary to
`tabdat-language`. The parser returns an owned typed dynamic-panel command for
`xtabond <y> [xvars] [, robust lags(#) instlag(#)]`, preserves ordered and
quoted variables, supports the flag-only `robust` option, defaults `lags(1)`
and `instlag(2)`, and enforces `instlag > lags` plus the recovered numeric
bounds.
Runtime deliberately returns the typed unsupported-command result; it does not
inspect panel metadata, construct instruments, fit a model, initialize DuckDB,
or mutate session state.

Evidence: [_workspace/runtime-xtabond/](_workspace/runtime-xtabond/), including
the [migration evidence](_workspace/runtime-xtabond/02-evidence-migration.md),
[review](_workspace/runtime-xtabond/03-review.md), and
[summary](_workspace/runtime-xtabond/04-summary.md). The pinned oracle parser
selection, focused Rust parser/runtime tests, locked Rust baseline, dependency/
advisory/unsafe-code policy, PR-head, squash merge, and merge-head workflows
all passed.

This accepted syntax slice leaves panel metadata ownership, dynamic-panel GMM,
instrument matrices and lag construction, weighting, covariance, missingness,
prediction, post-estimation state, labels, formatting, CLI, JSON, MCP, and
broad Python `xtabond` parity deferred. Phase 11.1 runtime `xtabond` remains
unchecked.

## Verified slice: bounded tokenizer API

Merged [PR #68](https://github.com/SaehwanPark/tabdat-explore-rs/pull/68)
([45da1ac](https://github.com/SaehwanPark/tabdat-explore-rs/commit/45da1ac86b5a7194bf020417fd11d2a540e78e1e))
adds an owned, backend-independent tokenizer API to `tabdat-language`.
`TokenKind`, `Token`, and `tokenize` recover the pinned Python `_tokenize`
behavior for identifiers, backtick identifiers, strings, numbers, two-character
operators, one-character symbols, Unicode-scalar offsets, and bounded lexical
diagnostics. Existing option/expression token consumers delegate through the
shared tokenizer.

This is a bounded lexical slice, not a complete parser. Command-specific
`parse_simple_body` grammar, varlists, option parsing, `if` clauses, expression
AST/precedence, prefixed commands, scripts, runtime execution, data behavior,
and statistical behavior remain deferred. The recovered quoted-string offset
quirk is documented and tested rather than normalized away.

Evidence: [_workspace/parser-tokenizer/](_workspace/parser-tokenizer/),
including the [migration evidence](_workspace/parser-tokenizer/02-evidence-migration.md),
[review](_workspace/parser-tokenizer/03-review.md), and
[summary](_workspace/parser-tokenizer/04-summary.md). The focused oracle,
locked Rust and policy checks, PR-head workflows, squash merge, and merge-head
workflows all passed.

## Verified slice: syntax-only `ttest` command

Merged [PR #69](https://github.com/SaehwanPark/tabdat-explore-rs/pull/69)
([4dcd892](https://github.com/SaehwanPark/tabdat-explore-rs/commit/4dcd892f59422774e49742019e5f4eefb459711b))
adds a bounded, backend-independent `ttest` syntax boundary to
`tabdat-language`. The parser returns an owned `TtestCommand` for
`ttest <var> == <numeric-value>`, `ttest <var> == <other-var>`, and
`ttest <var>, by(<group-var>)` with flag-only `welch`/`unequal` options. It
preserves the pinned diagnostics, decoded quoted names, repeated alias flags,
and source numeric spelling. Runtime deliberately returns the typed unsupported
command result; it does not inspect a relation, fit a model, calculate
inference, initialize DuckDB, or mutate session state.

Evidence: [_workspace/parser-ttest/](_workspace/parser-ttest/), including the
[migration evidence](_workspace/parser-ttest/02-evidence-migration.md),
[review](_workspace/parser-ttest/03-review.md), and
[summary](_workspace/parser-ttest/04-summary.md). The pinned oracle probes,
locked Rust and policy checks, PR-head workflows, squash merge, and merge-head
workflows all passed.

This accepted syntax slice leaves numeric conversion, type and missingness
semantics, sample construction, inference, covariance, post-estimation state,
reporting, CLI, JSON, MCP, and broad Python statistical parity deferred. The
Phase 7 statistical `ttest` item remains unchecked.

## Verified slice: syntax-only `sql` command

Merged [PR #77](https://github.com/SaehwanPark/tabdat-explore-rs/pull/77)
([feeeaf4](https://github.com/SaehwanPark/tabdat-explore-rs/commit/feeeaf4))
adds a bounded, backend-independent `sql` syntax boundary to
`tabdat-language`. The parser returns an owned `SqlCommand` for direct and
triple-quoted queries (`"""..."""`), with optional trailing case-insensitive
`into <table>` clauses. It validates table name identifiers and reserved names
(`active`, `__tabdat_*`), preserves query text opaquely, and reports exact
Python-compatible diagnostics. Runtime deliberately returns the typed unsupported
command result; it does not execute SQL, initialize DuckDB, or mutate session
state.

Evidence: [_workspace/parser-sql-syntax/](_workspace/parser-sql-syntax/),
including the
[migration evidence](_workspace/parser-sql-syntax/02-evidence-migration.md),
[review](_workspace/parser-sql-syntax/03-review.md), and
[summary](_workspace/parser-sql-syntax/04-summary.md). The pinned oracle probes,
locked Rust and policy checks, PR-head workflows, squash merge, and merge-head
workflows all passed.

This accepted syntax slice leaves multiline script block grouping, SQL
execution, named-table lifecycle, schema/type validation, reporting, CLI, JSON,
MCP, and broad Python database parity deferred. The Phase 6.5 runtime `sql` item
remains unchecked.

## Verified slice: syntax-only `regress` command

Merged [PR #79](https://github.com/SaehwanPark/tabdat-explore-rs/pull/79)
([e6b81d3](https://github.com/SaehwanPark/tabdat-explore-rs/commit/e6b81d3))
adds a bounded, backend-independent `regress` syntax boundary to
`tabdat-language`. The parser returns an owned `RegressCommand` with typed
`RegressEstimator` (`ols`, `wls`, `gls`), ordered predictors, optional weight
variable, robust covariance flag, optional cluster variable, and intercept
inclusion flag. It validates mutual exclusions (`robust` vs `cluster`, `wls` vs
`gls`), option single-use rules, variable count constraints, and flag option values,
reporting exact Python-compatible diagnostics. Runtime deliberately returns the
typed unsupported command result; it does not perform estimation, initialize backends,
or mutate model state.

Evidence: [_workspace/parser-regress-syntax/](_workspace/parser-regress-syntax/),
including the [contract](_workspace/parser-regress-syntax/01-contract.md) and
[summary](_workspace/parser-regress-syntax/04-summary.md). The pinned oracle probes,
locked Rust and policy checks, PR-head workflows, squash merge, and merge-head
workflows all passed.

This accepted syntax slice leaves statistical estimation, linear algebra, FFI
backends, model results, post-estimation, reporting, CLI, JSON, MCP, and broad
estimator parity deferred. Runtime linear regression execution (OLS, WLS, GLS,
robust, and cluster covariance) is implemented in PR #151; broader post-estimation,
reporting, CLI, JSON, and MCP remain deferred.

## Verified slice: syntax-only `logit` and `probit` commands

Merged [PR #81](https://github.com/SaehwanPark/tabdat-explore-rs/pull/81)
([a14d552](https://github.com/SaehwanPark/tabdat-explore-rs/commit/a14d552))
adds bounded, backend-independent `logit` and `probit` syntax boundaries to
`tabdat-language`. The parser returns owned `LogitCommand` and `ProbitCommand`
types with ordered predictors, robust covariance flag, optional cluster
variable, and intercept inclusion flag. It validates mutual exclusions
(`robust` vs `cluster`), single-variable requirement for `cluster`, option
single-use rules, variable count constraints, and flag option values,
reporting exact Python-compatible diagnostics. Runtime deliberately returns
the typed unsupported command result; it does not perform estimation,
initialize backends, or mutate model state.

Evidence: [_workspace/parser-logit-probit-syntax/](_workspace/parser-logit-probit-syntax/),
including the [contract](_workspace/parser-logit-probit-syntax/01-contract.md) and
[summary](_workspace/parser-logit-probit-syntax/04-summary.md). The pinned oracle probes,
locked Rust and policy checks, PR-head workflows, squash merge, and merge-head
workflows all passed.

This accepted syntax slice leaves statistical estimation, optimization algorithms,
link functions, FFI backends, model results, post-estimation, reporting, CLI,
JSON, MCP, and broad binary response estimator parity deferred. The Phase 7.2
binary model estimation items remain unchecked.

## Verified slice: syntax-only `bayes:` prefixed command

Merged [PR #83](https://github.com/SaehwanPark/tabdat-explore-rs/pull/83)
([9e2ba7c](https://github.com/SaehwanPark/tabdat-explore-rs/commit/9e2ba7c))
adds a bounded, backend-independent `bayes:` prefixed command syntax boundary
to `tabdat-language`. The parser returns an owned `BayesPrefixCommand` holding
an inner estimation command (strictly validated to be `Command::Regress` or
`Command::Logit`), MCMC parameters `draws`, `burnin` (with alias `tune`),
`chains`, `thin`, `seed` (with alias `rseed`), and ordered custom prior
specifications `Vec<(String, String)>`. It supports complex parenthesized
expressions such as `prior(x, normal(0, 10))` and backtick-quoted identifiers,
while enforcing exact Python-compatible diagnostics for missing commands,
malformed prefix options, non-numeric values, unsupported options, and
unsupported inner commands. Runtime deliberately returns the typed
unsupported-command result; it does not perform MCMC sampling, initialize
backends, or mutate model state.

Evidence: [_workspace/parser-bayes-prefix-syntax/](_workspace/parser-bayes-prefix-syntax/),
including the [contract](_workspace/parser-bayes-prefix-syntax/01-contract.md) and
[summary](_workspace/parser-bayes-prefix-syntax/04-summary.md). The pinned oracle probes,
locked Rust and policy checks, PR-head workflows, squash merge, and merge-head
workflows all passed.

This accepted syntax slice leaves MCMC sampling engines, posterior storage,
distribution calculus, model results, post-estimation, reporting, CLI, JSON,
MCP, and broad Bayesian model family parity deferred. The Phase 7.3 Bayesian
estimation items remain unchecked.

## Verified slice: syntax-only `poisson` and `nbreg` commands

Merged [PR #85](https://github.com/SaehwanPark/tabdat-explore-rs/pull/85)
([4244936](https://github.com/SaehwanPark/tabdat-explore-rs/commit/4244936))
adds bounded, backend-independent `poisson` and `nbreg` syntax boundaries to
`tabdat-language`. The parser returns owned `PoissonCommand` and `NbregCommand`
types with ordered predictors, robust covariance flag, optional cluster
variable, and intercept inclusion flag. It validates mutual exclusions
(`robust` vs `cluster`), single-variable requirement for `cluster`, option
single-use rules, variable count constraints, and flag option values,
reporting exact Python-compatible diagnostics. Runtime deliberately returns
the typed unsupported command result; it does not perform estimation,
initialize backends, or mutate model state.

Evidence: [_workspace/parser-poisson-nbreg-syntax/](_workspace/parser-poisson-nbreg-syntax/),
including the [contract](_workspace/parser-poisson-nbreg-syntax/01-contract.md) and
[summary](_workspace/parser-poisson-nbreg-syntax/04-summary.md). The pinned oracle probes,
locked Rust and policy checks, PR-head workflows, squash merge, and merge-head
workflows all passed.

This accepted syntax slice leaves statistical estimation, numerical optimization,
dispersion parameter estimation, FFI backends, model results, post-estimation,
reporting, CLI, JSON, MCP, and broad count model estimator parity deferred.

## Verified slice: syntax-only `zip` and `zinb` commands

Merged [PR #87](https://github.com/SaehwanPark/tabdat-explore-rs/pull/87)
([78cf4d8](https://github.com/SaehwanPark/tabdat-explore-rs/commit/78cf4d8))
adds bounded, backend-independent `zip` and `zinb` syntax boundaries to
`tabdat-language`. The parser returns owned `ZipCommand` and `ZinbCommand`
types with ordered predictors, ordered zero-inflation predictors (`inflate_predictors`),
robust covariance flag, optional cluster variable, and intercept inclusion flag.
It validates syntax structure, mandatory `inflate(<zvars>)`, mutual exclusions
(`robust` vs `cluster`), single-variable requirement for `cluster`, option
single-use rules, variable count constraints, and flag option values,
reporting exact Python-compatible diagnostics. Runtime deliberately returns
the typed unsupported command result; it does not perform estimation,
initialize backends, or mutate model state.

Evidence: [_workspace/parser-zip-zinb-syntax/](_workspace/parser-zip-zinb-syntax/),
including the [contract](_workspace/parser-zip-zinb-syntax/01-contract.md) and
[summary](_workspace/parser-zip-zinb-syntax/04-summary.md). The pinned oracle probes,
locked Rust and policy checks, PR-head workflows, squash merge, and merge-head
workflows all passed.

This accepted syntax slice leaves statistical estimation, numerical optimization,
zero-inflation parameter estimation, FFI backends, model results, post-estimation,
reporting, CLI, JSON, MCP, and broad zero-inflated count model estimator parity deferred.

## Verified slice: syntax-only `qreg` command

Merged [PR #89](https://github.com/SaehwanPark/tabdat-explore-rs/pull/89)
([cc6e656](https://github.com/SaehwanPark/tabdat-explore-rs/commit/cc6e656))
adds bounded, backend-independent `qreg` syntax boundaries to
`tabdat-language`. The parser returns an owned `QregCommand` type with
ordered predictors, quantile numeric string spelling (default `"0.5"`),
robust covariance flag, and intercept inclusion flag. It validates syntax
structure, quantile numeric bounds (`0 < quantile < 1`), option single-use
rules, variable count constraints, and flag option values, reporting exact
Python-compatible diagnostics. Numeric text for `quantile` remains an owned
`String` so that the public `Command` enum retains its `Eq` derive. Runtime
deliberately returns the typed unsupported command result; it does not perform
estimation, initialize backends, or mutate model state.

Evidence: [_workspace/parser-qreg-syntax/](_workspace/parser-qreg-syntax/),
including the [contract](_workspace/parser-qreg-syntax/01-contract.md) and
[summary](_workspace/parser-qreg-syntax/04-summary.md). The pinned oracle probes,
locked Rust and policy checks, PR-head workflows, squash merge, and merge-head
workflows all passed.

This accepted syntax slice leaves statistical estimation, linear programming,
quantile loss optimization, FFI backends, model results, post-estimation,
reporting, CLI, JSON, MCP, and broad quantile regression estimator parity deferred.

## Verified slice: syntax-only `tobit` command

Merged [PR #91](https://github.com/SaehwanPark/tabdat-explore-rs/pull/91)
([708e64f](https://github.com/SaehwanPark/tabdat-explore-rs/commit/708e64f))
adds bounded, backend-independent `tobit` syntax boundaries to
`tabdat-language`. The parser returns an owned `TobitCommand` type with
ordered predictors, required lower limit (`ll`), optional upper limit (`ul`),
robust covariance flag, cluster variable, and intercept inclusion flag. It validates
syntax structure, missing `ll` requirement, option single-use rules, cluster variable
count constraints, mutual exclusivity of `robust` and `cluster`, numeric parsing,
and flag option values, reporting exact Python-compatible diagnostics. Numeric text
for `lower_limit` and `upper_limit` remains an owned `String` so that the public
`Command` enum retains its `Eq` derive. Runtime deliberately returns the typed
unsupported command result; it does not perform estimation, initialize backends,
or mutate model state.

Evidence: [_workspace/parser-tobit-syntax/](_workspace/parser-tobit-syntax/),
including the [contract](_workspace/parser-tobit-syntax/01-contract.md) and
[summary](_workspace/parser-tobit-syntax/04-summary.md). The pinned oracle probes,
locked Rust and policy checks, PR-head workflows, squash merge, and merge-head
workflows all passed.

This accepted syntax slice leaves statistical estimation, likelihood optimization,
FFI backends, model results, post-estimation, reporting, CLI, JSON, MCP, and broad
censored regression estimator parity deferred.

## Verified slice: syntax-only `heckman` command

Merged [PR #93](https://github.com/SaehwanPark/tabdat-explore-rs/pull/93)
([b0dad85](https://github.com/SaehwanPark/tabdat-explore-rs/commit/b0dad85))
adds bounded, backend-independent `heckman` syntax boundaries to
`tabdat-language`. The parser returns an owned `HeckmanCommand` type with
ordered predictors, required selection dependent variable (`selectdep`),
required selection predictors (`select`), robust covariance flag, cluster
variable, and intercept inclusion flag. It validates syntax structure,
missing `selectdep` and `select` requirements, option single-use rules, cluster
variable count constraints, mutual exclusivity of `robust` and `cluster`,
and flag option values, reporting exact Python-compatible diagnostics. Owned
`String` and `Vec<String>` types are used so that the public `Command` enum retains
its `Eq` derive. Runtime deliberately returns the typed unsupported command result;
it does not perform estimation, initialize backends, or mutate model state.

Evidence: [_workspace/parser-heckman-syntax/](_workspace/parser-heckman-syntax/),
including the [contract](_workspace/parser-heckman-syntax/01-contract.md) and
[summary](_workspace/parser-heckman-syntax/04-summary.md). The pinned oracle probes,
locked Rust and policy checks, PR-head workflows, squash merge, and merge-head
workflows all passed.

This accepted syntax slice leaves statistical estimation, sample-selection
likelihood optimization, two-step estimation, FFI backends, model results,
post-estimation, reporting, CLI, JSON, MCP, and broad sample-selection regression
estimator parity deferred.

## Verified slice: syntax-only `nl` command

Merged [PR #95](https://github.com/SaehwanPark/tabdat-explore-rs/pull/95)
([b0077ff](https://github.com/SaehwanPark/tabdat-explore-rs/commit/b0077ff))
adds bounded, backend-independent `nl` syntax boundaries to
`tabdat-language`. The parser returns an owned `NlCommand` type with
an outcome variable, parsed expression tree (`GenerateExpression`), required
parameter names (`params`), required starting values (`start`), robust
covariance flag, and intercept inclusion flag. It validates syntax structure,
missing expression after `=`, missing target before `=`, duplicate `if` clause,
missing `params` and `start` requirements, duplicate parameter names,
start value count matching parameter count, numeric start value formatting,
option single-use rules, unsupported options, and flag option values, reporting
exact Python-compatible diagnostics. Owned `String` and `Vec<String>` types are
used so that the public `Command` enum retains its `Eq` derive. Runtime
deliberately returns the typed unsupported command result; it does not perform
estimation, initialize backends, or mutate model state.

Evidence: [_workspace/parser-nl-syntax/](_workspace/parser-nl-syntax/),
including the [contract](_workspace/parser-nl-syntax/01-contract.md) and
[summary](_workspace/parser-nl-syntax/04-summary.md). The pinned oracle probes,
locked Rust and policy checks, PR-head workflows, squash merge, and merge-head
workflows all passed.

This accepted syntax slice leaves statistical estimation, nonlinear least squares
optimization, Gauss-Newton / Levenberg-Marquardt solvers, FFI backends, model
results, post-estimation, reporting, CLI, JSON, MCP, and broad nonlinear
regression estimator parity deferred.

## Verified slice: syntax-only `streg` command

Merged [PR #97](https://github.com/SaehwanPark/tabdat-explore-rs/pull/97)
([735e6a1](https://github.com/SaehwanPark/tabdat-explore-rs/commit/735e6a1))
adds bounded, backend-independent `streg` syntax boundaries to
`tabdat-language`. The parser returns an owned `StregCommand` type with
a survival time variable, ordered predictor list, required failure indicator
variable (`failure`), required baseline distribution (`dist`, holding typed
`StregDistribution`), robust covariance flag, single cluster variable, and
intercept inclusion flag. It validates syntax structure, missing arguments,
conditions and assignment syntax, missing required `failure` and `dist` options,
case-insensitive `dist` values (`weibull` or `exponential`), option single-use
rules, cluster variable count constraints, mutual exclusivity of `robust` and
`cluster`, unsupported options, and flag option values, reporting exact
Python-compatible diagnostics. Owned `String` and `Vec<String>` types are used
so that the public `Command` enum retains its `Eq` derive. Runtime deliberately
returns the typed unsupported command result; it does not perform estimation,
initialize backends, or mutate model state.

Evidence: [_workspace/parser-streg-syntax/](_workspace/parser-streg-syntax/),
including the [contract](_workspace/parser-streg-syntax/01-contract.md) and
[summary](_workspace/parser-streg-syntax/04-summary.md). The pinned oracle probes,
locked Rust and policy checks, PR-head workflows, squash merge, and merge-head
workflows all passed.

This accepted syntax slice leaves statistical estimation, parametric survival
distribution fitting, maximum likelihood optimization, FFI backends, model
results, post-estimation, reporting, CLI, JSON, MCP, and broad survival regression
estimator parity deferred.

## Verified slice: syntax-only `spregress` command

Merged [PR #99](https://github.com/SaehwanPark/tabdat-explore-rs/pull/99)
([eaee45c](https://github.com/SaehwanPark/tabdat-explore-rs/commit/eaee45c))
adds bounded, backend-independent `spregress` syntax boundaries to
`tabdat-language`. The parser returns an owned `SpregressCommand` type with
a dependent outcome variable, ordered predictor list, spatial model specification
(`model`, holding typed `SpregressModelType`), coordinate variables (`coord`),
nearest-neighbor count (`knn`), external spatial weights path (`weights`),
required ID variable (`id`), spatial contiguity criterion (`contiguity`, holding
typed `SpregressContiguity`), and robust covariance flag. It validates syntax
structure, missing arguments, conditions and assignment syntax, missing spatial
specifications, mutual exclusivity of `coord` and `weights`, option compatibility
rules (`id`/`contiguity` only with `weights`; `knn` only with `coord`), single-use
rules, unsupported options, flag option values, and punctuation guards, reporting
exact Python-compatible diagnostics. Owned `String` and `Vec<String>` types are used
so that the public `Command` enum retains its `Eq` derive. Runtime deliberately
returns the typed unsupported command result; it does not perform estimation,
initialize backends, or mutate model state.

Evidence: [_workspace/parser-spregress-syntax/](_workspace/parser-spregress-syntax/),
including the [contract](_workspace/parser-spregress-syntax/01-contract.md) and
[summary](_workspace/parser-spregress-syntax/04-summary.md). The pinned oracle probes,
locked Rust and policy checks, PR-head workflows, squash merge, and merge-head
workflows all passed.

This accepted syntax slice leaves statistical estimation, spatial weight matrix
construction (k-NN / PySAL / Shapefile), 2SLS / GM / SARAR estimation, FFI backends,
model results, post-estimation, reporting, CLI, JSON, MCP, and broad spatial regression
estimator parity deferred.

## Verified slice: syntax-only regularized regression commands (lasso, postlasso, ridge, elasticnet)

Merged [PR #101](https://github.com/SaehwanPark/tabdat-explore-rs/pull/101)
([477e815](https://github.com/SaehwanPark/tabdat-explore-rs/commit/477e815))
adds bounded, backend-independent regularized linear regression syntax boundaries to
`tabdat-language`. The parser returns owned `LassoCommand`, `PostlassoCommand`, `RidgeCommand`,
and `ElasticnetCommand` types with a dependent outcome variable, ordered predictor list,
regularization penalty parameter (`alpha`, defaulting to `"1.0"` and validated positive),
elastic net mixing parameter (`l1_ratio` for elastic net, defaulting to `"0.5"` and validated
in `[0.0, 1.0]`), robust covariance flag (`robust` on postlasso), and intercept inclusion flag
(`noconstant`). It validates syntax structure, requiring the unquoted `linear` model specifier,
at least one predictor variable, rejects conditions and assignment syntax, single-use rules,
unsupported options, flag option values, numeric ranges, and punctuation guards, reporting
exact Python-compatible diagnostics. Owned `String` and `Vec<String>` types are used
so that the public `Command` enum retains its `Eq` derive. Runtime deliberately returns the
typed unsupported command result; it does not perform estimation, initialize backends,
or mutate model state.

Evidence: [_workspace/parser-regularized-regression-syntax/](_workspace/parser-regularized-regression-syntax/),
including the [contract](_workspace/parser-regularized-regression-syntax/01-contract.md) and
[summary](_workspace/parser-regularized-regression-syntax/04-summary.md). The pinned oracle probes,
locked Rust and policy checks, PR-head workflows, squash merge, and merge-head
workflows all passed.

This accepted syntax slice leaves statistical estimation, coordinate descent / proximal gradient
optimization, post-lasso refitting, cross-validation search, FFI backends, model results,
post-estimation, reporting, CLI, JSON, MCP, and broad regularized regression estimator parity deferred.

## Verified slice: syntax-only cross-validation regularized regression commands (cvlasso, cvridge, cvelasticnet)

Merged [PR #103](https://github.com/SaehwanPark/tabdat-explore-rs/pull/103)
([cc08faa](https://github.com/SaehwanPark/tabdat-explore-rs/commit/cc08faa))
adds bounded, backend-independent cross-validation regularized linear regression syntax boundaries to
`tabdat-language`. The parser returns owned `CvlassoCommand`, `CvridgeCommand`, `CvelasticnetL1Ratio`,
and `CvelasticnetCommand` types with a dependent outcome variable, ordered predictor list,
cross-validation folds (`cv`, defaulting to `5` and validated integer >= 2), elastic net mixing
parameter (`l1_ratio` for cvelasticnet, defaulting to `(0.1, 0.5, 0.7, 0.9, 0.95, 0.99, 1.0)` and
validated in `[0.0, 1.0]`), and intercept inclusion flag (`noconstant`). It validates syntax structure,
requiring the unquoted `linear` model specifier, at least one predictor variable, rejects conditions
and assignment syntax, single-use rules, unsupported options, flag option values, numeric ranges, and
punctuation guards, reporting exact Python-compatible diagnostics. Owned `String`, `Vec<String>`, and
`i64` types are used so that the public `Command` enum retains its `Eq` derive. Runtime deliberately
returns the typed unsupported command result; it does not perform estimation, initialize backends,
or mutate model state.

Evidence: [_workspace/parser-cv-regularized-regression-syntax/](_workspace/parser-cv-regularized-regression-syntax/),
including the [contract](_workspace/parser-cv-regularized-regression-syntax/01-contract.md) and
[summary](_workspace/parser-cv-regularized-regression-syntax/04-summary.md). The pinned oracle probes,
locked Rust and policy checks, PR-head workflows, squash merge, and merge-head
workflows all passed.

This accepted syntax slice leaves statistical estimation, coordinate descent / scikit-learn optimization,
K-fold splitting, cross-validation grid search, report generation, FFI backends, model results,
post-estimation, reporting, CLI, JSON, MCP, and broad cross-validation regularized regression estimator
parity deferred.

## Verified slice: syntax-only direct Bayesian linear regression command (bayes linear)

Merged [PR #105](https://github.com/SaehwanPark/tabdat-explore-rs/pull/105)
([79f0bc4](https://github.com/SaehwanPark/tabdat-explore-rs/commit/79f0bc49c65972a1f4e31c6174f7c6e250560b88))
adds bounded, backend-independent direct Bayesian linear regression syntax boundaries to
`tabdat-language`. The parser returns an owned `BayesCommand` type with a dependent outcome variable,
ordered predictor list, maximum iteration count (`n_iter`, defaulting to `300` and validated integer >= 1),
convergence tolerance (`tol`, defaulting to `"0.001"` and validated positive finite float), and intercept
inclusion flag (`noconstant`). It validates syntax structure, requiring the unquoted `linear` model
specifier, at least one predictor variable, rejects conditions and assignment syntax, single-use rules,
unsupported options, flag option values, numeric ranges, and delimiter guards, while preserving prefix
disambiguation (`bayes:`). Owned `String`, `Vec<String>`, and `i64` types are used so that the public
`Command` enum retains its `Eq` derive. Runtime deliberately returns the typed unsupported command
result; it does not perform estimation, initialize backends, or mutate model state.

Evidence: [_workspace/parser-bayes-linear-syntax/](_workspace/parser-bayes-linear-syntax/),
including the [contract](_workspace/parser-bayes-linear-syntax/01-contract.md) and
[summary](_workspace/parser-bayes-linear-syntax/04-summary.md). The pinned oracle probes,
locked Rust and policy checks, PR-head workflows, squash merge, and merge-head
workflows all passed.

This accepted syntax slice leaves statistical estimation, Bayesian evidence maximization / conjugate
linear regression, scikit-learn BayesianRidge optimization, FFI backends, model results, post-estimation,
reporting, CLI, JSON, MCP, and broad Bayesian linear regression estimator parity deferred.

## Verified slice: syntax-only direct predict post-estimation command

Merged [PR #107](https://github.com/SaehwanPark/tabdat-explore-rs/pull/107)
([aba64a8](https://github.com/SaehwanPark/tabdat-explore-rs/commit/aba64a8a07f7c469fefb3d1b72e5dcce251df83e))
adds bounded, backend-independent direct post-estimation prediction syntax boundaries to
`tabdat-language`. The parser returns an owned `PredictCommand` type with a target variable name,
prediction kind (`Xb`, `Residuals`, `Pr`, `SpatialLag`, `PosteriorPredictive`, defaulting to `Xb`),
interval flag (`interval`, defaulting to `false`), credible interval level (`level`, defaulting to `"95.0"` and
validated `0 < level < 100`), standard deviation flag (`std`), and optional file destination path (`saving`).
It validates syntax structure, requiring exactly one target variable name, rejects conditions and assignment syntax,
single-use rules, unsupported options, flag option values, numeric ranges, and delimiter guards (`predict:`, `predict=`, `predict==`).
It validates interdependencies: `interval`, `level`, `std`, and `saving` require `posterior_predictive`; `level` requires `interval`;
`saving` cannot combine with `std` or `interval`. Owned `String` and enum types are used so that the public `Command`
enum retains its `Eq` derive. Runtime deliberately returns the typed unsupported command result; it does not perform
model scoring, initialize backends, lookup post-estimation state, or mutate datasets.

Evidence: [_workspace/parser-predict-syntax/](_workspace/parser-predict-syntax/),
including the [contract](_workspace/parser-predict-syntax/01-contract.md) and
[summary](_workspace/parser-predict-syntax/04-summary.md). The pinned oracle probes,
locked Rust and policy checks, PR-head workflows, squash merge, and merge-head
workflows all passed.

This accepted syntax slice leaves model scoring, post-estimation state lookup, dataset mutation,
DuckDB column generation, Parquet draws persistence, FFI backends, reporting, CLI, JSON, MCP,
and broad post-estimation prediction execution parity deferred.

## Verified slice: syntax-only panel fixed-effects logit command

Merged [PR #109](https://github.com/SaehwanPark/tabdat-explore-rs/pull/109)
([c36d92a](https://github.com/SaehwanPark/tabdat-explore-rs/commit/c36d92a04860457d7aa1317fbdc67f8ded8e9217))
adds bounded, backend-independent panel fixed-effects logit syntax boundaries to
`tabdat-language`. The parser returns an owned `XtLogitCommand` type with a dependent variable (`outcome`),
ordered predictor variables (`predictors`), and a robust covariance flag (`robust`, defaulting to `false`).
It validates syntax structure, requiring at least 2 arguments (outcome and >= 1 predictors), rejects conditions
(`if ...`), assignment syntax (`xtlogit=`), delimiter guards (`xtlogit==`, `xtlogit:`), flag option values,
unsupported options, and requires the fixed-effects flag option `fe`. Owned `String` and `Vec<String>` types are used
so that the public `Command` enum retains its `Eq` derive. Runtime deliberately returns the typed unsupported command
result; it does not perform conditional logit optimization, compute standard errors, or mutate datasets.

Evidence: [_workspace/parser-xtlogit-syntax/](_workspace/parser-xtlogit-syntax/),
including the [contract](_workspace/parser-xtlogit-syntax/01-contract.md) and
[summary](_workspace/parser-xtlogit-syntax/04-summary.md). The pinned oracle probes,
locked Rust and policy checks, PR-head workflows, squash merge, and merge-head
workflows all passed.

This accepted syntax slice leaves conditional logit estimation, panel grouping validation,
numerical optimization, robust covariance estimation, FFI backends, reporting, CLI, JSON, MCP,
and broad panel logit execution parity deferred.

## Verified slice: syntax-only locally weighted regression smoother command

Merged [PR #111](https://github.com/SaehwanPark/tabdat-explore-rs/pull/111)
([d188b33](https://github.com/SaehwanPark/tabdat-explore-rs/commit/d188b338cb88f21950e8d048ba8c3e8a4a580663))
adds bounded, backend-independent locally weighted regression smoothing syntax boundaries to
`tabdat-language`. The parser returns an owned `LowessCommand` type with a dependent variable (`outcome`),
single predictor variable (`predictor`), target smoothed variable name (`target_variable`), and smoothing bandwidth
(`bandwidth`, defaulting to `"0.6666666666666666"`).
It validates syntax structure, requiring strictly 2 arguments (outcome and 1 predictor), rejects conditions
(`if ...`), assignment syntax (`lowess=`), delimiter guards (`lowess==`, `lowess:`), required option `gen(<newvar>)`
with strictly 1 variable name, single-use rules, unsupported options, and validates smoothing bandwidth strictly
between 0 and 1 exclusive. Owned `String` types are used so that the public `Command` enum retains its `Eq` derive.
Runtime deliberately returns the typed unsupported command result; it does not perform non-parametric smoothing,
evaluate kernel weights, or mutate datasets.

Evidence: [_workspace/parser-lowess-syntax/](_workspace/parser-lowess-syntax/),
including the [contract](_workspace/parser-lowess-syntax/01-contract.md) and
[summary](_workspace/parser-lowess-syntax/04-summary.md). The pinned oracle probes,
locked Rust and policy checks, PR-head workflows, squash merge, and merge-head
workflows all passed.

This accepted syntax slice leaves non-parametric regression smoothing, tricube kernel weighting,
local polynomial fitting, FFI backends, reporting, CLI, JSON, MCP, and broad smoothing execution parity deferred.

## Verified slice: syntax-only difference-in-differences command

Merged [PR #113](https://github.com/SaehwanPark/tabdat-explore-rs/pull/113)
([5c6a469](https://github.com/SaehwanPark/tabdat-explore-rs/commit/5c6a469ed17ecbb2718e811ce7ce56ef84497a13))
adds bounded, backend-independent difference-in-differences syntax boundaries to
`tabdat-language`. The parser returns an owned `DidCommand` type with a dependent variable (`outcome`),
optional control variables (`controls`), treatment indicator variable (`treatment_variable`),
post-treatment time period indicator variable (`post_variable`), and robust covariance flag (`robust`).
It validates syntax structure, requiring at least 1 argument (outcome and optional controls), rejects conditions
(`if ...`), assignment syntax (`did=`), delimiter guards (`did==`, `did:`), required options `treat(<var>)` and
`post(<var>)` each expecting strictly 1 variable name, single-use rules, unsupported options, and enforces variable
relationship constraints (`treat != post`, `treat != outcome`, `post != outcome`, `treat not in controls`, `post not in controls`).
Owned `String` and `Vec<String>` types are used so that the public `Command` enum retains its `Eq` derive.
Runtime deliberately returns the typed unsupported command result; it does not perform two-way fixed effects estimation,
evaluate parallel trends, or mutate datasets.

Evidence: [_workspace/parser-did-syntax/](_workspace/parser-did-syntax/),
including the [contract](_workspace/parser-did-syntax/01-contract.md) and
[summary](_workspace/parser-did-syntax/04-summary.md). The pinned oracle probes,
locked Rust and policy checks, PR-head workflows, squash merge, and merge-head
workflows all passed.

This accepted syntax slice leaves two-way fixed effects estimation, parallel trends diagnostics,
interaction modeling, parameter inference, FFI backends, reporting, CLI, JSON, MCP, and broad causal execution parity deferred.

## Verified slice: syntax-only doubly robust difference-in-differences command

Merged [PR #115](https://github.com/SaehwanPark/tabdat-explore-rs/pull/115)
([063bc3b](https://github.com/SaehwanPark/tabdat-explore-rs/commit/063bc3b940989f64bf74d75432616231d6d84877))
adds bounded, backend-independent doubly robust difference-in-differences syntax boundaries to
`tabdat-language`. The parser returns an owned `DrDidCommand` type with a dependent variable (`outcome`),
optional covariate variables (`covariates`), treatment indicator variable (`treatment_variable`),
post-treatment time period indicator variable (`post_variable`), estimation method enum (`method`: `Or`, `Ipw`, `Aipw`),
robust covariance flag (`robust`), bootstrap replications count (`bootstrap: Option<i64>`), and random seed
(`seed: Option<i64>`).
It validates syntax structure, requiring at least 1 argument (outcome and optional covariates), rejects conditions
(`if ...`), assignment syntax (`drdid=`), delimiter guards (`drdid==`, `drdid:`), required options `treat(<var>)` and
`post(<var>)` each expecting strictly 1 variable name, method enum validation (`or`, `ipw`, `aipw`, defaulting to `aipw`),
numeric bounds for bootstrap ($\ge 1$) and seed ($\ge 0$), seed dependency on bootstrap, single-use rules, unsupported options,
and enforces variable relationship constraints (`treat != post`, `treat != outcome`, `post != outcome`, `treat not in covariates`,
`post not in covariates`).
Owned `String`, `Vec<String>`, and enum/primitive types are used so that the public `Command` enum retains its `Eq` derive.
Runtime deliberately returns the typed unsupported command result; it does not perform propensity score estimation,
evaluate outcome regressions, run bootstrap iterations, or mutate datasets.

Evidence: [_workspace/parser-drdid-syntax/](_workspace/parser-drdid-syntax/),
including the [contract](_workspace/parser-drdid-syntax/01-contract.md) and
[summary](_workspace/parser-drdid-syntax/04-summary.md). The pinned oracle probes,
locked Rust and policy checks, PR-head workflows, squash merge, and merge-head
workflows all passed.

This accepted syntax slice leaves doubly robust estimation, propensity score modeling, outcome regression,
bootstrapping, statistical validation, FFI backends, reporting, CLI, JSON, MCP, and broad causal execution parity deferred.

## Verified slice: syntax-only double machine learning command

Merged [PR #117](https://github.com/SaehwanPark/tabdat-explore-rs/pull/117)
([f2e537c](https://github.com/SaehwanPark/tabdat-explore-rs/commit/f2e537c44ea092a95c4794e7df6504a794cb1f0e))
adds bounded, backend-independent double machine learning syntax boundaries to
`tabdat-language`. The parser returns an owned `DmlCommand` type with a dependent variable (`outcome`),
control variables (`controls`), treatment indicator variable (`treatment_variable`),
cross-fitting folds count (`folds`, defaulting to `5`), regularization penalty parameter (`alpha: String`,
retained as source spelling, defaulting to `"1.0"`), robust covariance flag (`robust`),
random seed (`seed: Option<i64>`), and intercept inclusion flag (`include_intercept: bool`).
It validates syntax structure, requiring at least 3 arguments (model identifier `linear`, outcome, and controls),
rejects non-linear models, rejects conditions (`if ...`), assignment syntax (`dml=`),
delimiter guards (`dml==`, `dml:`), required option `treat(<var>)` expecting strictly 1 variable name,
numeric bounds for folds ($\ge 2$), alpha ($> 0.0$), and seed ($\ge 0$), single-use rules, unsupported options,
and enforces variable relationship constraints (`treat != outcome`, `treat not in controls`).
Owned `String`, `Vec<String>`, and primitive types are used so that the public `Command` enum retains its `Eq` derive.
Runtime deliberately returns the typed unsupported command result; it does not perform cross-fitting,
regularized nuisance estimation, Neyman-orthogonal scoring, or mutate datasets.

Evidence: [_workspace/parser-dml-syntax/](_workspace/parser-dml-syntax/),
including the [contract](_workspace/parser-dml-syntax/01-contract.md) and
[summary](_workspace/parser-dml-syntax/04-summary.md). The pinned oracle probes,
locked Rust and policy checks, PR-head workflows, squash merge, and merge-head
workflows all passed.

This accepted syntax slice leaves double machine learning estimation, cross-fitting nuisance modeling,
Lasso regularized regressions, score evaluation, statistical validation, FFI backends, reporting, CLI,
JSON, MCP, and broad causal execution parity deferred.

## Verified slice: syntax-only control function regression command

Merged [PR #119](https://github.com/SaehwanPark/tabdat-explore-rs/pull/119)
([b800fda](https://github.com/SaehwanPark/tabdat-explore-rs/commit/b800fda0610f438cb56d2e67a030ef5ea7aa4486))
adds bounded, backend-independent control function regression syntax boundaries to
`tabdat-language`. The parser returns an owned `CfRegressCommand` type with a dependent variable (`outcome`),
optional exogenous variables (`exogenous`), endogenous regressor variable (`endogenous`),
instrumental variables (`instruments`), robust covariance flag (`robust`), cluster variable
(`cluster_variable: Option<String>`), and intercept inclusion flag (`include_intercept: bool`).
It validates syntax structure, requiring at least 1 argument (outcome and optional exogenous variables),
rejects conditions (`if ...`), assignment syntax (`cfregress=`), delimiter guards (`cfregress==`, `cfregress:`),
required option `endog(<var>)` expecting strictly 1 variable name, required option `iv(<vars>)` expecting $\ge 1$
variable names, cluster variable arity, option conflict (disallowing combining `robust` and `cluster`), single-use rules,
unsupported options, and enforces variable relationship constraints (`endog not in exogenous`).
Owned `String`, `Vec<String>`, and primitive types are used so that the public `Command` enum retains its `Eq` derive.
Runtime deliberately returns the typed unsupported command result; it does not perform first-stage residual
calculation, second-stage control function augmentation, bootstrapped standard errors, or mutate datasets.

Evidence: [_workspace/parser-cfregress-syntax/](_workspace/parser-cfregress-syntax/),
including the [contract](_workspace/parser-cfregress-syntax/01-contract.md) and
[summary](_workspace/parser-cfregress-syntax/04-summary.md). The pinned oracle probes,
locked Rust and policy checks, PR-head workflows, squash merge, and merge-head
workflows all passed.

This accepted syntax slice leaves control function estimation, first-stage modeling, residual generation,
second-stage estimation, standard error corrections, statistical validation, FFI backends, reporting, CLI,
JSON, MCP, and broad causal execution parity deferred.

## Verified slice: post-estimation linear combination syntax

The post-estimation linear combination hypothesis testing syntax slice (Phase 9)
adds bounded, backend-independent linear combination testing syntax boundaries to
`tabdat-language`. The parser returns an owned `LincomCommand` type with a linear combination
expression (`expression: GenerateExpression`).
It validates expression syntax structure, requiring a non-empty expression,
rejects incomplete expressions (`incomplete expression after <op>`), missing closing parentheses
(`missing closing ) in expression`), unsupported tokens in expression (`unsupported token in expression: <token>`),
and delimiter guards (`lincom:`, `lincom=`, `lincom==`, `lincom,`).
Owned `String`, `GenerateExpression`, and primitive types are used so that the public `Command` enum retains its `Eq` derive.
Runtime deliberately returns the typed unsupported command result; it does not perform parameter retrieval,
symbolic differentiation, covariance matrix transformation, standard error computation, or hypothesis testing.

Evidence: [_workspace/parser-lincom-syntax/](_workspace/parser-lincom-syntax/),
including the [contract](_workspace/parser-lincom-syntax/01-contract.md) and
[summary](_workspace/parser-lincom-syntax/04-summary.md). The pinned oracle probes,
locked Rust and policy checks, PR-head workflows, squash merge, and merge-head
workflows all passed.

This accepted syntax slice leaves post-estimation parameter retrieval, covariance calculations,
inference statistics, p-values, confidence intervals, reporting, CLI, JSON, and MCP parity deferred.

## Verified slice: classical linear hypothesis testing syntax

The classical linear hypothesis testing syntax slice (Phase 9)
adds bounded, backend-independent linear hypothesis testing syntax boundaries to
`tabdat-language`. The parser returns an owned `TestCommand` type with a list of
linear constraints (`constraints: Vec<GenerateExpression>`).
It parses variable list testing (`test <varlist>`), single unparenthesized constraints
(`test lhs = rhs` or `test lhs == rhs`), and multiple parenthesized constraints (`test (c1) (c2)`).
Equalities are normalized to subtraction expressions (`lhs - rhs`), and bare identifiers are retained
as `GenerateExpression::Identifier`. It enforces exact Python-compatible diagnostics for empty command
bodies (`test command expects a list of variables or constraints`), unexpected tokens outside parentheses
(`test command: unexpected tokens outside parentheses`), mismatched parentheses (`test command: mismatched parentheses`),
empty constraints in parentheses (`test command: empty constraint inside parentheses`), multiple equals
(`test command: multiple '=' in a constraint` or `test command: multiple '=' in a single constraint (use parentheses for multiple constraints)`),
missing operands around equals (`test command: missing left-hand side of constraint` or `test command: missing right-hand side of constraint`),
malformed constraints (`test command: malformed constraint`), expected variable names in varlist mode (`test command: expected variable name, got '<token>'`),
and delimiter guards (`test:`, `test=`, `test==`, `test,`).
Owned `String`, `GenerateExpression`, and primitive types are used so that the public `Command` enum retains its `Eq` derive.
Runtime deliberately returns the typed unsupported command result; it does not perform parameter retrieval,
linear restriction matrix $R$ and vector $r$ construction, Wald/F/chi-squared test statistics, or p-value computation.

Evidence: [_workspace/parser-test-syntax/](_workspace/parser-test-syntax/),
including the [contract](_workspace/parser-test-syntax/01-contract.md) and
[summary](_workspace/parser-test-syntax/04-summary.md). The pinned oracle probes,
locked Rust and policy checks, PR-head workflows, squash merge, and merge-head
workflows all passed.

This accepted syntax slice leaves post-estimation parameter retrieval, restriction matrix construction,
Wald/F/chi-squared statistics, degrees of freedom, p-values, reporting, CLI, JSON, and MCP parity deferred.

## Verified slice: visualization histogram syntax

The visualization histogram syntax slice (Phase 7 §7.4)
adds bounded, backend-independent visualization histogram syntax boundaries to
`tabdat-language`. The parser returns an owned `HistogramCommand` type with
target variable (`variable: String`), optional bin count (`bins: Option<i64>`),
optional save path (`saving: Option<String>`), and open-in-viewer flag (`open_artifact: bool`, defaulting to `true` unless `noopen` is supplied).
It enforces exact Python-compatible diagnostics for missing or multiple variables (`histogram expects exactly one variable`),
if clauses and assignment syntax (`histogram does not accept if clauses or assignment syntax`), assignment missing target or expression
(`histogram assignment requires a target before =` and `histogram assignment requires an expression after =`),
attached colons and double equals (`unsupported token in command: :` and `unsupported token in command: ==`),
trailing commas without options (`comma must be followed by at least one option`), unsupported options (`histogram unsupported option: <sorted_opts>`),
flag option values (`histogram option noopen does not accept a value`), bins validation (`histogram option bins must be at least 1`,
`histogram option bins expects an integer value`, `histogram option bins may only be supplied once`), and saving validation
(`histogram option saving expects a path`, `histogram option saving may only be supplied once`).
Owned `String`, `Option<i64>`, and primitive types are used so that the public `Command` enum retains its `Eq` derive.
Runtime deliberately returns the typed unsupported command result; it does not perform DuckDB binned frequency aggregation,
plot SVG/PNG rendering, artifact filesystem emission, or browser/viewer interaction.

Evidence: [_workspace/parser-histogram-syntax/](_workspace/parser-histogram-syntax/),
including the [contract](_workspace/parser-histogram-syntax/01-contract.md) and
[summary](_workspace/parser-histogram-syntax/04-summary.md). The pinned oracle probes,
locked Rust and policy checks, PR-head workflows, squash merge, and merge-head
workflows all passed.

This accepted syntax slice leaves DuckDB binned frequency aggregation, plot SVG/PNG rendering,
artifact filesystem persistence, browser/viewer opening, reporting, CLI, JSON, and MCP parity deferred.

## Verified slice: visualization scatter plot syntax

The visualization scatter plot syntax slice (Phase 7 §7.4)
adds bounded, backend-independent visualization scatter plot syntax boundaries to
`tabdat-language`. The parser returns an owned `ScatterCommand` type with
target y-variable (`y_variable: String`), target x-variable (`x_variable: String`),
optional save path (`saving: Option<String>`), and open-in-viewer flag (`open_artifact: bool`, defaulting to `true` unless `noopen` is supplied).
It enforces exact Python-compatible diagnostics for missing or extraneous variables (`scatter expects syntax: scatter y_var x_var`),
if clauses and assignment syntax (`scatter does not accept if clauses or assignment syntax`), assignment missing target or expression
(`scatter assignment requires a target before =` and `scatter assignment requires an expression after =`),
attached colons and double equals (`unsupported token in command: :` and `unsupported token in command: ==`),
trailing commas without options (`comma must be followed by at least one option`), unsupported options (`scatter unsupported option: <sorted_opts>`),
flag option values (`scatter option noopen does not accept a value`), and saving validation
(`scatter option saving expects a path`, `scatter option saving may only be supplied once`).
Owned `String` and primitive types are used so that the public `Command` enum retains its `Eq` derive.
Runtime deliberately returns the typed unsupported command result; it does not perform DuckDB data extraction,
Vega-Lite spec generation, plot SVG/PNG rendering, artifact filesystem emission, or browser/viewer interaction.

Evidence: [_workspace/parser-scatter-syntax/](_workspace/parser-scatter-syntax/),
including the [contract](_workspace/parser-scatter-syntax/01-contract.md) and
[summary](_workspace/parser-scatter-syntax/04-summary.md). The pinned oracle probes,
locked Rust and policy checks, PR-head workflows, squash merge, and merge-head
workflows all passed.

This accepted syntax slice leaves DuckDB data extraction, Vega-Lite spec generation, plot SVG/PNG rendering,
artifact filesystem persistence, browser/viewer opening, reporting, CLI, JSON, and MCP parity deferred.

## Verified slice: visualization bar chart syntax

The visualization bar chart syntax slice (Phase 7 §7.4)
adds bounded, backend-independent visualization bar chart syntax boundaries to
`tabdat-language`. The parser returns an owned `BarCommand` type with
target variable (`variable: String`),
optional save path (`saving: Option<String>`), flag to include missing values as a category (`include_missing: bool`, defaulting to `false` unless `missing` is supplied),
and open-in-viewer flag (`open_artifact: bool`, defaulting to `true` unless `noopen` is supplied).
It enforces exact Python-compatible diagnostics for missing or multiple variables (`bar expects exactly one variable`),
if clauses and assignment syntax (`bar does not accept if clauses or assignment syntax`), assignment missing target or expression
(`bar assignment requires a target before =` and `bar assignment requires an expression after =`),
attached colons and double equals (`unsupported token in command: :` and `unsupported token in command: ==`),
trailing commas without options (`comma must be followed by at least one option`), unsupported options (`bar unsupported option: <sorted_opts>`),
flag option values (`bar option missing does not accept a value`, `bar option noopen does not accept a value`), and saving validation
(`bar option saving expects a path`, `bar option saving may only be supplied once`).
Owned `String` and primitive types are used so that the public `Command` enum retains its `Eq` derive.
Runtime deliberately returns the typed unsupported command result; it does not perform DuckDB category aggregation,
Vega-Lite spec generation, plot SVG/PNG rendering, artifact filesystem emission, or browser/viewer interaction.

Evidence: [_workspace/parser-bar-syntax/](_workspace/parser-bar-syntax/),
including the [contract](_workspace/parser-bar-syntax/01-contract.md) and
[summary](_workspace/parser-bar-syntax/04-summary.md). The pinned oracle probes,
locked Rust and policy checks, PR-head workflows, squash merge, and merge-head
workflows all passed.

This accepted syntax slice leaves DuckDB category aggregation, Vega-Lite spec generation, plot SVG/PNG rendering,
artifact filesystem persistence, browser/viewer opening, reporting, CLI, JSON, and MCP parity deferred.

## Verified slice: visualization bayesplot syntax

The visualization bayesplot syntax slice (Phase 7 §7.4)
adds bounded, backend-independent visualization bayesplot syntax boundaries to
`tabdat-language`. The parser returns an owned `BayesPlotCommand` type with
target diagnostic plot kind (`kind: BayesPlotKind`),
optional save path (`saving: Option<String>`),
and open-in-viewer flag (`open_artifact: bool`, defaulting to `true` unless `noopen` is supplied).
It enforces exact Python-compatible diagnostics for missing or multiple arguments (`bayesplot expects syntax: bayesplot <trace|density|autocorrelation>`),
unrecognized or uppercase plot kinds (`bayesplot kind must be trace, density, or autocorrelation`),
if clauses and assignment syntax (`bayesplot does not accept if clauses or assignment syntax`), assignment missing target or expression
(`bayesplot assignment requires a target before =` and `bayesplot assignment requires an expression after =`),
attached colons and double equals (`unsupported token in command: :` and `unsupported token in command: ==`),
trailing commas without options (`comma must be followed by at least one option`), unsupported options (`bayesplot unsupported option: <sorted_opts>`),
flag option values (`bayesplot option noopen does not accept a value`), and saving validation
(`bayesplot option saving expects a path`, `bayesplot option saving may only be supplied once`).
Owned `String` and primitive/enum types are used so that the public `Command` enum retains its `Eq` derive.
Runtime deliberately returns the typed unsupported command result; it does not perform posterior draw extraction,
chain iteration processing, Vega-Lite spec generation, plot SVG/PNG rendering, artifact filesystem emission, or browser/viewer interaction.

Evidence: [_workspace/parser-bayesplot-syntax/](_workspace/parser-bayesplot-syntax/),
including the [contract](_workspace/parser-bayesplot-syntax/01-contract.md) and
[summary](_workspace/parser-bayesplot-syntax/04-summary.md). The pinned oracle probes,
locked Rust and policy checks, PR-head workflows, squash merge, and merge-head
workflows all passed.

This accepted syntax slice leaves posterior draw extraction, chain iteration processing, Vega-Lite spec generation,
plot SVG/PNG rendering, artifact filesystem persistence, browser/viewer opening, reporting, CLI, JSON, and MCP parity deferred.

## Verified slice: script engine

The script engine slice (Phase 5 §5.2) adds bounded, backend-independent `.td`
script parsing, macro expansion, directive evaluation, and conditional control
flow boundaries to `tabdat-language::script`. It provides `ScriptCommand` with
1-based start line tracking, `ScriptContext` for session macro definitions and
random seed state, and typed directive representations:
- Directives: `ScriptDirective::Seed(SeedDirective)`, `ScriptDirective::Let(LetDirective)`.
- Control flow: `ControlFlowDirective::If(IfDirective)`, `ControlFlowDirective::Else(ElseDirective)`, `ControlFlowDirective::End(EndDirective)`.
- Block execution tracking: `ScriptBlockState` with active status, match tracking, and else presence.
- Direct entry points: `parse_script(source, path)` and `read_script(path)` for streaming executable commands.
- Macro expansion: `expand_script_macros` expanding `$macro` against context while preserving literal `$` patterns.
- Expression evaluation: `evaluate_script_condition` evaluating truthiness and comparisons (`==`, `!=`, `<`, `<=`, `>`, `>=`).

It enforces exact Python-compatible behavior and diagnostics: comment stripping (`#`),
multiline triple-quoted SQL grouping (`sql """ ... """`) with internal newline preservation and opening line reporting,
unterminated SQL syntax errors (`<path>:<line>: unterminated triple-quoted sql command`),
directive syntax and value validation (`seed requires an integer value`, `invalid seed: ...`, `let expects syntax: let <macro> = <value>`, `invalid macro name: ...`),
undefined macro reporting (`<path>:<line>: undefined macro: <name>`),
and control flow block validation (`if requires a condition expression`, `else without matching if`, `end without matching if`, `else already defined for if block`, `unclosed if block`).

Evidence: [_workspace/script-engine/](_workspace/script-engine/),
including the [contract](_workspace/script-engine/01-contract.md) and
[summary](_workspace/script-engine/04-summary.md). The pinned oracle probes,
locked Rust and policy checks, PR-head workflows, squash merge, and merge-head
workflows all passed.

This accepted script engine slice leaves runtime script execution (`run <path>`), call stack recursion limits, DuckDB session integration, and CLI execution deferred.

## Verified slice: bounded runtime script execution

Merged PR #135 (`4717a09`) extends the runtime boundary with eager script execution
for the `run <path>` command (Roadmap Phase 5 §5.2). It exposes typed execution results
and error diagnostics in `tabdat-runtime`:
- `RunResult { path: PathBuf, executed_commands: usize }` representing the canonicalized
  script path and total executed command count.
- `ExecutionResult::Run(RunResult)` in the typed public execution result model.
- `RuntimeError::ScriptError(ScriptError)` wrapping script errors with source path and
  1-based line diagnostics.
- `Session::execute_run(&mut self, path)` entry point and recursive `execute_script_file`
  handler.

The runtime script engine enforces exact Python-compatible behavior and diagnostics:
- Script loading and parsing via `tabdat_language::script::read_script` with comment
  stripping and multiline triple-quoted SQL grouping.
- Sequential command execution on mutable active `Session` state.
- Nested `run <nested_path>` execution with relative path resolution against the
  enclosing script's parent directory.
- Recursion rejection: active canonical path call-stack cycle tracking returning exact
  diagnostic `<path>:1: recursive script inclusion is not supported`.
- Shared context inheritance: macros and random seed state defined or mutated in nested
  scripts propagate across child scripts and back to the parent.
- Control flow evaluation: `if` / `else` / `end` directives evaluate conditions and skip
  execution of inactive branches while honoring nested block constraints.
- Directives: `seed` sets the context random seed; `let` defines macros; `exit` cleanly
  terminates script execution early without error.
- Exact diagnostics: runtime execution failures encountered inside scripts are wrapped
  with the script file path and 1-based start line.

Evidence: [_workspace/runtime-run-execution/](_workspace/runtime-run-execution/),
including the [contract](_workspace/runtime-run-execution/01-contract.md) and
[summary](_workspace/runtime-run-execution/04-summary.md), `crates/tabdat-runtime/src/lib.rs`,
and `crates/tabdat-runtime/tests/run_contract.rs`. All 14 focused integration tests,
the updated 65-test `use_contract` suite, locked workspace checks, policy checks,
PR-head workflows, and squash merge passed.

This accepted runtime script execution slice leaves interactive REPL loop, terminal CLI runner,
and external script fixtures deferred.

## Verified slice: bounded runtime SQL query and named table execution

Merged PR #137 (`44686be`) extends the runtime boundary with bounded SQL query
execution and named-table lifecycle management for the `sql <query> [into <table>]`
command (Roadmap Phase 4 §6.5 & §6.1). It exposes typed execution results and error
diagnostics in `tabdat-runtime`:
- `TableResult { headers: Vec<String>, rows: Vec<Vec<CellValue>> }` representing typed
  tabular query results from direct `SELECT` or `WITH` queries.
- `SqlCreateResult { table_name: String, dataset: DatasetInfo }` representing target
  named table creation and active dataset publishing.
- `ActivateResult { table_name: String, dataset: DatasetInfo }` representing named table
  activation into the active relation.
- `ExecutionResult::Table`, `ExecutionResult::SqlCreate`, and `ExecutionResult::Activate`
  variants added to the typed public execution result model.
- `RuntimeError::SqlNotSelectOrWith`, `RuntimeError::SqlFailed`,
  `RuntimeError::SqlNoTableResult`, `RuntimeError::UseOptionsNotSupportedForNamedTable`,
  and `RuntimeError::UnknownTable { name }` with exact Python-parity diagnostics.
- `Session::execute_sql(&mut self, query, into)` entry point, `Session::named_tables()`,
  and `Session::active_table_name()` accessors.

The runtime SQL engine enforces exact Python-compatible behavior and diagnostics:
- Direct query execution (`sql <query>`): evaluates against DuckDB with the `active`
  view bound, returning headers and cell values without modifying the active dataset.
- Target query execution (`sql <query> into <table>`): evaluates query into temporary table
  `__tabdat_named_<table>`, updates `__tabdat_active`, registers the table in `named_tables`,
  and tracks `active_table_name`.
- Active named table synchronization: subsequent transforms (`keep`, `drop`, `generate`,
  `replace`, `rename`, `sort`, `gsort`, `recode`, `encode`, `decode`, `collapse`) that
  mutate `__tabdat_active` automatically update the underlying `__tabdat_named_<name>`
  table and in-memory registry.
- Named table activation: `use <table>` resolves against `named_tables`, publishes the
  relation as active, and rejects loader options (`delimiter`, `lazy`, etc.) with
  `use options are not supported for named table activation`.
- Query validation: ensures queries begin with `select` or `with` (case-insensitive);
  non-query statements return `sql only supports select or with queries in Phase 4`.
- Unresolved table diagnostics: `use <missing>` without file extension or path
  separators produces `unknown table: <name>`.
- Multi-statement `.td` script integration: scripts can execute multiline triple-quoted
  SQL queries and `into <table>` workflows seamlessly.

Evidence: [_workspace/runtime-sql/](_workspace/runtime-sql/),
including the [contract](_workspace/runtime-sql/01-contract.md) and
[summary](_workspace/runtime-sql/04-summary.md), `crates/tabdat-runtime/src/lib.rs`,
and `crates/tabdat-runtime/tests/sql_contract.rs`. All 10 focused integration tests,
the updated `use_contract` suite, locked workspace checks, policy checks, PR-head
workflows, and squash merge passed.

This accepted runtime SQL query and named table execution slice leaves multi-database
connections, remote DuckDB sessions, and CLI/JSON/MCP rendering deferred.

## Verified slice: bounded eager runtime `join` command execution

Merged PR #139 (`56babda`) extends the runtime boundary with bounded `join` command
execution against an active relation and a registered named table for the
`join <table> on <keylist> [, how=inner|left suffix(_right)]` command
(Roadmap Phase 4 §6.4 & §6.1). It exposes typed execution results and error
diagnostics in `tabdat-runtime`:
- `JoinResult { dataset: DatasetInfo }` representing the transformed dataset
  resulting from the join operation.
- `ExecutionResult::Join(JoinResult)` variant added to the typed public execution
  result model.
- `RuntimeError::JoinUnknownVariable { variables: Vec<String> }` and
  `RuntimeError::JoinUnknownVariableInTable { table_name: String, variables: Vec<String> }`
  with exact Python-parity diagnostics.
- `Session::execute_join(&mut self, command: &JoinCommand)` entry point, also
  routed from `Session::execute`.

The runtime join engine enforces exact Python-compatible behavior and invariants:
- Row order preservation: preserves active table row order primary and matching
  named-table row sequence secondary using collision-free internal row order
  identifiers (`__tabdat_join_order`, `__tabdat_join_right_order`).
- Right-side column collision handling: renames colliding columns from the right-hand
  relation using default suffix `_right` or user-specified `suffix(...)`, ensuring
  collision-free output identifiers via incremental suffix dedup.
- Inner and Left joins: supports `how=inner` (filtering to matching keys) and
  `how=left` (preserving all active rows, filling missing right columns with NULL).
- Multi-key joins: supports joining on multiple key columns (`join <table> on key1 key2`).
- Surviving label metadata retention: preserves left-table variable labels for
  surviving columns via `self.retain_label_metadata`.
- Active named table synchronization: updates underlying named table and registry if the
  active dataset was loaded from a named table.
- Atomic staging table lifecycle: builds the joined relation in `__tabdat_staging`
  and atomically publishes it to `__tabdat_active`, cleaning up staging on failure
  without altering session state.
- Multi-statement `.td` script integration: scripts can execute `join` commands
  seamlessly alongside `sql ... into <table>` and other transformation commands.

Evidence: [_workspace/runtime-join-execution/](_workspace/runtime-join-execution/),
including the [contract](_workspace/runtime-join-execution/01-contract.md) and
[summary](_workspace/runtime-join-execution/04-summary.md), `crates/tabdat-runtime/src/lib.rs`,
and `crates/tabdat-runtime/tests/join_contract.rs`. All 13 focused integration tests,
the existing `sql_contract`, `use_contract`, and `run_contract` suites, locked workspace checks,
policy checks, PR-head workflows, and squash merge passed.

This accepted runtime `join` execution slice leaves remote DuckDB sessions, external databases,
right/full outer joins (not supported in TabDat language), and CLI/JSON/MCP rendering deferred.

## Verified slice: bounded eager runtime `append` command execution

Merged PR #141 (`800a231`) extends the runtime boundary with bounded `append` command
execution against an active relation and a registered named table for the
`append <table>` command (Roadmap Phase 4 §6.4 & §6.1). It exposes typed execution results
and error diagnostics in `tabdat-runtime`:
- `AppendResult { dataset: DatasetInfo }` representing the combined dataset
  resulting from appending named-table rows to the active relation.
- `ExecutionResult::Append(AppendResult)` variant added to the typed public execution
  result model.
- Typed runtime error variants matching exact Python parity:
  - `RuntimeError::AppendUnknownVariable { variables: Vec<String> }`
  - `RuntimeError::AppendUnknownVariableInTable { table_name: String, variables: Vec<String> }`
  - `RuntimeError::AppendTypeMismatch { variable: String, left_type: String, right_type: String }`
  - `RuntimeError::AppendFailed`
- `Session::execute_append(&mut self, table_name: &str)` entry point, also
  routed from `Session::execute`.

The runtime append engine enforces exact Python-compatible behavior and invariants:
- Row order preservation: preserves active table row order primary (`side = 0`) and
  append table row sequence secondary (`side = 1`), preserving internal row sequence
  within each side via `row_number() OVER ()`.
- Column alignment: projects columns by explicit active dataset schema name on both
  sides before `UNION ALL`, guaranteeing correct alignment even if the named table
  columns were defined in a different order.
- Collision-free internal ordering columns: uses `unique_internal_name` to avoid
  colliding with existing columns named `__tabdat_append_side` or `__tabdat_append_row`.
- Detached transform behavior: sets `active_table_name = None` so that subsequent
  mutations on the active relation do not overwrite the named table from which active
  was originally loaded, preserving the named table snapshot.
- Variable label retention: preserves surviving variable labels from the active relation
  via `self.retain_label_metadata`.
- Atomic staging table lifecycle: builds the appended relation in `__tabdat_staging`
  and atomically publishes it to `__tabdat_active`, cleaning up staging on failure
  without altering session state.
- Multi-statement `.td` script integration: scripts can execute `append` commands
  seamlessly alongside `sql ... into <table>` and other transformation commands.

Evidence: [_workspace/runtime-append-execution/](_workspace/runtime-append-execution/),
including the [contract](_workspace/runtime-append-execution/01-contract.md) and
[summary](_workspace/runtime-append-execution/04-summary.md), `crates/tabdat-runtime/src/lib.rs`,
and `crates/tabdat-runtime/tests/append_contract.rs`. All 11 focused integration tests,
the existing `sql_contract`, `join_contract`, `use_contract`, and `run_contract` suites,
locked workspace checks, policy checks, PR-head workflows, and squash merge passed.

This accepted runtime `append` execution slice leaves remote DuckDB sessions, external databases,
schema evolution/union of mismatched columns, and CLI/JSON/MCP rendering deferred.

## Verified slice: bounded eager runtime `reshape` command execution

Merged PR #143 (`4d90584`) extends the runtime boundary with bounded `reshape` command
execution against an active eager relation for `reshape long|wide <stubs>, i(<identifiers>) j(<j_variable>)`
(Roadmap Phase 4 §6.4 & §6.1). It exposes typed execution results and error diagnostics in
`tabdat-runtime`:
- `ReshapeResult { dataset: DatasetInfo }` representing the transformed dataset
  resulting from unpivoting (long) or pivoting (wide) the active relation.
- `ExecutionResult::Reshape(ReshapeResult)` variant added to the typed public execution
  result model.
- Typed runtime error variants matching exact Python parity:
  - `RuntimeError::ReshapeUnknownVariable { variables: Vec<String> }`
  - `RuntimeError::ReshapeOutputColumnExists { variable: String }`
  - `RuntimeError::ReshapeLongFoundNoColumnsForStub { stub: String }`
  - `RuntimeError::ReshapeLongMissingColumn { stub: String, j_value: String }`
  - `RuntimeError::ReshapeWideFoundNoJValues`
  - `RuntimeError::ReshapeWideOutputColumnExists { variable: String }`
  - `RuntimeError::ReshapeFailed`
- `Session::execute_reshape(&mut self, command: &ReshapeCommand)` entry point, also
  routed from `Session::execute`.

The runtime reshape engine enforces exact Python-compatible behavior and invariants:
- **Reshape Long (`reshape long`)**: discovers stub `j_values` in column appearance order,
  validates that every stub column exists across all discovered `j_values` (erroring with
  `reshape long missing column <stub><j> for stub <stub>` on ragged stubs), ensures stubs
  match at least one column, and unpivots stubs via `UNION ALL` preserving row order primary
  (`row_order`) and stub discovery order secondary (`j_order`).
- **Reshape Wide (`reshape wide`)**: extracts distinct non-null `j_values` ordered
  lexicographically, errors if all `j` values are null (`reshape wide found no j values`),
  validates that output columns (`<stub><j>`) do not collide with non-participating columns,
  and pivots values via `MAX(CASE WHEN ... END)` grouped by identifier columns (`id_vars`),
  preserving initial identifier appearance order via `MIN(source_order)`.
- **Shared Invariants**:
  - Collision-free internal ordering columns: uses `unique_internal_name` to avoid
    colliding with existing columns named `__tabdat_reshape_row_order`, `__tabdat_reshape_j_order`,
    `__tabdat_reshape_group_order`, or `__tabdat_reshape_source_order`.
  - Detached transform behavior: sets `active_table_name = None` so that subsequent mutations
    on the active relation do not overwrite the named table from which active was originally
    loaded, preserving the named table snapshot.
  - Variable label retention: preserves surviving variable labels from the active relation
    via `self.retain_label_metadata`.
  - Atomic staging table lifecycle: builds the reshaped relation in `__tabdat_staging`
    and atomically publishes it to `__tabdat_active`, cleaning up staging on failure
    without altering session state.
  - Multi-statement `.td` script integration: scripts can execute `reshape` commands
    seamlessly alongside other transformation and inspection commands.

Evidence: [_workspace/runtime-reshape-execution/](_workspace/runtime-reshape-execution/),
including the [contract](_workspace/runtime-reshape-execution/01-contract.md) and
[summary](_workspace/runtime-reshape-execution/04-summary.md), `crates/tabdat-runtime/src/lib.rs`,
and `crates/tabdat-runtime/tests/reshape_contract.rs`. All 12 focused integration tests,
the existing `sql_contract`, `join_contract`, `append_contract`, `use_contract`, and
`run_contract` suites, locked workspace checks, policy checks, PR-head workflows,
and squash merge passed.

This accepted runtime `reshape` execution slice leaves remote DuckDB sessions, external databases,
complex nested stubs, and CLI/JSON/MCP rendering deferred.

## Verified slice: bounded CLI argument parsing and batch execution

Merged PR #145 (`231613f`) wires the root binary (`src/main.rs`) to `tabdat-language` and
`tabdat-runtime` for bounded CLI argument parsing and batch execution (Roadmap Phase 5 §7.1):
- **Workspace Wiring**: Connects root package `tabdat-explore-rs` to `tabdat-language` and
  `tabdat-runtime` via explicit path dependencies with versions, compliant with `deny.toml`
  wildcard dependency policies.
- **CLI Argument Parsing (`src/cli.rs`)**:
  - `-v`, `--version`: Prints `tabdat 0.1.0\n` and exits 0.
  - `-h`, `--help`: Prints standard usage and option summaries and exits 0.
  - `-c`, `--command <CMD>`: Repeated batch command execution against a `Session`.
  - `-f`, `--file <PATH>` and positional `<script>`: Executes TabDat `.td` script files.
  - Conflict detection: Rejects `-c` combined with script execution, `-f` combined with
    positional scripts, missing arguments, and unrecognized flags with exact Python-compatible
    diagnostics on stderr and exit code 2.
- **Execution Dispatch**:
  - Dispatches batch commands and script execution with exact error diagnostics and exit
    code conventions (0 for success, 1 for runtime error, 2 for parse/syntax/CLI error,
    3 for script file not found).
  - Preserves scaffold greeting (`Hello, world!\n`) when run with no arguments until
    interactive shell REPL is implemented in Phase 5 §7.2.
- **Testing**:
  - 10 unit tests in `src/cli.rs`.
  - 8 integration tests in `tests/cli_contract.rs`.
  - Original scaffold smoke test in `tests/scaffold.rs` continues to pass.

Evidence: [_workspace/cli-argument-parsing/](_workspace/cli-argument-parsing/),
including the [contract](_workspace/cli-argument-parsing/01-contract.md) and
[summary](_workspace/cli-argument-parsing/04-summary.md), `src/cli.rs`, `src/main.rs`,
and `tests/cli_contract.rs`. Locked workspace checks, policy checks, PR-head workflows,
and squash merge passed.

This accepted CLI slice leaves interactive REPL shell (§7.2), JSON serialization and
terminal table rendering (§7.3), visualization (§7.4), MCP server (§7.5), and discovery
flags deferred.

## Verified slice: CLI JSON and command discovery interfaces

Merged PR #147 (`c3ff323`) implements CLI discovery flags, in-app help topic retrieval,
syntax-only command explanation, and versioned JSON output envelopes (Roadmap Phase 5 §7.1):
- **Command Discovery & Schemas (`src/catalog.rs`)**:
  - `COMMAND_NAMES`: Canonical 81-command catalog sorted alphabetically.
  - `COMMAND_EFFECTS`: Declared command effects mapped across canonical categories (`read`, `write`, `control`, `plot`, `unknown`).
  - `COMMAND_SCHEMAS`: Complete syntax, argument descriptors, option descriptors, and help topic associations for all 81 commands.
  - `ResultEnvelope<T>` and `ErrorEnvelope`: Versioned machine-readable envelopes (`schema_version: 1`) with exact key-ordered serialization matching Python oracle `16b45d9`.
- **In-App Help Topics (`src/help.rs`, `src/help/topics/`)**:
  - Packaged all 79 canonical help topic markdown files.
  - Case-insensitive lookup via `load_help_topic_text()`.
- **CLI Discovery Flags (`src/cli.rs`)**:
  - `--json`: Machine-readable output mode; produces JSON error envelopes on execution and discovery errors.
  - `--list-commands`: Emits `CommandCatalogResult` (requires `--json`).
  - `--list-command-effects`: Emits `CommandEffectCatalogResult` (requires `--json`).
  - `--help-topic <topic>`: Emits `HelpTopicResult` with topic documentation (requires `--json`).
  - `--describe-command <cmd>`: Emits `CommandSchemaResult` (requires `--json`).
  - `--explain`: Syntax-only parsing and command preview via `-c`/`--command` without starting a session or initializing `duckdb` (requires `--json`).
  - Full mutual exclusivity and argument requirements validation matching Python `argparse` with exit code 2 on CLI errors.
- **Testing**:
  - 19 unit tests across `cli.rs`, `catalog.rs`, and `help.rs`.
  - 17 integration contract tests in `tests/cli_discovery_contract.rs`.
  - Backwards compatibility confirmed with `tests/cli_contract.rs` and `tests/scaffold.rs`.

Evidence: [_workspace/cli-json-discovery/](_workspace/cli-json-discovery/),
including the [contract](_workspace/cli-json-discovery/01-contract.md) and
[summary](_workspace/cli-json-discovery/04-summary.md), `src/catalog.rs`, `src/help.rs`,
`src/cli.rs`, and `tests/cli_discovery_contract.rs`. Locked workspace checks, policy
checks, PR-head workflows, and squash merge passed.

This accepted CLI discovery slice leaves interactive REPL shell (§7.2), runtime
execution JSON result serialization and terminal table rendering (§7.3), visualization
(§7.4), and MCP server (§7.5) deferred.

## Verified slice: statistical contracts substrate (tabdat-stats)

Create `crates/tabdat-stats` (`#![forbid(unsafe_code)]`) as an independent workspace
member crate implementing foundational statistical contracts and pure baseline inference
(Roadmap Phase 7 §9.1 and Phase 2 §4.1):
- **Core Domain Types (`crates/tabdat-stats/src/`)**:
  - `EstimationProblem`: Typed problem specification (`outcome`, `predictor_names`, `response`, `design_matrix`, `include_intercept`, `intercept_name`, `sample`, `weights`).
  - `EstimationSample`: Explicit sample provenance tracking (`retained_indices`, `total_observations`, `dropped_observations`, `weights`, `cluster_groups`).
  - `CoefficientEstimate`: Individual parameter estimate with name, value, standard error, test statistic ($t$/$z$), p-value.
  - `CovarianceMatrix` & `CovarianceType`: Symmetric parameter covariance representation supporting `NonRobust`, `RobustHc1`, and `Cluster(var)`.
  - `EstimationDiagnostics`: Estimation convergence metadata (`method`, `converged`, `iterations`, `objective_value`, `residual_sum_of_squares`).
  - `FitStatistics`: Goodness-of-fit statistics ($N$, degrees of freedom, R², adjusted R², Root MSE, RSS, TSS, F-statistic, log-likelihood).
  - `LeastSquaresResult`: Owned result struct for linear estimation.
  - `PredictionContract` (`predict_linear_response`): Out-of-sample and in-sample linear prediction.
  - `PostEstimationModel`: Post-estimation parameter access, linear combinations (`lincom`), and Wald linear hypothesis tests ($R \beta = r$).
  - `Estimator` trait: Backend capability trait.
  - `StatsError`: Normalized typed statistical error hierarchy.
  - `fit_least_squares`: Pure Rust baseline linear least squares fitting using Householder QR decomposition with back-substitution and triangular inverse computation for numerical backward stability.
- **Differential Validation & Testing**:
  - NIST Longley benchmark certified reference values matched to $< 10^{-9}$ relative tolerance (observed: $\sim 10^{-13}$ to $10^{-15}$).
  - Pinned Python TabDat oracle (`16b45d9`) matched to $< 10^{-9}$ relative tolerance.
  - 10 unit, contract, and differential tests in `crates/tabdat-stats/tests/`.

Evidence: [_workspace/statistical-contracts/](_workspace/statistical-contracts/),
including the [contract](_workspace/statistical-contracts/01-contract.md),
[design](_workspace/statistical-contracts/02-design.md),
[implementation](_workspace/statistical-contracts/03-implementation.md), and
[summary](_workspace/statistical-contracts/04-summary.md), `crates/tabdat-stats/`,
and PR #149 (squash merge `8370622`).

This accepted statistical contracts substrate slice leaves specific estimator command
dispatch (`regress`, `ivregress`, `logit`, etc.) and backend integration deferred.

## Verified slice: linear regression runtime execution (`regress`)

Implement end-to-end runtime execution of linear regression (`regress`) connecting
DuckDB tabular relations in `tabdat-runtime` with numerical estimation kernels in
`tabdat-stats` (Roadmap Phase 7 §9.2):
- **Core Estimators & Modes (`crates/tabdat-runtime/src/lib.rs`, `crates/tabdat-stats/src/`)**:
  - OLS `regress`: Ordinary least squares estimation using Householder QR decomposition with back-substitution.
  - WLS: Weighted least squares with precision weights ($w_i > 0$).
  - GLS: Generalized least squares with 1D variance specification ($\sigma_i > 0$) translated to precision weights $w_i = 1 / \sigma_i$.
  - Robust Covariance (`robust`): Stata HC1 robust sandwich covariance matrix $\frac{n}{n - k} (X^T W X)^{-1} \left(\sum_i (w_i e_i)^2 x_i x_i^T\right) (X^T W X)^{-1}$.
  - Clustered Covariance (`cluster(var)`): Cluster-robust sandwich covariance matrix with degrees-of-freedom correction $\frac{G}{G - 1} \frac{n - 1}{n - k} (X^T W X)^{-1} \left(\sum_g u_g u_g^T\right) (X^T W X)^{-1}$.
  - Intercept Suppression (`noconstant`): Fits models through the origin without constant term.
- **Data Extraction & Missingness Invariants**:
  - Requires active dataset; checks existence and numeric types for outcome, predictors, and weights.
  - Row missingness tracking: drops observations with null/NaN in outcome, any predictor, weight, or cluster variable while recording provenance in `EstimationSample` (`retained_indices`, `total_observations`, `dropped_observations`, `weights`, `cluster_groups`).
  - Strict validation of strictly positive weights ($w_i > 0$) and sigma ($\sigma_i > 0$).
  - Stores estimation state in `Session::last_regression` for post-estimation inspection.
  - Returns `ExecutionResult::Regression(Box<RegressionResult>)`.
- **Differential Validation & Testing**:
  - Classical, robust HC1, and clustered covariance verified against statsmodels and Python TabDat oracle (`16b45d9`) certified values ($< 10^{-13}$ observed relative error) in `crates/tabdat-stats/tests/robust_cluster_tests.rs`.
  - 12 comprehensive integration tests in `crates/tabdat-runtime/tests/regress_contract.rs`.

Evidence: [_workspace/linear-regression-runtime/](_workspace/linear-regression-runtime/),
including the [contract](_workspace/linear-regression-runtime/01-contract.md),
[design](_workspace/linear-regression-runtime/02-design.md),
[implementation](_workspace/linear-regression-runtime/03-implementation.md), and
[summary](_workspace/linear-regression-runtime/04-summary.md), `crates/tabdat-runtime/`,
`crates/tabdat-stats/`, and PR #151 (squash merge `9476053`).

This accepted runtime regression slice leaves broader post-estimation commands (`predict`,
`test`, `lincom`, etc.), reporting, CLI table rendering, JSON, and MCP deferred.

## Verified slice: reproducible build baseline


- Pin a Rust toolchain and commit the binary's lockfile.
- Enforce two-space indentation using rustfmt and EditorConfig.
- Forbid unsafe code in the ordinary binary and smoke-test its existing output,
  empty stderr, and successful exit without third-party runtime dependencies.
- Run formatting, all-target check/test, and warnings-as-errors Clippy in GitHub
  Actions on pull requests and pushes to `main`, using the lockfile.
- Document exact local checks and distinguish build health from migration parity.

Evidence: the smoke test passed locally; the two-space formatter rejected the
original four-space source before correction. All four baseline commands passed
locally and in [PR #2 CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/34186271522)
on the initial implementation revision. Every subsequent revision must also pass
CI before merge. The smoke test characterizes the scaffold, not a migrated public
CLI contract.

This build slice excluded parser/runtime/backend implementation, Python oracle
recovery, statistical parity, security auditing, benchmarks, packaging, and a
minimum-supported-Rust claim.

## Verified slice: Python migration authority

Pin an upstream-identifiable Python commit/tree, inventory behavioral authority and
fixture entry points, and define conflict/deviation and baseline-update rules.
Record the separate Rust repository decision and current/proposed architecture.
Acceptance: local clean checkout and GitHub commit/tree agree; every inventoried
path exists at that revision; bounded parser/script oracle checks are recorded
without implying full-suite or Rust parity; guidance has no stale missing-pin claim.
No Python source edits, dependency installation, backend work, or migrated commands.

## Verified slice: syntax-only parser foundation

Add the first language-layer crate without changing the scaffold binary or
initializing any runtime capability. `tabdat-language` exposes owned `Command`
variants for `Help`, `Status`, and `Exit`; `quit` is an alias for `exit`; and
`ParseError` has deterministic display text. The parser preserves the pinned
Python behavior for surrounding whitespace, case-insensitive command/topic names,
the `?` help alias, empty/unknown commands, quoted command-name rejection, and
unsupported arguments/options/assignment forms for these commands.

Evidence: `_workspace/parser-syntax-foundation/01-contract.md`,
`crates/tabdat-language/src/lib.rs`, and its unit/integration tests. The pinned
Python parser/script oracle suite passed with `516 passed in 0.44s` using
`PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider
tests/test_parser.py tests/test_script.py` from the clean sibling checkout at
revision `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`. Rust workspace checks pass
locally on the branch; hosted CI and review remain required before merge.

This slice does not execute commands, load data, provide the full tokenizer or
expression grammar, or establish parser/CLI/script parity beyond the listed forms.

## Verified slice: syntax-only inspection commands

Extend `tabdat-language` with typed, backend-independent syntax for `count`,
`head [n]`, and `tail [n]`. `head` and `tail` default to five rows and preserve a
canonical ASCII decimal `RowLimit`, including zero, leading-zero normalization,
quoted numeric arguments, values larger than `u64`, and substantially larger
values. Exact invalid-limit, unsupported-token, option, condition, assignment,
and trailing-comma diagnostics are covered by focused tests. No command executes,
reads session state, initializes DuckDB, or claims data-runtime support.

Evidence: `_workspace/parser-inspection-syntax/01-contract.md`,
`_workspace/parser-inspection-syntax/02-evidence-migration.md`,
`crates/tabdat-language/src/lib.rs`, and its unit/integration tests. The pinned
Python parser/script oracle suite again passed with `516 passed in 0.45s` using
`PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider
tests/test_parser.py tests/test_script.py` from the clean sibling checkout at
revision `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`.

This syntax-only slice did not itself establish `head`/`tail` execution,
active-dataset preconditions, row-order and missingness guarantees, backend
range conversion, results, or reporting. Separate bounded eager-runtime slices
now cover `describe`, `count`, `head`, `tail`, `summarize`, `codebook`, `missing`,
`duplicates`, and `isid`; lazy/materialized behavior and broader reporting remain
roadmap work.

## Verified slice: syntax-only describe command

Add the zero-argument `describe` form to `tabdat-language` as a typed,
backend-independent command. Case normalization, surrounding whitespace, and
the pinned Python diagnostics for arguments, conditions, options, assignments,
unsupported `==`/`-` tokens, and trailing commas are covered by focused tests.
This does not inspect schema metadata, access session state, execute a command,
or initialize a backend.

Evidence: `_workspace/parser-describe-syntax/01-contract.md`,
`_workspace/parser-describe-syntax/02-evidence-migration.md`,
`crates/tabdat-language/src/lib.rs`, and its unit/integration tests. The pinned
Python parser subset passed with `419 passed, 70 deselected`, and the full
parser/script oracle again passed with `516 passed in 0.44s` at revision
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`.

This slice leaves schema inspection results, active-dataset preconditions,
execution, reporting, serialization, and the full tokenizer/command inventory
to later roadmap slices.

## Verified slice: syntax-only doctor command

Add the zero-argument `doctor` form to `tabdat-language` as a typed,
backend-independent command. Case normalization, surrounding whitespace, and
the pinned Python diagnostics for arguments, conditions, options, assignments,
unsupported `==`/`-`/`+` tokens, missing `if` expressions, and trailing commas
are covered by focused tests. This does not probe the environment, inspect a
dataset, access session state, execute a command, or initialize a backend.

Evidence: `_workspace/parser-doctor-syntax/01-contract.md`,
`_workspace/parser-doctor-syntax/02-evidence-migration.md`,
`crates/tabdat-language/src/lib.rs`, and its unit/integration tests. The pinned
doctor parser subset passed with `6 passed, 8 deselected`, and the full
parser/script oracle again passed with `516 passed in 0.45s` at revision
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`.

This slice leaves environment/capability inspection results, active-dataset
preconditions, execution, reporting, serialization, prefixed commands, and
the full tokenizer/command inventory to later roadmap slices.

## Verified slice: syntax-only set command

Add the typed, backend-independent `set` form for the three currently
recognized setting names: `graph_format`, `artifact_dir`, and `graph_open`.
Setting names normalize case-insensitively; values remain owned strings with
quoted text unwrapped and spelling preserved. Exact diagnostics for missing or
extra arguments, unknown/backtick-quoted names, conditions, options,
assignments, unsupported punctuation, and trailing commas are covered by tests.
This slice parses commands only; it does not validate values, mutate
configuration/session state, inspect paths, open graphs, or initialize a
backend.

Evidence: `_workspace/parser-set-syntax/01-contract.md`,
`_workspace/parser-set-syntax/02-evidence-migration.md`,
`crates/tabdat-language/src/lib.rs`, and its unit/integration tests. The pinned
configuration parser test passed with `1 passed, 488 deselected`, and the full
parser/script oracle passed with `516 passed in 0.43s` at revision
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`.

This slice leaves typed configuration state, value validation, runtime effects,
persistence, reporting/serialization, prefixed commands, and the complete
tokenizer/option grammar to later roadmap work.

## Verified slice: syntax-only `datasignature` command

Add the direct, zero-argument `datasignature` form to `tabdat-language` as a
typed, backend-independent command. Case/whitespace normalization and the
pinned Python diagnostics for arguments, conditions, options, assignments,
unsupported argument tokens (`==`/`-`/`+`/`!`), missing `if` expressions, and
trailing commas are covered by focused tests. This slice does not hash data,
access an active relation, mutate session state, execute a command, or
initialize a backend.

Evidence: `_workspace/parser-datasignature-syntax/01-contract.md`,
`_workspace/parser-datasignature-syntax/02-evidence-migration.md`,
`crates/tabdat-language/src/lib.rs`, and its unit/integration tests. The pinned
`datasignature` parser test passed with `1 passed, 10 deselected`; the full
parser/script oracle remains `516 passed` at revision
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`.

This slice leaves SHA-256/signature semantics, active-dataset preconditions,
schema/row-order and missingness rules, result/reporting/serialization,
prefixed commands, and the full tokenizer/command inventory to later roadmap
work.

## Verified slice: syntax-only `use` command

PR #17 (`fc6e286`) adds direct, backend-independent `use` syntax to
`tabdat-language`:
local-path or raw-URI source classification, eager/lazy mode, lazy-engine
selection, and CSV delimiter/header options. The parser returns owned typed
values and deterministic diagnostics but does not inspect a path or URI, load a
file, resolve a named table, mutate session state, or initialize DuckDB/Polars.

Evidence: `_workspace/parser-use-syntax/{01-contract,02-evidence-migration,03-review}.md`,
`crates/tabdat-language/src/lib.rs`, and its unit/integration tests. The focused
Python parser subset passed with `419 passed, 70 deselected`, and the full
parser/script oracle passed with `516 passed in 0.46s` at the pinned baseline.
Locked Rust checks and policy scans pass locally, independent parser/contract/
workspace reviews report no open finding, and all six required hosted checks
passed before PR #17 was marked ready and merged.

This slice leaves data loading, format inference, named-table/session behavior,
lazy planning, execution/results, reporting/serialization, wrappers, and full
tokenizer parity to later roadmap work.

## Verified slice: eager local-Parquet runtime boundary

Merged PR #22 (`26dba2b`) accepted a Rust-owned `tabdat-runtime` session for one bounded path:
`Command::Use` with an existing local `.parquet` source in eager mode. The private
DuckDB adapter stages the file, reports ordered owned schema and row-count metadata,
and transactionally replaces active state only after a successful read. It rejects
lazy/URI/CSV-option forms and does not change the root scaffold binary.

The contract, pinned Python execution evidence, local checks, native
ownership/platform/license review, and hosted acceptance state are recorded in
`_workspace/use-eager-parquet/` and ADR 0007. This remains a bounded evaluation
rather than a supported product runtime; all broad Phase 4 session, relation,
load, inspect, transform, and reporting items remain unchecked.

## Verified slice: syntax-only `codebook` command

PR #18 (`1efc991`) adds direct, backend-independent `codebook [varlist]` syntax to
`tabdat-language`. The parser returns an owned ordered variable list, supports
the pinned quote and backtick forms, and preserves the recovered diagnostics for
conditions, options, assignments, missing `if` expressions, trailing commas,
and unsupported punctuation. It does not inspect an active dataset or schema,
validate variable names, expand wildcards/ranges, execute a command, or
initialize a backend.

Evidence: `_workspace/parser-codebook-syntax/{01-contract,02-evidence-migration,03-review}.md`,
`crates/tabdat-language/src/lib.rs`, and its unit/integration tests. Focused and
full pinned Python parser/script checks pass; local Rust and policy checks pass
(root smoke 1, language unit 22, integration 11). Independent
parser/contract/workspace review is complete. All six required hosted checks
passed before the PR was marked ready and merged: [Rust baseline and policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35212530397),
[ReadStat](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35212530584),
[libgretl feasibility](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35212530363),
and [libgretl OLS](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35212530418).

This slice leaves active-dataset/schema semantics, wildcard/range expansion,
conditions/options, prefixed commands, full tokenizer/varlist/expression
grammar, execution/results, reporting/serialization, and backend capability
initialization to later roadmap work.

## Verified slice: syntax-only `missing` command

PR #19 (`dc75c4d`) adds direct, backend-independent `missing [varlist]` syntax to
`tabdat-language`. The parser returns an owned ordered variable list, supports
the pinned quote and backtick forms, and preserves the recovered diagnostics for
conditions, options, assignments, missing `if` expressions, trailing commas,
and unsupported punctuation. It does not inspect an active relation or schema,
count nulls, validate variable names, expand wildcards/ranges, execute a command,
or initialize a backend.

Evidence: `_workspace/parser-missing-syntax/{01-contract,02-evidence-migration,03-review}.md`,
the implementation, and its unit/integration tests. Focused/full pinned Python
parser/script checks, locked Rust checks, policy scans, and independent review
all pass. All six required hosted checks passed before the PR was marked ready
and merged: [Rust baseline and policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35215073336),
[ReadStat](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35215073331),
[libgretl feasibility](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35215073321),
and [libgretl OLS](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35215073403).

This slice leaves active-relation/schema semantics, null-count and percentage
rules, wildcard/range expansion, conditions/options, prefixed commands, full
tokenizer/varlist/expression grammar, execution/results, reporting/serialization,
and backend capability initialization to later roadmap work.

## Verified slice: syntax-only `duplicates` command

PR #20 (`5460c7b`) adds direct, backend-independent
`duplicates [report] [varlist]` syntax in `tabdat-language`. It returns an
owned ordered variable list, strips a leading `report` alias according to the
pinned quote behavior, and preserves the pinned diagnostics for conditions,
options, assignments, missing `if` expressions, trailing commas, and unsupported
punctuation. It does not
inspect an active relation or schema, group rows, count duplicates, validate
variable names, expand wildcards/ranges, execute a command, or initialize a
backend.

Evidence: `_workspace/parser-duplicates-syntax/{01-contract,02-evidence-migration,03-review}.md`,
the implementation, and its unit/integration tests. Focused/full pinned Python
parser/script checks, locked Rust checks, policy scans, and independent review
all pass. All six required hosted checks passed before PR #20 was marked ready
and merged: [Rust baseline and policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35218042159),
[ReadStat](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35218042320),
[libgretl feasibility](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35218042170),
and [libgretl OLS](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35218042215).

This slice leaves duplicate-group and null-key semantics, active-relation/schema
behavior, conditions/options, prefixed commands, full tokenizer/varlist/
expression grammar, execution/results, reporting/serialization, and backend
capability initialization to later roadmap work.

## Verified slice: bounded eager-runtime `duplicates` command

Merged PR #38 (`6a10039`) adds the bounded eager local-Parquet
`duplicates [report] [varlist]` execution path to `tabdat-runtime`. It validates
active state and keys before querying, preserves requested/default key order and
duplicates, groups SQL NULL keys together, and returns an owned
`DuplicatesResult` with checked aggregate metrics. Successful and failed reads
are read-only with respect to active metadata and the private relation.

Evidence: `_workspace/runtime-duplicates/{01-contract,02-evidence-migration,03-review}.md`,
the implementation, and its unit/integration tests. The PR-head CI, policy, and
runtime workflows passed before merge; post-merge workflow links and branch
cleanup are recorded in the migration evidence.

This slice leaves lazy/materialized execution, last-operation state, labels,
wildcard/range expansion, formatting, CLI/REPL, JSON/MCP surfaces, and broader
relation APIs deferred.

## Verified slice: bounded eager-runtime `isid` command

Merged PR #39 (`e04def0`) adds the bounded eager local-Parquet
`isid [varlist] [, missok]` execution path to `tabdat-runtime`. It validates an
active dataset and every requested key before querying, preserves ordered and
repeated key variables, groups SQL NULL values equally, counts rows with any
missing key component, and returns an owned `IsidResult` when all requested
constraints pass. Missing-key rows fail unless `missok` is present; duplicate
key groups fail regardless of `missok`. Empty relations pass with zero counts,
and both successful and failed requests preserve active metadata and the private
relation.

Evidence: `_workspace/runtime-isid/{01-contract,02-evidence-migration,03-review}.md`,
the implementation, and its unit/integration tests. The pinned focused oracle,
locked Rust checks, policy scans, independent review, all PR-head and post-merge
hosted jobs, the squash merge, and branch cleanup are recorded in the migration
evidence.

This slice leaves lazy/materialized execution, last-operation state, labels,
wildcard/range expansion, formatting, CLI/REPL, JSON/MCP surfaces, and broader
relation APIs deferred.

## Verified slice: bounded eager-runtime `datasignature` command

Merged PR #40 (`9a141da`) adds the bounded eager local-Parquet
`datasignature` execution path to `tabdat-runtime`. It returns an owned
`DatasignatureResult` using the pinned SHA-256 schema/row/value protocol,
preserves public schema and active row order, accepts empty relations, and
leaves active metadata and the private relation unchanged. Exact fixtures cover
nonfinite and decimal values, nanosecond and nested temporal values, interval
encoding, temporal map keys, bare STRUCT fields, and escaped STRUCT names.

Evidence: `_workspace/runtime-datasignature/{01-contract,02-evidence-migration,03-review}.md`,
`docs/adr/0008-datasignature-sha256.md`, the implementation, and its
unit/integration tests. The pinned focused oracle, locked Rust checks, policy
scans, independent review, all PR-head and post-merge hosted jobs, the squash
merge, and branch cleanup are recorded in the migration evidence.

This slice leaves lazy/materialized execution, `last_operation`, labels/panel
metadata, CLI/REPL, JSON/MCP, direct DuckDB union values, and broader relation
APIs deferred.

## Verified slice: bounded eager-runtime `assert` command

Merged PR #41 (`019ceb1`) adds the bounded eager local-Parquet
`assert <boolean-expression>` execution path to `tabdat-runtime`. The typed
subset supports identifiers (including quoted identifiers), numeric/string/null
literals, unary minus, parentheses, arithmetic, and comparisons. It returns an
owned `AssertResult { checked, failed }`, treats false and SQL-NULL predicates
as failures, accepts empty relations, preserves active state on success and
failure, and uses checked/finite numeric normalization with unsigned safety
guards. Unknown names and non-boolean roots are rejected before querying.

Evidence: `_workspace/runtime-assert/{01-contract,02-evidence-migration,03-review}.md`,
the implementation, and its parser/runtime contract tests. The pinned focused
oracle, locked Rust checks, policy scans, independent review, final PR-head
hosted checks, squash merge, branch cleanup, and post-merge workflow matrix are
recorded there.

This slice leaves lazy/materialized execution, function calls and `e(sample)`,
`last_operation`, row-level diagnostics, formatting, CLI/REPL, JSON/MCP,
broader tokenizer/expression parity, and general relation APIs deferred.

## Verified slice: bounded eager-runtime `keep` projection

Merged PR #42 (`d43c923`) adds the bounded eager local-Parquet
`keep <explicit-varlist>` projection path to `tabdat-runtime`. The typed command
supports case-insensitive syntax, quoted/backtick identifiers, requested column
order, preserved row order, duplicate projection requests (with DuckDB's
deterministic duplicate names), exact bounded diagnostics, and staged
transactional publication. Unknown variables and dropped/corrupt active
relations fail before publication and preserve the previously published
metadata.

Evidence: `_workspace/runtime-keep/{01-contract,02-evidence-migration,03-review}.md`,
the implementation, and its parser/runtime contract tests. The pinned focused
oracle, locked Rust checks, policy scans, independent review, PR-head hosted
checks, squash merge, branch cleanup, and post-merge workflow matrix are
recorded there.

Predicate-form `keep if <expression>` remains deferred, including boolean/null
filtering, expression functions, arithmetic overflow reporting, and row-level
transform semantics. Lazy/materialized execution, wildcard/range expansion,
labels/panel metadata, `last_operation`, formatting, CLI/REPL, JSON/MCP, and
broader transformation sequencing remain deferred.

## Verified slice: bounded eager-runtime `drop` projection

Merged PR #43 (`50cf80c`) adds the bounded eager local-Parquet
`drop <explicit-varlist>` complement projection path to `tabdat-runtime`. The
typed command returns an owned `DropResult`, validates every requested name
before mutation, preserves source/schema order for the surviving columns, row
order and count, NULL values, and source metadata, and supports quoted/backtick
identifiers and duplicate requests. All-column removal, unknown variables, and
staging/backend failures are rejected atomically; the published metadata and
private active relation remain unchanged on failure.

Evidence: `_workspace/runtime-drop/{01-contract,02-evidence-migration,03-review,04-summary}.md`,
the implementation, and its parser/runtime contract tests. The pinned focused
oracle, locked Rust checks, policy scans, independent review, PR-head hosted
checks, squash merge, branch cleanup, and post-merge workflow matrix are
recorded there.

Predicate-form `drop if <expression>` remains deferred, as do
lazy/materialized execution, wildcard/range expansion, labels/panel metadata,
`last_operation`, formatting, CLI/REPL, JSON/MCP, and broader transformation
sequencing.

## Verified slice: bounded eager-runtime `select` projection

Merged PR #44 (`228fa50`) adds the bounded eager local-Parquet
`select <explicit-varlist>` projection path to `tabdat-runtime`. The typed
command returns an owned `SelectResult`, validates every requested name before
backend work, preserves requested column order and deterministic duplicate
projection names, row order and count, NULL values, source metadata, and quoted
or backtick identifiers. Empty typed requests, unknown variables, and
staging/backend failures leave the published metadata and private active
relation unchanged.

Evidence: `_workspace/runtime-select/{01-contract,02-evidence-migration,03-review,04-summary}.md`,
the implementation, and its parser/runtime contract tests. The pinned focused
oracle, locked Rust checks, policy scans, independent review, PR-head hosted
checks, squash merge, branch cleanup, and post-merge workflow matrix are
recorded there.

Predicate-form `select if <expression>` remains deferred, as do
lazy/materialized execution, wildcard/range expansion, labels/panel metadata,
`last_operation`, formatting, CLI/REPL, JSON/MCP, and broader transformation
sequencing.

## Verified slice: syntax-only `summarize` command

PR #21 (`ae12a65`) adds direct, backend-independent `summarize [varlist]` syntax
to `tabdat-language`. It returns an owned ordered variable list and preserves the
pinned diagnostics for assignments, conditions, options, missing `if` expressions,
trailing commas, and unsupported punctuation. It does not inspect an active
relation or schema, validate numeric columns, compute summary statistics, parse
structured expression/option forms, execute a command, or initialize a backend.

Evidence: `_workspace/parser-summarize-syntax/{01-contract,02-evidence-migration,03-review}.md`,
the implementation, and its unit/integration tests. Focused/full pinned Python
parser/script checks, locked Rust checks, policy scans, and independent review
all pass. All six required hosted checks passed before PR #21 was marked ready
and merged: [dependency policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35221567690/jobs/105202821398),
[Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35221567690/jobs/105202821628),
[ReadStat feasibility](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35221567716/jobs/105202725376),
[ReadStat Rust](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35221567716/jobs/105202725692),
[libgretl feasibility](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35221567602/jobs/105202724326),
and [libgretl OLS Rust](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35221567621/jobs/105202724462).

This slice leaves numeric-type and missingness semantics, active-relation/schema
behavior, conditions/options, prefixed commands, full tokenizer/varlist/
expression grammar, execution/results, reporting/serialization, and backend
capability initialization to later roadmap work.

## Verified slice: syntax-only `isid` command

PR #23 adds direct, backend-independent `isid [varlist] [, missok]` syntax to
`tabdat-language`. The parser returns an owned ordered key-variable list and an
exact-lowercase `missok` flag, preserving the bounded pinned-Python diagnostics
for missing keys, unsupported conditions/options/assignments, option values,
trailing commas, and unsupported punctuation. This syntax-only section does not
describe the bounded eager execution path documented separately below.

Evidence: `_workspace/parser-isid-syntax/{01-contract,02-evidence-migration,03-review}.md`,
the implementation, and its unit/integration tests. The focused `isid` oracle
check passed with `1 passed, 19 deselected`, and the full pinned parser/script
oracle passed with `516 passed` at Python revision
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`.

This syntax-only slice leaves wildcard/range expansion, unknown-variable
validation, null-key and duplicate-key semantics, active-relation/schema
behavior, key scans, result/reporting/serialization, full tokenizer/varlist/
option and expression grammar, scripts, CLI/JSON/MCP surfaces, and backend
capability initialization to later roadmap work; bounded runtime semantics are
covered separately above.

## Verified slice: syntax-only `run <script-path>`

Merged PR #24 (`77f4754`) adds a direct, backend-independent `run <script-path>`
parser form.
The parser returns the exact non-empty path token as an owned string and
preserves the pinned Python arity and command-boundary diagnostics. Runtime
execution remains an explicit typed unsupported-command error; no script file
is read, no command is executed, and no session or backend state changes.

Evidence and acceptance are tracked in
`_workspace/parser-run-syntax/{01-contract,02-evidence-migration,03-review}.md`.
The focused/full oracle, locked Rust, policy checks, independent parser/
contract/workspace review, all seven PR-head hosted jobs, the squash merge, and
all seven post-merge `main` jobs are recorded as passed. The temporary branch
was deleted locally and remotely.

This bounded form leaves path normalization, line-oriented script execution,
comments, multiline SQL, macros, control flow, nested/recursive scripts,
file/line diagnostics, full tokenizer parity, CLI/JSON/MCP surfaces, and
backend capability initialization to later roadmap work.

## Verified slice: syntax-only `rename` command

Merged PR #25 (`89f6c14`) adds direct `rename <old> <new>` syntax to
`tabdat-language` using the existing pure simple-body argument path. The parser captures two owned names,
normalizes command case and separator whitespace, unwraps the established quoted
and backtick identifier forms, and preserves the pinned arity, condition,
option, assignment, and punctuation diagnostics. Runtime execution remains an
explicit unsupported-command result; schema lookup, collision checks, relation
mutation, session effects, and all CLI/JSON/MCP surfaces are deferred.

Evidence and independent review are recorded in
`_workspace/parser-rename-syntax/`. The focused/full oracle, local locked and
policy checks, seven PR-head hosted jobs, squash merge, branch cleanup, and
post-merge `main` verification are recorded there as passed.

## Verified slice: syntax-only `select` command

Merged PR #26 (`5735b43`) adds direct `select <varlist>` syntax to
`tabdat-language` using
the existing pure simple-body argument path. The parser returns one or more
owned names, preserves their order and duplicates, reuses quote/backtick
unwrapping, and preserves the pinned condition, option, assignment, trailing
comma, and punctuation diagnostics. Runtime execution remains an explicit
unsupported-command result; active-schema lookup, wildcard/range expansion,
relation mutation, and execution are deferred.

The contract and migration evidence are in
`_workspace/parser-select-syntax/`. The focused/full oracle, local locked and
policy checks, seven PR-head hosted jobs, squash merge, branch cleanup, and
post-merge `main` verification are recorded there as passed.

## Verified slice: syntax-only `sort` command

Merged PR #27 (`7cf21ae`) adds direct, backend-independent `sort <varlist>`
syntax to `tabdat-language`. The parser returns an owned ordered variable list,
preserves duplicates and quoted/backtick names, and records the pinned Python
diagnostics for conditions, options, assignments, punctuation, and quote
boundaries. Runtime execution remains an explicit unsupported-command result;
active-schema lookup, wildcard/range expansion, stable/null/descending or
expression sorting, relation mutation, and execution are deferred.

The contract and migration evidence are in
`_workspace/parser-sort-syntax/`. The focused/full oracle, local locked and
policy checks, independent reviews, seven PR-head hosted jobs, squash merge,
branch cleanup, and post-merge `main` verification are recorded there as
passed.

## Verified slice: dependency and unsafe-code checks

Pin the security-tool versions used by CI, configure dependency license/advisory
policy for the current scaffold, run `cargo deny`, `cargo audit`, and `cargo geiger`
in CI, and document that these checks cover the Rust workspace rather than Python
or future native backends. Keep runtime dependencies unchanged.

Evidence: each tool has an explicit version and locked installation; local runs
passed on the pinned toolchain; [PR #4 CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/34187594609)
ran all three policy checks and the Rust baseline successfully on the latest
revision. Policy scope and expected future review are recorded in ADR 0002. No
dependency, backend, FFI, or product-command implementation was added.

## Verified slice: architecture ownership

Create `ARCHITECTURE.md` as the current-state and target-boundary authority. Distinguish
the implemented scaffold from proposed modules, define language → execution → backend
direction, typed state/effect boundaries, lazy capabilities, and unsafe/FFI ownership.
Evidence: `ARCHITECTURE.md` links the proposal, roadmap, ADRs, and migration policy;
it names current scaffold evidence and explicit exclusions; it introduces no crate,
command, backend, or parity claim. Documentation-only change; no code or dependency
change.

## Verified slice: DuckDB feasibility prototype

Evaluate an isolated `duckdb-rs` candidate for local CSV/Parquet loading, repeated
active-relation inspection, and Arrow result batches. Measure release orientation
costs and record ownership/unsafe/license/platform evidence without adding DuckDB to
the root runtime. Evidence: tiny inspectable tests passed locally and in the Linux path-scoped
[PR #6 workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/34194987287);
local macOS release evidence is recorded. Remote/S3, production session integration,
and parity remain explicitly deferred.

## Verified slice: ReadStat feasibility reconnaissance

Evaluate the pinned ReadStat `v1.0.0` release as a native DTA-ingestion candidate
without adding a Rust binding or product support. Verify the release archive,
record the macOS Apple Silicon build/test evidence and hosted Linux build/test
result, and document callback ownership, label/missingness surfaces, licensing,
and explicit adapter prerequisites. Do not claim DTA ingestion, labels,
missingness parity, or FFI safety from the native upstream test suite.

Evidence is recorded in [the ReadStat feasibility report](docs/feasibility/readstat.md)
and [ADR 0004](docs/adr/0004-readstat-feasibility.md); the path-scoped [PR #7
workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/34197952522)
built the pinned source and passed all four upstream tests on Linux.

## Verified slice: libgretl feasibility reconnaissance

Evaluate a pinned gretl/libgretl release as a native estimator-backend candidate
without adding a Rust binding or statistical product support. Verify the source
archive, record macOS Apple Silicon library/NIST evidence and hosted Linux
build/link/test evidence, and document initialization, native-handle ownership,
thread/dependency, and GPLv3 licensing constraints. Do not claim estimator parity,
FFI safety, or accepted backend status from native tests.

Evidence is recorded in [the libgretl feasibility report](docs/feasibility/libgretl.md)
and [ADR 0005](docs/adr/0005-libgretl-feasibility.md); the path-scoped [PR #8
workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/34201655560)
built the pinned source and passed all 11 NIST tests on Linux.

## Next

Resolve the DuckDB prototype's remaining platform/ownership/semantic gaps before
any production backend integration, or continue ReadStat/libgretl only through
fixture-backed low-level adapter contracts.

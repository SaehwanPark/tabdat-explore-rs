# TabDat Rust implementation state

## Current behavior

The Rust 2024 binary remains a scaffold: it prints `Hello, world!` and exits
successfully. The workspace also contains a backend-independent `tabdat-language`
crate with a deliberately small syntax-only parser for `help`/`?`, `status`,
`exit`/`quit`, `describe`, `doctor`, `set`, `datasignature`, `count`, `head`,
`tail`, `run <script-path>`, `save <path> [, replace]`, `export <path> [, replace]`,
and syntax-only `generate <target> = <expression>`, plus the verified direct `use`, `codebook [varlist]`,
`missing [varlist]`, `duplicates [report] [varlist]`, `summarize [varlist]`,
`isid [varlist] [, missok]`, and bounded direct `assert <boolean-expression>`
forms. Merged PR #25 (`89f6c14`) adds the
verified direct `rename <old> <new>` form, and merged PR #26 (`5735b43`) adds
the verified direct syntax-only `select <varlist>` form. Merged PR #22
(`26dba2b`) accepted a separate library-only
`tabdat-runtime` path for one eager local-Parquet `use` form; it is not wired into
the binary and does not provide a usable TabDat CLI, general data runtime, or
statistical model implementation.
The [proposal](docs/TABDAT_RUST_PORT_PROJECT_PROPOSAL.md) and
[roadmap](docs/TABDAT_RUST_PORT_ROADMAP.md) describe planned work, not support.

Merged PR #27 (`7cf21ae`) adds the verified syntax-only `sort <varlist>` form;
active-schema lookup, row sorting, and execution remain deferred.

Merged PR #28 (`fd94133`) adds the verified syntax-only `gsort [+|-]varlist`
form. Direction metadata is parsed only; active-schema lookup, ordering, and
execution remain deferred.

Merged PR #29 (`8b16223`) adds the bounded syntax-only `save <path> [, replace]` and
`export <path> [, replace]` forms. The language layer owns the lexical path and
replacement flag; filesystem validation, active-dataset access, output formats,
and persistence remain deferred.

Merged PR #45 (`63e65ec`) adds the bounded syntax-only
`generate <target> = <expression>` form. The language layer preserves an owned
expression tree, including function-call syntax, while expression evaluation,
schema/type validation, target mutation, and all runtime/output surfaces remain
deferred.

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

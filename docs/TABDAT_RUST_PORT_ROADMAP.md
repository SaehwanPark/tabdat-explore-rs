# TabDat Explore Rust: outcome-oriented roadmap

**Working plan, revised 2026-10-13.** Priorities are adjustable; this is not a
mandatory phase order or a commitment to full Python parity. Current support is
summarized in [SPEC.md](../SPEC.md). The historical checklist and recorded
acceptance decisions remain available in the [previous roadmap](https://github.com/SaehwanPark/tabdat-explore-rs/blob/333b97dd55524dbf5ce55fac510375df99e4649e/docs/TABDAT_RUST_PORT_ROADMAP.md).

## How work is selected

Choose the smallest vertical feature that improves a real workflow. Prioritize
user utility, correctness risk and dependency order, not the number of parser
forms migrated. It is acceptable to leave rare Python grammar/diagnostic cases
unsupported. Never waive statistical reference checks to speed delivery.

For each outcome, record only: **user behavior**, **2-5 focused acceptance cases**,
**important failure mode**, **what remains out of scope**, and **verification**.
No phase-wide checklist, repeated owner-review ritual or large handoff bundle is
required. Work independently of unrelated pending items.

## Now: finish one coherent core workflow

The bounded script slice in [PR #195](https://github.com/SaehwanPark/tabdat-explore-rs/pull/195)
records a signed 64-bit ASCII-decimal seed contract, retains all pinned oracle
outputs with five explicit deviations, and tests canonical/nested execution and
source diagnostics. See [script evidence](migration/phase3-scripts.md).
Seed currently records context only; RNG semantics remain future command work.

The minimal shell in [PR #196](https://github.com/SaehwanPark/tabdat-explore-rs/pull/196)
provides persistent line input, core inspection output, recoverable command errors
and clean exit/EOF. `use` → `describe` → `summarize` → `exit` is covered through
the binary. Capable-terminal editing, bounded in-memory history and input-level
Ctrl-C recovery are now installed (#203); piped input remains plain. Completion,
interactive multiline continuation and in-flight cancellation remain deferred; an executing command
may still terminate on SIGINT. Local Unix PTY evidence is not platform certification.

### 2. Close the core EDA loop

The shell now reports bounded frequency tables, common transformation metadata
and save/export outcomes ([PR #197](https://github.com/SaehwanPark/tabdat-explore-rs/pull/197)).
Binary fixtures cover ordering/missingness, transformed CSV/Parquet roundtrips,
and failed transformation/refused-overwrite recovery. Interactive `run` also
streams readable nested leaf results and preserves prior output/state on located
errors; output failure prevents subsequent mutations (#199). Broader options and other
result families remain command-specific; this is not whole-command parity.

`use` now accepts literal single/double-quoted paths with spaces, commas and
Unicode (#211), fixing a reproduced local-loading obstacle. Quoted CSV options,
CSV/Parquet roundtrips, nested JSON, source-classified failures and prior
model/data preservation are covered. No shell escape decoding, quoted paths for
other commands, or remote/lazy/DTA runtime is added.

**Goal:** common local-data workflows are functional through the actual CLI/REPL.

- `use`, `describe`, `head`/`tail`, `summarize`, `tabulate`, a practical subset of
  `keep`/`drop`, `generate`/`replace`, and save/export.
- Preserve/verify missingness, schema, ordering and failure-state guarantees for
  every newly supported option.
- Make unsupported options fail clearly; do not pre-parse every Python feature.

**Done when:** Canonical small CSV/Parquet fixtures run end to end and expected
rows, values, metadata and outputs match the declared supported semantics.

### 3. Make linear statistics reliably usable

The bounded shell now renders `regress`, `predict`, `lincom` and `test` results
([PR #198](https://github.com/SaehwanPark/tabdat-explore-rs/pull/198)). A noisy OLS
binary fixture compares displayed coefficients/inference/fit, lincom intervals,
F tests and predictions with statsmodels/SciPy, including exact sample row identity.
It also tests failed estimation/model clearing and recovery. This reporting-only
slice does not expand covariance/estimator support or claim validation of every
mode. CLI-v1 now includes bounded regress/predict/lincom/test results (#204),
with independent OLS/reference and error/null-policy checks, omitting full model
arrays. Script exit codes now use source categories (#205), fixing model errors
misclassified by “not found” and malformed scripts missed by text heuristics;
child locations and existing JSON fields are retained. Readable terminal `ttest`
results now cover one-sample, paired, pooled and Welch forms (#206), with a
SciPy display reference, asymmetric missingness, fractional DF, undefined
inference, state preservation and nested-stream checks. VIF already uses the
generic terminal table renderer. CLI-v1 now admits only `estat vif` among
`estat` forms and reports its owned ordered `TableResult` (#212), without
header-based type inference or changing terminal/library results. Independent
statsmodels fixtures cover intercept conventions, actual retained samples and
unweighted auxiliaries after WLS/GLS; nested/error/state paths are covered.
SQL and other `estat` JSON remain unsupported. CLI-v1 `ttest` JSON now covers the same four forms (#207), using
owned inference, explicit Student-t/95% metadata, nullable independent difference
counts, and the shared SciPy fixture; nested streams and error/null paths are
covered without changing calculations or model state.

Owned SQL-NULL missingness now reports in terminal/CLI-v1 JSON (#208), preserving
types, order/duplicates, counts, percentages and the empty-relation 0% convention.
Mixed NULL/NaN/infinity/stored-empty/sentinel checks distinguish NULL counts from
finite t-test samples; success/error/nested/state paths are covered. Summarize now
uses per-variable finite, non-NULL samples for all aggregates (#209), fixing the
reproduced non-finite aggregate failure. NumPy/SciPy references, loaded sample
indices, singleton/all-invalid/empty cases, exact wide integer/decimal extrema,
finite regressions and read-only state are checked. Native finite-arithmetic
overflow remains outside this correction; see MIG-0007 for the explicit policy.

Codebook now reports owned profiles in terminal/CLI-v1 JSON (#210): types,
SQL-NULL/non-NULL/native distinct counts, ordering/duplicates, and up to three
possibly repeated examples. Existing scalar encoding is reused; JSON metadata
explicitly marks non-finite example null encoding. Empty/all-NULL, stored-empty,
non-finite, scalar-boundary, nested/error and state paths are checked. Unsupported
composite example conversions remain a bounded execution limitation.

**Goal:** a complete, scientifically trustworthy linear-model workflow.

- `regress` with explicitly supported covariance modes, then `predict`, `test`,
  `lincom` and VIF where implemented.
- Confirm sample membership, coefficient/order conventions, degrees of freedom,
  covariance corrections, inferential values and post-estimation state.
- Add readable result tables and structured results for the supported subset.

**Done when:** Representative model fixtures agree with at least one trusted
independent reference at justified tolerances; invalid inputs and stale model
state are handled correctly.

## Next: improve ergonomics and automate reliably

- Extend machine-readable output **for executed commands**, not the entire
  speculative command catalog; version changes to stable public envelopes.
  CLI-v1 now includes ordered descriptive summaries and bounded frequency tables
  alongside load/count/preview results (#200/#201), including nested script
  streams and located failures. `describe` now reports owned current metadata
  (#213), completing canonical `use` → `describe` → `summarize` JSON scripts.
  Empty/current-prediction schema, nested equivalence, errors and state preservation
  are covered; label/panel fields retain existing null-placeholder semantics.
  Save/export JSON now reports owned requested destinations and output metadata
  (#215), supporting prediction-to-persistence workflows. CSV/Parquet roundtrips,
  overwrite refusal/replace, empty data, native/preflight failures and session
  preservation are covered. Injected write/flush failures directly prevent later
  file writes, without rolling back completed files. Native publication remains
  non-atomic; CSV is not a general type/label archive. Generate/replace JSON now
  reports owned post-transform metadata (#216), supporting transform → fit →
  predict → export. Exact previews/NULL conditions, closed-form OLS/sample identity,
  nested/persistence/error and label/model state paths are checked. Functions and
  affected/overflow-row diagnostics remain outside the installed result scope.
  Column-list keep/drop/select JSON now reports owned projected metadata (#217),
  enabling selected-schema fit/predict/persistence workflows. Request/remaining
  order, native types, NULLs, empty/duplicate/quoted names, nested streams and
  failure/output boundaries are checked; cached fits and existing label pruning
  are retained. Conditional keep/drop row filters remain uninstalled.
  Sort/gsort JSON now reports owned current metadata (#220), enabling ranked
  previews and ordered prediction/persistence. Mixed directions, NULL-last/stable
  prior-order ties, quoted/repeated/empty keys, native schema, nested streams and
  failure/output boundaries are checked. Labels and original fit/sample are
  retained without refitting; prediction uses current rows. No lazy/by execution
  or invented key/affected-row fields are implied.
  Single-column rename JSON now reports owned normalized schema metadata (#221),
  completing rename → transform → fit/predict → persistence. Values/types/position,
  NULLs, quoted names/empty data, label remapping, original cached-fit names/sample,
  validation/native case-collision, nested and output boundaries are checked.
  Renaming does not rewrite cached parameter names or install label/panel snapshots.
  Recode JSON reports owned generate/replace metadata (#222) for cleaning and
  categorization before tabulation/modeling. First-match/no-cascade ranges,
  SQL NULL versus stored empty strings, multi-output order/native schema,
  explicit cached-fit/label rules, nested/persistence/failure/output cases are
  checked. No guessed rule/changed-cell fields or new recode semantics are added.
  Label JSON now exposes owned actions/messages and explicitly tagged dictionary
  keys (#223), completing category definition/attachment/inspection workflows.
  Named-list subsets, clears/replacements/drops, transform reconciliation,
  data/model invariants, nested streams and output/file guards are checked.
  Dataset placeholders and session-local/non-persistent label limits remain.
  Encode/decode JSON now connects installed string category coding to dictionary
  inspection, explicit numeric transforms/modeling and readable persistence (#224).
  Sorted SQL-non-NULL codes, stored empty strings, native schema, inherited labels,
  integer-map decoding/unmatched NULLs, cached-state/native failure guards,
  nested streams and output boundaries are checked, without new backend semantics.
  SQL JSON now reports owned query rows and named-cohort metadata, including
  reactivation of registered tables (#225), for filtered analysis before fitting.
  SELECT/WITH ordering, existing scalar policy, empty/duplicate headers, labels
  and cached-fit rules, binder/validation failures and output/file boundaries
  are checked. SQL validation/publication boundaries are not sandbox/atomicity
  guarantees; no default row limit or backend execution changes are implied.
  Join JSON now reports owned lookup-enriched metadata (#226), connecting named
  SQL relations to duplicate-preserving inner/left joins and model/persistence.
  Multi/quoted keys, NULL nonmatches, suffixes, empty/self joins, cached original
  fits/labels/named updates, validation/native failures and output guards are
  checked; no guessed match diagnostics, cardinality checks or backend changes.
- Stabilize script reproducibility and practical error reporting, including
  canonical `.td` workflows and documented seed semantics. Noninteractive CLI
  streams now use fallible writes/flush boundaries (#214), fixing a reproduced
  broken-JSON-pipe panic/exit 101. I/O failures exit 1 and stop later commands;
  final/nested/write/flush and closed stdout/stderr cases are covered. Existing
  successful bytes/categories are retained; prior effects are not rolled back
  and no atomic file-publication guarantee is implied (MIG-0008).
- Measure responsiveness before optimizing. A dependency-free Rust example now
  records validated fresh-process version/core JSON workflows and separate session
  CSV load/summary/OLS/prediction times (#218); see the
  [measurement guide](performance/core-workflow.md). One excluded warmup, raw
  samples and profile/scope metadata are explicit. A matching local release
  baseline now retains two 10000-row runs and a 1000000-row run on macOS/aarch64
  (#219), with full release tests and unchanged correctness guards. This is not
  OS-cold-cache performance or cross-platform/release certification; interactive
  and other data shapes remain unmeasured. Optimize only demonstrated regressions
  from comparable profiles.
- Improve history/completion and display as needed for daily use. In-session
  `help`/`?` now provides bounded workflow guidance and packaged reference topics,
  including new `lincom`/`test` topics (#202); reference syntax is not an execution
  support guarantee.
- Add plotting or MCP transport only when a concrete workflow requires them.

## Later: optional estimator and backend breadth

Add estimators one at a time as **parse → execute → validate → report** slices.
Choose among GLM/count, IV/panel, causal, regularization, spatial/Bayesian, or
native backend integrations by actual demand and implementation cost. Syntax-only
coverage is not a milestone by itself. ReadStat/libgretl/plotting prototypes are
feasibility work until explicitly integrated and validated. Packaging/release
support must be evaluated for named platforms and dependencies.

## Always-on checks (risk-weighted)

| Risk | Minimum evidence |
| --- | --- |
| Routine docs/refactor | Diff, relevant local checks |
| Grammar, script or interface | Supported acceptance/diagnostic regressions |
| Data and file mutation | Data integrity, failure atomicity, lifecycle checks |
| Numerical inference | Sample/design + independent statistical reference |
| Native/FFI and distribution | Memory ownership, error/teardown, license/platform validation |

The root formatting, locked check/test, and Clippy commands are in
[CONTRIBUTING.md](../CONTRIBUTING.md). Run focused tests during iteration;
run the relevant full checks before sharing a completed code change. No need
to fuzz unrelated code or perform repetitive self-reviews by rote.

## Deliberate non-goals

- Exact Python parity for arbitrary error strings, whitespace and extremely
  large expressions/seeds unless they affect supported workflows.
- Full Stata/Python/R feature replacement.
- Building future backends, plugins, MCP resources or crates merely because
  they appear in an old proposal.
- Requiring entire early migration phases to be checked off before delivering
  useful runtime features.

## Keeping this roadmap useful

Maintain at most a handful of near-term outcomes. Replace completed outcomes
with the next highest-value work; link substantive test evidence rather than
adding PR-by-PR historical prose. Preserve old details in Git history and
existing evidence documents. Update [SPEC.md](../SPEC.md) when actual capability
changes, not merely when a syntax enum grows.

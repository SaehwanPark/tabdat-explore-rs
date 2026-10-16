# Archived workflow delivery ledger

Historical snapshot of the outcome roadmap before documentation cleanup.
This records completed slices and their evidence, not current priorities or new
implementation requirements. Use the [current roadmap](../TABDAT_RUST_PORT_ROADMAP.md)
and [SPEC](../../SPEC.md) for planning and supported behavior.

# TabDat Explore Rust: outcome-oriented roadmap

**Working plan, revised 2026-10-13.** Priorities are adjustable; this is not a
mandatory phase order or a commitment to full Python parity. Current support is
summarized in [SPEC.md](../../SPEC.md). The historical checklist and recorded
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
source diagnostics. See [script evidence](../migration/phase3-scripts.md).
Seed currently records context only; RNG semantics remain future command work.

The minimal shell in [PR #196](https://github.com/SaehwanPark/tabdat-explore-rs/pull/196)
provides persistent line input, core inspection output, recoverable command errors
and clean exit/EOF. `use` → `describe` → `summarize` → `exit` is covered through
the binary. Capable-terminal editing, bounded in-memory history and input-level
Ctrl-C recovery are now installed (#203); piped input remains plain. Bounded
core-command/help-topic Tab completion now supports word-end prefixes and preserves
arguments after the cursor. Local literal `use`/`run` path completion now supports
files/directories and compatible open quotes without shell expansion. Basic bare EDA
variable lists now complete current schema names from owned prompt-level snapshots,
without scans or cached-model inference. Bare `use` now also completes registered
identifiers from prompt-level registry keys, merging sorted/deduplicated path candidates;
name/file/directory precedence, partial-script refresh and plain-input behavior are checked.
Bare `append`/`join` targets now also complete registered names only, without filesystem
candidates, preserving typed join suffixes through real PTY append-order and join-suffix
workflows. Explicit `use` paths/quotes/options remain filesystem-only. EDA variable lists
also complete explicitly opened backtick names (including spaces, punctuation and Unicode),
with mixed lists and closing-quote/option suffix preservation; empty/control/embedded-backtick
names remain excluded. Expression/option/output-path completion, multiline continuation
and in-flight cancellation remain deferred; an executing command may still terminate on SIGINT. Local Unix PTY
evidence is not platform certification.

Bounded `status` now reports current dataset/active-named/registered-name metadata
separately from retained linear names/intercept/sample counts, without scans,
engine initialization or model-array clones. It supports the restore-required-columns
or explicit-refit recovery workflow, not a prediction-eligibility/staleness test or
uninstalled estimator/operation/materialization tracking.

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
other commands, or remote/lazy/DTA runtime was added by that slice. `run` now also
accepts one literal single/double-quoted script path, closing the replay gap for
script folders/names containing spaces or commas. Nested relative includes,
ordered text/JSON-script leaf results, source-classified failures, recursion guards
and retained model/data recovery are tested. No shell escapes, run options, process
working-directory changes are added; see MIG-0010 for the quoted-path correction.

Interactive category coding now reports encode/decode dataset outcomes and typed
label dictionaries (#235), replacing unsupported-details fallback in the
encode→inspect mappings→decode/persist workflow. Owned order/spelling and wide,
control/Unicode/empty labels, list filtering, clearing/drop, cached-state invariants,
nested recovery and all terminal write/flush boundaries are checked. Existing JSON,
parser/native coding and factor/ordinal semantics are unchanged; dictionary
persistence and general label-aware formatting remain uninstalled.

Every current owned result variant now has an explicit terminal display (#236):
named cohort creation/activation and join/append/reshape/recode outcomes reuse
owned metadata; grouped tables reuse raw scalar rendering with native-mean semantics
made explicit. Cohort enrichment/stacking/cleaning→analysis/persistence and reshape
layouts, exact owned/nested equivalence, native NULL/nonfinite/empty grouped means,
control-safe names/scalars, recovery/locations and every terminal I/O boundary are
checked. Exhaustive matching makes future result variants require a display decision.
No new backend/parser/JSON admission, rescans or transaction guarantees are implied.

Readable one-shot batch/script output is now opt-in via `--text` (#238), sharing
the owned renderer while retaining quiet defaults and existing CLI-v1 JSON.
Use→EDA→fit/inference/predict→persistence, native grouped means with explicit sample rules,
file/nested/batch-run leaf order, mode conflicts, source exits and every output/
diagnostic failure boundary are covered. Text is a human surface, not a machine
schema; no new scans, calculations or parser-command/backend breadth is added.

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
unweighted auxiliaries after WLS/GLS; nested/error/state paths are covered. The
single-predictor/no-intercept case now uses the zero-column auxiliary projection
(VIF 1 for a nonzero finite predictor), completing diagnostic→prediction/persistence
without relaxing general model validation. Independent NumPy projection references
cover retained OLS/robust/cluster/WLS/GLS samples, preserving all prior reference
values/tolerances; see MIG-0011. Other `estat` JSON remains unsupported. Direct SQL queries/named cohorts have their
own JSON admission (#225), sharing table/metadata envelopes rather than inheriting
support from VIF serialization. CLI-v1 `ttest` JSON covers the same four forms (#207), using
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
  Core `count`/`summarize`/`tabulate`/`regress` discovery syntax now includes installed
  conditional forms; tabulation adds frequency-option descriptors and regression
  correctly requires predictors. Existing reference descriptors remain; linked help
  and syntax-only previews distinguish parser acceptance from executable support.
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
  are retained. Conditional keep/drop row filters are now installed (#230):
  bounded TRUE selection with complementary UNKNOWN handling, preserved schema/
  relative order/labels, named synchronization and original cached fit/sample.
  Cohort→explicit refit→prediction/persistence, exact row identity, native NULL/
  nonfinite/arithmetic semantics, validation/staging and output guards are checked;
  functions/logical-expression breadth and general native atomicity remain out.
  Read-only conditional summaries (#231) now reuse bounded TRUE predicates and
  ordinary per-variable finite samples for every aggregate, without data/model
  mutation or grouped-conditional admission. NumPy/SciPy selected-row references,
  asymmetric missingness, exact extrema/ordering, empty/singleton/native-error
  paths, named/cache invariants, nested/persistence and output guards are checked.
  One bare `detail` flag now adds fixed 1/5/10/25/50/75/90/95/99 type-7 DOUBLE
  percentiles on the same per-variable finite sample, in terminal and a separate
  CLI-v1 `SummarizeDetailResult`. Independent NumPy samples/values, default/requested
  order/duplicates, empty/singleton/constant inputs, exact extrema, ordinary-result
  equivalence, named/label/fit state, native failures and nested output/file guards
  are covered. Higher moments, weights, configurable percentiles, extreme-value
  lists, grouped detail and other option combinations remain uninstalled.
  Read-only conditional frequency tables (#239) reuse bounded TRUE predicates,
  category SQL-NULL exclusions and existing one/two-way label/count/percentage
  assembly. Equivalent single-key rows()/columns() forms execute; pandas selected
  samples/counts/denominators, wide/quoted/nonfinite/empty keys, named/label/cache
  invariants, nested text/JSON/persistence and write/flush guards are checked.
  Multiple dimension keys, value aggregation, functions/logical-expression breadth
  and model-sample reinterpretation remain outside the installed scope.
  Read-only conditional counts (#240) now check full selected-cohort cardinality
  before finite summaries or category-complete frequency tables. Counts include
  every TRUE row regardless of unrelated NULL/nonfinite values; bare count and
  existing CountResult remain unchanged. Independent selected-index/cardinality
  contrasts, exact wide/quoted/native predicates, read-only state, nested text/JSON,
  analysis/persistence, recovery/errors and output guards are checked. No variable
  counts/options, grouped conditional counts or broader expression grammar is added.
  Selected-cohort linear estimation (#241) now fits the existing bounded regress
  estimators (OLS/WLS/GLS, nonrobust/HC1/cluster covariance, intercept
  conventions) on TRUE predicate rows without filtering active data. FALSE/UNKNOWN
  rows are excluded before finite outcome/predictor, cluster and weight validation;
  the estimation sample retains original active-row indices, total rows, exclusion
  counts and aligned weights/clusters. Successful fits replace the cached model and
  failed attempts clear it under existing policy; lincom/test/VIF consume the
  retained design while prediction stays current-data. Independent statsmodels
  selected-row references, bare-versus-selected sample contrasts, read-only state,
  nested JSON/text/persistence and failure/output guards are checked. No numeric/
  string truthiness, functions, logical breadth, richer formulas, temporary
  filtering or automatic refits are added.
  Key-integrity terminal/JSON reporting (#232) now exposes existing duplicates/
  isid aggregates for diagnosis→validated lookup→join/analysis/persistence.
  Exact native NULL/nonfinite/key counters, missok failure rules, empty/repeated/
  quoted keys, read-only state, nested/error/output guards are checked without
  duplicate repair, coercion or persistent cardinality enforcement.
  Assertion/fingerprint terminal/JSON reporting (#233) now completes bounded
  script-precondition→current-data record→analysis/prediction/persistence workflows.
  Exact owned fields, pinned digest/Parquet roundtrip, schema/order sensitivity,
  current-versus-model samples, read-only state, NULL/empty behavior, nested/terminal
  recovery and output guards are checked. No new predicate/hash semantics, file-byte
  checksum, authentication, saved-signature verification or lazy execution is added.
  Count/native-mean collapse JSON and generic terminal dataset outcomes (#234,
  #249) finish grouped aggregation→inspection→explicit aggregate-model refit/
  prediction→persistence. Counts exclude per-target SQL NULL versus COUNT(*);
  native means retain NaN/infinities, unlike finite summarize, and reuse the
  independent grouped-mean reference. NULL/empty values, requested schema order,
  multi/wide keys, label pruning/named synchronization, original cached fits,
  staged collision failures, nested/error/output boundaries and persistence are
  checked. JSON previews map nonfinite/all-NULL means to null without sample
  diagnostics. Quoted by() names and sum/min/max JSON remain unsupported;
  no automatic weighting, alias repair or aggregation-semantic change is added.
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
  Append JSON now reports compatible stacked metadata (#227), complementing
  lookup enrichment with left-then-right multi-source analysis/persistence.
  Name alignment/type rules, NULLs/duplicates, empty/self/repeated appends,
  cached/explicit-refit samples, preserved labels/named snapshots, detachment,
  validation and output guards are checked without backend or schema-union changes.
  Grouped count JSON (#228) now provides a bounded read-only subgroup EDA step:
  raw ordered keys, SQL NULL/empty-string counts, typed scalar policy and totals,
  repeated/quoted/empty groups, cached/state invariants and output/file guards.
  Bare `by ...: summarize [numeric_vars]` now also reports the existing owned
  ByResult native AVG table. A separate sample review preserves per-target SQL-NULL
  exclusion with NaN/infinity retained, not ordinary finite-summary samples. Python
  standard-library statistics.mean references record actual per-variable input
  indices and tagged IEEE results, checked before JSON's existing nonfinite-null
  encoding. Defaults/repeated targets, typed/quoted/repeated groups, empty/all-NULL
  results, cached/named/label state, nested fit/predict/persistence and output/failure
  guards are covered. No backend numerics, rescan, inferred counts or JSON fields
  change; conditional/options-bearing grouped summaries and other by children stay
  unsupported.
  Reshape JSON (#229) now connects bounded multistub wide/long preparation to
  grouped EDA, explicit refitting, prediction and persistence. Owned metadata,
  source/suffix ordering, native MAX duplicates/NULLs/types, selected-schema
  roundtrips, label/cache/named rules, failure-state and output/file guards are
  checked without installing a general lossless pivot or new panel semantics.
- JSON batch `-c "run ..."` now reuses script callbacks and bounded leaf admission,
  completing setup→nested replay→prediction/persistence in one shared batch session.
  Leaves keep their existing envelopes/order without a run wrapper; empty/directive/
  exit containers, quoted includes, unsupported-leaf preflight, located failures and
  every leaf write/flush plus diagnostic/error-envelope boundary are checked. Failed
  batch runs exit 1 while file/positional script source categories stay 1/2/3. No new
  result families, runtime semantics, rollback or native publication guarantees.
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
  [measurement guide](../performance/core-workflow.md). One excluded warmup, raw
  samples and profile/scope metadata are explicit. A matching local release
  baseline now retains two 10000-row runs and a 1000000-row run on macOS/aarch64
  (#219), with full release tests and unchanged correctness guards. This is not
  OS-cold-cache performance or cross-platform/release certification. The same
  measurement example now includes finite percentile detail with independent
  closed-form sorted-position guards and current matching release observations
  at 10000/1000000 rows; historical artifacts remain unchanged. This is execute-only
  latency, not memory measurement or evidence justifying an optimization.
  The existing Unix PTY acceptance tool now offers opt-in warmed persistent-shell observations
  for a fixed four-row load/count/summary/OLS/prediction cycle, including editor,
  rendering, transport and observer overhead. Two local release repeats retain raw
  samples/context; closed-form fixture guards, excluded warmup, cleanup and failure
  rejection are tested. Physical display paint, cold interaction and larger/other
  interactive data shapes remain unmeasured. Optimize only demonstrated regressions
  from comparable profiles.
- Improve history/completion and display as needed for daily use. Capable-terminal
  Tab completion now covers installed core command names and packaged `help`/`?`
  topics and the first local `use`/`run` path argument using the existing editor,
  without executing commands or changing session state. Path prefixes are literal
  and case-sensitive; compatible opening quotes enable spaces/commas/quotes, files
  close quotes and directories remain open for navigation. Existing quote/option
  suffixes survive; hidden names require a dot prefix. Parser roundtrips, native
  symlinks, unavailable entries/control-name guards, real PTY quoted loading/replay
  and unchanged piped tabs are checked alongside model-retention/input recovery.
  Basic bare EDA variable lists now complete literal case-sensitive current column
  names. An owned snapshot refreshes before every prompt, including failed operations,
  partial scripts and named-table switches, without backend scans or cached-fit names.
  Parser-safe/Unicode names, cursor suffixes, schema refresh, real PTY generated/
  renamed/scripted predictor completion and unchanged pipes are checked. Suggestions
  do not certify type/model eligibility or suppress repeated variables.
  Bare `use` registered identifiers now share literal sorted/deduplicated path candidates,
  refreshed from registry keys after partial scripts/errors without scans or model-array
  copies. Native exact-name precedence and explicit path navigation remain unchanged;
  real PTY named-cohort switching and same-name file/directory cases are checked.
  Bare `append`/`join` targets now also complete literal registered names, offering no
  filesystem candidates and preserving typed join suffixes; real PTY same-name append
  ordering and join-suffix workflows are checked. No extension/loadability inference
  or shell expansion is added. Explicitly opened backtick EDA variables now complete from
  the same current-schema snapshot, closing names without duplicating a typed closing quote.
  Mixed bare/quoted lists, Unicode, rename refresh and quoted summary→fit→prediction are
  covered; embedded-backtick/control/empty names, expressions/options/output paths and
  persistent history remain outside completion scope.
  In-session `help`/`?` provides bounded workflow guidance and packaged reference topics,
  including `lincom`/`test` topics (#202) and the four installed `ttest` forms
  (one-sample, paired, pooled and Welch, including the `unequal` alias). `ttest`
  discovery now links its topic and comparison/variance-option syntax; executable
  topic examples, sorted rendered-group contrast direction and retained-model
  help recovery are covered. Prediction help now teaches installed xb/residuals,
  current-data/retained-fit and fresh-target rules through inspection/export;
  executable examples reuse the independent linear reference and check recovery.
  Discovery corrects the rejected `probabilities` spelling to parser-reference `pr`
  without installing probability/Bayesian/spatial prediction (MIG-0012).
  Reference syntax is not a general execution guarantee.
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
[CONTRIBUTING.md](../../CONTRIBUTING.md). Run focused tests during iteration;
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
existing evidence documents. Update [SPEC.md](../../SPEC.md) when actual capability
changes, not merely when a syntax enum grows.

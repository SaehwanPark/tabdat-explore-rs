# TabDat Explore Rust: outcome-oriented roadmap

## Destination and current baseline

Build a practically usable, terminal-first tool for **graduate-level econometrics**.
The minimum long-term analytical target is the empirical analyses illustrated in
Cameron and Trivedi, *Microeconometrics: Methods and Applications* (2005), not merely
introductory OLS. This is a product commitment, **not a claim of current support**.
Exact Stata/Python syntax parity and implementation of every theoretical derivation
are not required; statistically equivalent, reproducible workflows are.

[SPEC.md](../SPEC.md) describes the installed baseline: bounded local CSV/Parquet,
EDA/preparation, scripts, terminal/JSON output and linear analysis. Most other
estimator families remain parser-only. The [coverage ledger](reference/microeconometrics-coverage.md)
maps required analytical work and textbook replication status. Completed-slice
history stays in the [archived delivery ledger](archive/delivered-workflows.md).

Maintain three distinct milestones:

1. **Dependable core:** usable data preparation and linear-analysis workflows.
2. **Graduate core:** common cross-section, IV, panel and nonlinear analyses with
   interpretation, diagnostics and reproducible reporting.
3. **Textbook empirical coverage:** validated replications of the book's empirical
   illustrations, including the more advanced families below.

The first milestone is useful but does not satisfy the second; the second does
not establish full textbook coverage. Missing required methods are **deferred
required work**, not optional breadth.

## How to select work

Choose a small, complete **load/prepare → specify → fit → diagnose/interpret →
report/reproduce** slice. Implement parse → execute → validate → report together;
parser catalog entries and native prototypes do not count as delivered analyses.

- Follow the priority order below, adjusting for demonstrated dependencies and
  correctness defects. The sections are not rigid phase gates: useful independent
  estimator slices need not wait for every foundation or editor improvement.
- Before implementation, state behavior, 2–5 acceptance cases, an important failure
  case, exclusions and the independent reference. For textbook work, identify the
  exact example/data and add or refine its coverage-ledger entry.
- Do not make all textbook inventory work a prerequisite to implementing one
  bounded example. Expand the ledger alongside delivery.
- Address statistical/data-integrity defects before breadth. Do not let repeated
  completion/help/editor polish indefinitely displace the missing graduate core.
- Use existing crates and boundaries. This roadmap does not authorize a new
  generic estimator framework, dependency or native integration by default.

## Priority 1: foundations that unblock real analyses

Preserve the supported core while removing concrete barriers to the next analysis.
These foundations should be delivered alongside the estimators that need them.

| Outcome | Required behavior and acceptance evidence |
| --- | --- |
| Reliable model design | Explicit categorical levels/reference categories, interactions and transformations; stable parameter order/names and prediction design. Test missing categories, absent/new prediction levels and collinearity policy against a trusted reference. Encoding numeric labels alone is not factor support. |
| Deliberate samples and weights | Report estimation exclusions; keep selected-cohort and current-data prediction semantics explicit. Distinguish analytic/frequency/probability weights before advertising them. Current WLS and inverse-diagonal-variance GLS are not general survey estimation or arbitrary-covariance GLS. |
| Interpretable results | Correlation/covariance tables, model-appropriate diagnostics, coefficient uncertainty and prediction uncertainty where defined. Preserve robust/cluster corrections and inference DF separately from fit residual DF. |
| Analytical graphics | Executable scatter/distribution, fitted/residual and later model-specific plots, with a documented terminal/file output path. Rendering syntax alone is not completion. |
| Reusable reporting | Named fit storage/comparison and exportable estimates/covariance/sample provenance; no silent refits or invented staleness guarantees. Preserve current advertised JSON contracts when extending results. |
| Reproducible data access | Tested local data preparation/persistence and textbook data ingestion. Provide native DTA or an explicit reproducible conversion path; document type/label/missingness losses. Do not require production Python/R. |

Keep ordinary finite summaries distinct from SQL-NULL missingness and native grouped
means. Keep failure/state and file-output behavior tested. Improve help/discovery
for each executed slice, rather than advertising future methods as available.

## Priority 2: required graduate econometrics core

Default estimator delivery order is IV → binary outcomes → count outcomes → linear
panels → quantile/resampling. A documented dependency or a concrete analysis may
justify a smaller reordering. Each row is multiple bounded deliverables, not a
single PR or a demand to expose every option immediately.

| Workflow | Minimum complete outcome |
| --- | --- |
| **IV/2SLS** | Exogenous/endogenous/instrument design, exact estimation sample, identification/rank checks, classical and appropriate robust/cluster inference, first-stage results and instrument-strength diagnostics. Add endogeneity, overidentification and weak-instrument-robust inference as applicable; label assumptions and unavailable tests, never imply ordinary Wald inference resolves weak identification. |
| **Logit/probit** | Fit binary outcomes; report convergence/separation failures and robust/cluster inference; predict probabilities and compute average marginal effects/discrete changes with delta-method uncertainty. Define factor/interactions and effect estimands. |
| **Poisson QMLE/negative binomial** | Count-domain validation, offsets/exposure, model-specific fit diagnostics, conditional means and effects. Specify the NB parameterization and distinguish Poisson robust QMLE from distributional assumptions. |
| **Linear panels** | Validate unit/time keys, order, gaps and duplicates; define lag/lead/difference semantics. The bounded [MMA POLS/annual-FD SQL workflow](../examples/mma21p1panel/README.md) now validates guarded keys/pairs and person-cluster inference. Bounded FE, Between, balanced/unbalanced Swamy–Arora RE-GLS, and balanced/unbalanced Gaussian RE-ML with positive-interior variance-component log-Wald intervals and a one-sided zero-unit-variance LR test are also validated. Interior ML additionally supports full joint unit-score inference; remaining illustrated ML inference and general panel semantics, panel/model-comparison diagnostics and remaining analyses are still required. |
| **Quantile regression** | Installed bounded unweighted IID-location inference and current-row predictions, validated .1/.5/.9 household fits; joint IID pairs-bootstrap covariance/marginal percentile intervals permit heteroskedasticity under IID regularity, with a shared-replicate B-1 SE and asymptotic normal test/95% Wald interval for one coefficient across exactly two quantiles. Cluster inference, general restrictions, simultaneous bands and Figure4.1 rendering remain. See MMA-QREG. |
| **Seeded bootstrap** | Real RNG/seed/stream contract, IID and cluster resampling, explicit replication failures and reproducible uncertainty. Bounded IID/whole-cluster pairs OLS now uses an explicit local xoshiro256++/SplitMix64 stream, B-1 covariance and percentile95 intervals, including MOM Between, within-transformed and fixed-theta RE-GLS whole-person workflows (conditional transform, not variance-component refits). Script context-only `seed` is separate; joint IID pairs quantiles add common draws/full cross-quantile covariance; residual/parametric/further-estimator resampling, automatic FE/RE refits and simulation remain required. |

Graduate-core acceptance requires representative end-to-end analyses across these
families, the design/reporting foundations they need, usable terminal and script
execution, and reference-validated interpretation and inference. An OLS result,
a passing parser test or a coefficient-only estimator is not that milestone.

## Priority 3: required applied microeconometrics coverage

Extend the same workflow/validation standard to:

- **Choice:** bounded shared-design model-based multinomial logit now supplies
  the [original fishing-mode Table15.2 MNL column](../examples/mma15p1mnl/README.md),
  current category predictions and retained-X continuous joint-delta AMEs with
  independent R evidence. Balanced single-choice conditional logit additionally
  supplies the price/catch CL column, explicit current choice-set probabilities,
  and section2B finite-change point contrasts with independent R/survival and
  original-token evidence. Explicit numeric design additionally validates the
  author's combined fixed-coefficient alternative-constant/income/price/catch
  column and conditional MNL encoding, including its different null/LR convention;
  this is not random-coefficients mixed logit. These workflows do not cover all
  historical float-storage/at-means effects, conditional effects uncertainty, IIA
  diagnostics or remaining Chapter15 illustrations. Explicit positive-scale
  non-normalized two-nest `nlogit` additionally supplies the source-audited
  [MMA §15.6.3 p511 workflow](../examples/mma15p1mnl/README.md#non-normalized-nested-choice),
  full observed covariance/IV=1 diagnostics, identified independence-restriction
  LR and current income/price/catch point contrasts. This earlier specification
  is not preferred common-utility RUM normalization; fitted IV-bound violations
  and the historical Stata8 stationary-point discrepancy remain explicit.
  A separately labelled [normalized common-scale extension](../examples/mma15p1mnl/README.md#normalized-common-scale-nested-choice)
  now supplies preferred common-utility normalization with one shared positive
  scale, full natural covariance/scale-one diagnostics and identified LR
  chi-square1;131 R quantities/41 covariance entries/23,640 source probabilities
  and an exact900-decision in-bound-scale fixture. The real source fit still
  violates RUM bounds; preferred normalization does not certify consistency.
  Unrestricted preferred nest-specific fitting, ordered/richer models, broader
  availability, effects uncertainty, diagnostics and all remaining illustrations
  stay required.
- **Limited outcomes:** the [classic lower-censored Gaussian Tobit workflow](../examples/mma16p1tobit/README.md)
  now supplies model-based full joint observed inference, retained continuous
  observed-mean delta effects, current latent/uncensoring/censored-mean prediction
  and48 original-source R reference gates/600 predictions. This bounded §16.2.1
  author-program extension is not a published table or complete Chapter16.
  The [explicit Heckman two-step workflow](../examples/mma16p1tobit/heckman.md)
  validates the same original200/130 selection/outcome rows with corrected
  generated-regressor covariance, owned state/five point projections and an
  independent source audit. Closed-form actual-script/failure controls,
  reverse-consumer checks and independent source review pass. This is not default
  ML, excluded-instrument/causal identification or RAND illustration parity.
  The [fixed-lower truncated Gaussian slice](../examples/mma16p1tobit/truncreg.md)
  now delivers native conditional ML/full inference/effects/current point projections.
  Original130-row R/independent audits, actual-script/failure checks, independent
  kernel/integration source reviews and local workspace regression gates pass
  (1515 tests/two pre-existing ignored,264 current artifacts). This fills the
  author's documented but unexecuted truncated-MLE gap,
  not historical/table parity. Richer truncation/censoring/inference, remaining
  selection/two-part workflows and illustrations stay required.
- **Survival:** the [original566-strike nonparametric workflow](../examples/mma17p1km/README.md)
  now composes existing SQL/table/export/SVG commands for tied-time Kaplan–Meier,
  Greenwood SE/pointwise log-log95% bounds and Nelson–Aalen/SE. Independent R,
  source-pinned author-output and actual SVG geometry audits pass. A separate
  [80-spell artificial right-censoring workflow](../examples/mma17p2kmextra/README.md)
  now validates event/censor ties, carried-forward censor-only times, KM intervals
  and NA log-Wald95% bands against R and both author tables. Exact executable
  40-failure/40-censor input takes precedence over mistaken prose labels; the
  source is artificial, not an empirical-regression substitute. These are bounded
  single-spell/entry-zero scripts, not general survival declarations, delayed
  entry/other censoring/truncation, Cox-baseline equivalence or a complete chapter.
  Those declarations plus empirical parametric/Cox models, baseline/survival/hazard
  outputs, diagnostics and remaining illustrations stay required.
- **GMM/systems and dynamic panels:** moments/instruments, weighting matrices,
  one-/two-step covariance, convergence, overidentification and serial-correlation
  diagnostics; control instrument proliferation rather than silently expanding it.
- **Treatment evaluation:** explicit ATE/ATT and identification assumptions,
  regression adjustment, propensity matching/weighting, balance/overlap diagnostics
  and estimator-appropriate uncertainty. Add DID where the selected analysis needs
  it; modern causal extensions must not displace the book's required workflows.
- **Complex surveys:** probability weights, strata/PSUs and design-based variance;
  one-way regression clustering alone does not satisfy this outcome.
- **Missing data:** imputation models, multiple-imputation pooling and diagnostics;
  finite-case deletion and missingness profiles alone do not satisfy it.

Choose a specific example before fixing detailed parameterizations, algorithms or
backend choices. Do not advertise a broad family based on one narrow estimator.

## Priority 4: complete the textbook empirical target

Use the coverage ledger to close remaining illustration-specific gaps:

- Nonparametric/semiparametric methods and quantile extensions.
- General ML/NLS, numerical optimization and systems/GMM cases not covered above.
- Mixture/unobserved-heterogeneity, hurdle/zero-inflated counts, frailty,
  competing-risk and multiple-spell survival models.
- Nonlinear panel models and remaining dynamic-panel analyses.
- Simulation-based and Bayesian estimation, including diagnostics and actual
  reproducible random draws.
- Measurement-error estimation and remaining missing-data analyses.

**Completion rule:** inventory every empirical illustration in the stated edition,
identify its data, preparation, model, inference and post-estimation requirements,
and link executable scripts plus independent reference evidence. Mark each entry
planned, in progress, blocked or validated. Any proposed exclusion requires an
explicit product-scope decision; it cannot silently count as coverage. The present
family-level ledger and seed examples are not a complete example inventory.

Equivalent native workflows are acceptable; reproducing Stata do-file syntax,
Python internals or every theoretical estimator discussed in prose is not the
acceptance criterion. Resolve edition/example ambiguities explicitly in the ledger.

## Supporting work and genuinely optional breadth

- **Interface reliability:** protect ordered script effects, source-classified
  errors, output-failure stopping and stable structured results. Add result access
  or script iteration when an actual replication requires it, not a speculative
  general programming language.
- **Interactive ergonomics:** completion/history/multiline/cancellation work follows
  demonstrated analysis friction, not command-catalog size.
- **Responsiveness/distribution:** use the existing [measurement harness](performance/core-workflow.md);
  packaging, licensing and platform claims need named-platform evidence.
- **Optional unless a required analysis demonstrates a need:** MCP transport,
  remote/lazy engines, spatial modeling, regularization and integrations beyond
  the analytical target. ReadStat/libgretl/plotting adapters are implementation
  choices, not product milestones or approved production dependencies.

## Next bounded actions

1. Advance another required econometric analysis or bounded panel diagnostic on an
   exact ledger-defined sample/design/reference. The
   [Table21.2 workflows](../examples/mma21p1panel/README.md) now include composed
   POLS/FD/Between, native FE, balanced Swamy–Arora model-based/unit-cluster GLS,
   and Gaussian ML with joint/constrained observed information and null/slope LR.
   The exact p719 robust auxiliary Hausman workflow now uses existing SQL/clustered
   OLS/test primitives:39 independent R quantities/95,760 cells and8 selected
   printed facts, with F(1,G-1) reference and explicit covariance assumptions.
   This does not install general native `estat hausman` or automatic model selection.
   Conventional nonclustered FE now also reports the pooled-versus-within
   unit-effects F test; independent R/plm comparisons cover balanced, unbalanced
   and interleaved synthetic fixtures, while clustered FE withholds the classical
   reference. The original MOM source is not bundled.
   Unbalanced ML adds a 4,738-row MOM sensitivity audit (65 scalars/52,118 cells)
   and531 synthetic numerical comparisons, with unequal-count null/information.
   Interior ML audits48 R quantities/58,520 cells and495 synthetic comparisons;
   zero-unit-variance ML adds378 base-R Gaussian ML/constrained-information
   comparisons with explicit boundary metadata. Either fitted boundary withholds
   ordinary slope-LR references; the separate unit-variance test uses its 50:50
   boundary mixture. The bounded profile and lack of a regular normal-Wald
   unit-variance SE remain explicit. A small unresolved difference from old printed
   Stata8 SEs prevents claiming the published uncertainty column. Interior ML
   unit-score inference now validates53 independent R quantities and58,520 cells
   on the same MOM design, plus292 synthetic comparisons across balanced/unequal
   counts and a restricted-null boundary. It uses full joint observed bread,
   G/(G-1), normal/log-Wald and rank-aware slope chi-square inference, with no
   robust LR references or few-cluster guarantee. Boundary sandwich and non-unit
   clusters remain unsupported; select further ML extensions from exact illustrated
   requirements. Individual unit-intercept uncertainty, cluster-robust unit-effects
   tests, other Hausman/panel diagnostics, bootstrap and remaining table/figure
   outputs stay required. No persistent
   panel declaration or whole-table/family coverage is claimed.

2. Advance the next bounded MMA-COUNT requirement from its now-resolved
   source/sample/design: the [native Table20.5 Poisson portion](../examples/mma20p1count/README.md)
   and separate NB2 script validate model-based/independent-row robust coefficients,
   full inference/diagnostics/continuous effects and all 20,186 means/indices
   against R (Poisson1,095/NB2 1,191 quantities). NB2 specifies variance
   mu+alpha*mu² and full joint observed-information covariance for beta/lnalpha,
   not conditional GLM/Fisher covariance. Neither completes the count family or
   panel analysis: remaining exposure workflows, household/site/multiway/survey inference,
   other formal NB specification tests and remaining table outputs stay required; the boundary LR p-value is currently model-based only.
   The native Poisson original-person extension now audits20,186 source rows/
   5,908 actual people and validates551 additional R quantities with G/(G-1)
   group-score/normal inference. It assumes independent people, not household/site
   dependence or FE/RE; development-only DTA identity conversion is not native DTA IO.
   The NB2 original-person extension now independently validates599 additional
   quantities/all source/ID/design/mean/index records using FULL joint beta/lnalpha
   observed bread and grouped joint scores; normal95/transformed dispersion CIs,
   not conditional/Fisher substitution, G-1 t, few-cluster guarantee or alpha=0 test.
   Retained `estat countprob` now supplies descriptive observed/mean-fitted bins,
   validated against2090 R scalar/95 empirical quantities and444092 individual
   native-helper probabilities. It assumes full laws, not QMLE calibration or a
   formal test. The materially different printed Table20.6 NB2 row remains
   unresolved; native/November2006 author-program evidence does not complete it.
   The shared log-gamma Lanczos coefficient is now corrected, with 153
   independent base-R regressions for positive log-gamma, Student t tails/quantiles,
   F tails and normal probabilities. This improves existing inference paths without
   changing sample/covariance/DF conventions or adding textbook coverage;
   integer-shape count probabilities remain on their separate numerical path.
   Fixed Poisson `offset(var)` is now separately validated on a synthetic
   [complete workflow](../examples/poisson-offset/README.md): joint finite sample,
   same-offset null, current offset predictions and retained conditional-offset
   effects/count bins, with 950 independent base-R quantities across three designs
   and three covariance choices. NB2 fixed offsets now have a separate
   [synthetic workflow](../examples/nb2-offset/README.md) validating 1,076 R/MASS
   quantities, full joint beta/lnalpha observed covariance/group scores, same-offset
   dispersion-refitted nulls, current predictions and retained conditional-offset
   effects/count bins. These are method evidence, not new book coverage.
   Current-row `predict ..., pmf(k)`/`cdf(k)` now supports scalar or row-specific
   integer cutoffs0..100000 with retained alpha/offsets, a 10,000,000-increment
   expanded-work bound above99 and atomic publication;308
   independent base-R helper values,432 fixed-cutoff and144 row-specific
   offset-fit/current-row probabilities validate the bounded
   [complete workflow](../examples/count-probability/README.md). Full laws remain
   assumptions; robust covariance does not establish distributional calibration.
   Select one exact specification/reference before extending the estimator. The defined
   §14.2 native published-analysis workflow is now validated, including all
   source/cohort/fitted summaries, Tables14.1–14.2 and Figure14.1; this does not
   complete the binary family. Needed Fisher/factor/interaction/discrete-change
   foundations, maximum score and other exact binary illustrations remain
   required. Native OLS/logit/probit named reports are not fit restoration or
   reporting for every future estimator. One-way clustered covariance remains
   external method evidence, not a newly validated textbook illustration.

The classical/robust-MLE portion of the [§14.2 binary
slice](../examples/mma14p1binary/README.md) is delivered: the logit column
supplies model-based MLE/normal coefficient and LR inference, current-data
probabilities/xb, and retained-X continuous AME with conditional-design delta
uncertainty; the probit column (`probit.td` / `reference_probit.R`) delivers the
same workflow on Newton steps against the per-observation observed information,
with the R reference polished to the analytic score fixed point. R4.6.1 validates
all 630 source-row identities and numeric outputs for both families, including
tiny test tails. Unclustered robust fits now use N/(N-1) score-sandwich
covariance with converged observed-information bread, independently validated
for both families and propagated to conditional-X AME uncertainty. Their LR
p/reference are unavailable, not falsely advertised as robust LR inference.
Robust/clustered omnibus Wald inference now tests all non-intercept coefficients
using selected covariance and asymptotic chi-square(q); singular/rank-limited
restrictions are explicitly unavailable, without dropping the fit or silently
reducing q. Independent R validates the exact §14.2 robust tests under the
converged-information convention, not exact historical Stata8 Wald equality.
Probit deliberately retains the
fixed-point convention rather than old Stata8 information lag.
Combined outlier/separating-direction tests cover the
independent-review convergence defect; absolute normalized full-step and score
bounds fail rather than publish an unconverged fit. One-way clustered fits now
use G/(G-1) cluster-score covariance and normal (not G-1 t) inference, with
independent external Bangladesh method evidence in the coverage ledger. That
is not survey inference or a small-cluster reliability guarantee. The native
six-fit script now supplies the numerical classical/robust logit/probit/OLS
comparison for Table14.2, with 94 quantities and every source/design/prediction
record independently checked in R. OLS Gaussian working likelihood and unbounded
linear fitted values are explicitly distinguished from Bernoulli probabilities.
The original development-only checker's joined display remains only a validation
convenience. A separate native named-report workflow now stores the selected
OLS/logit/probit coefficients, covariance and fit-time sample provenance, and
renders the six requested columns without refitting. All 94 table quantities
match independent R; normal binary versus t-based OLS inference and distinct
likelihood/R² conventions remain explicit. Reports survive edits/reloads/refits
but cannot restore fit caches or establish current-data staleness/identity.
A native SVG scatter/three-line overlay now
renders Figure14.1's 630 outcomes and 1,890 independently checked R prediction
vertices, including actual geometry at 1e-8 pixel absolute tolerance. The visible
OLS range stays unbounded; layout/fonts and deterministic y-only display jitter
are intentionally not historical Stata RNG/pixel parity. The shared command
preserves data/fit/labels/order and publishes without replacement; PNG, automatic
opening and broader graphics remain uninstalled. Table14.1's overall/pier/charter
summaries now run through native conditional summarize: all 60 quantities and
630 sample/design records independently match base R, with sample n−1 SD and
unchanged prepared source order. This is three ordered summary blocks, not a
joined-table command. A composed `analysis.td` now includes the remaining source,
full prepared-cohort and fitted-value summaries: independent base R validates all
465 quantities in eight blocks/93 rows, plus every field/ID of the 1,182 source
and 630 prepared records, using unchanged tolerances. The same run also passes
all six-model/table/prediction and SVG geometry checks. JSON/text/piped CSV/SVG
bytes match the leaf workflows. Source/program/output hashes are verified; the
example audits each author block against the native representation. This
validates the defined §14.2 published tables/figure and all descriptive blocks,
not every auxiliary Stata fit-log field/layout, historical float32/RNG/WMF/ASCII
parity or fit restoration. Individual OLS ANOVA/F/interval displays are not added
to the comparison table; native test/lincom remain separate. Earlier successful
script outputs remain on later failure, not whole-script rollback. Reporting for
future estimator families remains required. This completes that bounded native
analytical contract, not the binary milestone, IV illustration or full book.

Keep GMM/Hansen J, robust/cluster AR confidence sets, endogenous-subvector confidence sets and
any further endogeneity inference required by exact illustrations explicit as
remaining required work.
Conventional Wald results, Sargan/Wu diagnostics and first-stage relevance tests
do not establish weak-IV-robust coefficient inference.

The schooling-IV 2SLS core now reports ordinary and Shea partial R-squared
separately, validated against the author output on the DATA66 sample; see the
[coverage ledger](reference/microeconometrics-coverage.md). This does not establish
completion of the illustration. A separate `estat ar, null(...)` command now supplies
homoskedastic weak-IV-robust **joint tests of all endogenous coefficients** on the
retained fit sample. Single-endogenous fits additionally support analytic
homoskedastic confidence sets via `estat ar, ci [level(95)]`, including disjoint,
unbounded and empty sets within documented numerical limits. For multiple-endogenous
fits, `estat ar, ci coefficient(name) [level(95)]` analytically projects the joint
AR region onto one coefficient while leaving every other endogenous coefficient
unrestricted; this has at-least-nominal coverage and can be conservative. It is
validated on a synthetic R reference and the original DATA66 sample: N=3,010 and
the 95% grade76 set is [-0.01438755244, 0.50285323590], matching an independent
base-R profile. This does
not supply robust/cluster projected confidence sets, multidimensional projected
regions or the less-conservative subvector AR test. A separate
[synthetic robust AR workflow](../examples/robust-ar/README.md) now validates
retained-sample HC1/one-way CR1 auxiliary-regression joint tests against base R
under three designs, with explicit auxiliary corrections and asymptotic
chi-square(q) inference. This is method evidence, not new book coverage or
few-cluster validation. The separate
[overidentified AER cigarette-demand example](../examples/cigarettes_iv/README.md)
now validates `estat overid` Sargan N*R² with asymptotic homoskedastic chi-square
inference on 48 states in 1995. Its sample/design/reference is specified; this
advances method coverage, not the Cameron–Trivedi empirical-illustration ledger.
Sargan is not robust Hansen J, an endogeneity test, or proof of instrument validity.
Independent-row robust parent fits now use a distinct residualized-instrument
Wooldridge score, with uncorrected score OPG, retained structural residuals and
asymptotic chi-square(L-K). Independent R SVD/lm.fit matches the exact 48-state
source/design/score/tail and five synthetic q1/q2 designs; see the same example.
This is additional method evidence, not new textbook coverage or Hansen J.
Clustered overidentification and GMM remain required work.
The same specified sample additionally validates joint homoskedastic
`estat endogenous`: Wu–Hausman F(1,44)=3.067816272944078, p=0.08682504624130956,
using auxiliary original/innovation-augmented OLS. A separate retained-sample HC1/one-way CR1 `estat endogenous` now tests all
first-stage innovations jointly by augmented-OLS Wald with asymptotic chi-square
inference. Its auxiliary K/DF/corrections and exact 48-state HC1 R reference are
explicit in the same example; five synthetic designs independently validate HC1
and grouped-score CR1. This is additional method evidence, not new book coverage.
Subset forms, robust score/GMM C variants and few-cluster guarantees remain
unsupported; these conventional tests assume valid instruments and do not
establish exogeneity or instrument validity by failing to reject.

Implement the selected bounded plan without broadening scope. If it conflicts
with the codebase or reference semantics, stop and report the conflict rather than
improvising. Report changed files, tests, deviations and unresolved limitations.

## Verification and scope boundaries

Follow [AGENTS.md](../AGENTS.md) and [CONTRIBUTING.md](../CONTRIBUTING.md):

| Change | Evidence |
| --- | --- |
| Docs or low-risk refactor | Diff, links and relevant checks |
| Grammar, script, CLI | Supported acceptance/error regressions |
| Data, session or file effects | Invariants, failure state and representative end-to-end workflow |
| Numerical inference | Independent reference, exact samples/design, parameterization, covariance/DF and justified tolerances; convergence/failure cases when applicable |
| Native/FFI or distribution | Ownership, lifecycle, licensing and platform-specific checks |

Development-only reference tools may use R/Python/Stata; the production runtime
must not require them. Never widen tolerances to conceal unexplained differences.
Do not reinstate exhaustive historical parity gates, routine repeated reviews or
unrelated fuzz campaigns. Commit/push/PR/merge only within current authorization.
Update SPEC when capabilities change, the ledger when example evidence changes,
and this roadmap when priorities change; keep detailed logs beside tests or PRs.

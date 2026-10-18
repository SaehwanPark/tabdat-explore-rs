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
| **Linear panels** | Validate unit/time keys, order, gaps and duplicates; define lag/lead/difference semantics. Deliver FE, RE, between and first-difference workflows with appropriate cluster inference and panel/model-comparison diagnostics. |
| **Quantile regression** | Explicit quantile, estimation sample, solver/convergence policy, inference and predictions; independently validate coefficients and uncertainty. |
| **Seeded bootstrap** | Real RNG/seed/stream contract, IID and cluster resampling, explicit replication failures and reproducible uncertainty. Current context-only `seed` is not sufficient. |

Graduate-core acceptance requires representative end-to-end analyses across these
families, the design/reporting foundations they need, usable terminal and script
execution, and reference-validated interpretation and inference. An OLS result,
a passing parser test or a coefficient-only estimator is not that milestone.

## Priority 3: required applied microeconometrics coverage

Extend the same workflow/validation standard to:

- **Choice:** multinomial and conditional choice, then ordered/nested or richer
  specifications needed by the book's illustrations; choice-set structure,
  probabilities, effects and relevant diagnostics.
- **Limited outcomes:** censored/truncated regression, Tobit, Heckman selection
  and two-part workflows; likelihood/two-step distinctions and correct uncertainty.
- **Survival:** event/censoring/truncation declaration, Kaplan–Meier, parametric
  and Cox models, with baseline/survival/hazard outputs and diagnostics.
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

1. Choose the next required binary-family outcome from the coverage ledger and
   scope it against its exact illustration before implementation — currently the
   native reusable fit-comparison/reporting and remaining author outputs for
   §14.2, followed by the Fisher/factor/interaction and discrete-change foundations
   exact illustrations need. One-way clustered covariance is delivered as external method evidence,
   not a newly validated textbook illustration. Do not treat the binary family
   as one oversized PR or these classical logit/probit slices as completion.

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
A joined comparison printed by the development-only checker is not native
named-fit storage/table support. A native SVG scatter/three-line overlay now
renders Figure14.1's 630 outcomes and 1,890 independently checked R prediction
vertices, including actual geometry at 1e-8 pixel absolute tolerance. The visible
OLS range stays unbounded; layout/fonts and deterministic y-only display jitter
are intentionally not historical Stata RNG/pixel parity. The shared command
preserves data/fit/labels/order and publishes without replacement; PNG, automatic
opening and broader graphics remain uninstalled. Table14.1's overall/pier/charter
summaries now run through native conditional summarize: all 60 quantities and
630 sample/design records independently match base R, with sample n−1 SD and
unchanged prepared source order. This is three ordered summary blocks, not a
joined-table command. Other author exploratory/fitted-value summaries and
reusable reporting foundations remain unfinished. This advances
one textbook subset, not the full illustration or binary milestone; advancing
from these classical slices did not complete IV or the binary family.

Keep GMM/Hansen J, robust/cluster AR, endogenous-subvector confidence sets and
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
unbounded and empty sets within documented numerical limits. This does not supply
schooling-specific scalar confidence sets with endogenous nuisance parameters,
endogenous-subvector inference or heteroskedastic/cluster AR. The separate
[overidentified AER cigarette-demand example](../examples/cigarettes_iv/README.md)
now validates `estat overid` Sargan N*R² with asymptotic homoskedastic chi-square
inference on 48 states in 1995. Its sample/design/reference is specified; this
advances method coverage, not the Cameron–Trivedi empirical-illustration ledger.
Sargan is not robust Hansen J, an endogeneity test, or proof of instrument validity.
The same specified sample additionally validates joint homoskedastic
`estat endogenous`: Wu–Hausman F(1,44)=3.067816272944078, p=0.08682504624130956,
using auxiliary original/innovation-augmented OLS. Robust/cluster/subset forms
remain unsupported; this conventional test assumes valid instruments and does
not establish exogeneity or instrument validity by failing to reject.

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

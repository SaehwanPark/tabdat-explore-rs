# Graduate microeconometrics coverage and replication ledger

## Authority and status

The product target is practical graduate-level econometrics, with at least the
empirical analyses in Cameron and Trivedi, *Microeconometrics: Methods and
Applications* (Cambridge University Press, **2005**) as the long-term analytical
acceptance target. This is not *Microeconometrics Using Stata*, and it does not
require exact Stata/Python command compatibility.

- [SPEC](../../SPEC.md) records **current executable capabilities**.
- [Roadmap](../TABDAT_RUST_PORT_ROADMAP.md) sets **delivery priorities**.
- This ledger records **analytical gaps and replication evidence**. Planned rows
  are not promises of installed commands, numerical validation or release readiness.
- The [development-status assessment](../artifacts/research/graduate-microeconometrics-gap-assessment.yaml)
  is a commit-specific research snapshot, not a live backlog.

## Required method coverage

All rows below are required target areas. Delivery may be incremental; advanced
methods are not optional merely because they follow the graduate-core milestone.
Detailed scope is determined by the empirical illustrations and their assumptions,
not by implementing every estimator mentioned in a chapter.

| Book chapters / area | Installed baseline | Required remaining outcomes |
| --- | --- | --- |
| 4: Linear, IV and quantile models | Bounded OLS/WLS/diagonal GLS, HC1/one-way cluster; IID-location qreg and joint IID pairs quantile bootstrap with one two-quantile coefficient contrast | Remaining IV diagnostics, analytic/cluster quantile uncertainty, general quantile restrictions and simultaneous bands; factor/interactions and weight semantics |
| 5–6, 10: ML/NLS, GMM/systems, optimization | Least squares and bounded binary logit/probit MLE/QR | Remaining nonlinear/system estimation, identification, optimization and valid covariance/convergence diagnostics |
| 7–8: Testing and specification | Linear restrictions, mean tests, VIF; binary LR/normal coefficient, robust/clustered omnibus Wald and delta-AME inference | Further model-specific LR/LM/Wald and specification tests, nonlinear restrictions, comparison and uncertainty |
| 9: Nonparametric/semiparametric | No executable workflow | Density, local/kernel regression and methods required by illustrations |
| 11–13: Bootstrap, simulation, Bayesian | Native bounded IID/whole-cluster pairs OLS bootstrap with explicit local seed, B-1 SE/covariance/bias and percentile95 intervals; audited MOM Between independent-person means and within-transformed whole-person resampling | Residual/parametric/other-estimator bootstrap and automatic FE/RE refits, simulation/Bayesian estimation and diagnostics; context script seed remains separate |
| 14: Binary outcomes | Model-based/robust/one-way clustered logit and probit, probabilities/xb and continuous retained-X delta AME; validated native §14.2 published tables/figure and descriptive workflow under explicit conventions | General binary restrictions/specification tests, factors/interactions/discrete changes and uncertainty, other illustration outputs/graphics |
| 15: Multinomial outcomes | Bounded person-level MNL fit/probabilities/joint-delta AMEs; balanced price/catch and combined fixed-utility conditional choice with keyed probabilities/point contrasts; source audits of three principal Table15.2 coefficient columns and conditional MNL encoding | Nested and other source-resolved choice analyses, broader choice-set semantics, conditional effects uncertainty/IIA diagnostics; historical float-storage/at-means effects and remaining illustrations |
| 16: Censoring, truncation, selection | Validated bounded lower-censored Gaussian Tobit/source workflow with full observed inference/current predictions/retained continuous effects | Remaining censoring/truncation, selection and two-part workflows with correct effects/inference; Heckman two-step source/reference slice now WIP |
| 17–19: Survival, heterogeneity, multiple hazards | No executable workflow | Event/censoring/truncation semantics, KM/parametric/Cox, mixture/frailty and multiple-event analyses |
| 20: Counts | Model-based/independent-row robust unweighted Poisson and joint beta/lnalpha NB2, means/xb, descriptive deviance/Pearson and retained-X continuous delta effects; validated Table20.5 coefficient/inference portions, audited original-person clustered extensions and descriptive retained observed/fitted probability-frequency bins; current-row PMF/CDF cutoffs0..100000 with a bounded expanded-work contract | Remaining exposure workflows (fixed Poisson/NB2 offsets have synthetic method evidence below), household/site/multiway/survey/panel inference, unresolved printed Table20.6 NB2 row, other formal NB specification tests and robust/clustered NB boundary-LR reference, hurdle/zero-inflated/mixture and remaining illustrated outputs |
| 21: Linear panels | Native SQL/OLS POLS, annual FD and Between classical/HC1 portions, native balanced/unbalanced Swamy–Arora model-based/unit-cluster RE-GLS and balanced/unbalanced Gaussian RE-ML with interior full joint unit-score inference, plus bounded explicit-key within FE slope/conditional-prediction workflow, the conventional pooled-versus-within unit-effects F test for nonclustered fits (clustered fits withhold this classical reference), retained effects and independent original-source evidence | Persistent panel declaration/general lags, remaining illustrated ML inference, individual FE intercept uncertainty, cluster-robust unit-effects tests, two-way effects, panel-specific diagnostics/comparisons, bootstrap and remaining table/figure illustrations |
| 22–23: Extended/nonlinear panels | Relevant parser forms only | Dynamic GMM and nonlinear panels with instrument controls and appropriate inference |
| 24: Stratified/clustered samples | One-way linear cluster covariance | Survey weights, strata/PSUs and design-based workflows |
| 25: Treatment evaluation | Relevant parser forms only | Explicit estimands, adjustment/matching/weighting, balance/overlap and valid uncertainty |
| 26: Measurement error | No executable workflow | Illustrated measurement-error correction/IV/sensitivity methods |
| 27: Missing data | Missingness inspection, finite-case exclusions | Imputation models, multiple-imputation pooling and diagnostics |

Chapters 1–3 supply assumptions and data-structure context rather than a separate
estimator checklist. Reproducible random draws from Appendix B are an enabling
requirement for simulation methods. Existing linear reference tests establish
bounded numerical behavior, **not a validated textbook replication**.

## Seed empirical illustrations — inventory incomplete

These are starting points from the authors' [dataset index][data], not a complete
inventory or exact replication specifications. Page ranges are locators from that
index; inspect the book and accompanying program before naming a table, sample,
variable list or estimator variant. Source data must be checked for access,
provenance and redistribution terms before adding fixtures.

| ID | Book locator and data lead | Status | Next evidence / blocker |
| --- | --- | --- | --- |
| MMA-IV | pp. 110–112, schooling/weak instruments; `DATA66.dat` + dictionary | In progress — Table 4.5 2SLS core validated | Coefficients, classical/HC1 SEs, N, conventional first-stage F and Shea partial R² match the author output; scalar weak-IV-robust intervals remain planned; bounded joint AR method evidence below |
| MMA-QREG | §4.6.4 pp. 88–90, Vietnam living standards; author `mma04p2qreg.do`, original `qreg0902.asc` | Partial: native .1/.5/.9 conditional quantiles and IID-location inference/prediction; joint IID pairs-bootstrap covariance/marginal percentile intervals and one two-quantile shared-pairs coefficient contrast with asymptotic normal inference | Original 5,999 rows / 5,006 finite cases; R quantreg validation, source identity and prediction audit in [workflow](../../examples/mma04p2qreg/README.md). Native Figure4.2 SVG validates all observed/line coordinates and geometry; DTA precision and historical Stata SE differ. [Joint bootstrap method](../../examples/mma04p2qreg/bootstrap.md) documents the .25/.75 author slope comparison; Figure4.1 bands and rendering remain |
| MMA-NONPARAM | pp. 295–297, 300, PSID; `psidf3050.dat` | Planned | Identify density/regression specifications, bandwidths and reference outputs |
| MMA-BINARY | §14.2, pp. 464–466, Tables 14.1–14.2/Figure 14.1; `mma14p1binary.do` / `Nldata.asc` | Validated native published-analysis contract — composed source/cohort/fitted summaries, native tables and SVG; separate binary AME extension | Exact output audit/conventions below; not literal Stata log/storage/RNG/format parity or fit restoration; binary family remains incomplete |
| MMA-MAXSCORE | §14.7.5, p. 486; `mma14p2maxscore.lim` / `mma14p1binary.asc` | Planned — author-index locator only | Resolve maximum-score specification, preparation and nonstandard uncertainty; not covered by logit |
| MMA-CHOICE | §15.2.1–3 pp. 491–495, fishing modes; `Nldata.asc` / `mma15p1mnl.do`; later `mma15p4gev.asc` | Partial — principal Table15.2 MNL/CL/combined coefficient workflows under explicit float64/inference conventions; separate analytic MNL continuous-AME extension | [Original1,182-person workflows/audits](../../examples/mma15p1mnl/README.md):107/61/218 R quantities, exact source/design and predictions; remaining program/table effects, diagnostics and other illustrations required |
| MMA-NESTED | §15.6.3 p511; `Nldata.asc` / `mma15p2gev.do` sections2A–C | Partial — earlier non-normalized two-nest likelihood, joint observed inference, identified λ=1 restriction and current point contrasts | [Source-bound native workflow/audit](../../examples/mma15p1mnl/README.md#non-normalized-nested-choice):149 R quantities including52 covariance entries/23,640 probabilities; preferred RUM normalization remains required; historical stationary-point discrepancy explicit |
| MMA-TOBIT | §16.2.1 pp.530–531, author `mma16p1tobit.do` §2A/`mma16p1tobit.asc` | Partial — validated native classic lower-censored Gaussian MLE on the author's exported200-row generated data | Exact70 censored/130 uncensored; native source script/full observed inference/600 predictions/continuous effects validated against48 R gates, with current workspace regression evidence and independent source/evidence review. Estimation is the author's program extension, not a published coefficient table; theoretical Figure16.1, CLAD/truncation/selection remain separate |
| MMA-SELECT | §16.2.1 pp.530–531, author `mma16p1tobit.do` §2A Heckman comparison | WIP — validated bounded corrected two-step kernel on original200/130 selection/outcome rows, terminal workflow pending |58 original R gates plus98 synthetic quantities/3600 synthetic predictions; separate covariance blocks, no exclusion restriction; owned runtime/parser/reporting/scripts remain; distinct from the RAND selection illustration below |
| MMA-SELECTION | pp. 553–556, 565, RAND; `randdata.dta` / `mma16p3selection.asc` | Planned | Separate selection/outcome samples, estimator variants and uncertainty |
| MMA-DURATION | pp. 574–575, 582, strikes; `strkdur.asc` | Planned | Specify spells, censoring, parameterization and survival/hazard references |
| MMA-HAZARDS | pp. 603–608, 632–636, 658–662, displaced workers; `ema1996` | Planned | Identify event types, repeated-spell/heterogeneity requirements and samples |
| MMA-COUNT | §20.3 pp. 671–674, Table20.5 p. 673; later p. 692, RAND; `mma20p1count.asc` / `randdata.dta` | Partial — Poisson/NB2 coefficients and model-based/independent-row robust statistic columns, plus model-based Poisson-boundary NB2 LR test, validated | 20,186 all-year prepared person-years; [native workflows/source audit/R references](../../examples/mma20p1count/README.md), Poisson1,095/NB2 1,191 quantities/all source/design/mean/index records; native person-Poisson551/NB2 599-quantity extensions/5,908 original people; native/November2006 author-program frequency analysis95 quantities/444092 individual probabilities validated; current-row PMF/CDF above99 additionally checked against20 base-R values, printed Table20.6 NB2 row unresolved; remaining tables and richer counts remain required |
| MMA-PANEL | §21.3 pp.709–714, Table21.2 p.710, PSID; `MOM.dat` / `mma21p1panfeandre.do` | Partial — POLS/FD, Between/RE-GLS, unbalanced RE-GLS sensitivity, interior ML coefficient/likelihood, positive-interior variance-component uncertainty, the one-sided zero-unit-variance LR test and Within slope/conditional-prediction portions validated | [Guarded keys/annual pairs, native explicit-key FE, inference/effects/prediction and R evidence](../../examples/mma21p1panel/README.md); individual unit-intercept uncertainty, remaining illustrated ML inference, cluster-robust unit-effects testing, Hausman and other panel diagnostics/bootstrap, lowess/figures and remaining table outputs required; bounded p719 robust auxiliary comparison and synthetic R/plm unit-effects F references are validated below |
| MMA-DYNAMIC | pp. 754–756, PSID; `MOMprecise.dat` | Planned | Specify moments, instruments, weighting and diagnostic references |
| MMA-NLPANEL | pp. 792–795, patents/R&D; `patr7079.asc` | Planned | Identify nonlinear panel estimators and heterogeneity assumptions |
| MMA-SURVEY | pp. 848–853, Vietnam; `vietnam_ex1` / `vietnam_ex2` | Planned | Determine design/weight/cluster semantics, not just regression clustering |
| MMA-TREATMENT | pp. 889–895, NSW/PSID; `nswpsid` | Planned | Fix ATE/ATT, sample/control group, propensity/matching choices and inference |

### MMA-CHOICE bounded MNL contract and evidence

The March2005 author `mma15p1mnl.do` executable section identifies Table15.2,
p.493 (the overview's Table15.1 is a typo). Its `mlogit mode ydiv1000,
basecategory(1)` uses all1,182 original `Nldata.asc` people, income/1000 and an
intercept, beach1 as reference; pier2/private3/charter4 counts are178/418/452,
with134 beach observations. Reuse the [16-field token-stream converter](../../examples/mma14p1binary/nldata_to_csv.awk),
preserve source IDs/order, and make no stochastic preparation or sample filter.
The [development R/nnet reference](../../examples/mma15p1mnl/reference_mlogit.R)
covers the full joint model-based Fisher covariance, normal95 coefficient
inference, scalar category probabilities/indices and conditional-X continuous
AME/delta inference. The author p494 finite-difference averages and at-means
`mfx` must not be silently equated with analytic AMEs or their uncertainty.

The [native workflow and audit](../../examples/mma15p1mnl/README.md) verify107
independent-R scalar quantities, all21,276 source/design cells,4,728 category
probabilities and4,728 indices; additional synthetic references cover376
quantities, two predictors, no-intercept and nonlowest-base covariance/effect
invariants. Retained-state/current-row and exact-DECIMAL/failure contracts have
focused executable tests. Native historical coefficients/SEs agree within1e-6,
not every printed SE digit; author finite-difference versus analytic AME
differences remain explicitly separated. Their individual float-storage versus
optimizer contributions have not been isolated. That MNL-only evidence makes no
conditional/nested, alternative-specific, robust/cluster, factor/discrete-change,
IIA-test, whole-table/program or completed Chapter15 claim; the separately
validated conditional price/catch analysis follows below.

### MMA-CL bounded conditional-choice contract and evidence

The [native script/audit](../../examples/mma15p1mnl/README.md#conditional-logit-table152-cl-column-and-section2b-point-contrasts)
targets the same author's section2/Table15.2 CL column: all1,182 people × four
explicitly keyed alternatives =4,728 long rows, exactly one chosen row per person,
alternative-specific price/catch and no common intercept or alternative constants.
Original IDs/order, original choice flags versus mode and every price/catch input
are bound directly to the hashed source tokens in both native/R exports.

- **Sample/identity:** exact numeric/text/boolean groups; unique group/alternative
  pairs and the same2..16 exact integer alternative codes are validated before
  missingness. One missing/nonfinite outcome/predictor excludes the whole set;
  finite exact0/1/chosen-count errors are enforced on retained complete sets.
  DECIMAL keys/codes/outcomes are not rounded through f64 for identity/domain.
- **Inference:** independent unit is the decision, not the alternative row.
  Unweighted conditional MLE has full joint inverse conditional Fisher covariance,
  correction1, asymptotic normal95, equal-probability null, LR chi-square(K) and
  McFadden R². Explicit rank/separation/nonconvergence failure, no ridge or
  pseudoinverse. Bounds and covariance/sample metadata are in `help clogit`.
- **Usable interpretation/report:** SQL long preparation and `isid` guard, fit,
  baseline and eight current-set predictions, restored baseline inputs, grouped
  mean substitution contrasts, terminal/CLI-v1 and CSV. Current outcomes are
  unused; new group IDs are allowed but the complete retained alternative universe
  is required. Missing current predictors NULL the entire set. Failed fits clear
  inference without data mutation; failed predictions preserve data/cache.
- **Independent evidence:** R4.6.1/survival3.8.6 exact single-event conditional
  likelihood supplies coefficients/full2×2 covariance; Cox expected counts also
  match normalized original-set probabilities. All61 quantities agree at1e-9
  absolute and42,552 probabilities at1e-10. Observed maxima: coefficient1.45e-15,
  covariance3.30e-17, likelihood6.83e-13, price/catch contrasts1.09e-15/7.47e-15,
  probabilities1.45e-15. Altering both exported preparations identically is
  rejected by original-token binding, not accepted as mutual agreement.
- **Independent small cases/failures:** closed-form2/3/4-alternative MLE and full
  covariance, binary reduction, within-set common shifts, row/group permutations,
  independent-decision replication covariance halving, exact DECIMAL identity,
  whole-set exclusion precedence, stale-inference clearing and atomic prediction/
  export regression tests. These are bounded evidence, not platform certification.

Section2B price+1 uses100*deltaP; catch+.001 uses1000*deltaP. These are model point
contrasts, **not** analytic AMEs, causal effects or delta-method intervals. Both
inputs are restored before export, unlike the author's final extra catch+.001
restore after computing its last prediction. Native float64 contrasts are not
claimed identical to Stata8 float-storage digits; those differences remain
unisolated. Historical coefficient/SE comparison separately agrees within1e-6
and likelihood within four-decimal rounding, not every printed digit.

Ragged availability, multiple-success matched sets, weights/robust/cluster/survey,
automatic factors, IIA tests, effects uncertainty, named reports/general restrictions
and remaining nested/other illustrated analyses remain required work; the combined
fixed-coefficient analysis is separately validated below.
No whole-table/program or completed Chapter15 coverage is claimed.

### MMA-COMBINED bounded fixed-utility contract and evidence

The [native script/audit](../../examples/mma15p1mnl/README.md#combined-fixed-coefficient-utility-table152-final-column)
targets `mma15p1mnl.do` section4's eight-column Table15.2 p.493 combined fit,
not the preceding five-column exploratory specification. All1,182 original people/
four alternatives remain: `p q dummy2 dummy3 dummy4 d2y d3y d4y`, income/1000,
beach alternative-constant/income normalization and no common intercept. This is
fixed-coefficient utility, **not random-coefficients mixed logit**. Existing
numeric-design/kernel capabilities execute it; no new estimator/parser/factor
framework is introduced.

- **Inference:** full joint8×8 inverse conditional Fisher sum over independent
  decisions, correction1, asymptotic normal95; uniform zero-all-eight null,
  LR chi-square(8), McFadden R². Same balanced-set, whole-set exclusion,
  rank/convergence and current-prediction contracts as the conditional core.
- **Encoding:** author section3's six numeric dummy/income columns reproduce
  ordinary MNL's full coefficients/covariance/likelihood/probabilities after
  parameter permutation, but **nulls/LR DF differ**: conditional zero-all-six
  uniform null/df6 versus ordinary MNL estimated shares/three zero income slopes.
  Their omnibus tests/pseudo R² must not be equated as model comparisons.
- **Native interpretation/report:** source ID and mode-versus-four-flag guards
  before any fit, explicit long preparation/pair guard, three model fits,
  probabilities, design-aware +$1,000 income and charter-price+1/catch+.001 point
  scenarios, restored scalar/interaction/price/catch inputs, four-row terminal
  summaries and CSV/CLI-v1. All three income interactions change together;
  changing only raw income is not prediction design maintenance. These extra
  scenarios are not published effects, analytic AMEs, causal estimates or CIs.
- **Independent evidence:** R4.6.1/survival3.8.6 validates218 quantities, all100
  covariance entries across six/eight-column fits and28,368 probabilities, with
  exact source IDs/order/flags/price/catch/income/interaction/baseline binding.
  Absolute scalar tolerance1e-9, probability1e-10, positive p-value relative1e-9.
  Combined observed maxima: coefficient3.85e-14, covariance3.34e-15,
  likelihood9.10e-13; income/price/catch contrasts5.87e-16/1.12e-15/1.03e-14.
  Probability max1.65e-14. Native/R tolerances were not widened: R write.csv's
  computed-income rounding was fixed by explicit17-digit export strings.
  Historical combined coefficients/SEs separately agree within1e-6 and likelihood
  four-decimal rounding; storage/optimizer contributions remain unisolated.
- **Inspectability/failure:**138-decision, three-income-cell, four-alternative
  full-rank fixture has exact log-odds coefficients and scenario probabilities;
  CLI tests check full MNL covariance permutation, distinct nulls, all joint
  interaction changes/restores and native mean contrasts. Duplicate source IDs
  and inconsistent flags fail before any fit/export, original input bytes remain
  unchanged, and export collisions preserve previously written files.

All three principal Table15.2 coefficient columns now have explicit source-audited
workflows. This is not one formatted whole-table report, every exploratory fit
in the author program, historical float-digit parity or completed Chapter15.
Nested/other choice illustrations, IIA diagnostics, broader availability, effects
uncertainty, weights/robust/cluster/survey and automatic design/reporting remain.

### MMA-NESTED bounded non-normalized contract and evidence

[PR324](https://github.com/SaehwanPark/tabdat-explore-rs/pull/324) implements
§15.6.3 p.511, author `mma15p2gev.do` sections2A–C, not a new coefficient column
of Table15.2. Same1,182 people/4,728 long rows; shore={beach1,pier2},
boat={private3,charter4}; lower utility `p q`, upper shore constant/income/1000.
The authors explicitly identify Stata8's **earlier non-normalized** variant,
not the preferred normalization. Native support requires explicit
`nonnormalized` opt-in; a preferred RUM-normalized estimator remains separate
required work, not silently substituted or excluded.

For nest m, conditional probabilities normalize `exp(xβ)` within the nest;
upper utility is `zγ + λ_m*logsumexp(xβ)`, with positive free λ. Unlike preferred
common-utility normalization, case-common shifts of lower utility do NOT cancel
when λ differs. Full fitting must retain those levels rather than reusing the
conditional core's within-case baseline differences. Upper predictors must be
constant within each retained case/nest. First native scope is exactly two nests,
at least two alternatives each, balanced exact structural keys/maps, whole-set
finite exclusion and atomic current-set prediction. No automatic intercept.

The [development reference](../../examples/mma15p1mnl/reference_nested.R) executes
R4.6.1/mlogit2.0.0's `unscaled=TRUE` model and a separately automatically
first/second-differentiated scalar likelihood. It verifies package likelihood,
fresh score and probabilities at identical parameters, then tightly refines the
stationary point using exact observed information. Default package OPG covariance
and loose function-value convergence are not substituted. The R utility-level
shore coefficients require multiplying by λ_shore to obtain the top-level
parameterization. The [native script](../../examples/mma15p1mnl/nested.td) and
[source-bound checker](../../examples/mma15p1mnl/check_nested.py) now compare all149
quantities,52 covariance entries across restricted/full fits and23,640
probabilities, including exact original design/restores and native displayed means.

Reference log likelihood is−1180.0125384828, positive IVs32.1725778578 /
15.0747565917 (observed SEs29.9971412397 /14.0098795481). The λ=1 conditional
restriction has LL−1252.71104268689, LR145.397008408188, chi-square2 p≈2.676e-32.
Fitted IVs violate the RUM bounds; higher likelihood is not a consistency claim.
Planned covariance is full joint observed-Hessian inverse/correction1; normal
slope tests use zero, IV diagnostics explicitly use **one**, not the excluded
zero-scale boundary. No regular uniform-null omnibus χ² with unidentified IVs.

Historical Stata8 reports LL−1180.0156 and IVs29.98591/14.06438, not the tightly
stationary result. Explicit float32 input/generated-income replay still yields
stationary IVs≈32.172579/15.074757 and LL−1180.0125377643: float storage alone
is not the whole explanation. No tolerance or historical-digit claim is widened
to erase this difference. LIMDEP reports LL−1180.013 with yet another top/utility
coefficient parameterization. Preferred normalized fitting on these restricted
covariates instead showed a divergent shore scale/singular information; that
failed probe is not a validated alternative fit.

The pure Rust kernel now fits positive scales from two deterministic starts,
retains raw lower-utility levels, uses full observed information (expected Fisher
only as an optimization fallback), and requires local scale identification at
the λ=1 restriction before assigning a regular LR reference. A704-decision
exact-count fixture identifies all six parameters in closed form;116 independent
R scalar/covariance/restriction checks plus analytic-derivative finite differences,
permutations, replication, prediction/constancy/map and failure checks pass.
A direct kernel probe on the R-prepared original design also matches149 source
quantities/23,640 probabilities: coefficient max1.54e-10, covariance3.99e-8
(on large weak-IV variances), source LL5.01e-12 and probability1.20e-11.
New nested scalar checks use1e-9 absolute +1e-8 relative and positive-tail
relative1e-8; covariance scaling matters for these weakly identified natural
IVs. Existing standalone MNL/CL gates are unchanged. The new nested-workflow
positive-tail1e-8 contract includes its four-slope restriction: native~1e-11
slope stopping differences amplify to1.82e-9 relative at p=4.42e-56, with tails
matching independent `math.erfc` at native z. The independent derivative check
confirms the restricted package point is already stationary (zero refinement
shift), not an optimizer-cache substitution.

Native wiring includes explicit `nonnormalized` syntax, raw structural-key/map
validation before whole-set exclusion, owned retained state, atomic current-set
prediction, terminal/CLI-v1 inference metadata and help/discovery.89 scoped tests
cover parser/runtime, conditional-reader regressions, reporting, synthetic native
script/text/JSON, the actual book script, closed-form704-person joint scenarios/
restores/displays, source guards before fitting, input/export preservation and
boundary/discovery contracts. Original-data native comparison maxima are
coefficient1.53e-10, covariance3.98e-8, LL3.41e-12, probability1.20e-11;
income/price/catch contrasts1.40e-15/1.12e-13/1.08e-13. A fresh original native
text run produces byte-identical CSVs without changing input. Shared native/R
design corruption and a fake zero LR p-value are rejected. This bounded evidence
does not complete preferred normalization, historical Stata8 parity, effects
uncertainty or the remaining Chapter15 inventory.

### MMA-NESTED-NORMALIZED-COMMON bounded contract and evidence

This slice implements a **separately labelled normalized common-scale
extension** of §15.6.3's source design, not reproduction of the author's earlier
Stata8 fit or implementation of unrestricted preferred nest-specific scales.
Same original1,182 people/4,728 rows, shore={beach1,pier2}, boat={private3,charter4},
lower `p q`, upper `dshore dshorey` with income/1000. A single positive λ is shared
by both nests: conditional probabilities normalize `exp(xβ/λ)` and upper utility
is `zγ + λ*logsumexp(xβ/λ)`. Unlike the earlier variant, case-common lower shifts
cancel. Explicit syntax is `nlogit ... normalized common`; existing
`nonnormalized` behavior must remain unchanged.

Inference: inverse full joint observed information/correction1, G decision
units and normal95; shared-scale normal diagnostic tests1, not zero. Re-estimated
λ=1 conditional restriction has LR chi-square1 only after local identification.
No unidentified uniform-null omnibus or composite RUM-consistency test. Positive
scale estimation is unconstrained above1; descriptive `(0,1]` violations must be
reported, not called RUM-consistent or silently clipped. Existing structural/
whole-set missingness/current-prediction atomicity contracts remain required.

The [development reference](../../examples/mma15p1mnl/reference_normalized_common.R)
checks R4.6.1/mlogit2.0.0 `unscaled=FALSE, un.nest.el=TRUE` fresh likelihood, score
and probabilities against independent base-R `deriv3`, Newton stationarity and
observed information; survival3.8.6 supplies the λ=1 restriction. It emits131
quantities including41 covariance entries across four/five-parameter fits and
23,640 probabilities. Stationary LL−1211.53537250817, common λ25.009343096962,
observed SE50.033049627836, independence LR82.351340357452 and chi-square1
p1.1392419190882e-19. This real-data fit **violates RUM bounds**, even under the
preferred utility normalization; the model must disclose that distinction.
Package default OPG/scale-zero tests are not adopted. New-family scalar tolerance is1e-9
absolute +1e-8 relative, positive tails relative1e-8 and probabilities absolute
1e-10; standalone MNL/CL/non-normalized gates remain unchanged.

Acceptance contract:

1. A900-decision exact-count fixture identifies β=(ln2,ln3), upper γ=(ln2,ln3)
   and shared λ=.5, including full observed covariance/inference against R.
   Three covariate cells use shore weights9/16 and boat weights multiplied by
   k² for k=2,3,1; shore nest odds multiply2*3^income, income0/1. The true shared
   scale is inside RUM bounds; it is not validated only on the violating source.
2. λ=1 reduces to the conditional model; common lower shifts cancel; permutations,
   independent-decision replication, retained provenance and rank/iteration/
   restriction-identification failures preserve statistical contracts.
3. The actual source script guards IDs/flags before fit, audits all131 quantities/
   41 covariances/23,640 keyed probabilities, jointly updates income, restores
   design, displays means and preserves inputs/existing exports.
4. Native exact structural keys/maps before exclusion, whole-set NULL prediction,
   upper constancy, failed-fit clearing and prediction rollback are exercised;
   old non-normalized/parser/CLI contracts stay covered. No weights, robust/
   cluster/survey, richer trees, general tests/reports or effects uncertainty.

The [exact-count R fixture](../../crates/tabdat-stats/tests/fixtures/normalized_common_choice_reference.R)
now independently checks all six empirical cell distributions, zero score and
positive observed information at the five known natural parameters, plus99
coefficient/covariance/restriction quantities (41 covariance entries). Chosen
counts are216/384/108/192. Native kernel and actual-script tests independently consume those99 quantities.

The implementation reuses the existing earlier-variant case
likelihood with **equal** logged nest scales and internal lower coefficients
b_tilde=β/λ. Project its two scale score/Hessian coordinates onto one common
coordinate, checking identification of that projected system at λ=1 (not the
unrestricted two-scale system). Return natural β=λ*b_tilde, unchanged upper γ,
and λ=exp(eta). Full covariance must use the complete Jacobian, including lower
coefficient derivatives with respect to eta; multiplying only standard errors
would lose coefficient/scale cross terms. Case-common centering is safe in this
common-scale normalized fit, not in the existing free-scale earlier variant.
This equivalence avoids a second independently maintained derivative engine.

The native pure kernel and complete-set prediction are now implemented in
`tabdat-stats`, retaining the old non-normalized input name as a compatibility
alias for the shared design. Eight new numerical/shape/provenance/failure tests,
five existing earlier-variant tests and the existing away-from-MLE analytic
derivative test pass. A source **pure-kernel probe, not the CLI**, passes all131
quantities/41 covariance entries/23,640 keyed probabilities. Maximum original
coefficient error1.73e-11, covariance error4.43e-7 (weak natural-scale variance,
within the predeclared relative gate), LL5.01e-12 and probability1.20e-11;
income/price/catch contrast errors4.17e-17/7.44e-15/6.65e-15. Both starts converge,
source score bound1.17e-15 and step7.63e-14. Native λ25.0093430969551 and observed
SE50.0330496234127 diagnose the same RUM-bound violation.

The explicit native mode now owns raw retained state, predicts current complete
sets atomically, and reports a separate CLI-v1 `NormalizedCommonNestedLogitResult`/
terminal result with the RUM-bound warning. Three parser and nine runtime cases
cover both modes and failure-state boundaries. One initial test incorrectly
assumed named-model storage supported this family; it was corrected to verify the
existing explicit rejection/preservation boundary, not to expand that subsystem.

The actual source CLI run passes131 quantities/41 covariance entries/23,640
probabilities with source-input preservation. Two actual-script tests fit900
known decisions, consume99 R quantities and check current scenarios, restores,
displays, source guards and input/export collisions. Fresh explicit `--text`
execution yields byte-identical exports and the expected warning/LR df1. Shared
native/R design corruption and mutually zeroed native/R LR p-values are rejected
by the source-hash and independent-tail checks. Initial text validation omitted
`--text` and incorrectly expected batch-default output; explicit text and the
public terminal formatter both pass without changing quiet batch behavior.

Current workspace/all-target evidence verifies1,469 passed and2 pre-existing
ignored across253 executables, with every current executable SHA256 checked.
The original broad run failed one obsolete schema expectation that required the
old mode individually; the corrected two-test schema/legacy JSON/text/help target
passes, combined explicitly with252 successful broad artifacts, not misreported
as a clean original run. Full-workspace all-target Clippy and current root Clippy
pass with warnings denied; format/diff checks pass and workspace doctests succeed
with zero cases. No CI checks are available. These bounded local checks do not
complete unrestricted preferred scales, Chapter15 or platform certification.
Independent nest scales and remaining work stay required.

### MMA-NESTED-SOURCE-AUDIT bounded reference audit and evidence

Before offering unrestricted preferred nest-specific inference on the fishing
source, audit the exact same1,182 decisions/4,728 rows and lower `p q`, upper
`dshore dshorey` design. This does not add native estimator syntax or replace the
published earlier variant/common-scale extension. Existing numerical gates stay
unchanged; no unresolved entry is counted complete or excluded.

Acceptance/failure cases:

1. Bind the original source/data hashes and both author Stata8/LIMDEP programs;
   distinguish raw lower features from upper utilities and their normalization.
2. Check a stable preferred likelihood/score against fresh R/mlogit evaluations
   at finite positive nest scales, with independent symbolic derivatives where
   applicable. Audit the reciprocal-scale limit rather than treating a nearly
   singular, large-scale optimizer printout as a finite regular MLE.
3. Profile the source shore scale with multiple deterministic starts and validate
   interior stationarity/full observed information before any covariance claim.
   Boundary/failed/unresolved profiles do not supply ordinary unrestricted Wald
   or LR inference, global-maximum proofs, or an approved exclusion.
4. Audit the earlier published Stata8 point and author LIMDEP parameterization
   against explicit likelihood/score/curvature, including source preparation
   precision. Keep published output, a stationary reference and an inferred
   historical-software mechanism distinct; do not invent a cause or widen gates.

The [source/boundary audit](../../examples/mma15p1mnl/README.md#nested-choice-source-and-boundary-audit)
now emits/checks1068 diagnostics. Fresh mlogit2.0.0 controls at four finite positive
scale points agree with the centered reciprocal-scale likelihood/full score and
probabilities. Eleven deterministic starts reach the diagnostic closure
`ks=0,kb≈.0538106827551`, LL−1211.43655668914; the derivative into positive shore
reciprocal scale is−2.09714424048. Independent base-R `deriv3` validates value/full
score and positive five-free-direction observed curvature in the declared
numerical coordinates. Twelve profiles/three starts each retain possible active
boat bounds. Raw L-BFGS code52 remains visible; separate observed-Newton polishing,
positive free curvature, score/step and active-bound KKT checks certify the
reported candidates. Three Hessian step sizes provide derivative sensitivity.
This is a local boundary/profile diagnosis, **not a global proof or finite
positive-scale unrestricted MLE**. No ordinary boundary covariance/LR is emitted;
regular native unrestricted preferred fitting remains required.

The original Stata printed point reproduces its printed likelihood but has mean
scaled score≈1.31e-4 and scaled Newton step≈.658, versus the double stationary
control≈3.50e-14/5.92e-12. The64 rounding vertices under each of float64/explicit
float32 preparation remain nonstationary; this is sensitivity evidence, not a
certificate over every interior rounding-box point. Float32 holds the double
parameters fixed against changed inputs, not a re-fit. Source logs show236
iterations/33 BFGS resets; the exact proprietary stopping mechanism is not claimed.
LIMDEP's raw lower `dshorey=1-(d3y+d4y)` maps through
`gamma0=lambda_s*delta_shore+(lambda_s-lambda_b)*delta_y` and
`gammay=lambda_b*delta_y`; its printed point reproduces the stationary likelihood
and nearby parameters, not preferred-normalized or printed-SE parity.

The checker binds all five original source/data hashes and prepared tokens,
parses original printed points and independently recomputes boundary/historical
likelihoods/scores and128 rounding vertices. Corruption testing exposed a missing
stationary-control acceptance gate: coherent nonstationary control metadata could
pass agreement checks. Both layers now require the double control's score/step
below1e-10 and positive information; the float32 sensitivity control is not
mislabelled stationary. Fresh R/reference/checker and all eight permanent
corruption regressions, including standalone R rejection with no partial output,
pass while preserving nine original inputs. One independent read-only review
confirmed that finding; its targeted follow-up confirms resolution, with execution
performed by the parent rather than the reviewer. Python syntax/diff checks pass.
No Rust/native behavior changed or full-workspace rerun was needed; this is not
CI/platform certification, new native capability or an approved exclusion.

### MMA-TOBIT bounded classic lower-censoring contract and evidence

Use the original author `mma16p1tobit.asc`200-row export, not a replacement RNG
sample: `ycens` on `lnwage` plus an intercept, lower limit0,70 left-censored and
130 uncensored. The March2005 program identifies §16.2.1 pp.530–531/Figure16.1;
its §2A Tobit estimation is explicitly **not a coefficient table in the book**.
Raw ASCII SHA256 is `8ec7ba9cf3da6854217f873441e47fd5492e777904057c8d9ca8f8ee00244ba6`.
Preserve every source cell/order and the supplied generated outcomes; no attempt
to reproduce Stata8's uniform stream or float-generation sequence is required.

Intended first native scope is unweighted IID homoskedastic Gaussian
`tobit y xvars, ll(number) [noconstant]`: joint finite sample; responses≤limit
contribute a left-censor probability, greater responses an ordinary Gaussian
density. Original responses remain unchanged. Use full joint observed covariance
for beta/lnsigma, correction1/asymptotic normal95, positive log-Wald sigma intervals
and a same-sample/bound re-fitted slope-null LR. Old Stata8 t intervals are not
silently labelled normal or used to widen native tolerances. Upper censoring,
robust/cluster/weights, factors, general tests and named-model storage are excluded
from this first executable slice, not completed family outcomes.

Acceptance/failure cases before implementation:
1. Original200-row source fit/inference/full covariance agrees with independent
   R4.6.1/survival3.8.6 Gaussian `survreg`, checked/refined with independent base-R
   likelihood/derivatives; report retained censor counts/scale/likelihood/LR.
2. Current-row latent `xb`, probability of being uncensored (`pr`) and observed
   censored mean (`mean`) predictions; retained-X continuous observed-mean AMEs
   with full-joint delta uncertainty, not latent slopes/truncated means/causal effects.
3. All-uncensored Gaussian/OLS equivalence, shifted nonzero censor limits and
   no-intercept/multiple-predictor references; covariance/response-scale invariants.
4. Joint missingness/current-row NULL predictions, input preservation and atomic
   prediction/model failure; reject rank failure, all-censored/degenerate-scale
   fits, unsupported covariance/upper-censor options and occupied output paths.

The pure Rust kernel now optimizes the concave beta/sigma,positive inverse-scale
parameterization with strict score/step and positive joint information; full
Jacobian inference returns beta/lnsigma covariance. Five independent
R/survival/base-R fixtures contain1396 entries (540 input cells,540 predictions,
316 inference/gradient/covariance/metadata values): censoring, nonzero-limit/no
intercept, all-uncensored Gaussian, bound shift and response scaling. One private
derivative test/six kernel tests pass, including replication/order/sample and
explicit rank/scale/iteration/unsupported-inference failures. An initial saturated
mean-probability accumulation rounded above1 and failed an AME test; sum-then-divide
fixes it without widening a gate. Independent read-only review additionally
found that unused censor bounds/extreme below-limit recordings could distort
response scaling and spuriously fail positive information. Both counterexamples
were reproduced as failing regressions; scaling now uses only likelihood-effective
values, preserving raw responses and the unchanged information/convergence gates.
All six current kernel tests/current stats all-target Clippy pass. Probabilities/
observed means legitimately change when a prediction bound changes, even if an
entirely uncensored estimation likelihood is unchanged.

The original source numerical probe additionally checks all48 R quantities,
nine full joint covariance entries/600 probabilities/latent/observed-mean
predictions and joint-delta AMEs under declared1e-9 absolute+1e-8 relative scalars,
1e-8 positive-tail relative and1e-10 probability gates. Source conversion and
malformed/truncated/output-collision guards pass. Parser acceptance is existing
behavior. Owned Gaussian session state, command execution, retained continuous
observed-mean effects, current xb/pr/mean predictions and terminal/CLI-v1/help
are now implemented. Two focused parser/four runtime cases pass, including
joint sampling/full covariance, original-X effects after edits, NULL predictions
from missing/nonfinite X, unused outcomes, labels/order and failure atomicity.
Current all-target execution/reporting/language/runtime Clippy and the explicit
prediction discovery contract pass. The native original-source script now passes
all48 R reference gates (46 comparisons plus two independently gated R solver
certificates), nine full covariance entries/600 predictions and1400 original
cells/ordinals. It independently binds the natural-coordinate source score,
observed-information inverse, retained gradient/full delta variance and tails;
Rust/R convergence coordinates are not equated. Nine permanent corruption
regressions reject shared zero tails, wrong covariance/gradients and false
stationary controls alongside source/order/cell/inventory failures, preserving
all six input hashes. Two actual-script cases use a separate closed-form
Gaussian control, full joint covariance/delta and byte-identical JSON/text
exports, plus existing-target/missing-input failures. Added runtime controls
validate the independent-R nonzero-limit/no-constant/below-limit sample and
numerical prediction rollback/failed numerical refit clearing. Independent
read-only source/evidence review found no P0/P1 and one P2 CLI predictor-requirement
mismatch; corrected docs/schema and explicit parser/discovery regressions resolve
it without parser expansion. All256 current workspace test artifacts are
hash-reconciled:1486 passed/two pre-existing ignored, no omitted artifacts.
Original nextest/Cargo timeouts remain incomplete; completed passing artifacts
were retained and only never-started artifacts resumed under bounded runners.
Workspace Clippy/formatting and the applicable final schema/test checks pass;
local macOS evidence is not Actions/other-platform certification or a
whole-illustration claim. Theoretical Figure16.1 curves, truncated regression,
Heckman/two-part/CLAD and the rest of Chapter16 remain required.

### MMA-SELECT bounded Heckman two-step contract (WIP)

Next exact analysis: March2005 `mma16p1tobit.do` §2A's built-in Heckman
comparison, using the same original200-row export: selection `dy` on intercept
and `lnwage`, outcome `ytrunc` on intercept and `lnwage` for130 selected rows.
This is an author-program extension associated with §16.2.1, not a published
coefficient table or completion of Chapter16. Source assets/fingerprints and
preparation are shared with MMA-TOBIT; no replacement random sample.

The source expressly says the manual second-stage OLS SEs are incorrect. Use
first-stage probit observed ML covariance and the conventional model-based
Heckman/Greene generated-regressor correction for the augmented outcome
coefficients, not naive/HC second-stage OLS. Scale is reconstructed from
selected RSS/n_selected plus alpha^2*mean(delta), alpha=rho*sigma. Report
normal95 for selection/outcome/Mills coefficients; sigma/rho are points only.
Selection and augmented-outcome covariance are separate declared blocks.
Do not fabricate their uncertainty, a joint ML likelihood/LR, cross-equation
restrictions or causal identification. Original regressors have **no exclusion
restriction**; identification is functional-form dependent and weakly separated
from the inverse Mills regressor (author reports correlation−.9745).

Planned explicit command: `heckman y xvars, selectdep(d) select(zvars) twostep`.
Keep existing parser-only bare form distinct; do not silently present two-step
as default ML. No weights/robust/cluster/factors, ML, switching, general tests,
named storage or prediction uncertainty in this slice. Both equations share the
intercept option. Require regular finite probit/rank/covariance and interior
|rho|<1; reject rather than clip invalid correlations/adopt Stata's ad-hoc fixes.

Sample: first require finite exact0/1 selection and selection regressors;
selected units additionally need finite outcome/outcome regressors. Exclude a
bad selected unit from both stages, matching the reference's common eligibility;
missing outcomes/X on unselected units do not exclude them. Preserve original
positions for eligibility and its selected subset, and do not alter raw data.
Current point predictions distinguish latent xb, selection pr, Mills ratio,
conditional-selected mean and zero-filled observed-contribution mean; no Tobit
mean/count aliasing. Only the relevant current X/Z are needed, not current y/d.

Acceptance/failure cases before implementation:
1. Original200/130 source coefficients, both declared covariance blocks, sigma/
   rho and predictions against R glm/lm plus unmodified sampleSelection1.2-16
   `heckitVcov`, independently reconstructed/certified with base-R derivatives.
2. Independent synthetic controls with an exclusion variable and no-constant
   designs; exact eligible/selected membership including asymmetric missingness.
3. Owned stage/sample state, current predictions and source exports; explicit
   no-exclusion warning, first-stage-only likelihood label and sigma/rho points.
4. Reject nonbinary/separated/rank-deficient/all-selected/all-unselected,
   degenerate/out-of-range-rho and unsupported inference/method requests; failed
   new fits clear stale estimation, prediction/file failures preserve state/data.

Reference milestone: R4.6.1 glm/lm with base-R symbolic observed probit
score/Hessian refinement and the **unmodified** sampleSelection1.2-16
`heckitVcov` function (pinned external development source, not a full installed
package claim) emits58 exact-inventory quantities,13 covariance entries and
1200 finite point predictions. Independent algebra reconstructs the correction;
all1400 original cells/ordinals and five malformed/collision/reference-fingerprint
failure guards pass. Sigma740.143348254785, rho−.674188264626690;
outcome intercept745.306997010131, slope176.646780648963 and Mills
coefficient−498.995959534882, corrected SEs1617.58343858516,
425.002572278714 and760.500897353647. The first-stage LL is−105.296713010228;
there is no fitted joint two-step likelihood. These are declared float64 R
reference values, not exact historical Stata stopping/storage/digit parity.
Initial reference stdout/schema defects (top-level chol printing, named scalar
suffixes) were caught by CSV/inventory validation and corrected, with diagnostics
preserved. No numerical tolerance was widened.

Kernel milestone: `crates/tabdat-stats/src/heckman.rs` now implements explicit
two-problem sample identity, existing observed-probit/QR reuse, full augmented
outcome generated-regressor covariance, sigma/rho points and five projections.
Two synthetic independent R cases cover98 quantities/26 covariance entries and
3600 predictions; seven focused tests include affine units, permutation and
IID replication, exact identity and rank/separation/scale/rho/unsupported failures.
Original-source native kernel probe passes55 numerical quantities and3 separately
bounded coordinate-dependent certificates,13 covariance entries,1000 core-helper
projections plus200 fitted-selection indices and1400 exact source cells/ordinals.
Read-only independent frozen-source review found no issues; parent current-code
execution passed192 stats tests/1 pre-existing ignored across45 current
hash-checked artifacts. The all-stats nextest discovery timeout remains incomplete;
exact-inventory bounded direct execution supplied the regression evidence, with
all-target stats Clippy/workspace format/diff/zero-doctest checks. See the
[source/kernel notes](../../examples/mma16p1tobit/heckman.md) for numerical limits,
provenance and remaining validation. No native command, owned runtime state,
DuckDB sample/prediction path or actual-script workflow is yet delivered.

This remains native-integration WIP, not a delivered selection workflow.

### MMA-PANEL bounded POLS/first-difference contract and evidence

The [native workflows and independent audit](../../examples/mma21p1panel/README.md)
resolve §21.3 pp.709–714, Table21.2 p.710 against the March2005 author program/log.
Their header says Table21.1, but the POLS/FD section markers and final table block
identify Table21.2. This is chapter21, not chapter24's clustered-sample examples.

- **Data/design:** externally downloaded, checksum-pinned `MOM.dat`, not the
  higher-precision chapter22 file: 5,320 rows, 532 men, all years1979–1988, numeric
  IDs1–532. Strict token-preserving AWK ingestion adds original source-row IDs;
  atomic staged publication refuses existing targets. Native `isid` and SQL
  guards reject duplicate/missing/noninteger keys. POLS is `lnhr~1+lnwg`;
  partitioned, year-ordered SQL LAG admits only consecutive annual pairs before
  complete-finite fitting. FD is `dlnhr~1+dlnwg`, N4,788/G532, retaining the
  author's estimated differenced intercept. No optional covariates/year dummies.
- **Usable analysis:** both scripts fit classical/HC1/person-cluster covariances,
  tests and slope95 intervals, immutable named comparisons, fitted/residual
  summaries and exports through existing text/CLI-v1/script surfaces. Cluster
  correction is G/(G−1)*(N−1)/(N−K), K2; inference uses G−1=531 t/F DF while fit
  residual DF remains N−K. Classical/HC1 are independent-row comparison columns,
  not panel-robust inference; Gaussian working likelihood is not a panel LR test.
- **Independent evidence:** R4.6.1/sandwich3.1.1 validates180 scalar quantities,
  full covariance/fit/test/interval/DF and all10,108 exported rows/111,188 cells.
  Numeric tolerance1e-9 absolute, nonzero tails1e-8 relative; exact source/person/
  year/pair identities. Separate R float32 ingestion/generated-difference audit
  reproduces24 author coefficient/SE facts at original printed precision; native
  float64 is not rounded to mimic Stata8 storage. The small synthetic CLI test
  protects unsorted/interleaved/gapped/missing sample semantics, text/JSON export
  equivalence, located early errors, unchanged inputs and refused outputs.
- **Still partial:** POLS/FD compose SQL/OLS; FD is not within-FE at T>2.
  The separate native FE portion below does not install persistent panel/xtdata
  state or complete the table/family. Raw source redistribution/platform
  guarantees are not claimed.

### MMA-PANEL bounded Between portion

The [Between workflow](../../examples/mma21p1panel/README.md#between-mean-response-inference-and-reproduction)
executes Table21.2's classical Between coefficient/inference portion using
native SQL plus OLS, with a separately validated independent-person HC1
comparison. It audits raw keys, jointly selects finite lnhr/lnwg pairs, computes
one equally weighted mean row per person, and orders IDs. Original MOM.dat
contributes ten rows per each of 532 people; inference DF is G-2=530. Modified
input retains singleton people and drops people with no joint cases; no T_i
weighting or separate-variable averaging. Named reports, tests/intervals,
mean-response predictions/residuals and count/year/mean exports complete the
bounded workflow. These are 532 mean observations, not 5,320 panel fit rows.

R4.6.1/lm and sandwich3.1.1 independently validate **60 scalars** and every one
of **4,256 exported cells/532 mean rows**, using absolute1e-9/nonzero-tail
relative1e-8 tolerances and exact identities/counts/year bounds. Separate
float32 ingestion reproduces seven author printed classical facts at half-unit
printed precision. The source hash is pinned. Synthetic CLI cases protect
joint membership, unequal counts/equal person weights, singleton/all-excluded
units, inference DF, text/JSON identity, key/sample/rank failure and export
preservation. HC1 is not the author's bootstrap; R²/working likelihood are for
means, not within/overall panel statistics or cross-design LR inference.
No general `xtreg ..., be`, cluster/FE/RE bootstrap, Figure21.2 or whole-table completion
is claimed; correlation of wages with person effects remains an identification
concern. Source and reproduction instructions remain beside the example.

### MMA-PANEL bounded native RE-GLS portion

The [RE-GLS script/reference](../../examples/mma21p1panel/README.md#re-gls-balanced-swamyarora-workflow)
executes the classical Table21.2 RE-GLS coefficient/inference portion on all
5,320 MOM.dat rows/532 people/T10. Native `xtreg ..., re id(id) time(year)`
uses balanced Swamy–Arora components (within N-G-p; between G-p-1 with sigma_e²/T
subtraction, negative unit variance truncated/reported), transforms the
intercept/slopes/outcome, and uses transformed RSS/(N-p-1) covariance scale.
Normal coefficient intervals and joint chi-square slope Wald are explicit;
DF5318 is a residual-scale DF, not a t reference. Population xb/residuals use
current rows and retained coefficients; no recovered conditional effects.

R4.6.1/plm2.6.7 and explicit matrix algebra validate **30 quantities**, every
**58,520 exported cells/5,320 source/design/prediction rows**, and eight
separate author float32 printed facts. Synthetic fixtures add **352 R/plm/matrix
comparisons**, including two slopes/full covariance, outcome shifts,
interleaving and the negative-component boundary. Tolerances remain absolute
1e-9/nonzero-tail relative1e-8 with exact source identity. Runtime/CLI tests
protect raw-key auditing, joint-case/balanced retained membership, fully omitted
units, current predictions, state on error and text/JSON/export preservation.

For the published balanced fit, retained counts must be equal T>=2 and both
within/between designs must be full rank. Time-invariant predictors, unit effects,
remaining illustrated ML inference, Hausman, bootstrap and remaining table/figure outputs remain required.
Model-based inference assumes exogenous unit effects and the specified
error-components covariance with independent units. No panel likelihood,
within/between/overall R², general restrictions or named RE reports are supplied.

The separately validated [unit-cluster RE extension](../../examples/mma21p1panel/README.md#unit-cluster-re-extension)
uses grouped quasi-demeaned scores with multiplier G/(G-1)*(N-1)/(N-K),
K=p+1, matching R/plm Arellano `sss`; `robust` aliases declared-unit clustering.
Point fit/sample/components/predictions stay unchanged. Normal95 and
chi-square(p) inference remain explicit; singular slope covariance suppresses
the Wald test with a reason, preserving marginal SEs and the requested p.
R4.6.1/plm2.6.7 plus independent group-score algebra validates **31 quantities per
alias/62 across two fits** and **69,160 exported cells/5,320 source/design/prediction
rows**; **438 synthetic R comparisons** cover full covariance, shifts, zero
component and rank failures. Text/JSON exports are identical, with unchanged
1e-9 absolute/nonzero-tail1e-8 relative tolerances and exact source identity.
Deterministic group summation protects alias equality in the shared linear
cluster path. Independent people/exogenous unit effects are assumptions, not
conclusions; no bootstrap, few-cluster, household/survey/multiway or new printed
Table21.2 coverage is claimed.

### MMA-PANEL unbalanced RE-GLS sensitivity extension

The [unbalanced GLS workflow](../../examples/mma21p1panel/README.md#unbalanced-re-gls-sensitivity-workflow)
fits the same lnhr~1+lnwg specification on a deterministic ragged MOM cohort
(4,738 rows/532 people, retained counts5–10). This changed sample is a method
extension, not another author Table21.2 result. Balanced entry points and result
shapes stay intact; the distinct `UnbalancedRandomEffectsGlsResult` reports a
T_i-weighted Swamy–Arora between fit, trace-corrected unit variance and unit-
`cluster(id)`/`robust` uses grouped quasi-demeaned scores with
G/(G-1)*(N-1)/(N-K). Coefficient inference is normal and the slope Wald is
chi-square; no few-cluster t/F guarantee or variance-component uncertainty is
claimed.

R4.6.1/plm2.6.7 and explicit matrix algebra validate **9,515 reference scalars**,
retained source identity and model inputs plus **71,070 exported cells** across
4,738 rows. The audit includes model-based/cluster covariance and inference,
per-unit transforms, predictions/residuals, and exact `cluster(id)`/`robust`
alias equality; text and JSON data exports are byte-identical. Synthetic
unequal/interleaved/boundary references add321 comparisons. Singleton units are
permitted only when rank/DF checks pass; no RE unit effects, time-invariant
predictors, general restrictions are added by that GLS slice; interior ML unit-score evidence is separate below.

### MMA-PANEL interior Gaussian RE-ML portion

The [interior ML workflow](../../examples/mma21p1panel/README.md#interior-re-ml-coefficientlikelihood-workflow)
now executes `xtreg lnhr lnwg, mle id(id) time(year)` on the same5,320 original
MOM rows/532 people/T10. Gaussian marginal likelihood, full joint observed
information (beta and two log variance parameters), normal coefficient intervals,
slope Wald and re-estimated intercept-only null/LR, population predictions and
text/JSON/export paths are executable. R4.6.1/nlme3.1.169 and independent
information algebra validate46 full-source quantities and58,520 cells;447
synthetic comparisons cover two slopes/full covariance, shifts/interleaving and
a boundary null (LR reference withheld). Absolute1e-9/nonzero-tail relative1e-8
and exact source/sample identity remain unchanged. Base-R `optimHess` checks
analytic information. Runtime tests protect retained sample/state and failures.

Explicit limits: balanced, full within rank;
eta[0,24]/0.5-grid with polished detected maxima is not a general global-optimum
guarantee. Zero-unit fits use the constrained extension below. No boundary ML sandwich inference, non-unit clustering, unit effects,
Hausman, general restrictions or bootstrap. The exact stationary observed SEs
have a small unresolved discrepancy with old printed Stata8 uncertainty;
**not a validated published uncertainty column or whole-table replication**.
Assumptions include Gaussian error components, exogenous unit effects and
independent people. Positive-interior variance-component uncertainty now uses
joint observed information and log-Wald intervals. The one-sided zero-unit-variance
LR test uses the 50:50 chi-square(0,1) boundary mixture. Robust ML inference and
remaining panel methods remain required work.

The zero-unit-variance extension now supports selected eta=0 optima with a
nonpositive inward score, Gaussian ML RSS/N scale and beta/log(sigma_e²)
constrained observed information. Normal coefficient intervals/Wald remain
asymptotic; the unit-variance component has no regular normal-Wald SE, and no
finite-sample/post-selection coverage is claimed. CLI JSON exposes the boundary
and matching K+1 covariance parameter order; interior K+2 and strict-interior API
behavior remain intact. **378 synthetic base-R lm ML/optimHess/nlme-null
comparisons** validate points,
complete covariance/inference, likelihoods and fitted/residual values on shifted,
interleaved and both-boundary cases. Runtime/CLI tests protect successful state
replacement, data/predictions/exports, metadata and genuine degenerate failures.
Full or null boundary withholds ordinary LR references. This is additional method
evidence, not a newly validated published uncertainty row; the original-MOM
interior audit and old printed-SE limitation remain unchanged.

### MMA-PANEL bounded native Within portion

The same exact `MOM.dat` sample/design now runs `within.td`: 5,320 rows/532 men,
`lnhr` on `lnwg` with unit intercepts, no time effects/extra regressors. Native
`xtreg ..., fe id(id) time(year)` audits all raw keys before joint-finite model
selection and centers only retained cases; unbalanced/gapped/interleaved panels
and singletons are supported. Classical residual/inference DF is N-G-p=4787;
unit-cluster grouped-score covariance uses G/(G-1)*(N-1)/(N-p), matching plm
Arellano sss, with t/F inference DF G-1=531. `robust` aliases unit clustering,
not OLS HC1. Only slopes have covariance/tests; the common intercept is a
normalization point and unit deviations have observation-weighted mean zero.

Complete text/CLI-v1/script workflow includes slope tests/95 intervals,
within/conditional R² and group/sample diagnostics, current-data xb/xbu/u/residual
prediction from retained centers, descriptive summaries and refused-overwrite
exports. Missing/nonfinite/unestimated conditional IDs yield NULL, not zero
unit effects; exact ID logical type is required. Runtime fit failures clear
stale model only; prediction failures preserve data/model.

R4.6.1/plm2.6.7, explicit LSDV and independent group-score algebra validate
**64 scalars, all5,320 exported rows/90,440 cells and1,064 recovered unit-intercept
comparisons**. Separate float32 ingestion reproduces six printed Within facts
at half a unit of printed precision, never rounding the native float64 oracle.
Author bootstrap is not analytic cluster inference. Synthetic fixtures add855
R/plm/LSDV scalar comparisons, including the unit-effects F diagnostic;
runtime/CLI tests protect sample/key/type/order/state and identical text/JSON
exports. Tolerances remain1e-9 absolute/nonzero-tail
1e-8 relative with exact source identities.

**Still required:** persistent panel state, remaining illustrated ML inference, two-way effects, richer
design/weights, individual unit-intercept uncertainty and cluster-robust unit-effects
testing, Hausman and other panel-specific
diagnostics/comparisons, bootstrap/RNG, lowess/Figures21.1–21.4 and remaining
Table21.2/illustration outputs. No full Within column, table/chapter/panel-family
completion is claimed; conditional R² is not Stata overall/between R² and no
panel likelihood is supplied.

### MMA-BINARY native published-analysis contract and evidence

- **Identity:** Cameron–Trivedi (2005), §14.2 pp. 464–466; authors'
  [`mma14p1binary.do`](https://cameron.econ.ucdavis.edu/mmabook/mma14p1binary.do)
  and [output](https://cameron.econ.ucdavis.edu/mmabook/mma14p1binary.txt).
  The composed `analysis.td` validates published Tables14.1–14.2/Figure14.1,
  all source/cohort/fitted descriptive blocks and usable prepared-data output.
  This is the defined native analytical contract, not every auxiliary Stata
  fit-log field or literal log replication. Converged information, float64
  preparation, CSV/SVG, source order and native layout/display jitter are
  explicit conventions; see the example's author-output audit. Individual OLS
  ANOVA/F/interval displays are not added to the comparison table (`test`/`lincom`
  remain separate native commands). No historical Stata storage/pixel/RNG,
  headerless LIMDEP input or general estimates-restoration parity is claimed.
- **Data/preparation/sample:** external `Nldata.asc`, 1,182 people, 16 numeric
  fields per person; token-based CRLF/unterminated-last-line conversion preserves
  original 1-based source IDs. Select mode 2 (pier) or 4 (charter): N630,
  178 pier/452 charter, original row order, no additional complete-case drops.
  Outcome `1(mode==4)`, x `ln(pcharter/ppier)`, intercept; unweighted float64
  preparation. Executable author code/output resolve a reversed-code comment.
- **Inference/workflow:** model-based covariance without N-K scaling —
  inverse Fisher for logit, inverse per-observation observed information at the
  converged fit for probit; normal95 coefficient inference, LR chi-square(1),
  current-X probabilities/xb, retained-X continuous AME with full
  conditional-design normal delta inference. The AME is an independently
  validated extension, not a published table claim.
- **Evidence:** [runnable script, hashes, reference and tolerances](../../examples/mma14p1binary/README.md);
  independent R4.6.1 glm/Fisher/delta reference; the logit glm stop already sits
  at the score fixed point, while the probit reference polishes glm's solution
  with Newton steps on the analytic observed information until the gradient
  norm is < 1e-13; all 630 source IDs, outcomes, design values, probabilities
  and indices compared for both families. Logit intercept/slope
  2.0531249423051365/-1.8225304317903031; log likelihood -206.82697122830305;
  AME -0.18392143550221252, SE 0.0052985086445694511. Probit fixed-point
  intercept/slope 1.1943604331481330/-1.0555145307066702; log likelihood
  -204.4108645034095; AME -0.18675505365724743, SE 0.0054545071552692895.
  Four small independent synthetic designs additionally cover
  intercept/noconstant and multiple columns.
  [PR #270](https://github.com/SaehwanPark/tabdat-explore-rs/pull/270),
  [PR #271](https://github.com/SaehwanPark/tabdat-explore-rs/pull/271).
- **Failure/state evidence:** non-binary/nonvarying outcomes, rank/iteration limits,
  ordinary and combined outlier/quasi-separation contrasts reject without fit
  publication. Absolute normalized score/full-step convergence prevents a reviewed
  false-convergence trap; saturated but identified outlier models remain valid.
  Native tests verify retained sample identity, current-data prediction, preserved
  data/labels/order on prediction failure, and stale-fit clearing on failed refit.
- **Robust extension:** unclustered `, robust` retains identical MLE/sample/design
  and uses individual-score sandwich covariance with N/(N-1) ML scaling, not
  linear HC1. Converged inverse observed-information bread (Fisher for logit)
  propagates to normal coefficient and conditional-X delta-AME inference.
  [PR #272](https://github.com/SaehwanPark/tabdat-explore-rs/pull/272) adds
  runnable robust scripts, an independent R4.6.1 reference and an executable
  checker for all 44 coefficient/covariance/effect quantities on the exact
  630-person cohort; both existing R prediction references check all source
  identities and predictions. Eight synthetic family/design variants cover
  full covariance/normal inference; runtime/CLI tests cover retained samples,
  edits, state and invalid cluster/weight inputs. Existing absolute/relative 1e-10
  tolerances are unchanged. Probit robust intercept/slope SEs are
  0.0794501824989468/0.0692948032742171 at converged information, not the
  author Stata8 values 0.0794429/0.0692881; see the example for that explicit
  convention difference. Ordinary LR p/reference fields are null for robust
  fits. [PR #274](https://github.com/SaehwanPark/tabdat-explore-rs/pull/274)
  adds the selected-covariance omnibus Wald test of all non-intercept
  coefficients with asymptotic chi-square(q). The exact fishing tests match
  independent R: logit W=194.280378518603, p=3.69887865746942e-44; probit
  W=232.020896467226, p=2.16086165111978e-52. Author output rounds to
  194.28/232.07: probit's difference is the documented information convention,
  not an unexplained tolerance exception. The runnable checker now covers 50
  coefficient/covariance/effect/Wald quantities; full person/design/prediction
  checks remain. Sixteen synthetic robust/clustered family/design variants
  validate W/df/tails; singular/rank-limited restrictions produce explicit
  unavailable tests, not pseudoinverses/reduced-q inference or discarded fits.
- **OLS/six-column comparison:** [PR #275](https://github.com/SaehwanPark/tabdat-explore-rs/pull/275)
  adds `comparison.td`, independently checked by base R4.6.1 and a standard-library
  checker: all 94 classical/robust logit/probit/OLS numerical quantities and all
  630 source/outcome/design/prediction records match. OLS intercept/slope are
  0.784154230742988/-0.242913654440961; classical slope SE 0.0104328468939993,
  HC1 slope SE 0.00862919092660423, R² 0.463304487997616 and Gaussian working
  log likelihood -195.167320651517 (ML variance RSS/N, not RSS/(N-K)). OLS uses
  t(628), binary fits normal z; linear fitted values span -0.202734122 to
  1.307384388 and are not clipped or relabeled probabilities. Native JSON/text
  reports the six ordered fits and selected covariance; the optional checker's
  joined comparison is not native named-fit storage/table support. No R/Python
  production dependency is introduced, and no cross-family LR test is implied.
- **Figure14.1:** [PR #276](https://github.com/SaehwanPark/tabdat-explore-rs/pull/276)
  adds native `figure.td` (reusing `comparison.td`) and `check_figure.py`.
  Fresh checksum-verified conversion and R4.6.1 references regenerate identically;
  all 94 fit quantities/630 exported records remain checked. The actual SVG
  matches all 630 raw outcomes and 1,890 R prediction vertices, stable x/row
  ordering and screen geometry (1e-8 pixel absolute; unchanged statistical/design
  tolerances). The unbounded OLS range is visible, not clipped. Native SVG was
  visually inspected in Chrome on macOS27.0.1 arm64; no broad platform/font or
  pixel-parity claim. Display jitter is deterministic y-only ±0.03 outcome units,
  not the author's Stata jitter(3)/RNG convention. JSON and piped-terminal runs
  produce identical SVG/prediction bytes. Corruption probes reject clipped
  geometry, permuted row identities and fabricated un-jittered metadata.
  Data/model/labels/order and no-clobber publication invariants have native tests;
  SVG contains raw float64 metadata and is not anonymized. No author dataset or
  generated graphic is redistributed in the repository.
- **Table14.1 summaries:** [PR #277](https://github.com/SaehwanPark/tabdat-explore-rs/pull/277)
  adds `summary.td`, using existing conditional `summarize` for charter, pcharter,
  ppier and lnrelp overall, pier (N178) and charter (N452). All 60 quantities
  (N/mean/sample n−1 SD/min/max) match independent base R4.6.1, with every one of
  the 630 source/outcome/design records checked in original source-ID order.
  Fresh checksum-verified conversion is byte-identical; unchanged absolute
  1e-10 summary/1e-12 prepared-record tolerances. JSON/text/piped-terminal exports
  are byte-identical and native terminal blocks were inspected. Checker probes
  reject population SD, swapped cohorts, wrong counts and permuted IDs; native
  actual-script tests cover small samples, constant-outcome SD0, ordering,
  invalid data/keys and file-collision preservation. This uses three ordered
  summary blocks, not a joined table, and introduces no engine/schema change,
  production R/Python dependency or source-data redistribution.
- **Native Table14.2 comparison:** [PR #278](https://github.com/SaehwanPark/tabdat-explore-rs/pull/278)
  adds `stored_comparison.td` and immutable named-report store/table/drop commands.
  The native requested six-column table independently matches all 94 R4.6.1
  quantities, selected covariance/normal-versus-t/likelihood/sample metadata and
  all 630 source/design/prediction records. Fresh checksum-verified conversion
  and regenerated R references are byte-identical; unchanged absolute/relative
  tolerances. Actual native text table inspected; text/piped table and all
  JSON/text/piped prediction bytes match, including the previous sequential-fit
  workflow. Corruption probes reject zeroed tiny tails, binary t metadata,
  wrong covariance/sample positions/order and mislabelled OLS likelihood.
  Native runtime/CLI/serde tests cover owned snapshots, reload/refit/failure,
  alignment/missing terms, registry limits/atomicity and zero-variance nulls.
  Clustered OLS t(G−1) is distinguished from residual df and binary normal z.
  This is report-only storage with selected covariance and fit-time sample
  positions, not restore, persistence, current-data identity/staleness claims,
  arbitrary post-estimation or WLS/GLS/IV report support. Other estimator families
  will need their own explicit reporting contracts as delivered.
- **Composed preparation/descriptive workflow:** [PR #279](https://github.com/SaehwanPark/tabdat-explore-rs/pull/279)
  adds `analysis.td`, reusing the existing six-model stored comparison and native
  figure commands, plus independent base-R `reference_analysis.R` and standard-
  library `check_analysis.py`. Eight ordered blocks cover 17 source variables,
  20 prepared variables overall/pier/charter, the explicit four-variable
  Table14.1 blocks and four fitted-value summaries. All **465 quantities**
  (93 rows × N/mean/sample n−1 SD/min/max) match R4.6.1 at unchanged absolute
  1e-10; every field/ID of 1,182 source and 630 prepared records matches at
  1e-12. Added income/1000 and 100*ln(pcharter/ppier) transformations are checked
  explicitly. Source/program/output hashes were freshly verified; conversion,
  R references and leaf predictions/SVG regenerate byte-identically. All 94
  stored-table quantities/630 prediction records and 630+1,890 SVG vertices/
  geometry pass on the same composed native output. JSON/text/piped exports
  and SVG bytes match. Native terminal summaries/table inspected. Seven
  corruption probes reject population SD, swapped cohorts, clipped OLS summary,
  income/pratio scaling errors, permuted IDs and changed source prices.
  Actual-script CLI tests cover composition, original order/transforms/sample SD,
  nested paths, all three surfaces, source preservation and four output-collision
  stages. Failures report the originating script/line and stop subsequent work;
  earlier successful exports remain, not whole-script rollback. No engine,
  inference, structured-result schema, dependency or source-data redistribution
  change. The example README audits every author program block against native
  execution and documents remaining format/display differences.
- **Still required:** reporting for future estimator families, other exact binary
  illustrations and needed factor/interactions/discrete-change foundations.
  Maximum score is a separate required locator, not silently absorbed into logit
  or removed from the inventory.
  This is neither complete binary-family nor full-book validation.

### MMA-IV weak-instrument replication contract and evidence

- **Identity/source:** Cameron and Trivedi (2005), §4.9.6, pp. 110–112,
  Table 4.5; author program [`mma04p4ivweak.do`](https://cameron.econ.ucdavis.edu/mmabook/mma04p4ivweak.do)
  and its [Stata 8 output](https://cameron.econ.ucdavis.edu/mmabook/mma04p4ivweak.txt).
  The data page identifies `DATA66.dat`/`DATA66.dct` and cites Kling (2001),
  “Interpreting Instrumental Variables Estimates of the Return to Schooling.”
- **Prepared sample/design:** retain `nlsflt == 1` and nonmissing `wage76` and
  `grade76`; the regression then uses complete cases over every model variable.
  Outcome is `wage76` (log wages). Endogenous regressors are `grade76`, `exp76`,
  `expsq76`; included exogenous controls, in source order, are `black`, `south76`,
  `smsa76`, `reg2`–`reg9`, `smsa66`, `momdad14`, `sinmom14`, `nodaded`,
  `nomomed`, `daded`, `momed`, `famed1`–`famed8`. Excluded instruments are `col4`,
  `age76`, `agesq76`; the controls are also instruments. Include an intercept.
  The source reports 3,010 observations for the retained analysis.
- **Reference outputs:** Stata 8/Table 4.5 reports the `grade76` 2SLS coefficient
  as 0.1324, with classical SE 0.0493 and robust SE 0.0488 (four-decimal
  comparison). The first-stage excluded-instrument test for `grade76` is
  F(3, 2980) = 8.07; the source additionally reports Shea partial R² = 0.00640757.
  The first-stage F is a conventional relevance diagnostic, **not** weak-IV-robust
  inference. Preserve the source's reported degrees of freedom for comparison;
  diagnose any sample/design discrepancy rather than adjusting tolerances.
- **TabDat validation:** the runnable [`schooling_iv.td`](../../examples/mma04p4ivweak/schooling_iv.td)
  and fixed-width [`data66_to_csv.awk`](../../examples/mma04p4ivweak/data66_to_csv.awk)
  converter were executed against the authors' `DATA66.dat`/`.dct` without
  copying source data into this repository. On the specified sample, TabDat
  returned N=3,010, `grade76` coefficient 0.13244845, classical SE 0.04934188,
  HC1 SE 0.04878509, and first-stage F(3, 2980)=8.06710. These match the author
  Stata 8 output rounded to four decimals. The runtime's conventional first-stage
  partial R² is 0.00805582; it is distinct from the separately reported Shea
  partial R²=0.006407574586203226, matching the author's 0.00640757 to eight
  decimal places (absolute rounding tolerance 5e-9). The independent
  [`reference_shea.R`](../../examples/mma04p4ivweak/reference_shea.R) uses base
  R 4.6.1 residualization of original/projected columns on the identical
  selected complete-case design; it also reports Shea values 0.0876256840 for
  `exp76` and 0.0829397109 for `expsq76`. The synthetic multi-endogenous
  R reference and executable regression tests in `crates/tabdat-stats/tests/`
  check distinct ordinary/Shea statistics, covariance invariance, no-intercept
  and single-endogenous designs, and zero/constant outcomes. The original data
  SHA-256 is `a3fc1600cee50ccf565128118b4f901d03b07c46fb471a9c08464250eb39486a`;
  dictionary SHA-256 is `600912c801d7e05489fd8651bd2659243a8a74b243b907bd56de8a07b8582988`.
  These are bounded diagnostics, not a completed weak-instrument analysis.
- **Access/limits:** source data remain external and are not copied into the
  repository until redistribution rights are established. The converter retains
  numeric values but not source labels. The example is just-identified, so an
  overidentification test does not apply to this design. GMM remains unimplemented;
  the schooling-specific scalar interval has now been replayed on DATA66 as described
  below. A successful 2SLS fit or conventional first-stage
  F does not resolve weak identification.
- **Bounded joint AR method:** `estat ar, null(b1 b2 ...)` after a nonrobust 2SLS
  fit tests all endogenous coefficients jointly using retained sample/design.
  Exogenous coefficients are unrestricted nuisance parameters; inference assumes
  valid instruments and homoskedasticity (exact F additionally requires conditional
  normality). `crates/tabdat-stats/tests/fixtures/anderson_rubin_reference.R`
  uses independent R 4.6.1 `lm`/`anova` on the existing synthetic two-endogenous
  fixture: null(1.7 -0.6) gives F(2,56)=5.42272181993005 and p=0.00703483761854517.
  Rust tests cover zero/nonzero nulls, no-intercept/no-control designs, weak
  relevance, and auxiliary failures; runtime/CLI tests verify retained sample,
  state, scripts and reporting. This is **method evidence**, not a newly validated
  book illustration or an empirical scalar schooling confidence set with experience
  effects left as endogenous nuisance parameters. Robust/cluster AR confidence sets
  and the less-conservative subvector AR test remain required work.
- **Bounded scalar AR confidence-set method:** after a nonrobust fit with exactly
  one endogenous regressor, `estat ar, ci [level(95)]` analytically inverts the
  homoskedastic AR test, leaving exogenous coefficients unrestricted. Independent
  R 4.6.1 `lm.fit`/`qf`/`polyroot` evidence is in
  `crates/tabdat-stats/tests/fixtures/ar_confidence_set_reference.R`. The 40-row
  orthogonal fixtures validate bounded [1.282246370466847, 1.717753629533153],
  disjoint (-inf, -4.536179309204275] union [5.198096652710294, +inf), all-real and
  empty 95% sets, plus a 90% no-intercept reference. Tests compare roots/critical F
  with absolute tolerance 1e-10, check membership against null tests, and exercise
  linear/constant/singleton polynomial boundaries and retained state/reporting.
  Rank/DF/nonfinite/range failures and numerically degenerate auxiliary residual
  pairs are explicit errors; see `help estat` for the positive-variance condition.
  This is synthetic method validation, not a newly replicated empirical
  illustration or a schooling-only interval from the three-endogenous design.
- **Bounded projected scalar AR method:** after a nonrobust multi-endogenous 2SLS
  fit, `estat ar, ci coefficient(name) [level(95)]` profiles all other endogenous
  coefficients out of the homoskedastic joint AR region and reports the scalar
  projection. The joint region uses F(q, N-rank[W,Z]); projection therefore has
  at-least-nominal coverage, not the nominal coverage of a less-conservative
  subvector test. Profiling is analytic, with no finite grid or nuisance plug-in.
  The three-endogenous synthetic design has a bounded 95% grade interval
  [0.549119702658308, 2.050880297341690], matching independent R 4.6.1 `lm.fit`,
  `optim` and `uniroot` to 1e-10; a second non-orthogonal design with nonzero
  nuisance cross terms matches its nonsymmetric interval
  [1.381379408356060, 2.513745876229364], and a weak-nuisance design exercises the
  explicit conservative all-real fallback. On the author
  DATA66 sample the command retains N=3,010 and reports a 95% grade76 set of
  [-0.014387552440543, 0.502853235901166], matching the independent
  [`reference_ar_projected.R`](../../examples/mma04p4ivweak/reference_ar_projected.R)
  profile to 1e-10. The exact source/design are recorded above; the source data
  remain external and are not committed. Runtime tests cover retained sample,
  state, JSON metadata and robust/cluster rejection. This advances the weak-IV
  inference method for Table 4.5 but does not validate the entire illustration.
  Robust/cluster projection and the less-conservative subvector AR test remain open.

Missing seed rows (including simulation/Bayesian, measurement error and imputation)
are **un-inventoried required work**, not exclusions. Several locators may contain
multiple analyses: split them into separately verifiable rows as they are inspected.
Do not label the whole book covered until that inventory is complete.

## MMA-COUNT: defined Poisson/NB2 portions, not whole-illustration completion

[PR #280](https://github.com/SaehwanPark/tabdat-explore-rs/pull/280) implements
[`poisson.td`](../../examples/mma20p1count/poisson.td), the Poisson coefficients
and first/second statistic columns of Table20.5 under the November2006 author
program/output; NB2 is separately validated below, not the complete table.
The author executable uses all five
years despite a stale year2 comment: 20,190 original person-years minus four
missing-education rows give 20,186 complete ASCII records. Original DTA row
positions excluded are 9746/10153/10400/10859; prepared positions are not original
or person IDs. Source checksums/converter/optional DTA audit are recorded in the
[example](../../examples/mma20p1count/README.md). All 18 variables/order and exact
counts match DTA after exclusion; predictor differences <5e-6 are ASCII export
quantization, not a tolerance for fitting different inputs.

The exact prepared float64 design is MDU on the 17 ordered source predictors
and intercept, unweighted/no offset. N=20,186; count sum=57,746; zeros=6,308;
maximum=77. Independent R4.6.1 Poisson/Fisher/score/delta reference validates
1,095 coefficient/full-covariance/normal-CI/tail/LR/Wald/diagnostic/effect
quantities and every field/position/mean/xb of every record, with the documented
1e-10 ordinary, 1e-7 accumulated-total, 1e-12 design and relative-tail tolerances.
Model-based inverse information has no N-K scaling; robust is N/(N-1), not HC1 or
Pearson scaling. Robust LR p/reference are unavailable; selected-V Wald and
coefficient inference are asymptotic chi-square/normal. Retained-X continuous
column derivatives and descriptive deviance/Pearson dispersion are native
validated extensions, not extra claimed book outputs or formal panel tests.

The actual script has runtime/CLI schema, sample, failure/retention, prediction,
output-collision and JSON/text/piped regression coverage. Repeated person-years
require clustered inference for dependence: these two reproduced columns are
**not panel-robust**. The ASCII omits original person IDs; the native Poisson
extensions below recover audited DTA IDs separately. Household/site/multiway,
Remaining exposure workflows (Poisson/NB2 fixed offsets are separately validated method evidence), current-row probabilities/unresolved printed Table20.6 NB2 row,
other formal NB specification tests and robust/clustered NB boundary-LR reference,
complete Tables20.3–20.6 and other
count illustrations remain required. `estimates store` remains OLS/logit/probit
only; no count fit restoration/persistence or general count test/lincom is claimed.

[PR #281](https://github.com/SaehwanPark/tabdat-explore-rs/pull/281) adds
[`nb2.td`](../../examples/mma20p1count/nb2.td) on the identical prepared sample:
NB2 variance mu+alpha*mu², alpha=exp(lnalpha), joint ML for beta/lnalpha. Independent
R4.6.1/MASS7.3.65 validates 1,191 quantities and every source/design/mean/index
record for both covariances. Full converged observed-information inverse includes
beta/dispersion cross terms; MASS conditional/Fisher vcov is not substituted.
Robust is full joint-score N/(N-1), not HC1. Normal beta inference, transformed
log-alpha intervals, restricted NB nuisance-dispersion refitting, robust LR
suppression and beta-only selected-V Wald are explicit. Pearson/(N-K-1) and
NB deviance at fitted alpha are descriptive native extensions, not formal tests.

The bounded implementation accepts exact loaded counts0..100000 and lnalpha
strictly(-12,12), N>K+1, full rank/positive final undamped observed curvature and
absolute scaled score/full-step<=1e-10. Boundary/Poisson-limit/nonrepresentable
fits fail; there is no clipped-alpha or implicit Poisson fallback. Original
DTA/ASCII NB2 coefficient differences <=4.38e-8 are source quantization; native
validation uses exact ASCII. Historical stopping/rounded output is not the
float64 score fixed point (HLTHP approximately0.4256563 vs converged0.42565748).
General count restrictions, other formal NB specification tests and robust/clustered NB boundary-LR reference, multiway/survey,
current-row probability predictions and remaining illustration outputs stay required;
this addition does not complete the book's count workflow or panel inference.

[PR #282](https://github.com/SaehwanPark/tabdat-explore-rs/pull/282) adds the
[native original-person Poisson extension](../../examples/mma20p1count/poisson_clustered.td),
not a cluster column printed in Table20.5. Development-only `prepare_person.R`
compares every outcome/predictor/order against original DTA (exact counts,
predictor differences<5e-6 ASCII quantization) before attaching original row,
zper/person ID and study year. Four education-missing exclusions are unchanged;
G5908, group sizes1:265/2:246/3:3743/4:70/5:1584, unique person-year keys and all
five years are verified. No prepared positions are substituted for person IDs.

R4.6.1/sandwich3.1.1 `vcovCL(type='HC0',cadjust=TRUE)` matches467 synthetic and
551 empirical quantities plus every source/ID/design/mean/index field of all20,186
records. Group scores are summed BEFORE outer products, correctionG/(G-1), normal95
not G-1 t; LR reference unavailable and slope Wald guarded by G-1 rank. Converged
R weights are synchronized by fixed-point warm start, not a relaxed tolerance.
Source preparation refuses replacement and detects positional/source/key errors;
JSON/text/piped CSVs agree. Retained-X effects keep original covariance/groups,
current predictions need mean predictors only. This assumes independent PERSON
clusters; household/site/cross-person dependence, few-cluster methods, survey,
FE/RE/panel models and remaining count outputs stay required as
applicable. It is not full RAND design-based or causal inference, native DTA IO,
whole count-family or book completion.

[PR #283](https://github.com/SaehwanPark/tabdat-explore-rs/pull/283) adds the
[native NB2 original-person extension](../../examples/mma20p1count/nb2_clustered.td)
on the SAME audited20,186 rows/5,908 actual people and Table20.5 mean design,
not a printed cluster column. R4.6.1/MASS7.3.65 references535 synthetic/599
empirical quantities/all original/prepared/person/year/source/design/mean/xb
records. MASS starts only: full joint beta/lnalpha score/observed curvature is
polished and checked by normalized numerical score differentiation; each grouped
score is independently checked against grouped dnbinom log-PMF differentiation.
This is not `vcov(glm.nb)` or conditional/Fisher bread. All19x19 covariance
entries, dispersion SE/transformed CIs and retained-X delta effects match;
G/(G-1) full joint group scores, normal not G-1 t. Beta-slope Wald excludes
lnalpha and guards q>=G/rank. Clustered slope-LR references or normal-Wald
alpha=0 tests are not manufactured.

The singleton case in the535-quantity fixture matches141 shared existing
row-robust quantities exactly. References regenerate byte-identically; unchanged ordinary/total/design/
relative-tail tolerances, exact state/sample/key/failure/legacy-shape tests and
actual JSON/text/piped output agreement validate this bounded extension. Twenty
corruption probes reject missing cross covariance, row-robust substitution,
wrong correction/DF/null dispersion/fake alpha tests/symmetric intervals/tiny
tails/IDs/order/design/clipped means; prior complete Poisson/NB2/person-Poisson
empirical regressions remain passing. Person independence remains an assumption;
no household/site/multiway/survey/few-cluster guarantee, FE/RE, native DTA,
formal specification test or whole count/panel/book completion follows.

[PR #284](https://github.com/SaehwanPark/tabdat-explore-rs/pull/284) adds the
[native observed/fitted count-frequency analysis](../../examples/mma20p1count/count_probabilities.td):
`estat countprob [, max(10)]` uses original retained y/fitted means/NB2 alpha,
exact0..9 plus >=10 by default. Each fitted fraction averages individual
conditional probabilities, NOT the probability at an average mean. Model-based/
row-robust/cluster covariance choices do not establish the full Poisson/NB2 law
or alter these plug-in probabilities on the same sample. No uncertainty/formal
Pearson goodness-of-fit test follows; current-row probability prediction stays open.

R4.6.1/MASS7.3.65 independently validate2090 scalar PMF/survival values across37
stress cases,95 aggregate/model quantities and every source/design/mean/xb field
of all20186 records. A separate native stats-helper probe matches444092 individual
probabilities with relative-tail tolerance; it is development evidence, not an
installed rowwise probability export. Log PMFs, direct small gamma/beta tails,
well-conditioned complements and compensated integer log-factorials avoid
underflow/cancellation without normalization/clipping or legacy inference changes.
JSON/text/piped actual scripts agree;20 corruption probes reject wrong laws,
averaging/tails/samples/counts/means/probabilities/formal-test claims.

**Printed Table20.6 remains partial/unresolved**, not literally reproduced:
the printed NB2 percentages0..2 are30.9/19.6/13.6 versus independent native/
November2006 author-program31.82679994/19.07192866/12.75571956. The authors'
program explicitly acknowledges the unknown discrepancy. At zero the difference
is0.9268 percentage points, not float64 stopping/rounding. No parameter/tolerance
tuning is used to erase it. Exact source/sample/design/law are validated against
the author-program specification; the printed row is not silently excluded or
marked complete. Remaining table/illustration/formal-test/core requirements stand.
No author data/program/book redistribution or production R/Python dependency.

### NB2 Poisson-boundary likelihood-ratio method evidence

[PR #308](https://github.com/SaehwanPark/tabdat-explore-rs/pull/308) adds the
one-sided test `H0: alpha=0 (Poisson)` against `H1: alpha>0 (NB2)` using the
full NB2 and same-design/sample/offset Poisson likelihoods. The 50:50
chi-square(0)/chi-square(1) reference is reported only with model-based
covariance, under the regular one-parameter boundary conditions described by
[Self and Liang (1987)](https://doi.org/10.1080/01621459.1987.10478472).
Robust and clustered fits report the LR statistic without that classical
reference or p-value. R4.6.1 independently fits the Poisson null with `glm.fit`
and checks its likelihood against `dpois`; the full original Table20.5 source
audit matches the test along with 1,191 NB2 quantities and all 20,186 prepared
rows. The source LR is 34,620.0220788; its float64 p-value underflows to zero,
not an exact probability. This method extension does not reconcile the authors'
unresolved printed Table20.6 NB2 row or complete the count chapter.

## Additional fixed count offset evidence (not a textbook replication)

The [synthetic offset workflow](../../examples/poisson-offset/README.md) delivers
`poisson ..., offset(var)` with fixed coefficient one, joint finite sampling,
same-offset nulls, unchanged model-based/N/(N-1)/G/(G-1) covariance conventions,
current-data means/indices including offset, and retained effects/count bins.
Base R4.6.1 `glm` plus independent sandwich algebra validates 950 scalar quantities
across three designs and three covariance choices, including all means/indices,
full coefficient/covariance/normal inference, LR/Wald/diagnostics and continuous
delta effects. Native tests pin retained indices, zero-offset/shift invariance,
current-versus-retained behavior, atomic failures and JSON/text/script output.
The separate [NB2 offset workflow](../../examples/nb2-offset/README.md) validates
1,076 R4.6.1/MASS7.3.65 quantities across three designs/three covariance choices:
full beta/lnalpha observed information/grouped joint scores, all means/indices,
coefficient/dispersion/normal inference, NB null alpha/likelihood refitted with
the SAME offsets, LR/Wald/diagnostics and continuous delta effects. Base R numerical
score/curvature checks distinguish this from conditional GLM/Fisher covariance.
Native tests protect exact samples, offset/intercept shifts, provenance and
current-versus-retained/state/file contracts through the actual JSON/text script.
Exposure workflows may use equivalent explicit log-offset preparation rather than
requiring an `exposure()` alias. Neither synthetic method extension changes the
no-offset Table20.5 design, validates another book illustration or resolves the
printed Table20.6 discrepancy.

## Additional IV method evidence (not a textbook replication)

The external AER `ivreg` CigarettesSW example is specified and reproduced in
[`examples/cigarettes_iv/`](../../examples/cigarettes_iv/README.md). The script
selects all 48 states observed in 1995, orders state codes, and fits log packs on
log real price (endogenous) and log real per-capita income with an intercept;
excluded instruments are real sales-tax difference and real cigarette tax.
The README records source URL, CSV checksum and exact numerical/sample contract.
Independent R 4.6.1 QR fits yield coefficients (9.8949555411551628,
0.2804048250834234,-1.2774241334272722), Sargan N*R²=0.3326221419365246,
chi-square DF=1, p=0.5641191400175745; native replay matches within 1e-12 absolute
tolerance. The statistic is N*(u'P_Z u)/(u'u), not an N-K-corrected variant.

`estat overid` uses retained structural residuals/design without refitting or
mutating state. Numerical tests additionally validate two-endogenous,
three-/four-instrument, intercept/no-intercept fixtures and independent chi-square
tails; runtime/CLI cases cover exact retained indices, current-data changes,
just-identified/robust/cluster rejection, undefined variance and stop-before-export.
Inference is homoskedastic, correctly specified maintained model, conventional
strong-identification asymptotics. Failure to reject does not prove instrument
validity; Hansen J and GMM remain required work. Independent-row robust parent
fits now use a separate Wooldridge residualized-instrument score, not Sargan:
[`robust_overid.td`](../../examples/cigarettes_iv/robust_overid.td) uses the same
48-state design and structural residuals. With L=4, K=3 and q=1, independent base
R4.6.1 SVD/lm.fit gives chi-square(1)=0.3347358817061415 and
p=0.5628836468492445; native values match at 1e-12 absolute, along with all ordered
state identities/240 prepared cells and the pinned source hash. The score OPG is
uncorrected (factor1), not parent HC1. It projects the full instrument space off
Xhat=P_Z X, uses exactly q=L-K complement directions and regresses ones on their
products with structural residuals, without an intercept. Five independent R
synthetic designs validate q1/q2, mixed first stages, single/multiple endogenous,
intercept/no-intercept/no-control cases and native instrument-coordinate/unit
invariance at 1e-11 absolute. See the
[reference and checker](../../examples/cigarettes_iv/README.md#independent-row-robust-overidentification-score).
Retained state/failure and CLI/script tests cover rank/DF/undefined-score failures
without silently lowering q or running later file effects. This conventional
independent-row asymptotic score is not Hansen J, weak-IV or cluster/survey
inference; clustered overidentification remains required work. It adds external
method evidence only, not a completed textbook illustration.

The same exact sample/design additionally supports joint homoskedastic
`estat endogenous`, using the independent
[`reference_endogenous.R`](../../examples/cigarettes_iv/reference_endogenous.R).
Restricted/innovation-augmented OLS RSS are 1.578794093504594 and
1.475890440962176; Wu–Hausman F(1,44)=3.067816272944078, p=0.08682504624130956.
Native F/tail match at 1e-12 absolute tolerance with the same pinned state IDs.
The R script distinguishes an unimplemented HC0 contrast F=3.823467179945732,
not an alternate target for the classical test. Tests additionally cover five
R synthetic just-/overidentified, multiple-/single-endogenous, control/no-control,
intercept/no-intercept designs, unit invariance, null/rank/DF/degenerate-variance
cases and retained state/reporting. All m declared endogenous regressors are
jointly tested with N-K-m denominator DF, conditional on valid instruments and
the maintained model; no 2SLS refit or current-data rescan occurs. The conventional
F is not weak-IV-robust coefficient inference; exact finite-sample F additionally
requires conditional normality. The separate
[`robust_endogenous.td`](../../examples/cigarettes_iv/robust_endogenous.td)
now adds HC1 augmented-OLS joint innovation Wald inference: independent base
R4.6.1 gives chi-square(1)=3.504844914950468, p=0.06118957434570806 and correction
48/44, matched at 1e-12 absolute alongside every ordered state ID/240 prepared
cells by the [replay checker](../../examples/cigarettes_iv/check_robust_endogenous.py).
One-way clustered parent fits use grouped auxiliary scores and
G/(G-1)*(N-1)/(N-Kaux), with Kaux=K+m, not parent IV K. Five independent base-R
synthetic designs validate ten HC1/CR1 statistic/tail pairs (1e-11 absolute),
unit invariance, intercept/no-control variants and joint one/two-endogenous
just-/overidentified designs. Result metadata distinguishes correction DF from
the asymptotic chi-square(m) reference; singular Wald covariance is an error.
The command retains exact fit sample/group IDs across current edits without
mutating model/data. These conventional joint tests require valid instruments,
the maintained linear model and strong identification; clustered inference
needs independent growing groups, not a few-cluster guarantee. Robust score/GMM
C and subset tests, and any other inference required by exact illustrations,
remain unfinished, not excluded. **This is method evidence only: no Cameron–Trivedi illustration
row changes status or becomes validated because this external example works.**

## Additional clustered binary method evidence (not a textbook replication)

The [Bangladesh contraception example](../../examples/contraception_cluster/README.md)
uses the external `mlmRev::Contraception` CSV: N=1,934 women, 759 events, 60
districts; pooled unweighted logit/probit on an intercept and provided centered
age, clustered by district. Source checksum, preparation, scripts and independent
R4.6.1 numerical/prediction evidence are recorded there. All 1,934 person/group/
outcome/design/prediction records and 50 covariance/coefficient/delta-AME/Wald
quantities match (the omnibus extension is in PR #274); eight synthetic clustered family/design references validate
full covariance/normal inference. Singleton/relabeling, missing clusters,
wide-ID preservation, retained state and CLI failures have native coverage.
[PR #273](https://github.com/SaehwanPark/tabdat-explore-rs/pull/273) adds this
G/(G-1) cluster-score sandwich with converged observed-information bread.
Inference is asymptotic normal with independent clusters, not G-1 t or a
small-cluster coverage guarantee. LR p/reference are unavailable. This does
not fit a random-effects model, supply survey/design-based inference, or
validate any book illustration; the chapter-24 linear/Poisson requirements
and remaining binary work stay explicit.

## How to turn a seed into an implementation contract

Before implementing the chosen illustration, record alongside its test/fixture:

1. **Identity:** edition, chapter/page/table or figure, author program URL/name and
   relevant commands; retain source version/hash when available.
2. **Data:** origin, access/redistribution terms, ingestion/conversion steps,
   schema, labels, ordering and inspectable preparation script. If original data
   cannot be redistributed, document acquisition and keep a small synthetic
   regression fixture; that fixture alone does not validate the original example.
3. **Statistical contract:** outcome, design/reference levels, instruments or
   panel/survey structure, exact sample/exclusion rules, weights, estimand,
   parameterization, covariance corrections, degrees of freedom and diagnostics.
4. **Reference:** independent software/version, capture script or retained output,
   coefficients/covariance/inference/predictions/effects and justified tolerances.
   Compare samples/design before interpreting numerical disagreements.
5. **User workflow:** executable TabDat script, terminal and structured reporting
   appropriate to the slice, result export/plots when required by the illustration,
   and important invalid-input/convergence/state failure cases.
6. **Evidence/status:** links to implementation, tests and artifacts; explicitly
   list options/results still missing. Record files/tests/deviations at handoff.

Do not create a large document suite for each command: an inspectable script,
fixture/reference metadata and focused tests can carry this contract. Use existing
crate boundaries and fixture conventions; no new harness is required.

### Status and completion rules

- **Planned:** no complete executable replication; parser support does not advance it.
- **In progress:** bounded scope is specified and work exists, but required results
  or evidence are incomplete. Enumerate the remainder.
- **Blocked:** record a specific data, identification, numerical or interface blocker
  and the next action; do not invent missing reference semantics.
- **Validated:** the specified illustration executes end to end, with exact
  sample/design checks, independent numerical comparisons and required
  interpretation/reporting evidence linked. This validates only the named scope.

Splitting a partial replication into a validated narrow row plus planned remaining
rows is acceptable; hiding the remainder is not. Any exclusion from the empirical
book target requires an explicit product-scope decision and rationale. Prefer an
equivalent Rust-native analysis over source-language syntax parity. No runtime
Python/R dependency is permitted; development-only reference capture is allowed.

## Sources

- [Authors' book supplement][supplement]: programs, data and output.
- [Authors' chapter outline][outline] and part summaries: method-family scope.
- [Authors' empirical dataset index][data]: seed locators and file leads above.

[outline]: https://cameron.econ.ucdavis.edu/mmabook/mmaoutline.html
[data]: https://cameron.econ.ucdavis.edu/mmabook/mmadata.html
[supplement]: https://cameron.econ.ucdavis.edu/mmabook/mma.html

### MMA-PANEL unbalanced Gaussian ML sensitivity evidence

The [explicit MOM sensitivity cohort](../../examples/mma21p1panel/README.md)
retains4,738 rows/532 people with T_g5–10, preserving source order/identity.
Native ML uses each retained T_g, re-estimates the variance-weighted null, and
reports joint beta/log(sigma_e²)/log(sigma_u²) observed information. R4.6.1 /
nlme3.1.169 and numerical Hessians validated65 reported quantities and52,118
exported cells; 531 synthetic comparisons cover singleton, shifted/interleaved
and boundary cases. Existing balanced/interior and boundary evidence remains.
Runtime failure/sample-state and text/JSON/export tests protect the full workflow.
Absolute1e-9/nonzero-tail relative1e-8 tolerances are unchanged. This is method
and changed-sample sensitivity evidence, not a new published Table21.2 row.
Updated balanced/unbalanced R references add 65 component-uncertainty comparisons
across positive-interior and zero-boundary fits; the full synthetic fixtures now
contain 531 scalar comparisons. Positive interior components use
delta-method SEs and exponentiated log-Wald intervals from joint observed information;
the zero unit-variance component has no regular interval. The full-source audit
compares65 reported quantities, including10 component-uncertainty values and both
unit-variance LR values, plus52,118 exported cells across4,738 rows. Robust ML
inference and panel diagnostics remain required; no whole-table/family
completion or global optimizer guarantee is implied.

### MMA-COUNT current-row probability method extension

Bounded `predict ..., pmf(k)`/`cdf(k)` for integer0..100000 or the current numeric
cutoff variable adds conditional PMF/CDF values on current predictors/offsets
under Poisson/NB2 laws. Missing cutoffs return NULL; negative, fractional or
greater-than100000 values fail atomically. Above99, a prediction may add at most
10,000,000 cutoff increments across rows with usable model inputs. This is a method
extension of the existing Table20.5 mean-model specifications, not new printed
coverage. The [complete synthetic workflow](../../examples/count-probability/README.md)
validates308 base-R helper probabilities,432 fixed-cutoff and144 row-specific
offset-fit probabilities, including changed predictors/offsets, reversed order
and three covariance choices. CLI exports match across text/JSON and covariance
choices. Raw state/sample/labels, missing/overflow/zero-mean limits and atomic
failures, including the expanded-work guard, are covered; no distributional
guarantee follows from robust covariance. Parameter uncertainty/formal calibration,
and printed Table20.6 NB2
reconciliation remain unresolved. Underlying count-fit contracts are preserved.

### MMA-PANEL p719 robust auxiliary Hausman comparison

The [complete composed workflow](../../examples/mma21p1panel/README.md) validates
§21.4.3 p719 / March2005 mma21p2panmanual.do block7 on all5,320 MOM rows/532 people.
Joint-finite means, balanced Swamy–Arora theta and the auxiliary within-wage
restriction use existing SQL/OLS cluster covariance (K3) and native F(1,531)
inference. H=t²=1.649507405254, F p=0.199586037144; the asymptotic chi-square
tail is distinguished. Conventional covariance subtraction is not used under
within-person dependence. Independent R/plm/sandwich checks39 quantities and
95,760 raw/design/prediction cells at unchanged tolerances; separate float32
input matches8 selected author-printed facts. Small interleaved/excluded-unit
fixtures, sample/rank/theta guards and export failure/identity checks complete
the workflow. This adds one bounded illustrated comparison, not general native
`estat hausman`, arbitrary model selection, bootstrap or full panel coverage.

### MMA-PANEL Between seeded pairs-bootstrap method evidence

The [complete workflow](../../examples/mma21p1panel/README.md) extends the
Between-bootstrap block of March2005 mma21p2panmanual.do: equal-weight means on
all5,320 jointly finite MOM observations/532 people, then resample532 independent
mean-response/design records per replicate, B200/explicit native seed10001.
An independent Python xoshiro/SplitMix64/index oracle and base-R lm/cov/quantile
check18 scalar summaries and4,256 mean/design/prediction cells; all original
loaded cells are pinned separately. Synthetic references add1,440 exact sample
indices and76 R coefficient/covariance/SE/bias/type7-interval quantities across
multiple, no-intercept and shifted designs. This is native reproducible method
evidence, not the author's unspecified Stata draw stream or a printed bootstrap
SE replication. It does not add raw person-year/cluster/FE/RE bootstrap or complete
Chapters11–13. Any failed replicate aborts without redraw/discard. Bootstrap
reports do not replace retained fits or supply analytic test p-values.

### MMA-PANEL within-transformed whole-person pairs-bootstrap evidence

The [bounded workflow](../../examples/mma21p1panel/README.md) implements block4
of March2005 mma21p2panmanual.do: all5,320 jointly finite MOM rows/532 people,
joint-case within-person lnhr/lnwg deviations, transformed OLS with intercept,
B500/explicit native seed10001, resampling532 complete people per replication.
Independent Python group draws and R lm/cov/type7 check18 bootstrap quantities
and47,880 source/transform/prediction cells, with original loaded source pinned.
Synthetic balanced/unbalanced/no-intercept/shifted designs add640 exact group
indices and285 numerical/size quantities. Native text/JSON exports are identical.
The native stream differs from historical Stata; no printed SE equality is
claimed. Fixed-transform OLS resampling is installed, not automatic FE/RE refits,
RE nuisance-component bootstrap or FE intercept inference. The transformed
intercept is numerical zero. No few-cluster or nominal-coverage guarantee;
any bad replication aborts without redraw/discard and prior fit/data stay intact.

### MMA-PANEL fixed-theta RE-GLS whole-person bootstrap evidence

The [bounded workflow](../../examples/mma21p1panel/README.md) implements block5
of March2005 mma21p2panmanual.do: all5,320 MOM rows/532 people, guarded balanced
joint cases, Swamy–Arora theta estimated once, ordinary transformed OLS with
intercept, whole-person B500/native seed10001 bootstrap. The transformed intercept
is (1-theta)*population alpha. R/plm independently checks theta and original
parameter scaling, then independent group draws and R lm/cov/type7 validate18
bootstrap quantities plus theta,95,760 identity/design/prediction cells and all
original loaded rows. Text/JSON exports are identical. Synthetic interleaved
24-row evidence adds18 bootstrap quantities/432 cells and failure/export cases.
This conditions on fixed theta, not native RE variance-component re-estimation
or unconditional nuisance inference, historical Stata RNG/printed-SE parity,
few-cluster accuracy or a nominal-coverage guarantee.

### Robust joint Anderson–Rubin method extension

The [synthetic workflow](../../examples/robust-ar/README.md) validates retained
unweighted 2SLS joint nulls with auxiliary HC1/one-way CR1 Wald chi-square(q),
using unrestricted adjusted-outcome residuals, K_aux corrections and the entire
excluded-instrument block. Base R4.6.1 supplies six cases across controlled,
no-intercept and no-exogenous-control designs, with two endogenous coefficients,
three excluded instruments, explicit missingness and a separate fixed-adjusted-outcome zero-relevance check. Exact
source identities, N/K/G, corrections, statistics and tails are tested; unit/order
invariance, unchanged state and rank-limited failures are covered. The 12-group
fixture establishes arithmetic only, not few-cluster reliability. This is method
evidence advancing required IV inference, not a new textbook illustration,
robust confidence sets, endogenous-subvector inference, HAC or survey support.

### MMA-PANEL interior full joint unit-score ML method evidence

The [native extension](../../examples/mma21p1panel/README.md#interior-gaussian-ml-unit-score-inference)
uses the exact Table21.2 lnhr~1+lnwg design on all5,320 original MOM rows/532
people, with full joint beta/log-variance unit-score sandwich covariance and
G/(G-1). Independent R4.6.1/nlme3.1.169, dense marginal unit derivatives and
numerical joint Hessians validate53 scalar quantities and58,520 exported
source/prediction cells; all original values/IDs/order are checked against the
pinned source. Text/JSON exports are byte-identical. Four synthetic designs add292
comparisons, with exact finite-case identity, balanced/unequal counts, shifts,
interleaving and a boundary restricted null. Full-model boundaries are rejected.

Selected covariance propagates to normal coefficient/log-variance intervals and
rank-aware slope Wald chi-square inference. Both model-based LR references are
withheld. This is robust method evidence on the existing illustration's mean
design, not a newly validated printed uncertainty column, latent-variance claim,
full panel milestone, non-unit/multiway/survey inference or few-cluster guarantee.
Boundary robust ML and non-unit clusters remain unsupported; any extension must
be selected against an exact illustration. Conditional unit effects, general
restrictions, bootstrap, panel diagnostics and remaining illustration outputs
remain required.


### MMA-QREG joint IID pairs-bootstrap method evidence

The [joint household script](../../examples/mma04p2qreg/bootstrap.md) advances
mma04p2qreg.do block3 / Figure4.1's underlying joint fit: original5,006 complete
ASCII records, nineteen quantiles .05,.10,...,.95, intercept/log total expenditure,
B200 and explicit native seed10101. Each IID row resample is shared across all
quantiles. Full joint covariance uses B-1, with marginal R-type7 percentile95
intervals and no simultaneous-coverage claim. The read-only report preserves
the retained median fit and source/design/current-row prediction workflow.

An independent Python xoshiro/SplitMix64 index oracle and R4.6.1/quantreg6.1 br
fits validate1,672 quantities and25,030 cohort/retained-median prediction cells;
all5,999 raw records and source order are pinned. The original audit uses the
optimized native binary. Synthetic cases add477 joint-bootstrap quantities and
27 near-active coefficient/covariance/normal-inference checks at unchanged
1e-8 absolute / relative1e-7 tiny-tail tolerances. A reproduced near-active LP
failure now refines the same problem once at stricter solver precision without
weakening certificates, redrawing or discarding a sample. Failure/work limits,
exact fits without replicate sparsity, sample/state and native-script guards
are covered. This is IID method evidence, not survey-design inference or the
historical Stata RNG/SE column. The bounded .25/.75 slope contrast now has an
independent common-draw normal-inference reference; Figure4.1 bands/rendering,
general restrictions and cluster/survey uncertainty remain uninstalled.

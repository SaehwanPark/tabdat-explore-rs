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
| 4: Linear, IV and quantile models | Bounded OLS/WLS/diagonal GLS, HC1/one-way cluster | IV/2SLS and diagnostics, quantile estimation; factor/interactions and weight semantics |
| 5–6, 10: ML/NLS, GMM/systems, optimization | Least squares and bounded binary logit/probit MLE/QR | Remaining nonlinear/system estimation, identification, optimization and valid covariance/convergence diagnostics |
| 7–8: Testing and specification | Linear restrictions, mean tests, VIF; binary LR/normal coefficient, robust/clustered omnibus Wald and delta-AME inference | Further model-specific LR/LM/Wald and specification tests, nonlinear restrictions, comparison and uncertainty |
| 9: Nonparametric/semiparametric | No executable workflow | Density, local/kernel regression and methods required by illustrations |
| 11–13: Bootstrap, simulation, Bayesian | Context-only seed, no resampling/estimation workflow | Actual RNG contract, IID/cluster bootstrap, simulation/Bayesian estimation and diagnostics |
| 14: Binary outcomes | Model-based/robust/one-way clustered logit and probit, probabilities/xb and continuous retained-X delta AME; validated native §14.2 published tables/figure and descriptive workflow under explicit conventions | General binary restrictions/specification tests, factors/interactions/discrete changes and uncertainty, other illustration outputs/graphics |
| 15: Multinomial outcomes | No executable workflow | Required multinomial/conditional and richer choice models with choice-set semantics |
| 16: Censoring, truncation, selection | Relevant parser forms only | Tobit/truncation, selection and two-part workflows with correct effects/inference |
| 17–19: Survival, heterogeneity, multiple hazards | No executable workflow | Event/censoring/truncation semantics, KM/parametric/Cox, mixture/frailty and multiple-event analyses |
| 20: Counts | Model-based/independent-row robust unweighted Poisson and joint beta/lnalpha NB2, means/xb, descriptive deviance/Pearson and retained-X continuous delta effects; validated Table20.5 coefficient/inference portions, audited original-person clustered extensions and descriptive retained observed/fitted probability-frequency bins | Exposure/offset, household/site/multiway/survey/panel inference, current-row PMF/CDF predictions, unresolved printed Table20.6 NB2 row, formal NB boundary/specification tests, hurdle/zero-inflated/mixture and remaining illustrated outputs |
| 21: Linear panels | Panel/xtreg parser only | Unit/time declaration, gap/duplicate policy, FE/RE/between/differences and diagnostics |
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
| MMA-QREG | pp. 88–90, Vietnam living standards; `qreg0902.dta` | Planned | Identify quantiles, model and uncertainty method; resolve ingestion |
| MMA-NONPARAM | pp. 295–297, 300, PSID; `psidf3050.dat` | Planned | Identify density/regression specifications, bandwidths and reference outputs |
| MMA-BINARY | §14.2, pp. 464–466, Tables 14.1–14.2/Figure 14.1; `mma14p1binary.do` / `Nldata.asc` | Validated native published-analysis contract — composed source/cohort/fitted summaries, native tables and SVG; separate binary AME extension | Exact output audit/conventions below; not literal Stata log/storage/RNG/format parity or fit restoration; binary family remains incomplete |
| MMA-MAXSCORE | §14.7.5, p. 486; `mma14p2maxscore.lim` / `mma14p1binary.asc` | Planned — author-index locator only | Resolve maximum-score specification, preparation and nonstandard uncertainty; not covered by logit |
| MMA-CHOICE | pp. 491–495, fishing modes; `Nldata.asc` / `mma15p4gev.asc` | Planned | Resolve choice-set layout and exact multinomial model variants/effects |
| MMA-SELECTION | pp. 553–556, 565, RAND; `randdata.dta` / `mma16p3selection.asc` | Planned | Separate selection/outcome samples, estimator variants and uncertainty |
| MMA-DURATION | pp. 574–575, 582, strikes; `strkdur.asc` | Planned | Specify spells, censoring, parameterization and survival/hazard references |
| MMA-HAZARDS | pp. 603–608, 632–636, 658–662, displaced workers; `ema1996` | Planned | Identify event types, repeated-spell/heterogeneity requirements and samples |
| MMA-COUNT | §20.3 pp. 671–674, Table20.5 p. 673; later p. 692, RAND; `mma20p1count.asc` / `randdata.dta` | Partial — Poisson/NB2 coefficients and model-based/independent-row robust statistic columns validated | 20,186 all-year prepared person-years; [native workflows/source audit/R references](../../examples/mma20p1count/README.md), Poisson1,095/NB2 1,187 quantities/all source/design/mean/index records; native person-Poisson551/NB2 597-quantity extensions/5,908 original people; native/November2006 author-program frequency analysis95 quantities/444092 individual probabilities validated, printed Table20.6 NB2 row unresolved; current-row PMF/CDF/formal tests/remaining Tables20.3–20.6 and richer counts remain required |
| MMA-PANEL | pp. 708–715, PSID; `MOM.dat` | Planned | Verify unit/time keys, transformations, model comparisons and covariance |
| MMA-DYNAMIC | pp. 754–756, PSID; `MOMprecise.dat` | Planned | Specify moments, instruments, weighting and diagnostic references |
| MMA-NLPANEL | pp. 792–795, patents/R&D; `patr7079.asc` | Planned | Identify nonlinear panel estimators and heterogeneity assumptions |
| MMA-SURVEY | pp. 848–853, Vietnam; `vietnam_ex1` / `vietnam_ex2` | Planned | Determine design/weight/cluster semantics, not just regression clustering |
| MMA-TREATMENT | pp. 889–895, NSW/PSID; `nswpsid` | Planned | Fix ATE/ATT, sample/control group, propensity/matching choices and inference |

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
  overidentification test does not apply to this design. GMM and schooling-specific
  scalar weak-IV-robust intervals remain unimplemented. A successful 2SLS fit or conventional first-stage
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
  book illustration or a scalar schooling confidence set with experience effects
  left as endogenous nuisance parameters. Robust/cluster AR and endogenous-
  subvector confidence sets remain required work.
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
offsets/exposure, current-row probabilities/unresolved printed Table20.6 NB2 row,
formal NB boundary/specification tests,
complete Tables20.3–20.6 and other
count illustrations remain required. `estimates store` remains OLS/logit/probit
only; no count fit restoration/persistence or general count test/lincom is claimed.

[PR #281](https://github.com/SaehwanPark/tabdat-explore-rs/pull/281) adds
[`nb2.td`](../../examples/mma20p1count/nb2.td) on the identical prepared sample:
NB2 variance mu+alpha*mu², alpha=exp(lnalpha), joint ML for beta/lnalpha. Independent
R4.6.1/MASS7.3.65 validates 1,187 quantities and every source/design/mean/index
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
General count restrictions, formal alpha=0/specification tests, multiway/survey,
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
not a printed cluster column. R4.6.1/MASS7.3.65 references527 synthetic/597
empirical quantities/all original/prepared/person/year/source/design/mean/xb
records. MASS starts only: full joint beta/lnalpha score/observed curvature is
polished and checked by normalized numerical score differentiation; each grouped
score is independently checked against grouped dnbinom log-PMF differentiation.
This is not `vcov(glm.nb)` or conditional/Fisher bread. All19x19 covariance
entries, dispersion SE/transformed CIs and retained-X delta effects match;
G/(G-1) full joint group scores, normal not G-1 t. Beta-slope Wald excludes
lnalpha and guards q>=G/rank. LR reference/alpha=0 tests not manufactured.

The singleton case in the527-quantity fixture matches141 shared existing
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

R4.6.1/MASS7.3.65 independently validate1787 scalar PMF/survival values across34
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
validity; robust/cluster Hansen J and GMM remain required work.

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
requires conditional normality. Robust/cluster/subset endogeneity tests are not
installed; any such inference required by the roadmap or exact illustrations
remains unfinished, not excluded. **This is method evidence only: no Cameron–Trivedi illustration
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

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
| 5–6, 10: ML/NLS, GMM/systems, optimization | Least-squares machinery only | Executable nonlinear/system estimation, identification, optimization and valid covariance/convergence diagnostics |
| 7–8: Testing and specification | Linear restrictions, mean tests, VIF | Model-specific LR/LM/Wald and specification tests, nonlinear restrictions, comparison and uncertainty |
| 9: Nonparametric/semiparametric | No executable workflow | Density, local/kernel regression and methods required by illustrations |
| 11–13: Bootstrap, simulation, Bayesian | Context-only seed, no resampling/estimation workflow | Actual RNG contract, IID/cluster bootstrap, simulation/Bayesian estimation and diagnostics |
| 14: Binary outcomes | Logit/probit parser only | Fit, robust/cluster inference, probabilities, marginal effects/discrete changes and uncertainty |
| 15: Multinomial outcomes | No executable workflow | Required multinomial/conditional and richer choice models with choice-set semantics |
| 16: Censoring, truncation, selection | Relevant parser forms only | Tobit/truncation, selection and two-part workflows with correct effects/inference |
| 17–19: Survival, heterogeneity, multiple hazards | No executable workflow | Event/censoring/truncation semantics, KM/parametric/Cox, mixture/frailty and multiple-event analyses |
| 20: Counts | Poisson/NB/zero-inflated parser forms only | Poisson QMLE/NB, exposure/offset, hurdle/zero-inflated/mixture and other illustrated count models |
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

## Seed empirical illustrations — none validated yet

These are starting points from the authors' [dataset index][data], not a complete
inventory or exact replication specifications. Page ranges are locators from that
index; inspect the book and accompanying program before naming a table, sample,
variable list or estimator variant. Source data must be checked for access,
provenance and redistribution terms before adding fixtures.

| ID | Book locator and data lead | Status | Next evidence / blocker |
| --- | --- | --- | --- |
| MMA-IV | pp. 110–112, schooling/weak instruments; `DATA66.dat` + dictionary | In progress — bounded 2SLS runtime | Exact source program, sample, design, Table 4.5 reference and limits are recorded below; implement executable 2SLS and first-stage diagnostics without bundling the source data |
| MMA-QREG | pp. 88–90, Vietnam living standards; `qreg0902.dta` | Planned | Identify quantiles, model and uncertainty method; resolve ingestion |
| MMA-NONPARAM | pp. 295–297, 300, PSID; `psidf3050.dat` | Planned | Identify density/regression specifications, bandwidths and reference outputs |
| MMA-CHOICE | pp. 463–466, 486, 491–495, fishing modes; `Nldata.asc` / `mma15p4gev.asc` | Planned | Resolve choice-set layout and exact binary/multinomial model variants/effects |
| MMA-SELECTION | pp. 553–556, 565, RAND; `randdata.dta` / `mma16p3selection.asc` | Planned | Separate selection/outcome samples, estimator variants and uncertainty |
| MMA-DURATION | pp. 574–575, 582, strikes; `strkdur.asc` | Planned | Specify spells, censoring, parameterization and survival/hazard references |
| MMA-HAZARDS | pp. 603–608, 632–636, 658–662, displaced workers; `ema1996` | Planned | Identify event types, repeated-spell/heterogeneity requirements and samples |
| MMA-COUNT | pp. 671–674, 692, RAND; `mma20p1count.asc` / `randdata.dta` | Planned | Identify Poisson/NB and richer count specifications, offsets and effects |
| MMA-PANEL | pp. 708–715, PSID; `MOM.dat` | Planned | Verify unit/time keys, transformations, model comparisons and covariance |
| MMA-DYNAMIC | pp. 754–756, PSID; `MOMprecise.dat` | Planned | Specify moments, instruments, weighting and diagnostic references |
| MMA-NLPANEL | pp. 792–795, patents/R&D; `patr7079.asc` | Planned | Identify nonlinear panel estimators and heterogeneity assumptions |
| MMA-SURVEY | pp. 848–853, Vietnam; `vietnam_ex1` / `vietnam_ex2` | Planned | Determine design/weight/cluster semantics, not just regression clustering |
| MMA-TREATMENT | pp. 889–895, NSW/PSID; `nswpsid` | Planned | Fix ATE/ATT, sample/control group, propensity/matching choices and inference |

### MMA-IV weak-instrument replication contract (bounded first slice)

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
- **Access/limits:** source data remain external and are not copied into the
  repository until redistribution rights are established. This bounded slice
  does not implement GMM, overidentification tests (the example is just-identified),
  Shea's statistic, or weak-IV-robust inference. A successful 2SLS fit or
  conventional first-stage F does not resolve weak identification.

Missing seed rows (including simulation/Bayesian, measurement error and imputation)
are **un-inventoried required work**, not exclusions. Several locators may contain
multiple analyses: split them into separately verifiable rows as they are inspected.
Do not label the whole book covered until that inventory is complete.

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

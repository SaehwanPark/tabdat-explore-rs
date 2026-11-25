# TabDat Explore Rust: outcome-oriented roadmap

## Destination and current baseline

TabDat is a small, terminal-first Rust application for dependable exploratory
data analysis and graduate-level econometrics. The long-term analytical target is
the empirical work in Cameron and Trivedi, *Microeconometrics: Methods and
Applications* (MMA, 2005). This is a product commitment, not a claim that the
book or its syntax is currently supported. Equivalent native workflows are
preferred to copying Stata or Python internals.

[SPEC.md](../SPEC.md) is the current capability map. The
[coverage ledger](reference/microeconometrics-coverage.md) is the source of truth
for illustration-level status and numerical evidence. The examples directory
contains executable workflows and their local contracts. The roadmap sets
priority and acceptance direction. It does not duplicate delivery logs.

The installed baseline is a usable data-preparation, EDA and regression core.
It includes local eager CSV/Parquet data, scripts, terminal and JSON output,
bounded linear, IV, panel, binary, count and selected nonlinear workflows, and
reference-checked textbook slices. Parser acceptance, executable behavior,
independent numerical evidence and release readiness remain separate claims.
Remote/lazy input, native DTA loading, broad survey estimation, general survival
declarations, Bayesian estimation and many textbook illustrations remain open.

Maintain three distinct milestones:

1. **Dependable core** — usable data preparation and linear-analysis workflows.
2. **Graduate core** — representative cross-section, IV, panel and nonlinear
   analyses with diagnostics, interpretation and reproducible reporting.
3. **Textbook empirical coverage** — validated workflows for the book's empirical
   illustrations, including their samples, designs, inference and post-estimation.

The first milestone does not satisfy the second, and the second does not imply
complete textbook coverage. Missing methods remain required work when a selected
analysis needs them.

## How to select work

Choose a small, complete **load/prepare → specify → fit → diagnose/interpret →
report/reproduce** slice. Implement parsing, execution, validation and reporting
together. A parser entry, native prototype or coefficient-only estimator is not
a delivered workflow.

- Define the sample, design, parameterization, covariance and degrees of freedom,
  prediction cohort, failure behavior and independent reference before coding.
- Prefer a concrete data/analysis need over broad syntax or a generic estimator
  framework. Reuse existing crates and boundaries.
- Keep exact source, method and inference distinctions visible. Do not silently
  convert a source discrepancy into tolerance or whole-family support.
- Update the ledger and SPEC only when supported behavior or evidence changes.
  Keep detailed gates beside the example, test or ledger entry.

## Priority 1: foundations that unblock real analyses

Deliver these with the estimator that needs them.

| Outcome | Required behavior and acceptance direction |
| --- | --- |
| Reliable model design | Explicit factor levels and references, interactions and transformations, stable parameter names/order, prediction-design rules and collinearity policy. The bounded `regress i.var`/`ib(level).var` design is installed. Interactions and cross-estimator factor semantics remain. |
| Deliberate samples and weights | Report exclusions and retain selected-cohort versus current-data prediction semantics. Distinguish analytic, frequency, probability and design weights. Existing WLS/diagonal GLS and one-way clustering are bounded methods, not general survey estimation. |
| Interpretable results | Preserve model-appropriate diagnostics, covariance corrections, inference degrees of freedom, coefficient uncertainty and defined prediction/effect uncertainty. |
| Analytical graphics | SVG scatter plots with supplied or fitted line overlays are installed. Add distribution, fitted/residual and model-specific plots only when a selected workflow has a defined output and geometry contract. |
| Reusable reporting | Retain named estimates, covariance, sample provenance and model metadata without silent refits or invented restoration guarantees. Preserve stable JSON contracts. |
| Reproducible data access | Keep local preparation and persistence tested. Add a reproducible source conversion or native reader when an identified illustration requires it, with explicit type, label and missingness losses. |

Ordinary finite summaries must remain distinct from SQL NULL missingness. Data,
state and output failures must preserve the promised pre-failure state.

## Priority 2: graduate econometrics core

The default order is IV → binary outcomes → count outcomes → linear panels →
quantile/resampling. A concrete dependency or correctness defect can change the
order. Each row describes a bounded end-to-end outcome, not every option.

| Workflow | Minimum complete outcome |
| --- | --- |
| **IV/2SLS** | Exogenous/endogenous/instrument design, exact sample and rank checks, classical and appropriate robust/cluster inference, first-stage diagnostics, endogeneity and overidentification tests where identified, and weak-instrument-robust inference where required. Bounded homoskedastic tests/sets include the DATA66 `grade76` AR projection with other endogenous coefficients unrestricted; HC1/one-way CR0 Wooldridge overidentification scores, HC1/CR1 scalar AR sets for one endogenous regressor and one to nine excluded instruments, and closed robust AR projection for exactly two endogenous regressors/two excluded instruments are also installed. Broader robust projection and endogenous-subvector tests remain. Conventional first-stage F does not certify validity. |
| **Logit/probit** | Domain and separation/convergence failures, model-based/robust/cluster inference, probability and linear-index predictions, continuous average marginal effects plus a bounded numeric 0/1 probability change with explicit estimands and delta uncertainty, and retained-fit Wald tests of general linear restrictions. |
| **Poisson/NB2** | Count-domain checks, offsets/exposure, explicit variance parameterization, model-specific diagnostics, conditional means/effects and covariance conventions, retained-fit Wald tests and linear combinations of mean coefficients, a Poisson NB2-direction score test with selected model/row/cluster score variance, bounded model-based unclustered plus one-way cluster-robust NB2 grouped score-adjusted goodness-of-fit tests, and current-row PMF/CDF predictions with pointwise delta-method SEs and logit-Wald intervals under the retained covariance plus discrete equal-tail future-count intervals under fitted means/law and raw current-row Poisson/NB2 response residuals. Distinguish Poisson QMLE from distributional claims. |
| **Linear panels** | Explicit unit/time keys, gaps and duplicates, lag/lead/difference semantics, pooled/within/Between/FD/RE fits, model comparisons, predictions and inference. Read-only [`xtdes`](commands/xtdes.md) and [`xtsum`](commands/xtsum.md) take explicit keys without retaining panel state; broader model comparisons and diagnostics remain open. |
| **Quantile regression** | Conditional quantiles, current-row predictions, IID uncertainty and a bounded whole-cluster pairs bootstrap with explicit resampling unit. Analytic clustered covariance, general restrictions, simultaneous bands, effects and Figure 4.1 remain. |
| **Seeded resampling** | Explicit seed/stream, IID and cluster units, B−1 uncertainty, percentile conventions and failure policy. Current OLS, quantile and selected Poisson/panel compositions are bounded source workflows, not a general automatic bootstrap API. |

Graduate-core acceptance requires representative, independently checked workflows
across these families with interpretation and reporting. One passing parser test
or one estimator coefficient vector does not establish that milestone.

## Priority 3: applied microeconometrics target

Use the ledger for exact source, sample and evidence. The following summaries state
the next outcome without repeating the ledger's detailed delivery records.

| Area | Installed bounded baseline | Required next outcomes |
| --- | --- | --- |
| Choice | MNL, balanced conditional logit and restricted normalized/non-normalized nested-choice workflows with source audits | Broader availability and choice sets, ordered/richer models, effects uncertainty, IIA diagnostics and remaining illustrations |
| Censoring and selection | Gaussian Tobit, lower-truncated Gaussian ML and corrected Heckman two-step workflows with declared inference/effects | Richer truncation/censoring, two-part and selection workflows, default ML and remaining illustrations |
| Survival | KM/NA compositions plus bounded exponential, Weibull, Gompertz and Cox/Breslow source workflows | General declarations, delayed entry, time-varying covariates, clustered inference, competing-risk incidence and richer diagnostics |
| Counts | Poisson/NB2, fixed offsets and a bounded `exposure(var)` option, effects, probability bins, pointwise current-row PMF/CDF delta-method SEs and logit-Wald intervals under the retained covariance, discrete equal-tail future-count intervals under fitted means/law, raw current-row Poisson/NB2 response residuals, a Poisson NB2-direction score test, unclustered and one-way cluster-robust NB2 grouped GOF tests, and selected person/cluster workflows | Household/site/multiway/survey/panel inference, hurdle/zero-inflated/mixture models, other formal NB tests and unresolved Table 20.6 output |
| Linear and nonlinear panels | Explicit-key linear panels, IV/GMM composition, Gaussian GEE and selected Poisson FE/RE/GEE workflows | Persistent declarations, dynamic-panel methods, broader GEE/FE/RE inference, model comparisons and instrument-control diagnostics |
| Treatment and clustered samples | NSW/PSID comparisons, DID, overlap diagnostics, IPW, kernel and descriptive matching compositions; household/individual clustered workflows | General estimands, matching uncertainty, survey weights/strata/PSUs, design-based variance and remaining treatment/Chapter 24 analyses |
| Nonparametric and missing data | Bounded KDE/LOWESS source adaptations and complete-case/single-fill/Rubin-summary comparisons | General smoothing/uncertainty, tied-curve parity, imputation generation/diagnostics and finite-complete-DF multiple-imputation inference |
| GMM, measurement error and Bayesian methods | Selected IV/GMM and explicit-formula NLS evidence | General systems/dynamic GMM, measurement-error corrections, simulation, Bayesian estimation and diagnostics |

Each bounded workflow must keep its assumptions and unresolved source differences
visible. No row is a claim of chapter or book completion.

## Priority 4: textbook empirical coverage

The coverage ledger's inventory is incomplete. For every selected illustration,
record the edition and locator, source and provenance, preparation, estimation
sample, design and estimand, inference/reference, post-estimation outputs,
failure case, executable entry point and independent evidence. Mark it planned,
in progress, blocked or validated. An exclusion requires an explicit product-scope
decision.

The remaining families include flexible and semiparametric regression, general
optimization and systems, dynamic panels, mixtures and unobserved heterogeneity,
hurdle/zero-inflated counts, frailty and multiple-event survival, simulation,
Bayesian/MCMC methods, measurement error and the remaining missing-data work.
The [coverage ledger](reference/microeconometrics-coverage.md) and the linked
example inventories are authoritative for which bounded slices already exist.

## Supporting work and optional breadth

- Protect ordered script effects, source-classified errors, output-failure stopping
  and stable structured results. Add language features when a real replication
  requires them.
- Improve completion, history, multiline editing and cancellation when analysis
  friction demonstrates the need.
- Use the [performance harness](performance/core-workflow.md) for measurements.
  Packaging, licensing and platform claims require named-platform evidence.
- MCP transport, remote/lazy engines, spatial modeling, regularization and extra
  integrations are optional until a required analysis demonstrates a need.
  ReadStat, libgretl and plotting adapters are implementation choices.

## Next bounded actions

The Table 20.6 discrepancy audit is complete as a bounded investigation. The
official October 2005 program and the year-2-only NB2 fit did not explain the
printed row; PR [#478](https://github.com/SaehwanPark/tabdat-explore-rs/pull/478)
records the evidence and keeps the conflict explicit in the count ledger. Resume
that investigation only if new primary-source evidence becomes available; do not
tune estimates or tolerances.

Current-row Poisson/NB2 PMF and CDF predictions support pointwise delta-method
SEs and two-sided logit-Wald intervals through `predict target, pmf(k) se`,
`predict target, cdf(k) se`, and their `interval [level(95)]` forms, including
current-cutoff variants. They use the retained covariance, including joint
beta/lnalpha covariance for NB2, and treat current predictors and fixed
offsets/exposure as known. The interval endpoints append atomically alongside
the point estimate. Log probabilities, complements and transformed delta SEs
preserve underflowed probabilities. Zero means and zero transformed variance
yield degenerate intervals; invalid levels, arithmetic, cutoffs or work limits
fail without publishing partial columns. Forty-four independent base-R
finite-difference endpoint cases and runtime/CLI contracts are recorded in the
count workflow and coverage ledger. These are pointwise parameter-uncertainty
intervals, not simultaneous bands, future-count prediction intervals or a
distributional-law guarantee; separate intervals need not sum to one or be
monotone across cutoffs.

Current-row future-count intervals are installed with
`predict target, n interval [level(95)]`. The point column remains the
conditional mean; lower and upper columns contain discrete equal-tail counts
under the fitted Poisson/NB2 law. Their endpoints use direct log-tail
comparisons, so the upper target remains distinguishable when `1-tail` rounds
to one. Current predictors and fixed offsets/exposure are held fixed, and the
retained covariance does not widen these outcome-variation intervals. Quantiles
are bounded to 0..100000, all three columns publish atomically, and cumulative
CDF-prefix work is capped at 10,000,000 terms across usable current rows.
Eighty-seven independent base-R `qpois`/`qnbinom` endpoint cases record both
neighboring tail probabilities and attained coverage; runtime tests cover
covariance invariance, offsets/exposure, missing rows, collisions, support/work
limits and empty data. This discrete predictive interval is separate from the
pointwise parameter-uncertainty interval for an individual PMF or CDF.

Raw current-row Poisson/NB2 response residuals are installed through
`predict target, residuals`. The result is current `y - exp(xb)`, using retained
mean coefficients and current predictors plus fixed offset/exposure; covariance
selection does not change it. Missing/nonfinite outcomes or model inputs yield
NULL; finite outcomes must remain exact counts in the family domain or the
prediction fails atomically. Bare `xb`, `n`, fit/sample and row order remain
unchanged. A 36-row independent base-R `glm`/`MASS::glm.nb` fixture validates
changed current outcomes, predictors and offsets. Runtime/CLI tests cover
missingness, covariance invariance, collisions, empty data and invalid current
counts. These are raw response residuals, not Pearson, deviance or standardized
residuals.

The next bounded IV slice extends HC1/one-way CR1 robust scalar AR confidence
sets from two through nine to exactly ten excluded instruments for one
endogenous regressor. Preserve included-exogenous nuisance treatment, retained
sample/groups and existing covariance corrections; keep q=11+ and other
endogenous/covariance dimensions unsupported. Degree-20 boundaries may contain
20 simple real roots and 11 confidence-set components, so extend solver limits
only with direct degree-20 and ambiguity/failure tests. Validate a 240-row,
ten-instrument synthetic reference in base R for HC1 and CR1, including numeric
determinants at 21 Chebyshev nodes, `polyroot` endpoints, and direct Wald probes.
Correct the first-stage JSON availability flag and command-results prose to
report the actual q<=10 support. Update SPEC, help, examples and the coverage
ledger; do not claim an MMA illustration from synthetic method evidence.

Do not turn the entire textbook inventory into a prerequisite for one bounded
analysis. Preserve the assumptions and unresolved source differences in the ledger.

## Delivery record and historical detail

The [coverage ledger](reference/microeconometrics-coverage.md) records current
method and illustration status. Example READMEs and linked evidence record exact
commands, numerical gates and limitations. Historical delivery logs remain in
`_workspace/` and the archived delivery ledger for provenance, but they are not a
current backlog. The immutable [previous roadmap snapshot](https://github.com/SaehwanPark/tabdat-explore-rs/blob/133cfc0/docs/TABDAT_RUST_PORT_ROADMAP.md)
preserves the former detailed roadmap when that history is needed.

Verification and development follow [AGENTS.md](../AGENTS.md) and the
[local testing guide](../CONTRIBUTING.md#impact-scoped-testing).

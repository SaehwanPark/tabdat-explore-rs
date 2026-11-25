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
| **IV/2SLS** | Exogenous/endogenous/instrument design, exact sample and rank checks, classical and appropriate robust/cluster inference, first-stage diagnostics, endogeneity and overidentification tests where identified, and weak-instrument-robust inference where required. Bounded homoskedastic tests/sets include the DATA66 `grade76` AR projection with other endogenous coefficients unrestricted; HC1/one-way CR0 Wooldridge overidentification scores, HC1/CR1 scalar AR sets for one endogenous regressor and one to seven excluded instruments, and closed robust AR projection for exactly two endogenous regressors/two excluded instruments are also installed. Broader robust projection and endogenous-subvector tests remain. Conventional first-stage F does not certify validity. |
| **Logit/probit** | Domain and separation/convergence failures, model-based/robust/cluster inference, probability and linear-index predictions, continuous average marginal effects plus a bounded numeric 0/1 probability change with explicit estimands and delta uncertainty, and retained-fit Wald tests of general linear restrictions. |
| **Poisson/NB2** | Count-domain checks, offsets/exposure, explicit variance parameterization, model-specific diagnostics, conditional means/effects and covariance conventions, retained-fit Wald tests and linear combinations of mean coefficients, and a Poisson NB2-direction score test with selected model/row/cluster score variance. Distinguish Poisson QMLE from distributional claims. |
| **Linear panels** | Explicit unit/time keys, gaps and duplicates, lag/lead/difference semantics, pooled/within/Between/FD/RE fits, model comparisons, predictions and inference. Persistent panel state and broader diagnostics remain open. |
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
| Counts | Poisson/NB2, fixed-offset compositions, effects, probability bins, a Poisson NB2-direction score test, and selected person/cluster workflows | Household/site/multiway/survey/panel inference, hurdle/zero-inflated/mixture models, other formal NB tests and unresolved Table 20.6 output |
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

Select one of these outcomes with an exact ledger-defined contract, then add its
evidence beside the workflow. Keep the choice responsive to correctness defects
and demonstrated dependencies rather than treating the list as phase gates.

1. Advance one required econometric analysis or bounded panel diagnostic. The
   [Chapter 21 workflows](../examples/mma21p1panel/README.md) provide explicit
   panel samples, designs and open comparison/diagnostic choices, while the
   [coverage ledger](reference/microeconometrics-coverage.md) records the next
   evidence needed for IV, binary, quantile and panel slices.
2. Advance the next bounded MMA count requirement from its defined source and
   sample. The [native count workflows](../examples/mma20p1count/README.md)
   identify fixed-offset, clustered/person, NB2 and remaining Table 20.6 work;
   choose one exact estimand and reference without treating a single discrepancy
   as a prerequisite for unrelated count progress.

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

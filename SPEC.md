# TabDat Explore Rust: product and capability specification

## Purpose and scope

TabDat is a terminal-first, Rust-native tool for dependable exploration and
**graduate-level microeconometric analysis**. A usable workflow goes from loading
and preparing data through specification, fitting, diagnostics, interpretation,
and reproducible reporting. Statistically equivalent native workflows matter
more than Stata syntax or output parity; production does not depend on Python or R.

The existing long-term minimum target remains the empirical analyses in Cameron
and Trivedi, *Microeconometrics: Methods and Applications* (2005), abbreviated
**MMA** below. The chapter map adds an applied guide using the same authors'
*Microeconometrics Using Stata*, **second edition (2022), volumes I and II**,
abbreviated **MUS2**. The request for chapter alignment did not specify an edition;
this map uses the current two-volume edition. Its numbering is different from MMA
and from the 2010 revised edition. [Authors' MUS2 contents][mus2-contents];
[publisher's edition information][mus2-publisher].

Neither book is covered completely. The MUS2 map is a feature-oriented starting
point, **not an inventory or validation of its individual illustrations**. MMA
examples linked below provide evidence for related methods, not proof of MUS2
chapter replication. Read the [roadmap](docs/TABDAT_RUST_PORT_ROADMAP.md) for
priorities and the [MMA coverage ledger](docs/reference/microeconometrics-coverage.md)
for example-specific scope, references and unresolved discrepancies.

## How to read the map

- **Required** describes representative software outcomes for the chapter, not
  a promise to copy every Stata command or derivation. An exact illustration,
  sample, estimand, reference and failure case must be chosen before development.
- **Available** means an executable, bounded capability relevant to the topic;
  a linked composition may use scripts/SQL rather than a general command. It is
  not a claim of full coverage or of a validated MUS2 example.
- **Pending / revisit** identifies missing outcomes and a practical trigger, not
  a scheduled release. Deferred work remains required when it is needed for the
  target analyses. Revisit shared foundations alongside the first analysis that
  needs them; do not wait for a whole chapter to be inventoried.
- Parser acceptance, executable behavior, independent numerical evidence and
  release readiness are different things. Source and tests are authoritative.
  The [previous detailed capability snapshot][previous-spec] preserves the
  pre-rewrite command-level limits and evidence; it is historical, not live.

## Shared product baseline

| Workflow | Available now | Important boundaries / next need |
| --- | --- | --- |
| Session and automation | Persistent terminal session, batch commands, scripts, help and bounded JSON/text results; macros and conditional script flow | No general Stata-language parity, persistent history, in-flight cancellation guarantee or MCP transport. Script `seed` records context; bootstrap has its own explicit seed. |
| Data | Local eager CSV/Parquet, inspection, finite summaries, missingness, tabulation, key checks, transformations, joins, reshape, SQL, save/export | DTA runtime and broad remote/lazy input are not installed. Define sample, NULL/finite, ordering, type and label semantics per command. |
| Analysis and reporting | Bounded linear and nonlinear estimators, post-estimation, current-data predictions, selected immutable named reports and SVG scatter/line overlays | Options and inference vary by model. Named reports are not restorable fits; factors/interactions, broader diagnostics and reusable comparisons remain needed. |
| Reliability | Explicit supported data/state and statistical contracts, reproducible bounded bootstrap streams and independent reference evidence for selected workflows | Supported surface only: do not infer generalized capability, platform support or release certification from a parser or a single example. |

## MUS2 chapter feature map

Each row names the chapter's analytical need, the related installed baseline,
and a concrete reason to return to the gap. **All chapters are partial or
pending**, even when a related MMA example has independent evidence.

### Volume I: Cross-Sectional and Panel Regression Methods

| Chapter | Required software features | Available / satisfied in part | Pending / revisit when |
| --- | --- | --- | --- |
| 1. Stata basics | Interactive work, scripts, help, variables, reusable results and automation | Terminal, help, batch/scripts, macros, conditional flow and selected reports | General loops, programmable routines and result manipulation are not established; revisit with the first reproducible analysis needing them, without requiring Stata syntax. |
| 2. Data management and graphics | Import, clean, join, reshape, summarize and graph data | Bounded local data preparation, summaries, key checks and SVG scatter/overlays | Native DTA, more graph families and broader data semantics remain; revisit with each source dataset and required figure. |
| 3. Linear regression basics | OLS, robust inference, diagnostics, prediction and sampling weights | OLS, HC1/one-way cluster covariance, tests, VIF, prediction and bounded one-way factor design (`i.var`, `ib(level).var`) | Interactions, transformations, specification diagnostics and survey weights remain; revisit with a selected weighted or diagnostic illustration. |
| 4. Linear regression extensions | Prediction, margins/effects, decompositions and difference-in-differences | OLS predictions and [MMA treatment comparison](examples/mma25p1treatment/README.md), including a bounded difference-in-differences composition | General predictive margins, decomposition and causal-design inference remain; revisit when a specified target and comparison require them. |
| 5. Simulation | Seeded random draws, simulation loops, Monte Carlo summaries and integration | Explicit seeded streams in bounded bootstrap; some independently checked generated-data examples | General simulation/RNG and integration workflows are not established; revisit with a specified simulation, stream and Monte Carlo error criterion. |
| 6. Linear regression with correlated errors | GLS/FGLS, clustered and mixed models, systems and survey design | Bounded positive-weight WLS/inverse-diagonal-variance GLS, cluster inference, FE/RE and Gaussian GEE; [MMA household comparison](examples/mma24p1olscluster/README.md) | General covariance GLS, systems and survey strata/weights remain; revisit with an identified correlated-error or survey illustration. |
| 7. Linear instrumental-variables regression | 2SLS, identification, weak-IV inference, endogeneity and overidentification diagnostics; systems | Bounded 2SLS, first-stage diagnostics, AR tests/confidence sets and selected endogeneity/overidentification tests; [MMA IV/panel work](examples/mma22p1panel/README.md) | Robust projected weak-IV sets, clustered overidentification and systems remain; revisit for a specific weak-IV or systems design. Conventional first-stage diagnostics do not establish instrument validity. |
| 8. Linear panel-data models: Basics | Panel keys, pooled/within/between/FD/RE fits, comparisons and predictions | Explicit-key FE, RE-GLS/ML, bounded Gaussian GEE and SQL/OLS between/FD compositions; [MMA panel workflow](examples/mma21p1panel/README.md) | Persistent panel declaration, general lags and broader diagnostics/comparisons remain; revisit with a panel needing those semantics. |
| 9. Linear panel-data models: Extensions | Panel IV, dynamic panels, instrument controls and long-panel inference | Bounded first-difference 2SLS and clustered two-step GMM [composition/workflow](examples/mma22p1panel/README.md) | General panel IV, Hausman–Taylor, Arellano–Bond and long-panel methods remain; revisit with a specified dynamic or long-panel illustration and identification checks. |
| 10. Introduction to nonlinear regression | Binary models, NLS, predicted means and effects | Bounded logit/probit and explicit-formula NLS; continuous delta effects for binary fits; [MMA NLS](examples/mma05p2nls/README.md) | Factors, discrete effects and broader NLS inference remain; revisit with a nonlinear design requiring them. |
| 11. Tests of hypotheses and model specification | Wald/LR/score tests, specification, multiple testing and power | Linear restrictions and selected model-specific LR/Wald/score tests with stated covariance/reference conventions | General nonlinear tests, multiple testing, power and randomization tests remain; revisit with a specific model and valid reference distribution. |
| 12. Bootstrap methods | Reproducible pairs/cluster resampling, uncertainty, alternative schemes and jackknife | Bounded OLS, quantile and selected Poisson pairs bootstraps with explicit seeds; additional source-specific panel compositions | Wild, studentized, residual/parametric and general-estimator bootstrap and jackknife remain; revisit when an illustration specifies resampling unit and failure policy. |
| 13. Nonlinear regression methods | Nonlinear fits, covariance choices, predictions, margins and diagnostics | NLS, binary and count models with bounded inference and predictions; [MMA count workflow](examples/mma20p1count/README.md) | General nonlinear effects, diagnostics and covariance options remain; revisit with the next model-specific end-to-end analysis. |
| 14. Flexible regression: Finite mixtures and nonparametric | Mixtures, splines, local and partially parametric regression | Bounded [KDE composition](examples/mma09p1np/README.md) and ordered-row [LOWESS](examples/mma09p1np/lowess.md) | General mixtures, splines and nonparametric regression remain; revisit with a selected illustration and bandwidth/identification and uncertainty contract. |
| 15. Quantile regression | Conditional quantiles, uncertainty, comparisons and quantile treatment effects | Bounded `qreg` and joint IID pairs bootstrap with one two-quantile contrast; [MMA household workflow](examples/mma04p2qreg/README.md) | Cluster inference, general restrictions, bands and quantile treatment effects remain; revisit with a defined estimand and reference. |

### Volume II: Nonlinear Models and Causal Inference Methods

| Chapter | Required software features | Available / satisfied in part | Pending / revisit when |
| --- | --- | --- | --- |
| 16. Nonlinear optimization methods | Reliable optimization, derivative/identification checks, custom likelihood and GMM | Estimator-specific bounded optimizers, NLS and restricted clustered linear GMM | General user-defined likelihood/nonlinear GMM is not installed; revisit with a concrete estimator, convergence and numerical-reference contract. |
| 17. Binary outcome models | Binary fits, predictions, effects, clustered inference and endogenous/fractional extensions | Logit/probit with model-based, robust or one-way cluster inference, predictions and continuous delta AMEs; [MMA binary example](examples/mma14p1binary/README.md) | Discrete effects, endogenous and fractional models and broader diagnostics remain; revisit with an identified binary illustration. |
| 18. Multinomial models | Multinomial, conditional, nested and ordered choice, effects and diagnostics | Bounded MNL, balanced conditional logit and two restricted nested-logit modes; [MMA fishing workflows](examples/mma15p1mnl/README.md) | Ordered, multinomial probit, random-parameter choice, broader choice sets and effect uncertainty remain; revisit with a specified choice design. |
| 19. Tobit and selection models | Censoring, truncation, selection, missingness and attrition workflows | Bounded lower Gaussian Tobit/truncation and Heckman two-step; [MMA selection workflow](examples/mma16p1tobit/heckman.md), missingness inspection and a [single-fill comparison](examples/mma27p1milinear/README.md) | General selection, two-part, multiple imputation and attrition inference remain; revisit with explicit selection/missingness assumptions. Single filling is not multiple imputation. |
| 20. Count-data models | Poisson/NB, offsets, diagnostics, mixtures, endogeneity and clustered inference | Poisson/NB2, fixed offsets, selected robust/cluster inference, predictions, immutable named reports and probability bins; [MMA count workflow](examples/mma20p1count/README.md) | Hurdle, zero-inflated/mixture and endogenous count models remain; revisit with a chosen count distribution and diagnostic/reference. |
| 21. Survival analysis for duration data | Survival curves, censoring, hazard models and time-varying/clustered data | Bounded KM/NA compositions and native Weibull, Gompertz and Cox fits; [MMA duration workflow](examples/mma17p4duration/README.md) | General survival declarations, delayed entry, time-varying covariates and clustered inference remain; revisit with exact event, censoring and risk-set semantics. |
| 22. Nonlinear panel models | Binary, censored and count panels, GEE and endogenous extensions | Bounded Poisson conditional FE, gamma/normal RE and exchangeable GEE; [MMA patents workflow](examples/mma23p1pannonlin/inventory.md) | Binary/Tobit/quantile panels, broader GEE and endogeneity remain; revisit with a specified panel likelihood, sample and inference. |
| 23. Parametric models for heterogeneity and endogeneity | Mixtures, nonlinear mixed effects, structural equations and selection/endogeneity | Bounded Poisson random effects and Heckman two-step address specific forms, not a general framework | Mixtures, structural equations and general joint/endogenous models remain; revisit with an identified model and independently checked likelihood. |
| 24. Randomized control trials and exogenous treatment effects | Explicit treatment estimands, randomized comparisons, adjustment, uncertainty and balance | Bounded [MMA treatment comparison](examples/mma25p1treatment/README.md) includes regression and selected weighting/diagnostic compositions | General RCT/treatment commands, multilevel and quantile effects remain; revisit with the design, estimand and randomization/inference assumptions. |
| 25. Endogenous treatment effects | IV/LATE, selection, difference-in-differences, synthetic control and regression discontinuity | Bounded 2SLS and source-specific difference-in-differences and observational treatment compositions | General causal estimators and design-specific uncertainty remain; revisit with a specified identification strategy, overlap and sensitivity checks. |
| 26. Spatial regression | Spatial keys/weights, dependence diagnostics, spatial models and inference | No spatial-specific executable workflow established; generic data/OLS only | Revisit with a specific spatial illustration, weight-matrix provenance and independent reference; ordinary cluster SEs are not spatial modeling. |
| 27. Semiparametric regression | Kernel/series, partial-linear, single-index and additive models | Bounded ordered-row LOWESS and source-specific KDE; not general semiparametric estimation | Revisit with a specific estimator, smoothing/identification and valid uncertainty requirements. |
| 28. Machine learning for prediction and inference | Validation splits, regularization, prediction evaluation and inference after learning | Basic model prediction only; no general ML workflow established | Revisit with a specified predictive or inferential illustration, leakage controls, tuning and uncertainty; prediction alone is not causal inference. |
| 29. Bayesian methods: Basics | Priors, likelihood, posterior summaries, prediction and model comparison | No Bayesian estimation workflow established | Revisit with a specified model, prior, independent posterior reference and diagnostics. |
| 30. Bayesian methods: Markov chain Monte Carlo algorithms | Seeded chains, posterior sampling, convergence/Monte Carlo diagnostics and multiple imputation | Bootstrap RNG is not an MCMC implementation; supplied-summary Rubin pooling is a [bounded composition](examples/mma27p1milinear/rubin_pooling.md), not imputation generation | Revisit with an identified posterior, sampler, reproducible chains and convergence/Monte Carlo error criteria; generate and diagnose imputations before claiming MI support. |

The MUS2 Volume I appendices cover Stata programming, Mata and optimization. They
inform reusable scripting, matrix and numerical needs, but do not require embedding
Stata or Mata. Revisit a missing primitive when a selected analysis needs it.

The MUS2 map does not replace MMA-specific requirements absent from this chapter
list, such as measurement-error corrections, richer multiple-hazard analyses and
other textbook illustrations. These remain required under the existing MMA target;
revisit each with a concrete illustration and reference in the MMA ledger.

## Correctness and acceptance

For every advertised workflow, make the input types, missingness and sample,
ordering, model/design and parameterization, covariance corrections, degrees of
freedom, inference and retained state explicit where applicable. Failures must
preserve data and files as promised. Scripts and random streams need documented
reproducibility semantics; stable machine contracts require compatibility or an
explicit versioned change. Numerical changes need independent reference checks
and explained tolerances, not silently widened ones.

The next useful core release is an interactive, dependable EDA and linear workflow,
not textbook completion. Graduate-core acceptance requires representative
end-to-end, independently validated cross-section, IV, panel and nonlinear
analyses, with interpretation and reporting. MMA textbook acceptance additionally
requires an inventory of its empirical illustrations and linked reproducible
references, with explicit scope decisions for exclusions. MUS2 chapter completion
would likewise require an illustration-level inventory and evidence; this map
alone is not that evidence.

## Further detail

- [Roadmap](docs/TABDAT_RUST_PORT_ROADMAP.md): priorities and remaining outcomes.
- [MMA coverage ledger](docs/reference/microeconometrics-coverage.md): detailed
  examples, independent references, conventions and outstanding discrepancies.
- [Documentation index](docs/README.md): user guides and command/result contracts.
- [Architecture](ARCHITECTURE.md): module boundaries and deferred integrations.
- [Migration authority](docs/migration/README.md): compatibility decisions and evidence.
- [Historical detailed capability snapshot][previous-spec]: pre-rewrite limits and
  evidence; verify against current source/tests before relying on it.

Keep this specification navigable: update the affected summary and map when a
capability changes; keep detailed numerical evidence beside the workflow or in
the coverage ledger rather than appending a new section for every PR.

[mus2-contents]: https://cameron.econ.ucdavis.edu/mus2/mus2_contents.pdf
[mus2-publisher]: https://www.stata.com/bookstore/microeconometrics-stata/
[previous-spec]: https://github.com/SaehwanPark/tabdat-explore-rs/blob/61761c7/SPEC.md

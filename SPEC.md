# TabDat Explore Rust: current product and implementation contract

**Status:** current supported behavior is summarized below; source/tests are
authoritative. Verification details are in the relevant PR threads and
[script evidence](docs/migration/phase3-scripts.md). Update this summary when
implementation changes.

## Product intent

TabDat is a terminal-first, Rust-native, Stata-inspired statistical/EDA tool.
Its value is responsive exploration, dependable statistical analysis, simple
installation, and automation with consistent typed results. The intended audience
includes **graduate-level econometrics**. The minimum long-term analytical target
is reproducible support for the empirical analyses in Cameron and Trivedi,
*Microeconometrics: Methods and Applications* (2005). This target is **not current
capability**: the installed statistical subset remains primarily linear models.

It is **not** a full Stata, Python, R, or earlier TabDat clone. Equivalent native
workflows are preferred to source-language syntax parity. Deliver complete
specify → fit → diagnose/interpret → report workflows, not syntax catalogs. The
[roadmap](docs/TABDAT_RUST_PORT_ROADMAP.md) prioritizes required graduate methods;
the [coverage ledger](docs/reference/microeconometrics-coverage.md) tracks method
gaps and textbook replication evidence.

## Implemented and deferred capabilities

The distinction below is intentional. Individual options, errors, edge cases,
and known limitations are established by current source and tests, not inferred
from a parser enum. Historical per-command implementation evidence is preserved
in [the preceding SPEC revision](https://github.com/SaehwanPark/tabdat-explore-rs/blob/333b97dd55524dbf5ce55fac510375df99e4649e/SPEC.md).

| Area | Current state | Important limit |
| --- | --- | --- |
| Binary and CLI | Batch `-c`, scripts `-f`/positional, opt-in readable `--text`, help, discovery, bounded JSON; noninteractive write/flush failures exit 1 and stop later commands | No-argument invocation starts a persistent line session; default batch/script success stays quiet, JSON unchanged; text conflicts with JSON/discovery and requires execution; startup configuration-file loading and MCP transport/server flags are uninstalled |
| Interactive shell | Persistent session, `help`/`?`, explicit readable displays for every current owned result variant, including core EDA/linear, label dictionaries and cohort/layout outcomes; capable-terminal editing, core-command/help-topic, bounded literal local `use`/`run` path, bare `use`/`append`/`join` registered-name and current-schema bare/backtick EDA variable-list Tab completion, bounded session-local history and input Ctrl-C recovery; exit/EOF | No embedded-backtick-variable/expression/option/output-path completion or shell path expansion, persistent history, multiline continuation or in-flight cancellation guarantee; plain piped/dumb input; transformations report metadata outcomes, not invented diagnostics; parsed-command breadth remains bounded |
| Scripts | Line commands, comments, multiline SQL, macros, seed, `if`/`else`/`end`, nested `run` with literal single/double-quoted script paths and source-classified CLI exits (execution 1, syntax 2, file 3); interactive runs stream readable leaf results; JSON batch `run` streams supported leaves without a container envelope and shares surrounding batch state (failed run exits 1 with located error) | Bounded 91-call compatibility evidence with five explicit seed deviations; seed records context only, not RNG state |
| Data loading and storage | Bounded local eager CSV/Parquet, literal single/double-quoted `use` paths with spaces/commas, local save/export paths | Broad remote/lazy and DTA runtime not installed |
| Data exploration | Bounded inspection including read-only current-data/retained-estimator-family and linear-sample metadata `status`, TRUE-row conditional counts, per-variable finite summaries including read-only predicates and bounded `detail` type-7 DOUBLE percentiles, one/two-way frequency tables with bounded read-only predicates and selected-sample percentage denominators, key-integrity diagnostics/guards, bounded assertions, data fingerprints and previews; shell/CLI-v1 SQL-NULL missingness and codebook profiles with types, ordering, counts and bounded examples | Not all accepted syntax executes; summarize uses per-variable finite samples, with native finite-arithmetic overflow limits |
| Data transformation | Bounded projection, conditional keep/drop TRUE-row selection, expressions, sort, recode/labels, join/append/reshape, grouped collapse and named-table SQL | Exact support is command/option-specific; do not claim general parity |
| Statistics | Pure Rust linear model/inference utilities; bounded `regress` (bare or read-only TRUE-predicate selected-cohort fitting), `ivregress 2sls` with multiple endogenous regressors, classical/HC1/one-way-cluster covariance and conventional first-stage partial F/partial R² plus Shea partial R² summaries; retained-sample `estat ar, null(...)` homoskedastic joint Anderson–Rubin tests of all endogenous coefficients and single-endogenous `estat ar, ci [level(95)]` confidence sets within documented numerical limits (including disjoint/unbounded/empty sets); retained-sample homoskedastic Sargan `estat overid` (N*R², asymptotic chi-square) and joint homoskedastic Wu–Hausman `estat endogenous` (augmented-regression F); `ttest`, `predict`, `test`, `lincom`, VIF; readable shell results for the supported regression and t-test workflows, including explicitly labelled unweighted OLS Gaussian working-model likelihood (RSS/N ML variance; not Bernoulli or cross-family LR inference) | GMM/Hansen J, robust/cluster/subset endogeneity tests, robust/cluster overid, multi-endogenous/subvector confidence regions and robust/cluster AR are not installed; scalar AR confidence sets reject numerically degenerate auxiliary variance; conventional first-stage diagnostics do not resolve weak identification; remaining estimator families are largely parser-only |
| Binary outcomes | Pure Rust unweighted model-based, robust or one-way clustered `logit` and `probit` (Newton/QR core; logit Fisher-scoring, probit Newton on the per-observation observed information), normal coefficient/95% interval inference, model-based LR inference (robust fits suppress LR p-values), robust/clustered omnibus zero-slope Wald chi-square inference with explicit rank/unavailability guards, N/(N-1) score sandwich when unclustered robust or G/(G-1) cluster-score sandwich (normal, not G-1 t), explicit convergence/identification errors, current-data `predict ..., pr`/xb, retained-design continuous `estat margins` with conditional-X delta SE/95% intervals; terminal, CLI-v1 and scripts; [composed fishing-mode §14.2 workflow](examples/mma14p1binary/README.md) independently validates all source/cohort/fitted summaries, Table14.1, native six-model Table14.2 and Figure14.1 SVG under float64/converged-information/native-layout conventions, with immutable named reports (not restorable fit caches or a literal Stata log) | No weighted/survey/multiway binary fits, factor/interactions/discrete changes, general binary lincom/test or prediction intervals; other illustration graphics remain; strict complete-finite-case exact 0/1 with both classes; absolute normalized score/step bounds can reject extreme finite designs; not an analytic separation classifier |
| Counts | Pure Rust unweighted log-link `poisson` and NB2 `nbreg` (variance mu+alpha*mu², joint beta/lnalpha inverse observed information), model-based or independent-row robust N/(N-1) sandwich, plus one-way cluster G/(G-1) (NB2 full joint scores/bread), normal95 inference, robust zero-slope Wald/rank guards, descriptive likelihood/deviance/Pearson; Poisson fixed `offset(var)` with joint finite sampling, same-offset null and current-data `predict ..., n`/xb (index includes offset), retained-X continuous delta effects holding offsets fixed, and read-only retained `estat countprob` observed/mean-fitted distributional frequency bins (no test/uncertainty). [RAND Table20.5 portions](examples/mma20p1count/README.md): Poisson1,095/NB2 1,187 independent R quantities; native original-person Poisson551/NB2 597-quantity extensions/5,908 groups; all20,186 source/identity/design/mean/index records | Partial count family: no NB2 offsets or exposure shorthand, weights/survey/multiway/few-cluster methods, factors/discrete changes, formal NB alpha=0/specification tests, general count test/lincom, current-row PMF/CDF prediction or named count reports; robust is not panel-robust. Native/author-program probability-frequency analysis is independently validated, but the materially different printed Table20.6 NB2 row remains unresolved. Exact loaded counts: Poisson <=2^53-1/N>K; NB2 <=100000/N>K+1 and lnalpha strictly (-12,12), positive final undamped joint curvature; full-rank and strict bounded convergence, not clipped-alpha fallback |
| Structured interfaces | CLI-v1 JSON core EDA including read-only session `status` and describe metadata and missingness/codebook, read-only grouped counts/native AVG mean tables (per-target SQL-NULL exclusion, not finite summaries) and bounded conditional cohort counts, count/native-mean collapse dataset outcomes (explicit aggregate-model refit, no automatic weights), duplicates/isid key guards, bounded assert/datasignature records, ordinary/percentile-detail conditional summaries and one/two-way conditional frequencies, bounded bare/conditional regress with selected coefficient covariance/parameter order and labelled unweighted OLS working likelihood, 2SLS regression results with additive first-stage diagnostic rows, model-based/robust/one-way clustered logit/probit and continuous delta-AME results with optional cluster metadata and fit-level robust/clustered omnibus Wald metadata, model-based/independent-row robust Poisson/NB2 fit/continuous-delta results with count-specific diagnostic/null/normal-inference, optional one-way group metadata and full joint NB2 dispersion/covariance metadata, retained-sample `CountProbabilityResult` distributional bins with observed/fitted fractions and expected counts (no inference/test), and predict/lincom/test (latter two linear-only), retained-design `estat vif` tables and typed homoskedastic joint AR and single-endogenous confidence-set results, generate/replace/rename/recode/encode/decode and session-local label dictionaries, SQL tables/named-cohort metadata, named-table activation, lookup joins, compatible dataset stacking and bounded wide/long reshape metadata, column-list keep/drop/select and bounded conditional keep/drop, sort/gsort metadata, save/export outcomes, bounded SVG scatter artifact metadata, session-local `estimates store/table/drop` immutable OLS/logit/probit reports with selected covariance/sample provenance and explicit normal-versus-t reference metadata, and four-form ttest results with packaged help/discovery, ordered script and batch-`run` leaf streams, discovery including conditional core command syntax, and typed transport-independent MCP boundary | No full-results JSON surface or MCP transport/server |
| Visualization | SVG-only `scatter` with up to three numeric line overlays, optional deterministic y-display jitter/title, variable labels and JSON/text artifact counts; [native Figure14.1 workflow](examples/mma14p1binary/README.md) checked against all 630 outcomes/1,890 R prediction vertices and actual SVG geometry | Explicit new `.svg` saving path and `noopen`; 10,000 candidate rows/80-character labels, float64 coordinates and fixed layout; no PNG, automatic opening/default paths, replacement or other plot families; jitter is not RNG/anonymization |
| Native capabilities | Phase 1 DuckDB/ReadStat/libgretl/plotting feasibility decisions and prototypes | ReadStat, libgretl and raster/font adapters are not production integrations; SVG uses a pure first-party generator, not the resvg prototype |

**Never equate:** command catalog entry = parsed command = implemented execution =
validated statistical behavior = release-quality feature.

## What must be correct

For any **supported** operation:

1. **Observability:** A command must be reachable through its advertised user
   surface, and its output/errors must tell the user what happened.
2. **Data semantics:** Input types, missingness, selection, ordering, labels,
   joins, and post-operation state must be deliberate and tested where relevant.
   Avoid partial mutation on recoverable failure except documented contract cases.
3. **Statistical semantics:** Sample membership and design-matrix conventions,
   estimate parameterization, covariance/inference calculations and model state
   must be specified. Verify changed statistics against an independent trusted
   reference; record meaningful tolerances and disagreements.
4. **Reproducibility:** Scripts run in documented order; supported seed values
   have explicit semantics; saved/JSON results remain stable where promised.
5. **Safety and deployability:** Production application code remains safe Rust by
   default; native resource ownership is local; core runtime needs no Python/R.
   Dependency/license/platform claims require their own evidence.

Correctness is measured at the **supported behavior boundary**, not by achieving
Python equivalence on every incidental lexical input or error string.

The shared positive-domain log-gamma coefficient is corrected; affected t/F,
chi-square and normal/probit paths retain their sample, covariance and DF
conventions but may produce more accurate last digits than older builds.
[Independent base-R numerical regressions](crates/tabdat-stats/tests/gamma_inference_contract.rs)
cover 153 log-gamma/t/F/normal values, including small DF and tiny tails; existing
chi-square and estimator reference tests remain applicable. This is an accuracy
fix, not a new estimator, arbitrary-domain gamma guarantee or textbook replication.

## Compatibility policy

| Class | Examples | Default treatment |
| --- | --- | --- |
| Essential | Statistical values/inference, estimation sample, data integrity, script-visible state | Strongly preserve or explicitly fix with high-quality evidence |
| Stable interface | Supported command options, JSON schema/exit behavior, documented `.td` constructs | Preserve or version/document intentional changes |
| Convenience | Error wording, whitespace variants, unsupported parser-only syntax | Best effort; document material differences rather than blocking delivery |
| Historical/future | Unimplemented estimators, prototype adapters, broad Python behavior | Not a prerequisite for a usable Rust milestone |

If an existing supported behavior is deliberately changed, add a focused
regression test, briefly justify its impact, and update user-facing docs. Use a
short note in [migration decisions](docs/migration/decisions.md) for migration
changes; reserve a new ADR for consequential architecture, backend, or API choices.

## Acceptance for next usable release candidate

The first useful release milestone remains **an interactive, dependable core EDA
and linear-analysis workflow**, not completion of an exhaustive language phase.
It is a foundation release, **not graduate-core or textbook-coverage acceptance**.
Target:

- Run a session interactively without scaffold output; load local CSV/Parquet
  (minimal line session implemented; advanced editing/cancellation remain deferred).
- Inspect, summarize, tabulate, and transform representative small datasets.
- Fit and inspect a supported linear model; predict/test without silent state loss.
- Produce readable terminal results and stable structured output for the advertised subset.
- Run canonical scripts with meaningful path/line errors and explicit seed policy.
- Meet representative statistical reference and data-integrity tests.
- Measure startup/command responsiveness without premature performance promises.

These are **targets**, not statements of current support. Remaining gaps and
verification must be assessed against source/tests before declaring this milestone.

## Required development beyond the core release

The graduate-core target adds model-design semantics (factors/interactions),
diagnostics/graphics and reusable reporting alongside IV/2SLS, logit/probit with
marginal effects, remaining count extensions (NB offsets/exposure shorthand beyond installed Poisson fixed offsets), linear panels, quantile
regression and genuine seeded bootstrap. Required later coverage includes choice,
selection/censoring, survival, GMM/dynamic and nonlinear panels, treatment effects,
surveys, imputation, and the remaining semiparametric, simulation/Bayesian,
mixture and measurement-error illustrations. These are **planned requirements**,
not installed features or optional substitutes for the linear workflow.

Graduate-core acceptance needs representative end-to-end, independently validated
analyses across its required families. Full textbook acceptance additionally needs
an inventory of the empirical illustrations and linked scripts/reference evidence
for each; any exclusion requires an explicit scope decision. A passing parser test,
one estimator or one example does not validate a family or chapter. See the
[prioritized roadmap](docs/TABDAT_RUST_PORT_ROADMAP.md) and
[coverage ledger](docs/reference/microeconometrics-coverage.md).

## Where to look for detail

- Source/tests: authoritative for actual implementation and regressions.
- [Documentation index](docs/README.md): user guides, command/result contracts and
  historical reference navigation.
- [Architecture](ARCHITECTURE.md): current module/crate boundaries and deferred integration.
- [Migration authority](docs/migration/README.md): pinned Python oracle, compatibility and deviations.
- [Phase 3 language evidence](docs/migration/phase3-language.md): 672-case corpus and parser/fuzz status.
- [Script evidence](docs/migration/phase3-scripts.md): pinned API replay, bounded seed contract and canonical/nested workflow tests.
- [Phase 2 evidence](docs/migration/phase2-core-contracts.md), [Phase 1 report](docs/feasibility/phase1-final-report.md): completed historical decisions.
- [Previous detailed implementation ledger](https://github.com/SaehwanPark/tabdat-explore-rs/blob/333b97dd55524dbf5ce55fac510375df99e4649e/SPEC.md): immutable historical snapshot.

Keep this document concise. Do not append one section per PR or duplicate test
logs here. If detailed evidence is important, place it alongside the relevant
subsystem/test or in the migration archive and link to it.

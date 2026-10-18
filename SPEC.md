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
| Statistics | Pure Rust linear model/inference utilities; bounded `regress` (bare or read-only TRUE-predicate selected-cohort fitting), `ivregress 2sls` with multiple endogenous regressors, classical/HC1/one-way-cluster covariance and conventional first-stage partial F/partial R² plus Shea partial R² summaries; retained-sample `estat ar, null(...)` homoskedastic joint Anderson–Rubin tests of all endogenous coefficients and single-endogenous `estat ar, ci [level(95)]` confidence sets within documented numerical limits (including disjoint/unbounded/empty sets); retained-sample homoskedastic Sargan `estat overid` (N*R², asymptotic chi-square) and joint homoskedastic Wu–Hausman `estat endogenous` (augmented-regression F); `ttest`, `predict`, `test`, `lincom`, VIF; readable shell results for the supported regression and t-test workflows | GMM/Hansen J, robust/cluster/subset endogeneity tests, robust/cluster overid, multi-endogenous/subvector confidence regions and robust/cluster AR are not installed; scalar AR confidence sets reject numerically degenerate auxiliary variance; conventional first-stage diagnostics do not resolve weak identification; remaining estimator families are largely parser-only |
| Binary outcomes | Pure Rust unweighted model-based, robust or one-way clustered `logit` and `probit` (Newton/QR core; logit Fisher-scoring, probit Newton on the per-observation observed information), normal coefficient/95% interval inference, model-based LR inference (robust fits suppress LR p-values), robust/clustered omnibus zero-slope Wald chi-square inference with explicit rank/unavailability guards, N/(N-1) score sandwich when unclustered robust or G/(G-1) cluster-score sandwich (normal, not G-1 t), explicit convergence/identification errors, current-data `predict ..., pr`/xb, retained-design continuous `estat margins` with conditional-X delta SE/95% intervals; terminal, CLI-v1 and scripts; [exact fishing-mode classical/robust logit and probit subset](examples/mma14p1binary/README.md) independently validated under converged-information conventions | No weighted/survey/multiway binary fits, factor/interactions/discrete changes, general binary lincom/test, prediction intervals or rendered figure; strict complete-finite-case exact 0/1 with both classes; absolute normalized score/step bounds can reject extreme finite designs; not an analytic separation classifier |
| Structured interfaces | CLI-v1 JSON core EDA including read-only session `status` and describe metadata and missingness/codebook, read-only grouped counts/native AVG mean tables (per-target SQL-NULL exclusion, not finite summaries) and bounded conditional cohort counts, count/native-mean collapse dataset outcomes (explicit aggregate-model refit, no automatic weights), duplicates/isid key guards, bounded assert/datasignature records, ordinary/percentile-detail conditional summaries and one/two-way conditional frequencies, bounded bare/conditional regress, 2SLS regression results with additive first-stage diagnostic rows, model-based/robust/one-way clustered logit/probit and continuous delta-AME results with optional cluster metadata and fit-level robust/clustered omnibus Wald metadata, and predict/lincom/test (latter two linear-only), retained-design `estat vif` tables and typed homoskedastic joint AR and single-endogenous confidence-set results, generate/replace/rename/recode/encode/decode and session-local label dictionaries, SQL tables/named-cohort metadata, named-table activation, lookup joins, compatible dataset stacking and bounded wide/long reshape metadata, column-list keep/drop/select and bounded conditional keep/drop, sort/gsort metadata, save/export outcomes and four-form ttest results with packaged help/discovery, ordered script and batch-`run` leaf streams, discovery including conditional core command syntax, and typed transport-independent MCP boundary | No full-results JSON surface or MCP transport/server |
| Visualization | Several command forms parse | Rendering/plot execution not installed |
| Native capabilities | Phase 1 DuckDB/ReadStat/libgretl/plotting feasibility decisions and prototypes | ReadStat, libgretl and plotting adapters are not production integrations |

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
marginal effects, Poisson/NB with offsets/exposure, linear panels, quantile
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

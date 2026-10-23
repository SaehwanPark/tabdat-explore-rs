# Command and CLI-v1 result contracts

This reference preserves the detailed supported command, sample, state and JSON
contracts formerly in the root README. Source/tests remain authoritative. Paths to
source and fixtures below are relative to the repository root; use in-session
`help <command>` for syntax. This is not full-command JSON or parser-catalog support.

- [Session status](#session-status)
- [Named fit reports and comparison](#named-fit-reports-and-comparison)
- [Summaries and percentiles](#summaries-and-percentiles)
- [Frequencies and conditional cohorts](#frequencies-and-conditional-cohorts)
- [Selected-cohort regression](#selected-cohort-regression)
- [Multinomial logit and continuous marginal effects](#multinomial-logit-and-continuous-marginal-effects)
- [Binary logit/probit and continuous marginal effects](#binary-logitprobit-and-continuous-marginal-effects)
- [Expressions and recoding](#expressions-and-recoding)
- [Grouped collapse](#grouped-collapse)
- [Assertions and fingerprints](#assertions-and-fingerprints)
- [Labels and category coding](#labels-and-category-coding)
- [SQL and named relations](#sql-and-named-relations)
- [Joins and appends](#joins-and-appends)
- [Key integrity](#key-integrity)
- [Conditional summaries](#conditional-summaries)
- [Conditional selection](#conditional-selection)
- [Reshape](#reshape)
- [Grouped counts and means](#grouped-counts-and-means)
- [Rename and column projection](#rename-and-column-projection)
- [Sorting](#sorting)
- [Persistence](#persistence)
- [VIF](#vif)
- [Linear inference fields and conventions](#linear-inference-fields-and-conventions)
- [Script exit categories](#script-exit-categories)
- [Mean comparisons](#mean-comparisons)
- [Missingness and finite samples](#missingness-and-finite-samples)
- [Column profiles](#column-profiles)

Use `--json -c "use data.csv" -c "summarize"` (or `--json -f analysis.td`)
for versioned line-delimited envelopes. Successful execution output currently
supports use, describe, count, codebook, summarize, missing, tabulate,
generate/replace, save/export, bounded SVG scatter artifact metadata,
`estimates store/table/drop` reporting snapshots,
head/tail, `estat vif/ar/overid/endogenous` and the bounded
regress/predict/lincom/test workflow plus ttest; nested scripts stream leaf results.
Existing result shapes remain unchanged. `describe` uses `schema_version: 1`,
`result_type: "DescribeResult"` and `data.dataset`, reusing the load/prediction
metadata shape: source `path`, integer `row_count`, ordered `columns` with names
and native type strings, `execution_mode` and `lazy_engine`. It reports the
current schema (including generated prediction columns) without rescanning or
changing data/model state. The source path remains the load source, not a new
output destination. `label_metadata` and `panel_metadata` remain the existing
`null` placeholders, not a full session-label/panel snapshot.

## Session status

`status` reports current dataset/named-table metadata separately from the retained
linear fit's names, intercept convention and original sample counts. It is read-only,
works before loading without initializing DuckDB, and never clones model arrays.
CLI-v1 adds `StatusResult` with `data_initialized`, nullable `dataset`, nullable
`active_table_name`, sorted `named_tables`, nullable `estimation_family` and nullable
`linear_model`. The family is the retained cache's snake-case tag (currently
`linear`, `instrumental_variables`, `logit`, `probit`, `multinomial_logit`, `poisson`,
`negative_binomial` or `panel_linear` for native within FE), not a claim of
current-data prediction eligibility. `linear_model` remains OLS/2SLS-only and is
null after other families, including FE; its existing fields are unchanged.
Within-FE sample/DF/normalization metadata is in the preceding `FixedEffectsResult`. Status is not an automatic refit/stale flag and
does not invent covariance, operation or materialization tracking. See `help status`.

## Named fit reports and comparison

`estimates store NAME` snapshots a successful current unweighted OLS, logit or
probit fit into a session-local report registry. `estimates table NAME [NAME ...]`
returns the explicitly requested columns in that order; `estimates drop NAME
[NAME ...]` removes reports only. Names are case-sensitive ASCII identifiers of
1–32 bytes. At most 32 stored reports, 64 coefficients per report, 1–8 unique
explicit names per table and a 64-name coefficient union. There is no implicit
all/latest, silent overwrite, wildcard, restore, persistence or formatting option.
For intentional reuse, explicitly drop a name first. Missing/duplicate names and
multi-drop failures validate before any registry mutation. WLS/GLS/IV, missing
estimator provenance and ambiguous duplicate coefficient names fail explicitly.

CLI-v1 adds three result types, with the existing envelope version/shapes unchanged:

- `EstimateStoreResult`: `name`, `estimator`, `stored_count`.
- `EstimateDropResult`: requested `names` and remaining `stored_count`.
- `EstimateTableResult`: `parameter_names` (first-seen union) and `models` (ordered
  immutable `EstimateSnapshot` records). Each model contains `name`, `estimator`,
  `outcome`, `predictors`, `include_intercept`, selected `covariance` label,
  nullable `cluster_variable`/`cluster_count`, `covariance_parameter_names`, full
  `covariance_matrix`, and `coefficients` with name/value/SE/statistic/p-value.
  It also contains `observation_count`, nullable binary `event_count`, full
  `sample` (fit-time retained candidate indices, total/dropped counts, weights
  and cluster groups), `reference_distribution`, nullable
  `inference_degrees_of_freedom`, `log_likelihood`/`log_likelihood_convention`,
  OLS `r_squared`/`root_mse`/`df_residual`, and binary `mcfadden_r_squared`.
  Inapplicable/undefined/nonfinite optional quantities are null, not zero.

Coefficients and covariance are copied from the selected fit, never refitted or
reinferred. Binary reference is `asymptotic_standard_normal` with null inference
DF; OLS is `student_t` with residual DF except clustered OLS uses G−1.
`df_residual` is separately retained and must not be substituted for G−1.
Covariance labels match the original fit results: linear nonrobust/robust/cluster,
binary model-based information or N/(N−1)/G/(G−1) sandwich. OLS likelihood convention
is `gaussian_working_model_ml_variance_rss_over_n`, binary is `bernoulli`.
Zero/nonfinite OLS variance has null likelihood/convention. OLS R² and binary
McFadden pseudo-R² are distinct; no cross-family or robust LR test is implied.

Terminal output aligns exact coefficient names, displaying coefficient/statistic
rows, model/sample counts and conventions, with six-decimal tab-separated values.
Missing terms are NA, not invented zero coefficients; JSON keeps full precision,
SEs/p-values/covariance/sample provenance. Different outcomes/samples are allowed
and disclosed; equal N is not proof of identical rows. Fit-time sample positions
are not current-data identities, fingerprints or automatic staleness guarantees.
Review sample/group/name metadata before sharing reports.

Store/table/drop do not change active estimation, predictions, AME, data, labels
or row order and require no backend reads/initialization. Reports survive data
reloads/edits, successful/failed refits and active-model invalidation, but do not
restore model caches. Failed refits still clear active estimation under existing
semantics. Domain `ExecutionState.stored_estimates` defaults empty and is omitted
when empty; retained linear state adds optional `estimator` provenance. Old states
remain readable with unknown estimator provenance, never guessed to be OLS.
This adds no status staleness flag or saved-dataset registry metadata.

The [native Table14.2 workflow and independent R reference](../../examples/mma14p1binary/README.md)
validate six columns/all 94 comparison quantities and all 630 prediction/sample
records; tests cover alignment, immutable ownership, samples, inference DF,
missing/zero-variance terms, limits, failure atomicity and script/terminal/JSON.
This bounded reporting subset is not full Stata estimates parity, stored-model
prediction, disk persistence or whole-illustration completion.

## Multinomial logit and continuous marginal effects

`mlogit y [xvars] [, baseoutcome(integer) noconstant]` fits unweighted,
independent-row shared-design multinomial logit. CLI-v1 `MultinomialLogitResult`
has `estimator: "mlogit"`, sorted `category_levels`, explicit `base_outcome`,
`category_counts`, complete-finite `sample`, outcome/predictor/intercept metadata,
coefficients and full `covariance_matrix`. Coefficient rows include category IDs
and names; covariance order is category-major nonbase, intercept then predictors.
The default base is the lowest retained numeric category. Codes must be exact
integers within ±(2^53−1); categorical strings/factors are not interpreted.

`covariance: "model_based_joint_fisher_information"`, correction1.0,
`reference_distribution: "asymptotic_standard_normal"` and confidence95 apply.
Likelihood/null/LR/McFadden metadata identify the intercept-only estimated-shares
null or, with `noconstant`, equal probabilities. LR df is `(J−1)*slope_count`
and its reference is asymptotic chi-square; df0 has a null p-value. Convergence
requires averaged RMS-scaled score and full undamped Newton step both≤1e-10.
Bounds:2..16 levels,N≤10000,K≤32,P≤128,N*P²≤20million. Rank, separation-like
numerical degeneracy and iteration-limit failures clear prior inference without
mutating data; no ridge/pseudoinverse fallback is presented as an identified fit.

`predict newvar, pr outcome(code)` or `xb outcome(code)` appends scalar current-X
probabilities/base-relative indices (base index0). Current outcomes are unused;
missing/nonfinite predictors yield NULL. Invalid categories/targets/schemas or
nonfinite arithmetic fail without publishing a column. `estat margins` returns
`MultinomialMarginalEffectsResult`, one continuous derivative/normal95 delta
inference row per category/predictor, using retained X and the full joint
covariance. Metadata explicitly mark conditional-on-observed-X uncertainty.
An intercept-only fit returns no predictor effects. Analytic AMEs are neither
at-means effects, discrete changes nor historical float/finite differences.

No robust/cluster/weighted/survey inference, factors, alternative-specific design,
conditional/nested models, IIA tests, prediction uncertainty or named reports/
general test/lincom is installed for this family. See the [original-source
workflow/reference audit](../../examples/mma15p1mnl/README.md) for precise
Table15.2 scope and historical numerical differences.

## Binary logit/probit and continuous marginal effects

`logit y x1 ... [, robust cluster(group) noconstant]` fits an unweighted
Bernoulli logit model.
Outcome/predictor columns must be numeric. SQL NULL/nonfinite model values are
excluded together; the retained outcome must contain both exact 0/1 classes.
Other finite outcome values are errors, not silently recoded. Parameter order is
intercept first (if included), then declared predictor columns; names are unique.
A new fit attempt clears prior estimation state even if validation/estimation fails,
without changing data/labels. `robust` and `cluster` are mutually exclusive.

CLI-v1 `BinaryLogitResult` includes `estimator: "logit"`, outcome/predictors,
intercept/N/event counts, `sample` with exact retained indices, `coefficients`,
`covariance_matrix`, log/null likelihoods, LR statistic/df/p/reference and McFadden
pseudo-R². Default covariance is `model_based_fisher_information`, with **no N-K scaling**.
Each coefficient has `name`, `value`, `standard_error`, normal `statistic`/`p_value`
and `confidence_lower`/`confidence_upper`; `confidence_level` is fixed at 95.
`reference_distribution` is `asymptotic_standard_normal`, not Student t. The LR
reference is asymptotic chi-square: estimated constant-probability null with an
intercept, otherwise probability 0.5; intercept-only fits have df 0 and null LR p.

With `, robust`, `covariance` is `robust_sandwich_n_over_n_minus_1`: the
unclustered sandwich of individual scores with converged observed-information
bread and **N/(N-1)** scaling, not OLS HC1's N/(N-K). Logit's observed and Fisher
information coincide. Coefficients, sample, likelihoods and predictions are
unchanged; normal coefficient and conditional-X delta-AME inference use the
selected covariance. `likelihood_ratio_p_value` and
`likelihood_ratio_reference_distribution` are **null** for robust fits: LR/df
and pseudo-R² remain descriptive, not robust LR inference. Survey inference
and weights remain unsupported.

With `, cluster(group)`, `covariance` is `cluster_sandwich_g_over_g_minus_1`:
sum individual scores within group before outer products and apply G/(G-1),
without the linear-regression (N-1)/(N-K) factor. The same converged-information
bread and normal coefficient/conditional-X delta-AME inference apply, not G-1
Student t. Clusters must be independent; few/imbalanced clusters can give poor
coverage. At least two retained groups are required, not an adequacy guarantee.
Fit/margins JSON includes additive `cluster: {variable, group_count}` metadata;
`cluster` is null for classical/unclustered fits. LR p/reference remain null for
clustered fits. The retained sample also owns its aligned `cluster_groups` keys.
Numeric, text and boolean IDs are supported, with NULL/nonfinite cluster IDs
jointly excluded before outcome-domain validation. Wide integer/Decimal identity
is exact; floating -0/+0 share a group. Keys are opaque within their typed source
column, not a new cross-type equality or prediction requirement. Group edits do
not redefine retained margins, and current-data prediction needs only predictors.
No multiway, random-effects, survey or small-cluster t inference is implied.

Robust/clustered fits also include additive `omnibus_wald` metadata testing
`all_non_intercept_coefficients_zero`: W = b_s' V_ss^-1 b_s using the selected
covariance, `degrees_of_freedom` q equal to the number of declared predictor
columns, and `reference_distribution: "asymptotic_chi_square"` (not F).
Without an intercept every coefficient is tested. Classical fits return
`omnibus_wald: null` and retain their existing LR inference. This is not a
conditional-mean specification test or a general binary `test`/`lincom` API.

If no slopes exist (stats API), q >= G for a clustered fit, or the slope
correlation covariance is numerically singular, the fitted model remains usable
but Wald `statistic`, `p_value` and `reference_distribution` are null.
`unavailable_reason` is respectively `no_non_intercept_coefficients`,
`insufficient_cluster_rank` or `singular_slope_covariance`; nonrepresentable
W/tail inference gives `nonfinite_wald_inference`. Requested q is retained;
no pseudoinverse, reduced-q test or small-cluster F reference is substituted.
Rank decisions use the slope correlation matrix and the existing inverse's
1e-12 pivot threshold, not raw predictor units. This guard does not guarantee
reliable inference for nearly singular covariance or few/imbalanced clusters.

Only successful finite, numerically identified fits produce results. `converged`,
`iterations` (information evaluations), `convergence_coordinate_system:
"unit_column_norm"`, `convergence_absolute_tolerance: 1e-10` and final
`max_abs_scaled_score`/`max_abs_scaled_newton_step` expose the stopping contract.
Both the score and **full undamped** normalized Newton step must meet the absolute
bound. Scaled QR/backtracking is limited to 100 evaluations. A flat likelihood or
small score alone cannot certify a finite MLE under separation. This is not an
analytic separation classifier: numerical singularity, insufficient resolution,
backtracking failure and iteration exhaustion are explicit errors. Extreme finite
models can fail these conservative numerical bounds. Training probability/weight
arrays are not added to the public CLI envelope.

After logit, `predict name, pr` appends stable sigmoid probabilities and `, xb`
appends linear indices on **current** predictor rows, without the outcome. Bare
`predict` deliberately keeps the existing xb default. Missing/nonfinite predictors
or an unrepresentable index yield NULL. The canonical staging path preserves
order, labels, row count and fit; target/type/required-column failures do not publish
an output column. `PredictionResult` retains its existing metadata-only shape.
Use head/tail/export to inspect or persist values. Residuals, intervals and other
prediction forms are not installed after logit; linear prediction semantics stay
unchanged.

`estat margins` returns `LogitMarginalEffectsResult` with `estimator: "logit"`,
`effect: "average_continuous_predictor_column_derivative"`,
`sample: "retained_estimation_design"`, N and ordered `effects` (the same normal
inference fields as coefficients, excluding the intercept). It averages
`beta_j*p*(1-p)` over the retained design. `uncertainty:
"conditional_on_observed_design_delta_method"` includes the full intercept/slopes'
covariance through the delta gradient, **not** sampling uncertainty in the
covariate distribution. Current-data changes do not redefine the estimand or
rescan/refit the model. These are not automatic discrete changes or interaction
chain rules. `lincom`/`test` after logit are explicitly unsupported rather than
silently using linear Student-t/F inference.

`probit y x1 ... [, robust cluster(group) noconstant]` shares the contract with these
family deltas. The engine fits Newton steps against the per-observation observed
information with a per-iteration Fisher fallback, and the default covariance is
`model_based_observed_information` (inverse observed information at the
converged fit from the converged QR), not the logit's Fisher matrix. Robust
fits use that observed-information inverse as sandwich bread, not Fisher or
penultimate-iterate information; their covariance label and null LR-inference
fields follow the robust contract above. CLI-v1
returns `BinaryProbitResult` (`estimator: "probit"`, same field layout, same
convergence fields) and `ProbitMarginalEffectsResult` after `estat margins`
(`estimator: "probit"`; margins average `beta_j*phi(beta_0 + beta'x)` over the
retained design). `predict ... , pr` computes probabilities with the same
registered `tabdat_gauss_phi` Gaussian function the fit used, keeping SQL
probabilities bit-identical to engine probabilities under the same index; `, xb`
appends the finite linear index, and unsupported predict forms fail with the
probit wording. With no supported binary/Poisson/NB2 fit, `estat margins` keeps the established
prior-logit message.

See the [exact fishing-mode contract and R reference](../../examples/mma14p1binary/README.md).
Weighted/survey/multiway binary inference and factors/interactions/discrete
changes remain required work where applicable. The bounded Figure14.1 SVG
workflow below does not complete broader illustration graphics.

## Bounded Poisson ML / independent-row robust QMLE

`poisson y [xvars] [, offset(var) robust cluster(var) noconstant]` executes unweighted log-link regression.
The joint complete-finite estimation sample preserves fit-time positions;
missing/nonfinite model cells are excluded before outcome-domain rejection.
Loaded outcome values must be exact nonnegative integers <=2^53-1, with DECIMAL
and wide integer identity checked before conversion. This is not lexical recovery
of digits a loader already rounded to DOUBLE. N>K/full rank and bounded numerical
convergence are required. Weights are not installed; one-way cluster selection
and inference are specified below.

`offset(var)` adds a numeric fixed log-mean term: eta=X beta+offset and mu=exp(eta).
Offset missing/nonfinite cells join the model/group exclusions BEFORE count-domain
validation. It is not a parameter or weight and does not increase K or LR DF.
The nested null retains the same offsets: with an intercept,
mu0_i=exp(offset_i)*sum(y)/sum(exp(offset)); without it, mu0_i=exp(offset_i).
A guarded log-sum-exp evaluates the intercept-only starting/null model. Extreme
finite offset designs can still fail the existing representability/convergence
policy. Exposure shorthand is not installed; NB2 fixed offsets have the distinct
joint-dispersion null contract below.

CLI-v1 adds **PoissonResult** (`estimator: "poisson"`):

- `outcome`, ordered `predictors`, `include_intercept`, `observation_count`,
  `zero_count`, float64 `count_sum`, and the exact optional `sample` record.
- Ordered `coefficients` and `covariance_parameter_names`, plus the full
  `covariance_matrix`. Each coefficient has `name`, `value`, `standard_error`,
  nullable `statistic`, `p_value`, `confidence_lower` and `confidence_upper`.
  The reference is `asymptotic_standard_normal`, `confidence_level: 95.0`.
  Zero robust variance retains the fit with unavailable z/p/CI, not point intervals.
- `covariance` is `model_based_poisson_information` (inverse Fisher, no N-K
  scaling) or `robust_sandwich_n_over_n_minus_1` (independent-row score sandwich,
  not OLS HC1, Pearson-dispersion scaling or panel/cluster inference).
- `log_likelihood` uses `log_likelihood_convention: "poisson_count_pmf"`.
  `null_log_likelihood` and `null_model` distinguish
  `intercept_only_estimated_mean` from `unit_mean_all_coefficients_zero` without
  an intercept. The latter does not silently estimate a null intercept.
  Offset fits additionally report `offset_variable`, `offset_coefficient: 1.0`,
  `linear_index: "x_beta_plus_offset"` and use `null_model` values
  `intercept_only_with_same_fixed_offset` or
  `all_coefficients_zero_with_same_fixed_offset`. No-offset shapes are unchanged.
- `likelihood_ratio`, `likelihood_ratio_df`, nullable `likelihood_ratio_p_value`
  and nullable `likelihood_ratio_reference_distribution`. Only a model-based
  positive-DF LR has chi-square reference/tail; robust LR remains descriptive.
  `omnibus_wald` is null for model-based fits; robust uses the guarded selected-V
  zero-slope chi-square object described for binary fits, with explicit
  unavailable reasons/no reduced-rank pseudoinverse. Intercept-only has no slope test.
- `mcfadden_r_squared`, `residual_deviance`, `pearson_chi_square`, `df_residual`
  and `pearson_dispersion` (Pearson/(N-K));
  `dispersion_inference: "descriptive_not_a_panel_or_specification_test"`.
  Residual DF is diagnostic, not a Student-t coefficient reference. These
  diagnostics do not rescale covariance or supply formal specification p-values.
- `converged: true`, `iterations`, `convergence_coordinate_system: "unit_column_norm"`,
  `convergence_absolute_tolerance: 1e-10`, `max_abs_scaled_score` and
  `max_abs_scaled_newton_step`. Both absolute score and FULL Newton step must
  meet that tolerance, at most100 outer Newton iterations (up to50 likelihood
  backtracking candidates each; reported iterations exclude candidates). Rank, boundary,
  iteration and nonrepresentable quantities fail rather than publish a fit;
  some extreme finite inputs may be rejected. This is not an analytic existence test.

`PoissonMarginalEffectsResult` reports `estimator: "poisson"`,
`effect: "average_continuous_predictor_column_derivative"`,
`sample: "retained_estimation_design"`,
`uncertainty: "conditional_on_observed_design_delta_method"`, selected `covariance`,
normal reference, fixed confidence level, retained N and `effects` in predictor
order with the same nullable inference fields. The continuous effect is
βj*mean(μ), full gradient βj*mean(μX)+ej*mean(μ), conditional on retained X. This
is not a discrete change, original-variable chain rule or random-X uncertainty.
Offset fits hold retained offset values fixed in both derivatives and delta
uncertainty, including if the same variable is also a predictor: no chain rule
through that offset is implied. Their effects additionally report `offset_variable`
and `offset_treatment: "fixed_at_retained_values_in_derivatives_and_delta_uncertainty"`.
Changing current offsets does not change retained effects/count-frequency reports.

`predict newvar, n` appends exp(xb) on current predictor rows; explicit/bare xb
remains the index. For offset fits BOTH xb and n use the CURRENT named offset
alongside current predictors; xb includes the offset, not just X beta. No outcome
or cluster ID is needed. Missing/nonnumeric offset columns fail without publishing
a target or clearing the retained fit. Nonfinite/unrepresentable indices and
overflowing means become NULL, finite negative exponential underflow is zero.
Means are not probabilities and are not clipped at one. Prediction publication
preserves labels/order/fit and fails atomically on invalid schema/existing targets.
`PredictionResult` is still metadata-only. Noncount families reject `n`; Poisson
rejects `pr`, residuals and auxiliary prediction options.

Domain command snapshots add `PredictKind::MeanCount` (`mean_count` on wire).
Owned executable state is `EstimationState::PoissonFit` (`family: "poisson_fit"`);
the legacy `Poisson(ModelState)`/`family: "poisson"` placeholder remains readable,
not an installed usable fit. A failed new runtime fit clears stale estimation,
not data/labels/order. No fit restoration/persistence is added. `estimates store`
continues to reject Poisson. See [exact source/R workflow](../../examples/mma20p1count/README.md)
and MIG-0017; bounded NB2 has its distinct contract below. The synthetic
[fixed-offset workflow](../../examples/poisson-offset/README.md) is independent
method evidence, not a textbook replication. Remaining exposure workflows,
multiway clustering, weighted/survey inference, current-row PMF/CDF predictions, factors/discrete
changes and general count test/lincom remain required work.

### Poisson one-way clustered inference

`cluster(var)` and `robust` are mutually exclusive. Joint finite model/cluster
selection precedes finite invalid-count rejection. Cluster type may be numeric,
text or boolean; SQL NULL/nonfinite numeric IDs are excluded, while actual loaded
empty text is a group. Wide integer/DECIMAL IDs retain identity before f64 conversion.
At least two retained groups are required. Point/design/convergence/diagnostics
and prediction behavior are unchanged; only selected covariance/inference changes.

Use converged Poisson information inverse B and S_g=sum_i_in_g X_i*(y_i-mu_i):
V=G/(G-1)*B[sum_g S_g*S_g']B. Aggregate BEFORE outer products; no HC1/Pearson
rescaling or score centering. Coefficients/delta effects stay asymptotic normal95,
not G-1 t; LR p/reference are unavailable. Zero-slope Wald retains requested q;
q>=G gives `insufficient_cluster_rank`, with no pseudoinverse/reduced q. This
correction alone does not ensure reliable few-cluster inference.

CLI-v1 retains PoissonResult/PoissonMarginalEffectsResult and adds **only on
clustered results** `covariance: "cluster_sandwich_g_over_g_minus_1"` and
`cluster: {variable, group_count, finite_sample_multiplier: G/(G-1),
independence_assumption: "across_clusters"}`. Noncluster result shapes/labels are
unchanged. Stats/owned fit/effect records add default/omitted-None cluster metadata;
old owned state remains readable, not restorable. Retained-X margins keep groups
and V across edits/drop ID; current means/xb need mean predictors only, not IDs.
Failed new runtime fits clear stale inference only; bad predictions preserve state.

[Native original-person RAND extension](../../examples/mma20p1count/README.md)
audits DTA identities against every ASCII model row;551 independent R quantities/
all20,186 source/original-row/person/year/design/prediction records and5,908 groups
are validated. This is not a printed Table20.5 column, household/site/multiway
correction, survey or FE/RE/panel-family implementation. See MIG-0019. NB2
uses its distinct full-joint clustered law below.

## Bounded jointly estimated NB2

`nbreg y [xvars] [, offset(var) robust cluster(var) noconstant]` executes unweighted log-link NB2:
mean mu=exp(X beta), variance mu+alpha*mu², alpha=exp(lnalpha), size=1/alpha.
It shares Poisson's exact loaded-integer/joint complete-finite sample semantics,
but bounds counts to **0..100000** to cap integer gamma-ratio/derivative cache
work. It requires N>K+1 (K mean parameters plus dispersion), full rank and
lnalpha strictly **(-12,12)**. These are supported numerical limits, not clipping
or an existence classifier. Boundary/Poisson-limit or nonrepresentable variance
fails rather than substituting Poisson. `lnalpha` is a reserved parameter name.

`offset(var)` adds a finite numeric fixed coefficient-one log-mean term, with
joint model/offset/group exclusions before finite count validation. It does not
increase K or LR DF. BOTH full and restricted NB likelihoods retain the same
offsets. The restricted model reestimates intercept AND dispersion when an
intercept exists; otherwise mu=exp(offset), with dispersion still reestimated.
A same-offset Poisson fit supplies starting values only: it does not replace
the restricted NB likelihood, its nuisance alpha or the full joint covariance.
No-offset fitting entry points and structured shapes remain unchanged.

CLI-v1 adds **NbregResult**, `estimator: "nb2"`. Common outcome/predictor/sample,
count, coefficient/normal95, likelihood/LR/Wald and descriptive diagnostic fields
follow the Poisson contracts above, with these explicit NB2 differences:

- `mean_parameter_count` K and `parameter_count` K+1; `coefficients` contains
  beta only. `covariance_matrix` is FULL joint (K+1)², ordered by
  `covariance_parameter_names` (intercept/slopes, then `lnalpha`). Covariance is
  `model_based_joint_observed_information` or
  `robust_sandwich_n_over_n_minus_1`. Both use the converged joint observed
  information including beta/lnalpha cross terms, not conditional GLM/Fisher V.
- `variance_function: "mu_plus_alpha_mu_squared"`,
  `dispersion_parameterization: "alpha_exp_lnalpha_size_inverse_alpha"`.
  `dispersion` contains `alpha`, `log_alpha`, both `*_standard_error`, and
  nullable `alpha_confidence_lower/upper`, `log_alpha_confidence_lower/upper`.
  Alpha SE uses alpha*SE(lnalpha); its CI exponentiates the log-alpha interval.
  Zero variance does not invent a point interval. No normal dispersion z/p is
  supplied. The existing `dispersion_boundary_test` string
  `"not_implemented_no_normal_alpha_zero_test"` continues to mark the absent
  normal-Wald test; additive `dispersion_test` reports the Poisson-boundary LR
  statistic and hypotheses. Its reference distribution and
  p-value use the asymptotic 50:50 chi-square(0)/chi-square(1) mixture only with
  model-based covariance. For robust covariance both are null and
  `unavailable_reason` is `"robust_covariance_selected"`; for clustered fits it
  is `"clustered_likelihood_not_independent"`. The test compares the
  NB2 and Poisson likelihoods on the same retained sample, design and fixed
  offset; it assumes the specified likelihood and is not a robust/cluster test.
- `log_likelihood_convention: "nb2_count_pmf"`; `null_model` is
  `intercept_only_mean_and_dispersion_reestimated` or
  `unit_mean_dispersion_reestimated`. `null_alpha` records the restricted NB
  nuisance estimate, not full alpha or a Poisson null. Positive-DF model-based
  LR is chi-square; robust LR remains descriptive with null p/reference.
  Robust selected-V Wald tests beta slopes only, never lnalpha.
  Offset fits additionally report `offset_variable`, `offset_coefficient: 1.0`,
  `linear_index: "x_beta_plus_offset"`, with `null_model` values
  `intercept_and_dispersion_reestimated_with_same_fixed_offset` or
  `all_beta_zero_dispersion_reestimated_with_same_fixed_offset`.
- Residual NB deviance is evaluated at fitted alpha. Pearson uses NB2 variance;
  `df_residual`=N-K-1 and `pearson_dispersion`=Pearson/(N-K-1), descriptive only,
  not a Student-t reference, formal test or covariance multiplier.
- `iterations_role: "full_model_outer_iterations"`; full and restricted fits
  each have at most100 outer Newton iterations, excluding candidate evaluations
  (at most20 damping attempts/50 backtracking trials each per iteration).
  Reported iterations exclude restricted-fit work. Final UNDAMPED information
  must be positive definite and both
  absolute score and full Newton step <=1e-10 in
  `convergence_coordinate_system: "unit_mean_design_column_norm_and_log_alpha_sqrt_n"`.
  `supported_maximum_count: 100000`, `supported_log_alpha_open_bounds: [-12,12]`.

**NbregMarginalEffectsResult** uses `estimator: "nb2"` and the same retained-X
continuous derivative/conditional-design normal95 fields as Poisson.
`covariance_role: "beta_block_of_full_joint_beta_lnalpha_covariance"` is explicit:
the joint gradient is `[beta_j*mean(mu*X)+e_j*mean(mu),0]`, not a reason to
replace joint V with fixed-alpha conditional covariance. Offset fits additionally
report `offset_variable` and the same fixed-retained-offset `offset_treatment`
string as Poisson effects. Offsets are fixed in derivatives/uncertainty, not a
chain rule through the offset even if its variable is also a predictor.

Current-row `predict ..., n`/xb uses the common atomic exponential/finite-index
path, no outcome required; means are not probabilities. BOTH xb and n include
the CURRENT named offset when specified (xb=X beta+offset); missing/nonnumeric
offset columns fail without clearing the retained fit or publishing a target.
Retained effects/count bins keep original fitted means/offsets/alpha despite
current-data edits. Bare predict stays xb.
Noncount fits reject n; NB2 rejects pr/residual/auxiliary forms. The existing
`PredictMeanCountRequiresPoisson` error identity is retained with wording extended
to installed count fits. Executable owned state uses
`negative_binomial_fit`; the old `negative_binomial` placeholder remains readable,
not a usable/restorable fit. Failed new fits clear stale estimation only.

Poisson/NB2 current-row `pmf(k)` and `cdf(k)` accept either a scalar integer or
the name of a current numeric cutoff variable, with exact values0..100000.
Missing cutoffs yield NULL; negative, fractional, or greater-than100000 values
fail before publishing a column. For cutoffs above99, one request may add at most
10,000,000 cutoff increments across rows with usable model inputs. Exceeding that
work bound fails before publication. These use current predictors/offsets and
retained NB2 alpha, and assume the fitted full conditional law; robust covariance
does not establish distributional calibration. The separate `estat countprob`
table retains its tail cutoff1..100.

See [source/R workflow](../../examples/mma20p1count/README.md) and MIG-0018;
[synthetic fixed-offset evidence](../../examples/nb2-offset/README.md) validates
full joint inference/nulls without claiming a new textbook illustration.
FE/RE/panel-family inference, weights/remaining exposure workflows, richer count
specifications, other formal specification tests, discrete
changes/general restrictions/named reports and full illustration outputs remain
required as applicable. Independent-row robust is not panel-robust.

### NB2 one-way full-joint clustered inference

`nbreg ..., cluster(var)` uses the same exact joint model/ID exclusions and
numeric/text/boolean identity rules as clustered Poisson, including G>=2 and
mutually exclusive robust/cluster options. Counts0..100000, lnalpha strictly
(-12,12), N>K+1, full mean rank and final positive undamped JOINT observed
information/absolute convergence remain unchanged. For the Poisson starting fit
only, detach group metadata while retaining the exact same rows/design; retain
original groups in owned NB2 state. Point/null/diagnostic results are unchanged.

With the FULL joint beta/lnalpha observed information inverse B and joint scores
s_i, let S_g=sum_i_in_g s_i. V=G/(G-1)*B[sum_g S_g*S_g']B, including cross
parameters AND within-group cross-observation terms. No conditional MASS/Fisher
bread, ignored dispersion, row-score substitution, HC1/centering/Pearson scaling.
Normal95 beta/log-alpha intervals and transformed alpha intervals use selected
joint V. No normal-Wald alpha=0 or G-1 t/few-cluster guarantee. The boundary LR
statistic is descriptive under clustered covariance and has no reported p-value.
Descriptive NB null still refits dispersion; slope LR p/reference unavailable.
Wald tests beta slopes ONLY,
not lnalpha, retains q and guards q>=G/rank without pseudoinverse/reduced q.
Residual DF remains N-K-1, not G-1.

CLI-v1 keeps NbregResult/NbregMarginalEffectsResult and adds ONLY on clustered
results `covariance: "cluster_joint_score_sandwich_g_over_g_minus_1"` and the
same four-field `cluster` metadata as Poisson. Full covariance parameter order
remains beta then lnalpha; effects use its beta block, not conditional covariance.
Noncluster shapes stay unchanged. Raw fit/effect cluster metadata defaults to and
omits None for old owned reads, not restoration. Retained-X margins keep original
groups/V across edits/drop IDs; current means/xb need mean predictors only. New
fit failures clear stale inference only; bad predictions/options preserve state.

[Native original-person NB2 extension](../../examples/mma20p1count/README.md)
validates535 synthetic/599 empirical quantities/all20,186 source/original-row/
person/year/design/mean/xb records/G5908, with numerical grouped-log-PMF score
and joint curvature checks. Not a printed Table20.5 cluster column, complete
RAND household/site/randomization correction, causal or full count/panel/book
workflow. No author data redistribution/production R/Python/native DTA dependency.
See MIG-0020. Remaining multiway/survey/few-cluster, exposure workflows/weights,
current-row probabilities/formal tests/general restrictions/named reports and richer count
requirements are not removed.

## Retained count probability-frequency reports

`estat countprob [, max(10)]` is read-only after an owned Poisson/NB2 fit,
including model-based, independent-row robust or one-way clustered covariance.
`max(m)` is a decimal integer1..100, default10. It reports exact counts0..m-1
and an inclusive >=m tail, not exactly m. Uses ORIGINAL retained outcomes/fitted
means/NB2 alpha and sample, not current rows, displayed rounded parameters or a
refit. Edits/drop y/X/IDs do not change it; failures preserve data/model state.
With no usable count fit, including legacy placeholders, runtime reports
`EstatCountProbRequiresPriorCount`; invalid direct cutoff/numerical inputs report
`EstatCountProbFailed`. CLI batch execution errors retain the established
`ExecutionError` envelope and meaningful message; parser errors retain exit2.

CLI-v1 adds **CountProbabilityResult** with:

- `estimator: "poisson"|"nb2"`, `outcome`, `observation_count`, `tail_from`;
- `distribution: {family: "poisson"}` or `{family: "nb2", alpha: ...}`;
- exact original `sample`, `sample_role: "retained_estimation_sample"`;
- ordered `bins`: `lower_bound`, inclusive `upper_bound` (null ONLY for final
  tail), `observed_count`, `observed_fraction`, `fitted_fraction`, `expected_count`;
- `probability_role: "mean_of_individual_plugin_conditional_probabilities"`,
  `covariance_role: "does_not_establish_distribution"`,
  `uncertainty: "not_computed"`, `goodness_of_fit_test: "not_performed"`.

Fitted fraction is mean_i Pr(Y_i in bin | X_i, fitted parameters), NOT the
probability at mean(mu). Expected count is the sum of individual probabilities.
Full conditional Poisson/NB2 laws are assumptions; robust/cluster QMLE covariance
does not validate them or adjust these probabilities. No binomial SEs, intervals,
p-values, formal Pearson goodness-of-fit test or guarantees of calibrated
probabilities follow. Terminal labels the same descriptive distinction.

Log PMFs preserve higher-count mass when p0 underflows. Small tails use direct
gamma/beta calculations, with complements ONLY in well-conditioned branches;
integer log-factorials/products avoid general log-gamma approximation and
large-argument cancellation. Bounded convergence and finite[0,1]/partition
checks fail explicitly, with no clipping/renormalization. New numerics reuse
bounded beta CF machinery without changing legacy beta/F value paths.

[Native author-program analysis](../../examples/mma20p1count/count_probabilities.td)
and [reference/checker](../../examples/mma20p1count/README.md) validate2090
independent R scalar probabilities/37 selected stress cases,95 empirical
aggregate/model quantities and every source/design/mean/index field of20186
records; a separate native scalar-helper probe matches all444092 individual
PMF/tail values. The probe is development evidence, NOT an installed rowwise
probability export. Actual JSON/text/piped exports agree and20 corruption probes
reject wrong laws/averaging/tails/samples/frequencies/probabilities/claims.

The printed Table20.6 NB2 row differs materially from the authors' November2006
program/reference (zero share30.9% vs31.82679994%); it remains UNRESOLVED, not
forced into a tolerance or declared literal whole-table replication. No book/data/
program redistribution. Current-row PMF/CDF `predict`, uncertainty/formal tests,
named count reports and other required count/core/book scope remain open.
See MIG-0021.

## Summaries and percentiles

A summary uses `schema_version: 1`,
`result_type: "SummarizeResult"` and `data.rows`, with each row containing
`variable`, `count`, `mean`, `std_dev`, `minimum` and `maximum`.

Summary rows preserve requested order and duplicate variables; omitted variables
select numeric columns in schema order. `count` is the finite, non-NULL count and
`std_dev` is sample standard deviation (undefined below two observations).
Missing/undefined and non-finite values serialize as `null`, consistent with
CLI-v1 preview output. Extrema reuse preview scalar encoding: integers remain
JSON numbers, decimals are exact scaled strings. Consumers needing large-integer
precision should use a lossless JSON number parser. This is an additive result
surface, not full-command JSON support.

`summarize [varlist] [if <predicate>], detail` additionally reports percentiles
1/5/10/25/50/75/90/95/99 on that same per-variable finite sample. It uses type-7
linear interpolation after DOUBLE conversion, not an exact integer/decimal quantile
or a universal Stata percentile convention. Terminal output pairs each ordinary
summary with a percentile table. CLI-v1 adds `SummarizeDetailResult` with
`data.quantile_method: "linear_type_7"` and ordered rows containing `summary`
(the unchanged ordinary fields) and `percentiles` (`percentile`, nullable `value`).
Empty samples have undefined percentiles; singleton percentiles equal the one value.
Ordinary `SummarizeResult` is unchanged. No higher moments, weights, extreme-value
lists, configurable percentiles, grouped detail or other option combinations are
installed. Native finite-arithmetic limits remain. See `help summarize` and the
independent NumPy fixture `tests/fixtures/summary-detail-reference.json`.

## Frequencies and conditional cohorts

Bounded one/two-way `tabulate` uses `result_type: "TabulateResult"` with
`data.headers` and `data.rows`. Header/cell order, value labels, counts and
percentages are supplied by execution without recomputation. Missing categories
are excluded by default and included last with `missing`; their cells serialize
as `null`. Two-way row/column percentage headers remain explicit. This does not
install the broader parser-only tabulation/aggregation forms.

Read-only `count if expr` returns existing `CountResult.data.row_count` for all
TRUE predicate rows, including rows with unrelated NULL/nonfinite values. FALSE
and UNKNOWN are excluded; empty selections return zero. It preserves the active
dataset, labels, named relation and cached fit/sample. Use it to check cohort size
before `summarize` (per-variable finite samples) or `tabulate` (category exclusions).
Bare count is unchanged; variable lists/options, grouped conditional counts and
function/logical-expression breadth remain unsupported. See `help count`.

Read-only `tabulate r [c] if expr` and equivalent single-variable
`rows()`/`columns()` forms select only TRUE predicate rows. FALSE/NULL rows are
excluded; category SQL NULLs are then excluded unless `missing`. All percentage
denominators use that selected category sample, not original dataset totals or a
cached estimation sample. Arithmetic/comparisons/boolean columns/NULL reuse the
bounded predicate subset; functions/logical-expression breadth, multiple keys
per dimension and value aggregation remain unsupported. Active data, labels,
named relations and cached fits/samples are preserved. Selected counts and
percentages are checked against a development-only pandas reference with exact
counts and absolute percentage tolerance `1e-10` (no production Python dependency).

## Selected-cohort regression

`regress y xvars if expr` now fits the existing bounded linear estimators
(OLS/WLS/GLS with nonrobust/HC1/cluster covariance and intercept conventions)
on the TRUE predicate cohort without filtering active rows. FALSE/UNKNOWN rows
are excluded before finite outcome/predictor, cluster and weight validation, and
the estimation sample keeps original active-row indices, total rows and
exclusion counts alongside aligned weights/clusters. Active data, schema, order,
labels, named relations and source stay read-only; a successful fit replaces the
cached model, while a failed attempted regression clears it under existing
policy. `lincom`/`test`/`estat vif` consume the retained selected design and
`predict` remains current-data prediction with fitted coefficients. Predicates
reuse the bounded TRUE subset (no numeric/string truthiness, functions or
logical breadth); temporary filtering and automatic refits stay unsupported.
Coefficients, inference, VIF and predictions are checked against a
development-only statsmodels selected-row reference with exact sample/cluster/
weight membership (no production Python dependency).

## Expressions and recoding

`generate` and `replace` return additive `GenerateResult`/`ReplaceResult`
envelopes with `data.dataset` post-transform metadata. Source path, total row
count and ordered native schema are supplied, not changed-cell counts or overflow
diagnostics. Inspect values with `head`/`tail`. Numeric arithmetic generate and
numeric/string/NULL replace with boolean/NULL conditions use existing runtime
rules; function calls remain unsupported. Failed transforms preserve prior state.
Successful transforms retain the cached fit without refitting; replace removes
the target value-label attachment under existing rules. Metadata label/panel
fields remain null placeholders. See `help generate`/`help replace`.

`recode` returns additive `RecodeResult` with owned `data.dataset` metadata,
not rules, changed-cell counts or overflow diagnostics. Exactly one `generate()`
(new outputs in requested order) or `replace` target is required. Conditional
rules match original values in order (first match, no cascade); numeric ranges
are inclusive, `min`/`max` unbounded, and `else` is the final fallback. Unmatched
values remain unless an `else` rule supplies a value. `missing`/`nonmissing` mean
SQL NULL/non-NULL, not empty strings or general non-finiteness. Native output
schema/promotion is reported; text output uses text conversion. Replacing removes
selected value-label attachments, retains variable labels/definitions and cached
fits without refitting. Metadata label/panel fields remain null placeholders.
Parquet retains stored empty strings; CSV reload may turn empty fields into NULL.
See `help recode` for bounded eager syntax and failure limits.

## Grouped collapse

`collapse count x y, by(group)` and `collapse mean x y, by(group)` emit
`CollapseResult` with owned `data.dataset` metadata in CLI-v1 JSON and a readable
terminal row/column outcome. They replace active data with ordered group keys and
native BIGINT `count_x`/`count_y` or DOUBLE `mean_x`/`mean_y` columns. Counts exclude
SQL NULL independently per variable, unlike `by group: count` row totals; stored
empty/nonfinite values count as non-NULL. Means require numeric targets and use native
AVG with per-target SQL-NULL exclusion, retaining NaN/infinities, **not** finite-sample
`summarize`. Inspect values with `head`; its existing JSON policy maps both nonfinite
means and all-NULL means to null, without sample counts/diagnostics. NULL keys group
natively, NULL-last; empty input remains empty. Group labels survive, aggregate-source
labels prune, dictionaries remain and active named relations synchronize. Cached fits/
samples are not refitted: use aggregate columns for explicit refit/prediction before
persistence. Group aggregates are not individual observations or automatic model weights.
Repeated/colliding names can fail staged schema validation without repair. Sum/min/max
JSON remains unsupported; see `help collapse` for syntax and state/output limitations.

## Assertions and fingerprints

Read-only `assert` and `datasignature` now report in terminal/CLI-v1 JSON, enabling
script preconditions and current-data fingerprint records before analysis/persistence.
Successful `AssertResult` owns `data.checked`/`failed`; FALSE or SQL-NULL predicates
fail (empty data passes), and failure is an execution error, not a successful
assertion envelope. `DatasignatureResult` owns `data.algorithm`, `signature`,
`row_count` and `column_count`: the existing versioned SHA-256 protocol covers
public names/canonical types, row order and values, excluding labels/source paths.
Both inspect current data without changing named ownership, labels or cached models/
samples. No functions/logical breadth/Boolean literal keywords, `if`/options or `by:`
execution is added. Fingerprints are not file-byte checksums, authentication or
cross-package signatures; CSV re-inference may change them. See `help assert` and
`help datasignature` for bounded syntax and output-failure rules.

## Labels and category coding

Interactive `encode`/`decode` now report owned row/column outcomes, and `label`
responses show the action/message plus readable variable labels, attachments and
value-label sets. Names/text are quoted/control-safe. Mapping keys are explicitly
`integer`, `number` (exact source spelling) or `text`, retaining owned order and
wide integer precision without converting categories. Empty/absent dictionaries
are explicit. Filtered `label list names` still includes all variable labels.
These displays do not rescan data, change dictionaries/models or change JSON;
labels are not automatic factors or general metadata-aware formatting for other
commands. Nested terminal scripts stream the same leaf displays and can recover
from located errors; output failure stops later effects without rolling back
completed encoding, labels or files.

Bounded session-local `label variable`, `define`, `values`, `list` and `drop`
return `LabelResult` with `data.action`, `message` and owned `metadata` (null
when empty). Metadata contains ordered `variable_labels` and `attachments`
`[name, text-or-set-name]` pairs, and `value_sets` with `name` and ordered
`mappings` pairs `[key, display_text]`. Keys retain tagged `{kind, payload}`
forms: `integer` with an i64 JSON number (use lossless consumer parsing),
`number` with an exact numeric spelling string, or `text` with a string.
Names/attachments/sets are producer-normalized; mappings retain definition
order. `label list names` filters sets and matching attachments but still
includes all variable labels. Defining/replacing/clearing/dropping labels
changes metadata only, not data or cached models. Other dataset envelopes
retain their existing null label/panel placeholders. Labels are session-local:
a new `use` clears them, and CSV/Parquet save/export does not archive them.
`label save`/`label use` are not installed. See `help label`.

`encode`/`decode` report additive `EncodeResult`/`DecodeResult` with owned
`data.dataset` metadata, not code/mapping/count diagnostics. Encode maps sorted
non-NULL strings to 1..K, preserving SQL NULL and treating stored empty strings
as categories. Default/custom label dictionaries are inspected with `label list`;
custom names replace existing definitions, preserving other attachments to them.
Decode requires attached integer-key mappings and numeric input, uses native
SQL numeric equality (no explicit fractional truncation), and emits SQL NULL
for unknown/missing codes. Cross-type wide comparisons follow native coercion,
not a lossless DOUBLE/i64 equality guarantee.
Source columns/order/count/path and cached fits remain; generated variable labels
inherit their source label. Native schema is reported (small nonempty mappings
use INTEGER; empty/all-NULL encode uses BIGINT). Other dataset label/panel placeholders remain
null. Numeric codes alone do not install factor/dummy/ordinal model semantics;
choose a supported explicit design/transform. Dictionary persistence remains
uninstalled; CSV reload may turn empty strings into NULL. See `help encode`/`decode`.

## SQL and named relations

`sql SELECT/WITH ...` returns `TableResult` with owned `data.headers`/`rows`
using the existing scalar policy; headers can repeat, empty queries retain
headers, nonfinite numbers encode as null, and decimals stay exact strings.
Rows are not automatically truncated: specify SQL `LIMIT` when appropriate.
`sql ... into name` returns `SqlCreateResult`, and `use name` for a registered
named table returns `ActivateResult`, each with owned `data.table_name` and
`dataset`. Their source path is the named-table identifier, not the original
file. Publication/activation retains the cached fit/sample without refitting,
clears session labels, and reports native schema/count; transformations update
an active named table under existing rules. Dataset label/panel fields stay null.
Use explicit `ORDER BY` plus tie keys for reproducible query/cohort ordering.
Existing statement validation only checks the leading SELECT/WITH word; this
is not a SQL sandbox or general read-only guarantee. Native named publication
uses multiple steps, with no general atomic rollback guarantee. Ordinary
validation/binder/unsupported-cell failures are tested; do not infer whole-SQL,
composite-type, filesystem, lazy/remote or security support. See `help sql`.

## Joins and appends

Bounded `join lookup on keys [, how=inner|left suffix(...)]` returns
`JoinResult` with owned `data.dataset` metadata, not keys/matching/cardinality
or affected-row diagnostics. Join a registered named table after SQL setup and
primary activation. Inner is the default; left retains unmatched rows with
NULL right fields. SQL NULL keys do not match each other. Duplicate matches
multiply rows, grouped by prior active-row order and stored right-match order.
Right keys are omitted; nonkeys append with existing suffix/collision rules;
final native names/types/count/source are reported, without a wide cross-type
matching or case-collision naming guarantee. Labels/definitions/attachments for
retained left variables and the original cached fit/sample remain; explicitly
refit on enriched rows. Active named tables are updated under existing rules.
Dataset label/panel fields stay null placeholders; no cardinality validation,
right-label propagation, by/lazy/remote execution or general merge parity is
implied. See `help join`.

`append followup` returns `AppendResult` with owned `data.dataset` metadata,
not input paths or inferred added-row diagnostics. It requires the same column
names and equal canonical native types (including numeric width/decimal scale),
aligns right columns by name into left schema order, and preserves duplicates,
NULLs and left-then-stored-right row order. No schema union, automatic coercion,
sorting or deduplication is installed. Left labels/definitions/attachments and
the original cached fit/sample remain; explicitly refit combined data.
Append detaches active named-table ownership while retaining its source
identifier; registered prior named relations are unchanged, and `use name`
restores the preserved snapshot, not the stacked data. Metadata label/panel
fields stay null placeholders. Empty/self/repeated appends and ordinary
validation/type/file/output boundaries are checked; native publication faults
are not injected and no general atomicity guarantee is added. See `help append`.

## Key integrity

Read-only `duplicates [report] [varlist]` and `isid varlist [, missok]` now
report readable terminal diagnostics and additive CLI-v1 `DuplicatesResult` /
`IsidResult` fields, connecting key diagnosis/validation to lookup preparation.
Counters are owned current-data aggregates; `unique_groups` means all distinct
combinations (including duplicate/NULL groups), not only singletons. Default
isid rejects any SQL-NULL component; missok permits unique NULL-containing
combinations but never duplicates. Failures remain execution errors, with no
success-shaped or invented failure counters. Native grouping and stored-empty/
nonfinite distinctions are unchanged. Data/labels/named/model state is read-only;
no duplicate repair, key coercion, persistent cardinality rule or lazy loading
is installed. See `help duplicates` and `help isid` for exact fields/limits.

## Conditional summaries

Read-only `summarize [varlist] if <predicate>` (no options) now supports the
terminal and CLI-v1 `SummarizeResult` fields. It selects TRUE rows, then uses
per-variable finite/non-NULL samples for all aggregates; counts can differ,
not a fabricated global/model sample. Exact extrema, variable order/duplicates,
empty/singleton policy and native finite-overflow limits remain unchanged.
Predicates reuse the bounded keep-if rules, including explicit SQL-NULL and
native direct nonfinite versus safe-arithmetic handling. Data/labels/named state
and cached fits are untouched. One bare `detail` flag adds the percentile report
above. Functions/logical-expression breadth, other summary options and conditional
grouped summaries remain uninstalled. See `help summarize` and the
development-only NumPy/SciPy conditional-summary reference.

## Conditional selection

Conditional `keep if <predicate>` / `drop if <predicate>` now execute in the
CLI/REPL and CLI-v1 JSON, using existing KeepResult/DropResult dataset metadata.
Keep retains TRUE rows; drop removes TRUE rows and keeps FALSE/UNKNOWN, so they
partition input. Schema/types/source/relative order and labels survive; active
named tables synchronize. Cached inference/sample stay original until explicit
refit, while prediction uses current rows. Predicates reuse bounded comparisons,
arithmetic, Boolean columns, strings and SQL-NULL equality; no numeric truthiness,
functions, logical conjunction/disjunction or extra options. Direct NaN/infinity
comparisons remain native, not finite-sample filtering; safe arithmetic can turn
nonfinite/failed arithmetic into NULL/UNKNOWN. Stored empty strings are not NULL.
No predicate/affected/overflow counts or general native atomicity are implied;
column keep/drop remains unchanged. See `help keep`, `help drop`, MIG-0009.

## Reshape

Bounded `reshape long|wide` returns additive `ReshapeResult` with owned
`data.dataset` metadata, not inferred direction/stubs/changed-row fields.
Long requires complete `stub_suffix` columns and emits identifier/j/stub columns
in source-row then requested-stub/schema first-appearance suffix order; native
UNION promotion and NULL values are retained. Wide casts non-NULL j keys to text,
orders generated suffixes lexically, and uses native MAX for duplicate cells,
not uniqueness rejection or a general lossless pivot; NULL j rows contribute no
cell. Unrelated columns are omitted. Source identity is retained, named ownership
detaches (registered snapshots survive), and only surviving label names/attachments
remain; definitions and cached original fit/sample persist until explicit refit.
Missing old predictors fail rather than silently fitting a new model. Native
publication is not generally atomic; no panel/factor state is inferred. Empty
long works with valid stubs; empty/all-NULL-j wide fails. See `help reshape`.

## Grouped counts and means

`by groups: count` and `by groups: summarize [numeric_vars]` support CLI-v1 JSON
`ByResult` with owned `data.headers`/`rows`. Raw group keys preserve requested
order/duplicates; native group order is ascending NULLS LAST. SQL-NULL keys form
groups and stored empty strings remain distinct; no value-label substitution is
performed. Counts use COUNT(*) over all group rows. Means use existing native
AVG: SQL NULLs are excluded independently per target, but NaN/infinity remain.
Explicit targets preserve order/duplicates; defaults are numeric non-group columns
in schema order. This returns `mean_<variable>` columns only, not ordinary finite
`SummarizeResult` counts/SD/extrema or inferred sample diagnostics.

Empty input has headers and no groups. Existing scalar policy applies: exact
scaled decimals/wide integers and nonfinite cells encoded as null. Distinct native
NaN and SQL NULL keys can encode identically without merging groups; an all-NULL
mean and a native nonfinite mean can both encode null without implying the same
sample. Terminal output preserves native NaN/infinity explicitly. For x=[3,infinity],
the native mean is infinity (JSON null), while finite summarize reports N=1, mean=3.
A development-only standard-library `statistics.mean` fixture records per-variable
non-NULL input indices and tagged IEEE outputs; raw results are checked before JSON
encoding. No overflow/stability guarantee or numerical/backend change is added.

Inspection/reporting retains data/schema/source/order, labels, named state and
cached fits/samples. Conditional/options-bearing grouped summaries, other `by`
children and nested by remain unsupported. See `help by` for limits and reference
regeneration; this is not general by execution or full grouped descriptive statistics.

## Rename and column projection

Single-column `rename old new` returns additive `RenameResult` with the same
owned `data.dataset` post-operation metadata: native schema position/type, source
and total row count, not old/new-name or affected-cell fields. Values/NULLs and
row order are retained, including empty data and backtick-quoted names. Existing
targets (including the old name) and native case-insensitive collisions fail
without changing prior data/session state. Variable labels/value-label attachments
move to the new name; definitions remain. Cached fits retain their original names
and sample without refitting: post-estimation uses that fit, but prediction needs
its original predictor columns in current data. Normalize before fitting or
explicitly refit after renaming a predictor. Label/panel fields remain null
placeholders. See `help rename`.

Column-list `keep`, `drop` and `select` return additive `KeepResult`, `DropResult`
and `SelectResult` envelopes with the same `data.dataset` post-projection metadata.
`keep`/`select` use request order; `drop` preserves remaining schema order. Rows,
NULLs, native types and source are retained. Repeated keep/select names get native
unique aliases (e.g. `x`, `x_1`); repeated drop names remove a column once. Empty
data is supported; unknown columns or dropping every column fail without changes.
Labels/attachments for absent names are pruned, definitions and cached fits retained
without refitting. Prediction requires current predictors; post-estimation retains
the owned fit. Label/panel JSON fields remain null placeholders, not full snapshots.
Conditional row filtering also executes; see [conditional selection](#conditional-selection).
See `help keep`/`help drop`/`help select`.

## Sorting

`sort` (ascending) and `gsort` (per-key directions) return additive `SortResult`
and `GsortResult` envelopes with the same owned `data.dataset` metadata. Ordered
previews/predictions and exports use the actual current rows; result metadata
supplies no key list, changed-row count or cell values. NULLs are last for either
direction and full-key ties preserve their prior active-row order. Native schema,
source, row count, labels and the cached fit are retained without refitting;
retained sample indices still refer to the original fit, not reordered data.
Prediction evaluates current rows. Unknown keys fail without changes; unsupported
options/conditions fail before execution. Bounded local eager runtime only, not
lazy/remote or `by:` execution. Label/panel fields remain null placeholders.
See `help sort`/`help gsort`.

## Persistence

`save` (local Parquet) and `export` (local CSV) return additive `SaveResult` and
`ExportResult` envelopes with requested `data.path` and output-oriented
`data.dataset`, reusing the existing metadata shape. The reported dataset path
is the destination, not the unchanged active source shown by `describe`.
Prediction columns are included; data/model state and label/panel placeholder
policy are unchanged. Existing targets require `replace`; refused overwrite
preserves their bytes. Paths with spaces/commas can be quoted.

Persistence is **not atomic file publication**: a native failure may leave
partial output or alter a replaced file, and created parent directories may
remain. CSV is text interchange, not a general type/label archive. Result I/O
failure stops later commands but does not roll back files already written by
the completed command. Roundtrips and these failure boundaries are tested;
no general filesystem rollback or platform guarantee is claimed. See `help save`,
`help export` and MIG-0008.

## SVG scatter artifacts

`scatter y x, saving("figure.svg") noopen [lines(p1 p2 p3) jitter(number) title("text")]`
returns additive `PlotArtifactResult`, not a dataset mutation or fitted model.
The CLI-v1 envelope shape/version and all existing results are unchanged. Batch,
script and typed execution share publication; readable text reports the same
counts and conventions. A result is returned only after complete no-clobber
publication. Example data payload for eight candidate rows and one line:

```json
{
  "path": "figure.svg",
  "format": "svg",
  "width": 900,
  "height": 660,
  "candidate_rows": 8,
  "x_variable": "x",
  "scatter": {"variable": "y", "point_count": 6, "omitted_rows": 2},
  "lines": [{"variable": "fitted", "point_count": 7, "omitted_rows": 1}],
  "jitter_y": 0.03,
  "jitter_convention": "deterministic_row_index_pattern_outcome_units",
  "coordinate_precision": "float64"
}
```

These are current-data display samples, not estimation exclusions. Each series
independently selects finite x/its own y; `point_count` means scatter marks or
finite line vertices, not stroke-segment count. Line vertices sort by ascending
x then original candidate-row index; finite x with missing/nonfinite line y
breaks the path. Missing/nonfinite x is omitted because it has no sortable
location. A scatter needs one finite pair and every requested line needs at
least one adjacent finite pair; a requested series is not silently discarded.
There are at most three unique line columns and 10,000 candidate rows.

Numeric coordinates are float64 conversions, not lossless wide integer/decimal
identities. The fixed 900x660 layout uses all displayed values with nominal 5%
padding (constant ranges expand; unrepresentable ranges fail), rounded tick text
and viewer-resolved fonts. Line values, including OLS fits, are not clipped to
[0,1]. Variable labels supply axis/legend labels; labels/title have 1–80 XML-valid
characters, without general long-label/font-metric layout guarantees.

`jitter_y` is a finite nonnegative y-only outcome-unit amplitude, default zero.
For original candidate row i (zero-based, before exclusions/sorting), the display
offset is amplitude * ((73*i mod 251)/125 - 1). This stateless ordinal pattern
is not RNG, Stata percentage jitter, seed state, uncertainty or anonymization.
The SVG embeds raw float64 x/y and display y for scatter marks, plus sorted raw
line vertices/original row indices and coordinate domains. It therefore contains
underlying data even when jittered; review before sharing. No arbitrary SVG,
external asset/script/font, browser opening or raster API is accepted.

Schema/data/render failures leave data, labels, row order and retained model
unchanged and create no artifact. Rendering finishes before filesystem effects;
parents may then be created. A fully written/closed sibling stage is published
by atomic no-clobber hard link, requiring a local hard-link-capable filesystem.
Existing files, directories and symlinks are never replaced. Owned-stage cleanup
is best effort on filesystem/crash failure; this is not crash durability or
atomic publication for the separate CSV/Parquet commands above. Unsupported
extensions/opening/default paths and `replace` are rejected with actionable
errors. Native state/file/concurrency tests and the exact
[Figure14.1 reference/geometry workflow](../../examples/mma14p1binary/README.md)
cover this subset; PNG/other plot families and broad platform/font parity remain
uninstalled/unverified.

## VIF

`estat vif` returns an additive `TableResult` envelope with `data.headers`
(`Variable`, `VIF`) and ordered `data.rows`, including the supplied `mean_vif`
aggregate when available. It preserves the owned table rather than identifying
results by header/label heuristics. A predictor may itself be named `mean_vif`;
rows and labels are not deduplicated. Unavailable/non-finite cells encode as
`null`; finite zero remains numeric. Generic table serialization alone does not admit
commands: SQL has separate JSON admission, while other `estat` forms remain unsupported.

VIF uses **unweighted auxiliary regressions on the retained parent-fit sample**,
with the parent's intercept convention, including after WLS/GLS. It does not
rescan/refit changed current data or modify the dataset/model. Parent fit counts,
estimator and sample metadata remain in the preceding `RegressionResult`.
A lone no-intercept predictor now has VIF/mean-VIF 1: its zero-column auxiliary model
predicts zero, giving SSR = uncentered TSS and R² = 0. The library's all-zero design has
undefined R²; invalid/nonfinite input and general regression rank/zero-parameter rules
still reject as before. Other auxiliary failures remain execution errors. This corrects
the former valid-fit diagnostic failure, so subsequent script/batch commands can proceed
(MIG-0011). JSON null does not distinguish unavailable/infinite cells. See `help estat`.

## One-way within fixed-effects results

Bounded unweighted `xtreg y xvars, fe id(unit) time(year) [robust | cluster(unit)]`
adds CLI-v1 **FixedEffectsResult**, `estimator: "within_fixed_effects"`. RE remains
unsupported. Explicit distinct keys are mandatory for execution; both absent
remain parseable legacy syntax. All raw keys are audited (nonmissing/finite,
unique pair, exact numeric integer time) before joint-finite y/X selection and
unit centering. Source order is preserved; gaps/unbalanced units/singletons are
allowed. Invalid rank/DF or runtime fit failures clear stale estimation without
data mutation. No new production dependencies or persistent panel state.

Fields:
- `outcome`, ordered `predictors`, `entity_variable`, `time_variable`,
  `entity_data_type`, `observation_count`, `group_count`, `slope_count`,
  `absorbed_intercepts` (=G), `df_residual` (=N-G-p), `inference_df` and
  `reference_distribution: "student_t"`.
- `covariance`: `nonrobust` or `cluster(unit)`, selected slope-only
  `covariance_matrix`, explicit `covariance_parameter_names`, ordered
  `coefficients` (`name`, `value`, `standard_error`, `statistic`, `p_value`).
  Classical variance uses RSS/(N-G-p), t/F DF N-G-p. Unit clustering uses grouped
  within scores, multiplier G/(G-1)*(N-1)/(N-p), and t/F DF G-1. `robust` aliases
  unit clustering here, unlike OLS HC1. Optional `cluster` has `variable`,
  `group_count`, `finite_sample_multiplier`,
  `correction_convention: "plm_arellano_sss_nested_unit_effects"` and
  `independence_assumption: "across_units"`.
- `within_r_squared`, `conditional_r_squared` (level fits including unit effects,
  **not Stata overall/between R²**), `root_mse`, `residual_sum_of_squares`,
  `within_total_sum_of_squares`; `minimum_group_size`, `maximum_group_size`,
  `singleton_groups`. Undefined/nonfinite optional measures encode as null.
- `common_intercept` is only mean(y)-mean(X)beta on the estimation sample,
  `common_intercept_inference: "normalization_point_only"`,
  `unit_effect_normalization: "estimation_observation_weighted_zero_mean"`.
  No covariance, standard errors or intervals are reported for the common
  normalization point or individual unit-intercept coefficients, and no
  implicit intercept restriction is imposed.
- `unit_effects_f_test`, when present, tests that all unit intercepts are equal
  by comparing pooled and within RSS on the same estimation sample. It contains
  `f_statistic`, `df_num` (=G-1), `df_denom` (=N-G-p), `p_value`, and the
  classical pooled-vs-within reference label. The exact finite-sample F
  calibration assumes independent homoskedastic normal errors. It is withheld
  for unit-clustered fits; `unit_effects_f_test_unavailable_reason` records why.
- `sample` has `total_observations` and `dropped_observations`, not row vectors.
  No adjusted within R², panel/Gaussian likelihood, unit vectors or named FE
  snapshots are emitted. Existing non-FE result shapes remain unchanged.

`LincomResult`/`TestResult` operate on retained slopes only, using the inference
DF above even though fit residual DF remains N-G-p. Predictions append current-row
DOUBLE columns atomically: `xb` (also bare predict) is common intercept+X beta;
`xbu` is retained total unit intercept+X beta; `u` is retained normalized deviation;
`residuals` is current y minus xbu. Conditional modes require matching ID name and
exact logical type, not time; xb needs no ID, u needs no y/X. Missing/nonfinite/
unestimated conditional IDs or required numeric cells yield NULL, never zero
effects; overflowing calculations also yield NULL. Retained centers/effects do
not change with current data. Missing columns, type mismatch, existing target
or unsupported modes/intervals preserve fit/data and publish nothing. Existing
`PredictionResult` metadata is unchanged; use export/head to access values.
See [audited Table21.2 Within portion](../../examples/mma21p1panel/README.md).

## One-way random-effects results

`xtreg y xvars, re id(unit) time(year)` returns the existing
`RandomEffectsResult` for balanced Swamy–Arora GLS. Unbalanced retained counts
return the distinct `UnbalancedRandomEffectsGlsResult`; unbalanced Gaussian
marginal ML remains `UnbalancedRandomEffectsMlResult`. Balanced result fields and
the strict balanced statistics entry points retain their existing contracts.

Unbalanced Swamy–Arora uses a T_i-weighted between regression and the trace
correction
`(sum_i T_i ebar_i² - (G-p-1) sigma_e²) /
(N-tr((Z'WZ)^-1 Z'W²Z))`, where W=diag(T_i) and Z has an intercept plus unit
means. It transforms each retained unit with its own theta_i;
the intercept column is transformed by1-theta_i. The final model-based
quasi-demeaned covariance uses RSS/(N-K), K=p+1. Negative raw unit variance is
clipped to zero while preserving the raw estimate. Singleton units can be kept
when the within/between designs and residual degrees of freedom identify the fit.
Inference is asymptotic normal for coefficients and chi-square for the slope
Wald test; no t/F or few-cluster guarantee is implied.

The unbalanced JSON data includes the existing outcome/predictor/fit/covariance
fields and adds `balanced: false`, `minimum_group_size`, `maximum_group_size`,
`theta_at_minimum_group_size`, `theta_min`, and `theta_max`. Common `group_size`
and `theta` are null. `variance_component_method` is
`unbalanced_swamy_arora`; the `estimator` string is
`unbalanced_swamy_arora_random_effects_gls`. `sample` keeps total and dropped
counts without exposing row vectors. For `cluster(unit)` or its RE-specific
`robust` alias, `covariance` is `cluster(unit)`, covariance convention is
`unit_grouped_quasi_demeaned_scores`, and additive cluster metadata reports the
G/(G-1)*(N-1)/(N-K) multiplier and independence across units. The cluster option
changes covariance/inference, not variance components or coefficients.

For model-based balanced and unbalanced Gaussian marginal ML, `ml.variance_component_inference`
reports observed-information uncertainty for `sigma_e_squared` and, at an interior
fit, `sigma_u_squared`. Each component has an estimate, variance-scale
delta-method `standard_error`, `log_scale_standard_error`, and exponentiated
95% log-Wald `confidence_interval`. The object identifies its asymptotic normal
reference and method. At the zero-unit-variance boundary, `sigma_u_squared` is
null and `sigma_u_squared_unavailable_reason` explains why regular normal
inference is withheld; residual variance uncertainty remains available. These
are model-based interior intervals, not profile intervals or boundary tests.
Interior unit-cluster inference is the separate extension below.

These semantics apply to supported one-way random-intercept workflows with
explicit audited ID/time keys; they do not install random-effects unit
predictions, general panel state, weights, time-invariant predictors, general
restrictions, boundary sandwich covariance or general profile intervals.
The documented MOM sensitivity
cohort is an altered-sample method check, not a published textbook result.

### Interior RE-ML unit-score covariance

`xtreg ..., mle id(unit) time(year) [robust | cluster(unit)]` supports regular
positive-unit-variance fits for balanced/unequal retained counts. `robust` aliases
clustering on the declared unit. Existing result types, samples, point fits,
likelihood values, coordinate names and predictions are retained. The selected
covariance uses full joint marginal-likelihood unit scores with inverse joint
observed-information bread, multiplier G/(G-1), no N-K scale. Balanced log-variance
coordinates are log(a), log(a+T*u); unequal-count coordinates are log(a), log(u).

The JSON `covariance` is `cluster(unit)` and `covariance_convention` is
`unit_grouped_joint_ml_scores_observed_bread`. `covariance_matrix` is the selected
beta block; `ml.joint_covariance` is the full selected sandwich in the existing
`joint_parameter_names` order. `ml.cluster` adds `group_count` and
`finite_sample_multiplier`. Top-level `cluster` retains variable/count/independence
metadata with `correction_convention:
"gaussian_marginal_ml_full_joint_scores_g_over_g_minus_1"`; GLS keeps its different
correction and convention. Coefficients/intervals remain normal; the slope Wald
reference is asymptotic chi-square(p). Singular slope restrictions yield null
statistic/p with the existing reason field, retaining positive marginal SEs.

`ml.variance_component_inference.method` is
`unit_cluster_joint_score_log_wald`, using the selected full covariance for delta
SEs and exponentiated normal/log-Wald 95% limits. Under within-unit misspecification,
these concern working covariance parameters, not automatically true error-component
variances. Independent units, many-group asymptotics, mean/exogeneity assumptions
and a regular interior optimum remain necessary; there is no few-cluster guarantee.

`ml.likelihood_ratio_p_value` is null with an explicit unavailable reason. The
model-based `ml.unit_variance_test` is omitted and
`ml.unit_variance_test_unavailable_reason` explains the withheld mixture reference.
The raw slope LR/likelihoods remain descriptive; a restricted-null boundary is
allowed. These optional new fields default to absent when reading older/classical
serializations, preserving their prior shape. Non-unit clustering and full-model
zero-unit-variance boundary sandwich requests fail explicitly under the existing
failed-refit model-clearing/data-preservation rule. No silent model-based fallback,
G-1 t/F, survey/multiway inference, or robust boundary reference is supplied.

## Linear inference fields and conventions

Linear JSON results reuse the supplied statistics without recalculation:

- `RegressionResult`: `outcome`, ordered `predictors`, `estimator`, `covariance`,
  `observation_count`, `include_intercept`, `df_residual`, `r_squared`,
  `adjusted_r_squared`, `root_mse`, `sample` and ordered `coefficients`.
  Additive `covariance_matrix` contains the selected coefficient covariance,
  with explicit `covariance_parameter_names` row/column order. It is not a
  design or row-level data matrix. Additive `log_likelihood` is reported only
  for unweighted OLS (including robust/cluster covariance), with
  `log_likelihood_convention: "gaussian_working_model_ml_variance_rss_over_n"`.
  This Gaussian working-density value uses ML variance RSS/N, not the RSS/(N-K)
  variance used for coefficient covariance; it is not Bernoulli likelihood,
  robust LR inference or a cross-family LR test. WLS/GLS/2SLS and zero/nonfinite
  residual variance return null likelihood/convention, rather than inventing
  an unsupported weighted/IV or degenerate convention.
  `sample` contains `total_observations`/`dropped_observations`, or is null when
  unavailable. Coefficients contain `name`, `value`, `standard_error`, `statistic`
  and two-sided `p_value`. Training design matrices/row arrays are deliberately omitted.
  For `ivregress 2sls`, `first_stage` is additive and contains `test` (the
  conventional homoskedastic excluded-instrument partial F),
  `weak_iv_robust_inference_available: false` (these first-stage/coefficient
  summaries do not contain weak-IV-robust inference),
  `joint_ar_postestimation_available` (true for classical/HC1/one-way-cluster fits;
  actual null-specific auxiliary rank/variance checks still apply),
  `scalar_ar_confidence_set_available` (true only for nonrobust/single-endogenous
  fits; numerical/rank/variance validation can still fail),
  `coefficient_inference = "conventional_t_not_weak_iv_robust"`, and ordered
  `rows` with endogenous name,
  excluded instruments, N, numerator/denominator DF, ordinary `partial_r_squared`,
  `shea_partial_r_squared`, F and p-value. Shea's statistic accounts for the other
  endogenous regressors and is independent of the requested covariance on an
  identical sample; with one endogenous regressor it equals ordinary partial R²
  when that diagnostic is defined. Missing/non-finite diagnostic scalars are null.
  OLS results omit `first_stage`.
- `WuHausmanResult` (`estat endogenous` after a nonrobust 2SLS fit):
  `test: "wu_hausman_augmented_regression_f"`, `covariance: "homoskedastic"`,
  `null_scope: "all_endogenous_joint"`,
  `null_hypothesis: "all_declared_endogenous_regressors_exogenous"`,
  `weak_iv_robust: false`,
  `assumptions: "valid_instruments_homoskedastic_errors_maintained_model_strong_identification_exact_F_requires_conditional_normality"`,
  ordered `endogenous` and `excluded_instruments`, `observation_count`,
  `numerator_df`, `denominator_df`, `statistic` and upper-tail `p_value`.
  The joint null sets all full-instrument first-stage innovation coefficients to
  zero in an OLS augmentation of the original model. F uses m and N-K-m DF,
  where m is the endogenous count and K the original parameter count including
  intercept. This is not Durbin LM or a covariance-difference/robust Hausman test.
  Just-identification is allowed. Valid instruments, homoskedasticity, maintained
  specification and conventional strong-identification regularity are required;
  exact F additionally requires conditional normality. Endogenous-subset forms
  are unsupported; robust/cluster parents return the distinct result below.
  Auxiliary predictors are rescaled,
  not redefined; all rows/designs come from the retained problem. No 2SLS refit,
  current-data scan or model/data mutation occurs. Rank/DF/finite/variance failures
  and relative innovation/augmented-residual norm at most 1e-12 (see `help estat`)
  are errors, not a claim of exogeneity. Failure to reject does not prove
  exogeneity or instrument validity. Text/scripts expose the same owned result.
- `RobustEndogeneityResult` (`estat endogenous` after HC1/one-way clustered 2SLS):
  `test: "endogeneity_augmented_regression_robust_wald"`,
  `covariance: "robust_hc1"` or `"cluster(variable)"`,
  `reference_distribution: "asymptotic_chi_square"`,
  `null_scope: "all_endogenous_joint"`,
  `null_hypothesis: "all_declared_endogenous_regressors_exogenous"`,
  `weak_iv_robust: false`,
  `assumptions: "valid_instruments_maintained_linear_model_strong_identification_independent_rows_or_independent_clusters_many_clusters_required_no_few_cluster_guarantee"`,
  ordered `endogenous`/`excluded_instruments`, `observation_count`,
  `auxiliary_parameter_count`, `auxiliary_residual_df`, `covariance_correction`,
  nullable `cluster_group_count`, `degrees_of_freedom`, `statistic`, `p_value`.
  Tests all m first-stage innovation coefficients jointly in retained-design
  augmented OLS, with HC1 N/(N-Kaux) or CR1 G/(G-1)*(N-1)/(N-Kaux), Kaux=K+m.
  Auxiliary residual DF is correction metadata, not an F denominator; Wald uses
  asymptotic chi-square(m). This is not an exact F, robust score/GMM C, subset,
  few-cluster or weak-IV procedure. Valid instruments and conventional strong
  identification are required. Uses retained cluster IDs, not current columns;
  no data/model mutation or 2SLS refit. Classical innovation/variance/rank guards
  still apply; singular/rank-limited Wald covariance fails, not a reduced test.
  Text/scripts expose the same result. Failure to reject proves neither
  exogeneity nor instrument validity.
- `RobustOveridScoreResult` (`estat overid` after independent-row robust 2SLS):
  `test: "wooldridge_residualized_instrument_overid_score"`,
  `covariance: "independent_row_uncentered_score_opg"`,
  `covariance_correction: 1.0`,
  `reference_distribution: "asymptotic_chi_square"`, `weak_iv_robust: false`,
  `null_hypothesis: "overidentifying_restrictions_hold_given_maintained_model"`,
  `assumptions: "independent_rows_correctly_specified_maintained_model_valid_instruments_under_null_strong_identification"`,
  ordered `endogenous`/`excluded_instruments`, `observation_count`,
  `instrument_parameter_count` L, `structural_parameter_count` K,
  `degrees_of_freedom` q=L-K, `statistic` and upper-tail `p_value`.
  Regresses ones without an intercept on retained structural residuals times
  q independent directions from (I-P_Xhat)Z, where Xhat=P_Z X. N-RSS is computed
  as the fitted projection norm. The score OPG is uncorrected, not parent HC1.
  The chi-square(q) reference assumes independent rows and conventional strong
  identification; this is not Hansen J, classical Sargan, a finite F or a
  weak-IV/survey/cluster procedure. No fit/data mutation, row selection or
  structural refit occurs. Just-identification, singular Z/Xhat/score spaces,
  zero/nonfinite structural residuals and auxiliary DF failures are errors;
  rank is not silently reduced. Cluster parents remain unsupported. Rejection
  may mean invalid instruments or misspecification; failing to reject proves
  neither validity nor identification. Text/scripts expose the same result.
- `SarganResult` (`estat overid` after an overidentified nonrobust 2SLS fit):
  `test: "sargan_n_r_squared"`, `covariance: "homoskedastic"`,
  `reference_distribution: "asymptotic_chi_square"`, `weak_iv_robust: false`,
  `null_hypothesis: "overidentifying_restrictions_hold_given_maintained_model"`,
  `assumptions: "homoskedastic_errors_correctly_specified_maintained_model_strong_identification"`,
  ordered `endogenous` and `excluded_instruments`, `observation_count`,
  `degrees_of_freedom`, `statistic` and upper-tail `p_value`. Statistic is
  N*(u'P_Z u)/(u'u), with retained structural residuals and full instrument design;
  uncentered N*R² also defines noconstant behavior. DF = excluded instruments minus
  endogenous regressors. No N-K correction, Basmann, robust Hansen J or exact F.
  Strong-identification asymptotics, homoskedasticity and the correctly specified
  maintained model are required. Just-identified/cluster fits, zero or nonfinite
  residual variance and auxiliary rank/DF/numerical failures are errors. Robust
  parents return the distinct score result above, not Sargan.
  No sample rescan, 2SLS refit or state mutation occurs. Rejection can reflect
  invalid instruments or model misspecification; failure to reject does not prove
  instrument validity or resolve weak identification. The external AER example
  in `examples/cigarettes_iv/` records independent numerical and sample evidence.
- `AndersonRubinResult` (`estat ar, null(...)` after a nonrobust 2SLS fit):
  `test: "anderson_rubin_joint"`, `covariance: "homoskedastic"`,
  `null_scope: "all_endogenous_joint"`, `weak_iv_robust: true`, `assumptions`,
  ordered `endogenous`/`null_values`, `excluded_instruments`, `observation_count`,
  `numerator_df`, `denominator_df`, `statistic` and upper-tail `p_value`.
  Null values specify every endogenous coefficient in fit order. Included exogenous
  coefficients are unrestricted nuisance parameters. F uses q excluded instruments
  and N − rank[W,Z] residual DF, not 2SLS residual DF. Valid instruments and
  homoskedasticity are required; exact finite-sample F also requires conditional
  normality. Retained fit data are used without refitting 2SLS or mutating state.
  Wrong-sized/nonfinite nulls and undefined auxiliary variance/rank/DF are errors.
  Robust/cluster parent fits instead produce the distinct result below. No confidence-set or endogenous-subvector inference
  is implied; ordinary coefficient/tests remain conventional. The preceding
  first-stage availability flag remains false for those conventional summaries;
  the separate joint-AR availability flag advertises this post-estimation command.
- `RobustAndersonRubinResult` (`estat ar, null(...)` after an HC1 or one-way
  clustered 2SLS fit): `test: "anderson_rubin_joint_robust_wald"`,
  `covariance: "robust_hc1"` or `"cluster(variable)"`,
  `reference_distribution: "asymptotic_chi_square"`,
  `null_scope: "all_endogenous_joint"`, `weak_iv_robust: true`, `assumptions`,
  ordered `endogenous`/`null_values`, `excluded_instruments`, `observation_count`,
  `auxiliary_parameter_count`, `auxiliary_residual_df`, `covariance_correction`,
  optional `cluster_group_count` (null for HC1), `degrees_of_freedom`, `statistic`
  and upper-tail `p_value`. No denominator DF/F reference is reported. Auxiliary
  OLS regresses y-X_endog*b0 on included exogenous variables and all excluded
  instruments, then tests the complete excluded block with unrestricted
  auxiliary residual-score sandwich covariance. K_aux includes any intercept;
  HC1 correction is N/(N-K_aux), CR1 is G/(G-1)*(N-1)/(N-K_aux). The full Wald
  statistic uses asymptotic chi-square(q), with q equal to the excluded-instrument
  count. Assumes valid instruments, a correctly specified linear model and
  independent rows (HC1) or independent clusters and many-group asymptotics (CR1).
  This is neither an exact finite-sample test nor a few-cluster guarantee.
  Rank-limited restriction covariance (including q >= G), invalid nulls and
  auxiliary design/variance/DF failures are errors; restrictions are never dropped
  to manufacture a p-value. Retained sample/design/groups are used without 2SLS
  refits, current-column reads or state mutation. Robust confidence sets and
  less-conservative endogenous-subvector tests remain unsupported. The synthetic
  `examples/robust-ar/` workflow records independent numerical/sample evidence.
- `AndersonRubinConfidenceSet` (`estat ar, ci [level(95)]`): scalar nonrejection
  set for exactly one endogenous coefficient, leaving included-exogenous
  coefficients unrestricted. Fields: `test: "anderson_rubin_scalar_confidence_set"`,
  `covariance: "homoskedastic"`, `weak_iv_robust: true`,
  `endogenous_nuisance_policy: "none_single_endogenous_only"`, `assumptions`,
  `endogenous`, `excluded_instruments`, `confidence_level` (percent),
  `observation_count`, `numerator_df`, `denominator_df`, `critical_f`, `shape`
  and ordered `intervals` with `lower`/`upper` bounds. Finite endpoints are closed;
  **null lower/upper means negative/positive infinity**, not an unavailable
  estimate. Shapes: `bounded`, `disjoint`, `all_real`, `empty`, `lower_ray`
  ((-inf, upper]), `upper_ray` ([lower, +inf)), `singleton` ([b,b]). Empty sets
  have no intervals; all-real sets have one interval with both bounds null.
  Analytic quadratic inversion uses no finite grid/truncation. Confidence levels
  are finite and strictly between 0 and 100. Multi-endogenous and robust/cluster
  fits are rejected. Rank/DF/nonfinite/range failures and auxiliary residual
  variance that cannot remain positive for all nulls are errors, not empty sets
  (see `help estat` for the relative 1e-12 innovation-norm policy). Assumptions and
  retained sample/state semantics match the homoskedastic joint test; use the
  projected result below to leave other endogenous coefficients unrestricted.
- `ProjectedAndersonRubinConfidenceSet`
  (`estat ar, ci coefficient(name) [level(95)]` after a nonrobust multi-endogenous
  2SLS fit): analytic projection of the joint homoskedastic AR nonrejection region
  onto one named endogenous coefficient, leaving all other endogenous coefficients
  unrestricted. It reports `test: "anderson_rubin_projected_scalar_confidence_set"`,
  `coverage: "at_least_confidence_level_by_projection"`, the selected `coefficient`,
  ordered `nuisance_endogenous`, instruments, level, N, F numerator/denominator DF,
  critical F, `shape`, `intervals`, and
  `all_real_conservative_fallback`. The joint-region critical value uses
  F(q, N − rank[W,Z]); the projection is at least nominal and can be conservative.
  If unit-diagonal-scaled nuisance curvature has a Cholesky pivot at or below
  1e-12, the all-real set is returned conservatively and the fallback field is true.
  Valid instruments and
  homoskedastic errors are required; exact finite-sample coverage also requires
  conditional normality. Robust/cluster covariance, multidimensional regions and
  the less-conservative subvector AR test remain unsupported. The command uses the
  retained fit sample/design and does not mutate state.
- `PredictionResult`: `dataset`, using the existing load metadata shape; follow
  with head/tail to inspect values. Current prediction kinds are xb/residuals.
- `LincomResult`: `label`, `estimate`, `standard_error`, `statistic`, `p_value`,
  `ci_lower`, `ci_upper`, `ci_level`, `df_residual` (supplied Student-t inference).
- `TestResult`: ordered `constraints`, `statistic`, upper-tail `p_value`, `df`,
  `df_residual`, `is_chi2`; false means F with numerator/residual denominator DF.

The current least-squares engine reports fit residual DF N − P. Nonrobust/HC1
coefficient and `lincom` Student-t inference and `test` F denominators use N − P;
clustered coefficient/`lincom` inference and `test` F denominators instead use
G − 1, where G is the number of retained clusters. Thus `RegressionResult`'s
`df_residual` remains N − P while clustered `LincomResult`/`TestResult` report
G − 1 in their inference `df_residual` fields.
`robust` means HC1 (N/(N − P)); `cluster(name)` uses covariance correction
G/(G − 1) × (N − 1)/(N − P). Reporting changes none of these conventions.

Finite statistics are JSON numbers; unavailable/non-finite values are null,
not zero. IV uses complete cases over the outcome, included and endogenous
regressors, excluded instruments, and requested cluster variable. Its first-stage
partial F is a relevance diagnostic, not weak-IV-robust inference; ordinary 2SLS
coefficient tests do not resolve weak identification. Samples and inference remain
estimator/mode-specific; reference tests cover named fixtures, not every accepted mode.

## Script exit categories

Script execution (`-f` or a positional script) exits 0 on success, 1 for command
execution/preflight or recursive inclusion failures, 2 for invalid script syntax,
directives, macros or control structure, and 3 for script-loading failures or a
missing data file. Nested failures retain the child source location and category.
Classification comes from the error source, not words such as “not found” or
“expected” in diagnostics. These deliberate corrections replace the old
message-based heuristic; JSON envelope fields and diagnostic text are unchanged.
Batch `-c` exit behavior is unchanged.

## Mean comparisons

Interactive mean comparisons now display the existing bounded `ttest` results:
`ttest x == 0` (one-sample), `ttest x == y` (paired), `ttest x, by(group)`
(pooled equal-variance), and `ttest x, by(group) welch` (unequal-variance;
`unequal` is an alias). `help ttest`/`?ttest` and `--json --help-topic ttest`
explain these forms, samples and limits; discovery links the packaged topic.
Output includes ordered sample summaries, fixed 95% Student-t intervals, the
mean/difference null, t statistic, reference DF (fractional for Welch), and
left-/two-sided/right-tail probabilities. Paired samples use joint complete
finite observations; independent samples exclude invalid tested values and
missing group keys. Independent group labels are rendered, then sorted as strings;
the contrast is the first label minus the second, not input-row or numeric key order.

Independent difference rows show `NA` for N because the owned legacy field is
not a sample count; real group counts and reference DF are shown separately.
Undefined statistics display `NA`, not invented zeros. Rendering does not change
calculations, samples, data, or the saved regression. Nested interactive `run`
streams these results; default quiet batch output is unchanged, with readable `--text` as a separate opt-in.

CLI-v1 JSON also supports all four bounded `ttest` forms. `TtestResult.data`
contains `varname1`, nullable `varname2`/`by_variable`, `is_paired`, `is_welch`,
`null_value`, `reference_distribution: "student_t"`, `ci_level: 95.0`,
`group1_stats`, nullable `group2_stats`/`combined_stats`, `difference_stats`,
`t_statistic`, `degrees_of_freedom`, and `p_left`/`p_two`/`p_right`.
Each summary contains `name`, `count`, `mean`, `standard_error`,
`standard_deviation`, `ci_lower`, and `ci_upper`. Counts are integer observation
counts, except independent difference `count` is null; its legacy DF/zero field
is deliberately not exposed as N. Sorted rendered-label order defines group1 minus
group2; paired tests use varname1 minus varname2. One-sample differences
report the tested mean, not the mean minus the null. Group intervals use their
own sample DF; the tested interval and probabilities use the supplied reference
DF, including fractional Welch DF. Missing summaries and non-finite statistics
serialize as null; finite zero remains a number. Rendering and serialization do
not change the estimation sample or saved model.

## Missingness and finite samples

`missing [variables]` now reports owned missingness rows in the shell and
CLI-v1 JSON. `MissingResult.data` contains `missing_definition: "sql_null"` and
ordered `rows` with `variable`, `data_type`, `total`, `missing`, `nonmissing`,
and `missing_percent`. Default order follows the loaded schema; explicit order
and repeated variables are preserved. Counts remain integer JSON numbers and
finite percentages are unrounded; non-finite percentages serialize as null.
An empty relation keeps the existing zero-count/0% convention.

Missingness means SQL NULL **after loading**. Stored NaN/infinity, empty strings
and sentinel codes are nonmissing; CSV loading can first turn empty fields into
NULL. `summarize`, `ttest`, and `regress` exclude non-finite numeric inputs
separately. Each summary variable uses one finite, non-NULL sample for count,
mean, sample SD (N-1), and extrema. All-invalid samples report count zero and
undefined statistics; singleton SD is undefined. Filtering preserves exact
integer/decimal extrema without casting the aggregate input. Very large finite
values can still overflow native aggregates; broader numerical stability is not
claimed. This deliberately corrects formerly poisoned/failed non-finite summaries
without changing result field shapes or finite-data behavior. Reporting
is read-only, retains types/data/model state, and streams in nested scripts.
Current runtime support is eager local CSV/Parquet through DuckDB, not broader
lazy/remote modes.

## Column profiles

`codebook [variables]` now reports owned column profiles in the shell and
CLI-v1 JSON. `CodebookResult.data` contains `missing_definition: "sql_null"`,
`example_limit: 3`, `examples_nonfinite_encoding: "null"`, and ordered `rows`
with `variable`, `data_type`, `nonmissing`, `missing`, `distinct`, and `examples`.
Schema/requested order and duplicates are preserved. Distinct counts use native
non-NULL semantics; examples are up to three non-NULL scan values and can repeat.
Empty/all-NULL profiles have empty example arrays.

JSON examples reuse preview scalar encoding: integer numbers, exact decimal
strings, base64-prefixed bytes, and null for non-finite floats. Thus an encoded
null example can be a selected non-NULL NaN/infinity, not SQL missingness. The
terminal displays those non-finite examples directly. Types/counts, scans, data,
and model state are unchanged; unsupported composite example conversions still
fail meaningfully rather than produce a false profile.

## BootstrapOlsResult

`bootstrap y xvars, seed(integer) [reps(200) noconstant]` returns a read-only
`BootstrapOlsResult` with estimator `iid_pairs_bootstrap_ols`, current complete
sample metadata, outcome/predictor/intercept information, N/B, exact decimal-text
u64 seed and versioned RNG identifier. Coefficients carry original estimate,
bootstrap mean, bias, bootstrap SE and percentile95 bounds; covariance parameter
names and the full B-1 covariance matrix use the same order. Interval method is
`percentile_r_type7`; no analytic t/normal p-values or installed-fit replacement.
A failed replicate produces an indexed execution error, not partial uncertainty.
The pinned Python command catalogs remain unchanged as historical fixtures;
bootstrap is an explicit native read-only catalog/effects/help extension.

## ClusterBootstrapOlsResult

`bootstrap y xvars, cluster(unit) seed(integer) [reps(200) noconstant]` returns
flat CLI-v1 `ClusterBootstrapOlsResult`, estimator `cluster_pairs_bootstrap_ols`.
Common coefficient/covariance/seed/sample fields match BootstrapOlsResult, whose
IID wire contract is unchanged. Additive cluster fields are `cluster_count`,
`cluster_sizes` in first-seen retained order, `cluster_order` equal to
`first_seen_complete_case`, and `replicate_observation_counts` in replication
order. Each draw selects G whole groups with replacement; unbalanced groups
produce variable replicate N. Keys support numeric/text/boolean values, exact
integer/decimal identity, merged floating signed zeros and excluded floating
NaN/infinity; other types fail. Complete-finite y/X and nonmissing keys jointly
define the sample. The cap is G*largest_cluster_size*B*K²<=100,000,000.

This is current-column OLS with fixed supplied transforms, not automatic FE/RE,
wild/residual or survey resampling. B-1 covariance/SE and fixed type7 percentile95
intervals do not provide a few-cluster or nominal-coverage guarantee. Any failed
replication aborts without discard/redraw or retained-model/data mutation.

## Conditional quantile regression

`qreg y xvars [, quantile(value) noconstant]` adds CLI-v1
`QuantileRegressionResult`. Its `data` contains outcome, ordered predictors,
include_intercept, quantile, observation_count, ordered coefficients (intercept
first when present), covariance_matrix/covariance_parameter_names, check_loss,
nullable null_check_loss/pseudo_r_squared, sparsity, sparsity_observation_count,
bandwidth_probability, solver_iterations, relative_duality_gap,
max_score_violation and sample `{total_observations,dropped_observations}`.
Coefficient fields are name, value, standard_error, statistic, p_value, ci_lower
and ci_upper. No retained design or individual sample mapping is serialized.

Convention fields are `estimator: "linear_conditional_quantile"`,
`covariance: "iid_location_hall_sheather_residual_spacing"`,
`reference_distribution: "asymptotic_standard_normal"`, confidence_level 95 and
`solver: "clarabel_0.11.1_rust_qdldl_dual_lp_qr_polish"`. Covariance has no
finite-sample multiplier; the local spacing design uses ranks divided by N-K.
Loss is `sum(r*(tau-I[r<0]))`; the intercept null uses an empirical type-1
quantile. Null loss/pseudo R² are absent without an intercept (or pseudo R²
without a positive null loss). These are not likelihood fields.

Sample preparation uses the ordered joint complete-finite numeric cases.
QuantileFit is retained separately from legacy Quantile ModelState, so linear
fit APIs and existing serialized command fields are unchanged. Status reports
family quantile, with no invented linear regression metadata. Bare/current-row
`predict` uses xb; only xb/residuals are installed, following linear NULL/NaN/
infinity arithmetic. A failed new fit clears old inference, while prediction
errors preserve fit/data. Robust/cluster/weighted inference, cross-quantile
tests, margins, named quantile reports and prediction intervals are unsupported.
See `help qreg` and the [household workflow](../../examples/mma04p2qreg/README.md)
for numerical bounds, assumptions and independent evidence.


### Joint IID pairs-bootstrap quantiles

`bootstrap y xvars, quantiles(.25 .5 .75) seed(integer) [reps(200) noconstant]`
returns CLI-v1 `QuantileBootstrapResult`, estimator
`iid_pairs_bootstrap_joint_quantiles`. Existing OLS bootstrap result/command
shapes remain unchanged; typed commands use `QuantileBootstrapCommand` wrapping
its common bootstrap options and original numeric quantile spellings. It has
read-only requirements/effects and never installs/clears a retained fit.

The result contains outcome/predictors/intercept, N, ordered normalized quantiles,
B, exact decimal-string seed, versioned RNG, confidence_level95 and
interval_method `percentile_r_type7`. Coefficient rows contain name, estimate,
bootstrap_mean, bias, standard_error, ci_lower/ci_upper. Covariance names/order
are quantile-major, intercept then predictors; names are `q{normalized tau}:name`.
The full covariance includes cross-quantile blocks from common IID row draws,
with `covariance_convention=common_iid_pairs_b_minus_1` and
`simultaneous_coverage=false`. No analytic p-value/reference distribution is
implied by marginal percentile bounds. Singular covariance is retained rather
than regularized. Optional `contrast(variable)` is accepted only with exactly
two quantiles and adds a `contrast` object; the field is omitted when the option
is absent, preserving the serialized result shape. Its `parameter`, `lower_quantile`, `upper_quantile`
and `estimate` identify the higher-minus-lower original coefficient difference.
The `standard_error` is the B-1 sample SD of shared-replicate differences. When
inference is available, `statistic` is normal z, `p_value` is two-sided,
`reference_distribution` is `asymptotic_standard_normal`, `interval_method` is
`normal_wald`, and `ci_lower`/`ci_upper` form the normal 95% interval. These
nullable inference fields are null with `inference_unavailable_reason` for zero
or unrepresentable variance; zero variance retains standard_error 0. This is
one coefficient contrast, not a general restriction or simultaneous test.

CLI sample reports total/dropped counts; owned Rust results also retain the
complete-case source-index map. The same ordered finite/type rules and local
xoshiro256++/SplitMix64 draw contract as IID OLS apply. Weights/cluster modes are
rejected. Any original or replicate fit failure aborts without redraw or partial
results. Numerical/work limits and statistical assumptions are in help bootstrap.

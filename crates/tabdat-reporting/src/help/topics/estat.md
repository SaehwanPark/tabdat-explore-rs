# estat

How to invoke:
`estat vif`
`estat ar, null(b1 b2 ...)`
`estat ar, ci [level(95)]`
`estat ar, ci coefficient(name) [level(95)]`
`estat overid`
`estat hausman` (after bounded balanced nonclustered RE-GLS)
`estat endogenous`
`estat margins` (after logit, probit, Poisson, NB2, Gaussian Tobit or truncreg)
`estat margins, discrete(variable)` (after logit or probit)
`estat margins, factor(variable)` (after logit or probit with a factor term)
`estat overdispersion` (after Poisson)
`estat overdispersion, nb1` (after Poisson; linear NB1 variance direction)
`estat countprob [, max(10)]` (after Poisson or NB2)
`estat countgof [, max(10)]` (after a model-based unclustered or one-way
clustered NB2 fit)
`estat gof, group(10)` (after an unweighted model-based logit or probit fit)
`estat classification [, cutoff(#)]` (after logit or probit; default cutoff 0.5)

What it does:
For `vif`, report variance-inflation factors from the retained linear-regression predictor design, in predictor order; an active dataset and prior successful bounded `regress` fit are required. The IV forms below provide Anderson–Rubin coefficient inference, Sargan overidentification testing, and joint Wu–Hausman endogeneity testing from a retained 2SLS design.

What problem it answers:
How much does the fitted predictor design inflate variance through linear dependence among predictors?

Examples:
- `regress y x1 x2` then `estat vif`
- `regress y x1 x2, noconstant` then `estat vif`
- `regress y x1 x2, wls(weight)` then `estat vif`
- Batch JSON: `--json -c "use data.csv" -c "regress y x1 x2" -c "estat vif"`

Notes:
- Each auxiliary regression is **unweighted**, using the parent's retained sample and intercept convention, including after WLS/GLS. Missing/non-finite outcome/predictor values and required weight/group exclusions are inherited from the parent fit. This is not a weight-adjusted VIF diagnostic.
- The retained design is used even if current data changes after fitting; `estat vif` does not refit the model, rescan current data, or mutate dataset/model state.
- A supplied `mean_vif` aggregate row follows predictor rows when values are available. Labels are not unique identifiers; a predictor can itself be called `mean_vif`. Undefined factors appear as missing cells; the aggregate is absent if no values are available. An auxiliary-fit failure is an execution error.
- A lone predictor without an intercept uses the zero-column auxiliary model: its fitted values are zero, so SSR equals uncentered TSS and VIF/mean-VIF are 1 for a nonzero finite predictor. The library returns an undefined factor for an all-zero predictor; ordinary regression rank and zero-parameter validation are unchanged. This corrects the former diagnostic failure, allowing batches/scripts to continue normally (MIG-0011).
- Terminal output remains the ordered `Variable`/`VIF` table. CLI-v1 JSON returns `result_type: "TableResult"`, `data.headers` and ordered `data.rows`. Cells use the existing scalar policy: unavailable/non-finite numbers become `null`, finite zero remains numeric. JSON null does not distinguish unavailable values from infinity. No VIF-specific result type or extra model arrays are included; parent-fit metadata is in the preceding `RegressionResult`.
- Other `estat` forms may parse, but are not installed in this bounded runtime or admitted by CLI-v1 JSON. The supported forms are `vif`, `ar`, `overid`, classical panel `hausman`, `endogenous`, binary/count/Tobit/truncated-Gaussian continuous `margins`, binary numeric-0/1 discrete changes and retained-factor probability contrasts, grouped binary `gof`, the Poisson NB2-direction score test `overdispersion`, its explicit NB1 variance-direction option, descriptive count `countprob`, and the model-based NB2 grouped distribution test `countgof` (described below). Bare `estat gof` remains outside this slice, as do other binary specification tests. Generic table serialization alone does not admit additional commands. Direct `sql` SELECT/WITH queries are independently admitted as `TableResult`; `sql ... into name` returns `SqlCreateResult`, and `use name` returns `ActivateResult`. See `help sql` for supported forms and native validation/state limits.

## Classical panel FE–RE Hausman comparison

After `xtreg y x, re id(person) time(year)`, run `estat hausman`. The existing
scalar path requires balanced, nonclustered Swamy–Arora GLS, an intercept and
exactly one slope. It refits FE and RE on the owned joint-finite design; current
data edits do not change the test. `ClassicalPanelHausmanResult` CLI-v1 JSON and
text keep the same scalar slopes, variances, difference, sample/group counts,
chi-square(1), asymptotic p and assumptions. Nonpositive or cancellation-
dominated scalar covariance differences fail; its wire contract is unchanged.

After `xtreg y xvars, re id(group) time(key) vcmethod(equal_group_between)`,
`estat hausman` additionally compares all1..16 numeric slopes jointly, with an
intercept and N<=40000. Balanced/unbalanced groups and singletons follow the
original model sample. It excludes the common intercept, keeps **separate native
FE/RE residual covariance scales**, and computes d'(V_FE−V_RE)^−1d with asymptotic
chi-square(p), p=number of slopes. This method, including the one-slope case,
returns additive `JointPanelHausmanResult` JSON/text: both slope vectors and full
covariances, covariance difference, group/sample counts, residual DF/scales,
minimum parent-scaled Cholesky pivot, statistic/p and assumption/method labels.
A safely positive-definite covariance difference is required; parent-scaled
Cholesky pivots <=128 machine epsilons, rank deficiency, indefiniteness and
nonfinite results are errors. No pseudoinverse, absolute value, ridge, dropped
restrictions or common-variance rescaling is used. Only roundoff-level parent
asymmetry is averaged, not eigenvalue-repaired.

The joint path refits/verifies the complete retained RE result before comparing
FE, independently of active data (even if no dataset is active). Older states
without the owned design require refitting a supported model. Diagnostic
success/failure does not mutate the fit or data. Unbalanced/multiple-slope
**default Swamy–Arora**, ML and robust/clustered comparisons remain unsupported;
there is no arbitrary named-fit comparison. This is not IV Wu–Hausman or the
separate robust auxiliary comparison. Classical validity requires RE efficiency
under H0 and IID homoskedastic idiosyncratic errors; remaining within-group
correlation can invalidate it. Non-rejection does not establish exogeneity,
survey validity or causal identification. See `help xtreg` for sample rules.

## Observed versus fitted count frequencies

After a successful `poisson` or `nbreg`, run `estat countprob` to compare observed
count frequencies with the fitted distribution. Default `max(10)` reports exact
counts0..9 and a final **>=10** bin. Set `max(m)` to a decimal integer1..100;
the last bin is >=m, not exactly m. Invalid options leave data and fit unchanged.

For each bin, output shows observed N/fraction, fitted fraction and expected N.
The fitted fraction is the **average of individual conditional probabilities**
over the ORIGINAL retained estimation sample, not the probability evaluated at an
average mean. It uses retained y/means and NB2 alpha, even after current rows,
outcomes, predictors or cluster IDs change/drop. It does not refit or mutate data.
CLI-v1 returns `CountProbabilityResult` with inclusive bounds (`upper_bound: null`
for the tail), distribution parameters and original sample provenance.

Example: `poisson visits age income, robust` then `estat countprob, max(10)`;
repeat after `nbreg visits age income`. The model-based/row-robust/cluster
covariance choice does not alter the plug-in probabilities on the same sample.
**Correct conditional Poisson/NB2 distributional assumptions are required**:
robust/cluster QMLE covariance does NOT establish them. This is a descriptive
comparison, with NO formal goodness-of-fit test, p-value, SE or interval.

PMFs are evaluated in log coordinates; small tails use direct incomplete
gamma/beta calculations, with complements only in well-conditioned branches.
Representable small probabilities are not lost by subtracting a near-one sum;
probabilities are never clipped/renormalized. Numerical nonconvergence is
an error. There is no current-row PMF/CDF `predict` or probability export command.
See `examples/mma20p1count/count_probabilities.td`; the printed Table20.6 NB2 row
differs materially from the authors'2006 program and remains unresolved, not
forced into a tolerance or claimed as a literal replication.

## Poisson overdispersion score tests

After a successful Poisson fit, `estat overdispersion` tests
`H0: Var(Y|X)=mu` against the one-sided NB2 variance alternative
`Var(Y|X)=mu+alpha*mu^2`, `alpha>0`. It evaluates the restricted NB2 score
`u_i = 0.5*((y_i-mu_i)^2-y_i)` at the retained Poisson fit; `mu_i` includes any
fixed offset. It does not fit NB2 and does not establish the full negative-
binomial distribution.

Use `estat overdispersion, nb1` for the linear variance direction
`Var(Y|X)=mu+alpha*mu`, `alpha>0`, with row score
`u_i = ((y_i-mu_i)^2-y_i)/(2*mu_i)`. NB1-direction testing requires strictly
positive retained fitted means. This is a regression-based variance test, not
NB1 maximum-likelihood estimation or a claim that the full count distribution
is negative binomial.

For NB2, model-based score variance is the Poisson information
`0.5*sum(mu_i^2)`; for NB1 it is `N/2`. Either direction can use row-score
variance with N/(N-1) after `robust`, or grouped scores with G/(G-1) after
`cluster(group)`. The signed Z statistic has a one-sided asymptotic-normal
p-value. Robust/cluster score variance allows the selected independent unit to
differ from the Poisson likelihood's working independence variance; it still
requires a correctly specified conditional mean. Cluster inference assumes
independent groups and many-group asymptotics, with no few-cluster guarantee.
The NB2 score remains separate from NB2's model-based boundary likelihood-ratio
result. See `examples/nb2-overdispersion-score/README.md` for Base R evidence
and limits.

## NB2 grouped count goodness of fit

After an unclustered, unweighted, model-based `nbreg` fit without an offset,
`estat countgof` compares observed count cells with the mean fitted NB2
probabilities and reports the Andrews score-adjusted grouped chi-square test.
The default `max(10)` defines cells0..9 and >=10; `max(m)` accepts an integer
1..10 and defines cells0..m-1 and >=m. For J=m+1 cells, one cell is omitted and
the no-intercept auxiliary regression includes the remaining J-1 cell residuals
and every NB2 maximum-likelihood score, including `lnalpha`. The reported
statistic is N times the uncentered R-squared, with asymptotic chi-square(J-1)
reference. The result includes observed/fitted frequencies for every cell.

The unclustered reference requires independent rows and a correctly specified
conditional NB2 distribution. A one-way clustered NB2 fit without weights or an
offset instead uses the cluster-summed score-adjusted influence function and
`G/(G-1)` moment-covariance multiplier. It retains all beta/`lnalpha` scores and
the model-based observed-information bread. The covariance must have full rank
for the chi-square test to be reported. Its reference requires independent
clusters, correctly specified conditional NB2 marginals, and many clusters;
the multiplier does not give a few-cluster guarantee. Row-robust fits, weights,
offsets, and multiway clusters remain unsupported.

In particular, the RAND doctor-visit example has repeated person-years; the
authors say clustering is needed, while their §20.7 goodness-of-fit code does
not cluster. The unclustered p-value is therefore method reproduction, not
panel-robust inference. The cluster extension is validated on a synthetic
dependent-within-cluster NB2 reference, not the source panel; the prepared
author ASCII omits person identifiers.

For the default 11-cell partition the book's J-1 rule gives df=10. The November
2006 author program passes df=9 to its p-value calculation; this command follows
the book's rule and keeps the program discrepancy explicit. The diagnostic uses
the retained NB2 sample and does not refit or mutate model/data state. See
`examples/mma20p1count/count_gof.md` for the independent Base R reference and
source comparison, and `examples/mma20p1count/cluster_count_gof.md` for the
cluster influence-function derivation and independent reference.

## Grouped binary goodness of fit

After an unweighted, model-based `logit` or `probit` fit, run
`estat gof, group(10)`. The requested group count must be an integer from 3 to
20. The diagnostic uses the original retained response and fitted
probabilities; it does not rescan current data or refit the model. Ordered
fitted probabilities are divided at approximately equal-size quantiles. Rows
with exactly tied probabilities stay together, so the reported actual number
of nonempty groups can be smaller than requested. At least three actual groups
and strictly positive expected event and non-event counts in every group are
required.

For each group, the report gives the inclusive upper probability boundary,
observed and expected event/non-event counts, and N. The Hosmer–Lemeshow
statistic is the grouped Bernoulli Pearson statistic
`sum((observed_events - expected_events)^2 /
(expected_events * expected_nonevents / group_N))`. Estimation-sample inference
uses asymptotic chi-square with `actual_groups - 2` degrees of freedom and its
upper-tail p-value. CLI-v1 JSON returns `BinaryGoodnessOfFitResult` with the
ordered group details, retained sample, method, reference and assumptions.

Robust, clustered and weighted fits are rejected. Their covariance choices do
not justify the independent-Bernoulli chi-square calibration. This grouped
calibration diagnostic is sensitive to the requested group count and can be
unreliable when groups have sparse expected cells. It does not supply the
default Pearson test across covariate patterns or any other binary
specification test. See
[`examples/binary-hosmer-lemeshow/README.md`](../../../../../examples/binary-hosmer-lemeshow/README.md)
for the independent Base R comparison and edge-case evidence.

## Binary classification table

After a successful `logit` or `probit` fit, `estat classification` classifies
each retained estimation-sample observation as positive when its fitted
probability is greater than or equal to the cutoff. The default cutoff is 0.5;
`cutoff(#)` accepts a finite value from 0 through 1. The event is exactly
`outcome == 1`.

The table reports actual/predicted class totals and true-positive,
false-positive, false-negative and true-negative counts. It also reports
sensitivity, specificity, positive/negative predictive values, false-positive
and false-negative rates among actual classes and predicted classes, and the
correct-classification percentage. Percentages use a 0–100 scale. A percentage
is `NA` if its denominator is zero. The result is descriptive in-sample
classification: it has no standard error or hypothesis test and does not select
a cutoff or assess probability calibration/specification. Robust and clustered
fits use the same retained probabilities; their covariance does not affect
classification counts.

The calculation uses the response and probabilities owned by the successful
fit, so current-data edits do not change it. The command does not accept `all`,
row qualifiers, weights, ROC curves or cutpoint selection. JSON returns
`BinaryClassificationResult`; see the corresponding result contract and the
[independent synthetic reference](../../../../../examples/binary-classification/README.md).

## Binary continuous average partial effects

After a successful model-based, robust or one-way clustered binary fit (`logit`
or `probit`, optionally `, robust` or `, cluster(group)`), `estat margins` reports
the family-specific average derivative for each original continuous variable
listed directly or used in a continuous interaction:

- logit: `mean(p*(1-p) * d eta/dx)`
- probit: `mean(phi(eta) * d eta/dx)`

The average uses the **retained complete-finite estimation design**, not current
rows. Current-data edits do not rescan, refit or change the estimand.
Expanded factor dummy columns are not treated as continuous slopes. For
`c.x#c.z`, the product contributes one design column and no automatic main
effects; its contribution to `d eta/dx` is `beta_xz*z`, and its contribution to
`d eta/dz` is `beta_xz*x`. If either main effect is also listed, it contributes
its coefficient to the corresponding derivative. Factor levels and other
covariates stay at their observed retained-row values. The full gradient
includes all fitted coefficients, including factor and interaction coefficients.
Factor-only designs have no continuous effect and fail clearly. Use
`factor(variable)` below for retained-factor probability contrasts and their
uncertainty.

Uncertainty is the full delta method using the fit's selected covariance:
model-based (logit: inverse Fisher information; probit: inverse observed
information), unclustered score sandwich with N/(N-1) correction after
`, robust`, or one-way cluster sandwich with G/(G-1) correction after
`, cluster(group)`. Cluster inference is asymptotic normal, not G-1 t, and
requires independent clusters; few-cluster reliability is not guaranteed.
It includes intercept/slopes' covariance, **conditional on observed
X**. It is not uncertainty in the covariate distribution. Output includes normal
z/two-sided p and fixed 95% intervals; CLI-v1 returns
`LogitMarginalEffectsResult` or `ProbitMarginalEffectsResult` by fitted family.

These are original-variable continuous derivatives, not discrete changes. In
particular, a generated log-price column gives a derivative per log-price unit,
not per original price unit. No
multiway-cluster/weighted effects or level option are installed. The one supported
numeric binary contrast is described below. Additional model-family margins are
limited to the Poisson/NB2 continuous effects below. The retained fit
selects the family; with no supported fit the call keeps the established
prior-logit error wording. Refit successfully after a failed new runtime
estimation attempt; see `help logit`, `help probit`, `help poisson` and `help predict`.

## Binary discrete probability changes

After logit or probit, run `estat margins, discrete(variable)` for one retained
original numeric input whose values are exactly 0 and 1, with both levels
present. The input may appear as a main effect, in one or more continuous
product terms, or both. Product columns involving it are rebuilt for each
counterfactual. It can also be used beside additive factor terms, whose levels
stay fixed. Expanded factor dummy columns are not selectable; use
`factor(variable)` for a retained categorical predictor.
It reports the average fitted probability difference after setting that
predictor to 1 versus 0 for every retained row; all other predictor values stay
at their observed row values. The result is a finite change, not a continuous
derivative or a causal effect.

The calculation and full delta-method gradient use the owned retained design and
selected model-based, independent-row robust, or one-way clustered covariance.
Uncertainty is conditional on observed X, with asymptotic-normal z inference and
a fixed 95% interval. Cluster inference assumes independent groups; the G/(G-1)
correction does not guarantee reliable few-cluster inference. Current data edits
do not alter the result. The command rejects an unknown predictor, a factor
dummy, an input with values other than 0/1, or a retained sample missing either
level. It does not add
factor-variable encoding for the selected contrast, multiple contrasts, factor
interactions, weights, multilevel changes, or a causal interpretation. CLI-v1 JSON returns
`BinaryDiscreteChangeResult` with the estimand, predictor, covariance and sample
count. See `examples/binary-discrete-change/README.md` for additive evidence and
`examples/binary-interaction-discrete-change/README.md` for interaction evidence.

## Binary factor probability contrasts

After a logit or probit fit containing `i.variable` or `ib(level).variable`,
run `estat margins, factor(variable)`. The command returns one average fitted
probability difference for each non-baseline level versus a single baseline.
With an intercept, the baseline is the retained fit-time omitted reference.
With `noconstant`, it is the first sorted retained level. Every contrast
averages over the original retained sample and changes only the selected factor;
all other continuous and categorical terms stay at their observed row values.
Current data edits do not change the estimand.

For level `l` and baseline `b`, the estimate is
`N^-1 sum_i [F(x_i, l) - F(x_i, b)]`. The full gradient is the average
difference in link-density-weighted design rows. Delta-method uncertainty uses
the fitted model-based, row-robust or one-way clustered covariance, including
all coefficient cross terms. Output includes a two-sided asymptotic-normal z
test and 95% interval; the contrast is conditional on observed X and is not a
causal effect. JSON returns `BinaryFactorContrastsResult` with the factor,
baseline and ordered level contrasts.

The selected term must be a retained factor. Interactions, weights, multiway
clustering, simultaneous intervals and factor effects after a nonbinary family
remain unsupported. A missing fit, absent factor term, invalid retained layout
or unavailable delta variance fails without changing the fit.

## Poisson continuous average partial effects

After model-based or independent-row robust `poisson`, `estat margins` reports
`beta_j*mean(mu)` for each predictor column, excluding the intercept. The average
uses the **owned retained estimation design**, not current rows; edits do not
rescan/refit or change this estimand. Intercept-only fits have no predictor effects.

The full gradient is `beta_j*mean(mu*X) + e_j*mean(mu)`, including intercept/slopes'
covariance. The delta variance uses selected inverse Poisson information,
independent-row N/(N-1) or one-way group-score G/(G-1) sandwich. It does not use
Pearson dispersion, OLS HC1 or Student t; clustering assumes independent groups. Output has normal z/two-sided p and
fixed 95% intervals (`PoissonMarginalEffectsResult`); zero variance gives
unavailable z/p/CI rather than an exact interval.

These are **continuous column derivatives conditional on observed X**, not
binary-variable discrete changes, log/interaction chain rules or uncertainty in
the covariate distribution. For a generated log-income predictor the derivative
is per log-income unit, not per currency unit. One-way Poisson `cluster(var)`
uses the retained group-score G/(G-1) covariance and carries original variable/G
metadata through margins, with normal inference and independence across clusters.
It does not require current cluster IDs, resample X, or guarantee few-cluster
coverage. There are no weighted, factor/discrete-change or user-level-option
count margins. NB2 one-way clustering uses full joint scores as described below.

## NB2 continuous average partial effects

After `nbreg`, `estat margins` uses the owned retained X to report
`beta_j*mean(mu)` with gradient `[beta_j*mean(mu*X)+e_j*mean(mu), 0]` in joint
beta/lnalpha order. The zero direct lnalpha derivative does **not** justify
substituting conditional/Fisher covariance: the beta block comes from the FULL
joint inverse observed information, independent-row N/(N-1) or one-way grouped
FULL joint-score G/(G-1) sandwich. Group variable/G metadata is retained; no
conditional/Fisher or row-score substitution, G-1 t or few-cluster guarantee.

Output is conditional-X delta normal95 (`NbregMarginalEffectsResult`), not
Student t, random-X uncertainty, discrete changes or original-variable chain
rules. Current data edits do not change the retained effect. Intercept-only
models have no predictor effects. Original groups/V survive edits/drop ID;
current predictions need mean predictors only. There are no weighted/multiway/
discrete-change or user-level-option NB2 margins; see `help nbreg` for bounds.

## Sargan overidentification test

After a **nonrobust**, overidentified 2SLS fit:

```text
ivregress 2sls y w, endog(x1 x2) iv(z1 z2 z3)
estat overid
```

This reports the homoskedastic **Sargan N*R²** test. Let Z be all included
exogenous/intercept columns plus excluded instruments and u the retained
**structural** 2SLS residuals against the original regressors. The statistic is
`N * (u' P_Z u) / (u' u)`, with asymptotic chi-square DF equal to excluded-instrument
count minus endogenous-regressor count. The uncentered denominator also defines
`noconstant` behavior; there is no N-K correction. This is not Basmann, robust
Hansen J, or an exact finite-sample F test.

The null is that the overidentifying restrictions hold, given the maintained
model. Homoskedasticity, correct specification and standard strong-identification
asymptotics are required. A small p-value can reflect invalid instruments or
model misspecification; it does not identify which instrument is invalid.
Failure to reject does not prove instrument validity or resolve weak identification.

Just-identified fits are rejected explicitly, as are zero/nonfinite structural
residual variance and auxiliary rank/DF failures. Robust and clustered parent
fits use the score test below, not Sargan.
Sargan uses the retained sample/design/residuals even after current data changes;
it does not refit 2SLS, rescan current columns or mutate dataset/model state.
CLI-v1 JSON returns `SarganResult` with test convention, assumptions, N, endogenous
and excluded-instrument names, DF, statistic and upper-tail p-value. Scripts and
`--text` expose the same owned results. Endogeneity tests are described below;
clustered Hansen J/GMM overidentification remains separate work. See the
reproducible external-data example under `examples/cigarettes_iv/`.

### Robust overidentification score

After `ivregress 2sls ..., robust` or `cluster(group)`, `estat overid` returns
`RobustOveridScoreResult`, a Wooldridge residualized-instrument score test.
Let X be the original regressors, Z the full instrument design (both including
any intercept), Xhat=P_Z X, L=rank(Z), K=rank(Xhat), and q=L-K. Residualize Z on
Xhat and choose exactly q independent directions Q from that complement. For
the independent-row path, regress ones on `diag(u)Q` **without an intercept**,
using retained structural 2SLS residuals u. The statistic is N-RSS, computed as
the fitted projection norm to avoid cancellation. It has an **asymptotic
chi-square(q)** reference under the null and conventional strong identification.

The independent-row score OPG is **uncorrected**: `covariance_correction=1`, not
the parent coefficient HC1 N/(N-K) adjustment. For a clustered parent fit, the
test sums each residualized-instrument score within its retained group and uses
the uncentered CR0 covariance of those group sums. It also has
`covariance_correction=1`; the parent coefficient CR1 multiplier is not applied
to this score test. Its statistic is `T' V^-1 T`, where `T` is the sum of group
scores and `V` is their outer-product sum. The cluster path reports its variable
and group count and requires more groups than q plus a full-rank score covariance.

The projection removes the first-order 2SLS nuisance-estimation effect; testing
all raw moments with an arbitrary weight is not equivalent. This is not Hansen J,
classical Sargan, Basmann, a finite F, a survey test or weak-IV-robust inference.
The independent-row path requires independent rows. The cluster path requires
independent groups and many-group asymptotics; it gives no few-cluster guarantee.
Both require the correctly specified maintained model, valid instruments under
the null and conventional strong identification. Failure to reject does not
prove instrument validity. Rejection may reflect invalid instruments or
misspecification, not a named instrument's guilt.

The result reports ordered endogenous/instrument names, N, L, K, q, correction,
statistic and upper-tail p, with optional cluster variable/group count and the
same text/script/JSON reachability. The original fit sample, design, cluster IDs
and structural residuals are retained; there is no current-data scan, structural
refit or data/model mutation. Full-rank Z/Xhat and a rank-q score design are
required. The complement uses unit-scaled instruments and pivoted
twice-reorthogonalized directions with a 1e-12 relative norm rank bound; score
columns are separately unit-scaled before QR. Rank is not silently reduced.
Just-identification, insufficient auxiliary DF/groups, nonfinite or zero
structural residuals and singular row/group score spaces fail explicitly.

See the [documented score construction, p23](https://www.stata.com/manuals/rivregresspostestimation.pdf),
`examples/cigarettes_iv/robust_overid.td` and
`crates/tabdat-stats/tests/fixtures/robust_overid_score_reference.R` for method
evidence. This does not mark a Cameron–Trivedi empirical illustration complete.

## Joint Wu–Hausman endogeneity test

After a **nonrobust** 2SLS fit, including a just-identified model:

```text
ivregress 2sls y w, endog(x1 x2) iv(z1 z2)
estat endogenous
```

The null is that **all** regressors declared in `endog()` are exogenous, given
valid instruments and the maintained model. Full-instrument first stages use the
parent's included exogenous variables/intercept and excluded instruments. The
restricted auxiliary OLS fits y on the original regressors; the augmented OLS adds
every first-stage innovation. Original coefficients are unrestricted in both
fits; the m added innovation coefficients are jointly zero under the null.
The homoskedastic Wu–Hausman statistic is
`((RSS_restricted - RSS_augmented)/m) / (RSS_augmented/(N-K-m))`, where K includes
the original intercept and regressors. It reports F(m, N-K-m), **not** the
first-stage or 2SLS residual DF. No Durbin chi-square LM, covariance-difference
Hausman or endogenous-subset test is implied. Robust/cluster fits use the separate
Wald convention below, not this F reference.

The conventional reference requires valid instruments, homoskedastic errors,
correct specification and standard strong-identification regularity. Exact
finite-sample F additionally requires conditional normality. Rejection of joint
exogeneity is conditional on these assumptions; invalid instruments or a
misspecified model can invalidate that interpretation. Failure to reject does
not prove exogeneity, instrument validity or weak-IV-robust coefficient inference.
The classical result and its reference are not silently substituted for robust
or clustered inference.

Auxiliary predictors are norm-rescaled without changing their column spaces.
An innovation norm at most 1e-12 relative to the endogenous residual norm after
included-exogenous controls is rejected as numerically degenerate. So is an
augmented residual norm at most 1e-12 relative to the centered outcome norm
(uncentered for noconstant). Constant outcomes with an intercept, nonfinite
arithmetic, insufficient N-K-m DF, singular instrument/augmented designs and
undefined variance fail explicitly; innovation rank is not silently reduced.

Auxiliary fits use the retained complete-case problem even after current data
changes; no 2SLS refit, current-column scan or dataset/model mutation occurs.
`--text`, scripts and CLI-v1 JSON expose `WuHausmanResult`: tested endogenous
names, excluded instruments, N, numerator/denominator DF, F, upper-tail p-value,
joint-null scope and assumptions. The external AER example in
`examples/cigarettes_iv/endogenous.td` distinguishes the classical F from an HC0
contrast; neither is silently substituted for the other.

### Robust/one-way clustered joint endogeneity Wald

After `ivregress 2sls ..., robust` or `cluster(unit)`, the same `estat endogenous`
command jointly tests all first-stage innovation coefficients in the augmented
OLS regression. It uses **auxiliary** HC1 `N/(N-Kaux)` or one-way CR1
`G/(G-1)*(N-1)/(N-Kaux)` score covariance, where `Kaux = K+m`, not the parent IV
parameter count. First stages remain OLS; cluster IDs come from the retained fit
sample, not current data. All classical innovation/variance/rank guards still
apply, and a singular/rank-limited innovation Wald covariance is an error rather
than a reduced-rank test.

`RobustEndogeneityResult` reports the Wald statistic with an **asymptotic
chi-square(m)** reference, auxiliary K/residual DF, correction, covariance mode
and optional group count. Residual DF is correction metadata, not a chi-square
denominator or G-1 t/F reference. This is not an exact finite-sample F, robust
score test, GMM difference-in-Sargan test, subset test or weak-IV-robust inference.
Valid instruments, the maintained linear model and conventional strong
identification are required. HC1 assumes independent rows; CR1 allows within-group
dependence but needs independent, growing groups, with no few-cluster guarantee.
Failure to reject does not establish exogeneity or instrument validity.

The [Stata methods discussion, p22](https://www.stata.com/manuals/rivregresspostestimation.pdf)
describes residual inclusion and appropriate robust covariance, citing Cameron
and Trivedi (2005, p276); native chi-square/correction conventions are explicit,
not a claim of historical Stata finite-F parity. The independent base-R workflow
`examples/cigarettes_iv/robust_endogenous.td` and synthetic HC1/CR1 references
provide method evidence, not another validated textbook illustration.

## Joint Anderson–Rubin test

After an `ivregress 2sls` fit, run for example:

```text
ivregress 2sls y w, endog(x1 x2) iv(z1 z2)
estat ar, null(1.7 -0.6)
```

`null()` must specify a finite numeric value for **every** endogenous coefficient,
in the parent's `endog()` order. Decimal and scientific notation are accepted;
whitespace separates coefficients, with no whitespace inside a signed number.
Included exogenous coefficients (including the intercept when fitted) remain
unrestricted nuisance parameters. This is a joint test, not a scalar/subvector
test with other endogenous coefficients left unspecified.

The test regresses `y - X_endog * b0` on the included exogenous variables alone
and on the included exogenous plus excluded instruments, then compares residual
sums of squares. It reports F(q, N − rank[W,Z]), where q is the excluded-instrument
count. Valid instruments and homoskedastic errors are required; exact finite-sample
F inference additionally requires conditional normality. The test is robust to
weak identification and does not test instrument validity. This F form applies
only to a nonrobust parent fit. Undefined auxiliary variance, rank or residual
DF failures are explicit errors.

After a `robust` or `cluster(group)` parent fit, the same command instead tests
all excluded-instrument coefficients in the unrestricted auxiliary regression
using HC1 or one-way CR1 covariance. It reports the full Wald statistic with an
**asymptotic chi-square(q)** reference. HC1 uses N/(N−K_aux); CR1 uses
G/(G−1)*(N−1)/(N−K_aux), with K_aux counting the intercept, included exogenous
variables and excluded instruments. These are auxiliary residual scores and
auxiliary DF, not the parent's structural residuals or parameter count. HC1
requires independent rows; CR1 requires independent clusters and many-group
asymptotics. Neither is an exact finite-sample test or a few-cluster guarantee.
Singular excluded-instrument covariance (including q >= G) is an explicit error;
the test never drops restrictions to obtain a p-value.

The retained complete-case fit sample/design is used even after current data is
changed; AR does not refit 2SLS, reread current columns or mutate model/data state.
`--text`, scripts and CLI-v1 JSON expose the joint null, endogenous names/order,
excluded instruments, N, both DF, F, p-value and assumptions (`AndersonRubinResult`).
A small p-value rejects the **entire supplied joint null** under these assumptions;
it does not identify which endogenous coefficient differs.
Robust results use the distinct `RobustAndersonRubinResult` type with covariance,
correction, auxiliary K/DF, optional G and asymptotic reference metadata; the
classical `AndersonRubinResult` contract remains unchanged. See
`examples/robust-ar/README.md` for an independently validated complete workflow.
Endogenous-subvector inference is not implemented. Robust confidence sets are
limited to one endogenous regressor and at most fourteen excluded instruments, as
described below.
Ordinary coefficient p-values and `test` remain conventional, not weak-IV-robust
inference.

## Scalar Anderson–Rubin confidence sets

After a **nonrobust** 2SLS fit with **exactly one endogenous regressor**:

```text
ivregress 2sls y w, endog(x) iv(z1 z2)
estat ar, ci
estat ar, ci level(90)
```

`ci` explicitly requests a confidence set; it cannot be combined with `null()`.
The default confidence level is 95 percent; `level()` must be finite and strictly
between 0 and 100. Included exogenous coefficients remain unrestricted nuisance
parameters. Multi-endogenous fits are rejected: the app does not plug in their
estimated effects or silently interpret a joint region as a scalar interval.

The set contains all scalar null values not rejected by the homoskedastic AR
F test. Analytic quadratic inversion has no finite grid or arbitrary search
range. Results may be a bounded interval, two disjoint rays, the whole real line,
or an empty set. Linear/constant boundaries and singleton sets are retained.
Finite endpoints are closed; terminal infinity is `-inf`/`+inf`. An empty set
means every scalar null is rejected at the chosen level under these assumptions,
not a convergence failure; an all-real set provides no finite bound.

The retained fit sample/design and homoskedastic/valid-instrument assumptions
are the same as for the joint test; exact F inference requires conditional
normality. Rank, residual-DF, nonfinite arithmetic or endpoint-range failures are
errors. This bounded implementation requires auxiliary residual variance to stay
positive for every scalar null: zero residual outcome variance or an outcome/
endogenous residual pair with relative innovation norm at most 1e-12 is rejected
as numerically degenerate. This avoids inventing confidence bounds across a null
with undefined variance. No near-zero quadratic coefficient is silently dropped;
coefficient scaling underflow is an explicit error. A confidence level whose
complementary probability or positive F multiplier loses floating-point resolution
is also an error, even if it is syntactically valid. Distinct bounded/disjoint
endpoints that coalesce when translated back to the original coefficient scale
are rejected rather than relabelled as genuine singleton or all-real sets.

CLI-v1 JSON returns `AndersonRubinConfidenceSet` with the endogenous name, level,
N, both DF, critical F, `shape` and `intervals`. An absent lower/upper bound
(`null`) means **negative/positive infinity**, not missingness; `empty` has an
empty interval list and `all_real` has one interval with both bounds null.
This homoskedastic form does not imply robust confidence sets or
endogenous-subvector/nuisance projection. A separate projected form is available
for multi-endogenous homoskedastic fits below.

After a `robust` or `cluster(group)` fit with exactly one endogenous regressor
and exactly one excluded instrument, the same `ci` syntax inverts the HC1/CR1
auxiliary Wald test instead:

```text
ivregress 2sls y w, endog(x) iv(z) robust
estat ar, ci
ivregress 2sls y w, endog(x) iv(z) cluster(group)
estat ar, ci level(90)
```

The result uses an **asymptotic chi-square(1)** reference. HC1 correction is
N/(N−K_aux); CR1 is G/(G−1)*(N−1)/(N−K_aux), where K_aux counts the intercept,
included exogenous regressors and the excluded instrument. The analytic
quadratic inversion reports bounded, disjoint, ray, all-real, empty or singleton
sets. Included exogenous coefficients remain unrestricted nuisance parameters.
HC1 assumes independent rows; CR1 assumes independent groups and many-group
asymptotics. Neither offers finite-sample or few-cluster coverage guarantees.

Multiple excluded instruments, multiple endogenous regressors, projected robust
sets, and HAC/survey covariance are unsupported for this one-instrument result.
Rank, residual-DF, cluster, nonfinite arithmetic and degenerate auxiliary-score
covariance failures are explicit errors. CLI-v1 JSON returns the distinct
`RobustAndersonRubinConfidenceSet` type with the covariance, correction, optional
cluster count, critical value, shape and intervals; the homoskedastic JSON
contract is unchanged.

With two through fourteen excluded instruments, `estat ar, ci` tests their
coefficients jointly using an **asymptotic chi-square(q)** reference, where q is the number of
excluded instruments:

```text
ivregress 2sls y w, endog(x) iv(z1 z2) robust
estat ar, ci
ivregress 2sls y w, endog(x) iv(z1 z2) cluster(group)
estat ar, ci level(90)
ivregress 2sls y w, endog(x) iv(z1 z2 z3) robust
estat ar, ci
ivregress 2sls y w, endog(x) iv(z1 z2 z3 z4) robust
estat ar, ci
ivregress 2sls y w, endog(x) iv(z1 z2 z3 z4 z5) robust
estat ar, ci
ivregress 2sls y w, endog(x) iv(z1 z2 z3 z4 z5 z6) robust
estat ar, ci
ivregress 2sls y w, endog(x) iv(z1 z2 z3 z4 z5 z6 z7) robust
estat ar, ci
ivregress 2sls y w, endog(x) iv(z1 z2 z3 z4 z5 z6 z7 z8) robust
estat ar, ci
ivregress 2sls y w, endog(x) iv(z1 z2 z3 z4 z5 z6 z7 z8 z9) robust
estat ar, ci
ivregress 2sls y w, endog(x) iv(z1 z2 z3 z4 z5 z6 z7 z8 z9 z10) robust
estat ar, ci
ivregress 2sls y w, endog(x) iv(z1 z2 z3 z4 z5 z6 z7 z8 z9 z10 z11 z12) robust
estat ar, ci
ivregress 2sls y w, endog(x) iv(z1 z2 z3 z4 z5 z6 z7 z8 z9 z10 z11 z12 z13) robust
estat ar, ci
ivregress 2sls y w, endog(x) iv(z1 z2 z3 z4 z5 z6 z7 z8 z9 z10 z11 z12 z13 z14) robust
estat ar, ci
```

The auxiliary regression includes the retained controls, intercept and all
excluded instruments. HC1 uses N/(N−K_aux); CR1 uses
G/(G−1)*(N−1)/(N−K_aux), where K_aux counts all auxiliary regressors. The
confidence boundary has degree at most 2q, up to degree twenty-eight for fourteen
instruments and up to fifteen accepted components.
The implementation isolates every real root by recursively partitioning at
derivative roots, then checks the sign of each region; it does not use a finite
grid. Sets may be bounded, disjoint, rays, all-real, empty, or `multiple` sets
with up to fifteen components. Repeated/ambiguous roots and singular or
ill-conditioned score covariance are explicit errors. The distinct
`RobustMultipleInstrumentAndersonRubinConfidenceSet` JSON result identifies the
instruments, reports two through fourteen degrees of freedom and follows the same
null-infinity convention. More than fourteen excluded instruments, multiple endogenous
regressors outside the robust projected two-by-two slice, robust projection
outside that slice, and HAC/survey covariance remain unsupported.

## Projected Anderson–Rubin confidence sets

For a nonrobust 2SLS model with multiple endogenous regressors, name the
coefficient of interest:

```text
ivregress 2sls wage w, endog(grade76 exp76 expsq76) iv(col4 age76 agesq76)
estat ar, ci coefficient(grade76) level(95)
```

TabDat analytically profiles every other endogenous coefficient out of the joint
homoskedastic AR nonrejection region. Included exogenous coefficients remain
unrestricted nuisance parameters. This is a projection of the joint region, so
coverage is at least the requested level and can be conservative. It does not
plug in nuisance estimates and does not use a finite search grid. The reported
critical value remains F(q, N − rank[W,Z]), for the complete endogenous null.
Exact finite-sample coverage requires valid instruments, homoskedastic errors
and conditional normality; without normality the F calibration is the usual
homoskedastic large-sample reference.

The nuisance profile is inverted exactly when its curvature is numerically
positive definite after unit-diagonal scaling, with every Cholesky pivot above
1e-12. Otherwise the command reports the all-real set as a conservative fallback
and marks `all_real_conservative_fallback: true` in JSON.
This may reflect a truly unbounded projection or numerical/rank-degenerate
nuisance curvature. The command uses the retained estimation sample and does not
change session state.

For an HC1 or one-way clustered fit with exactly two endogenous regressors and
two excluded instruments, the same syntax computes a robust projection:

```text
ivregress 2sls y w, endog(x1 x2) iv(z1 z2) robust
estat ar, ci coefficient(x1) level(95)
```

This projects the complete asymptotic chi-square(2) robust AR nonrejection region
over the other endogenous coefficient. HC1 uses N/(N−K_aux); CR1 uses
G/(G−1)*(N−1)/(N−K_aux), with K_aux counting the intercept, included controls
and both excluded instruments. It does not plug in a nuisance estimate or use a
finite grid. The command reports the closed conservative envelope, so a boundary
approached only as the nuisance coefficient diverges may be included. Root or
covariance ambiguity returns an all-real conservative fallback; a score
covariance singular throughout the null plane is an error. CR1 requires at least
three retained groups and independent groups with many-group asymptotics; there
is no finite-sample or few-cluster guarantee. Other dimensions and covariance
estimators remain unsupported. The retained fit sample/design/groups are used,
and session state is unchanged.


Links:
- `SPEC.md`

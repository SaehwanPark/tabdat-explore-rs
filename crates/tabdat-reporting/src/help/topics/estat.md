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
`estat countprob [, max(10)]` (after Poisson or NB2)

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
- Other `estat` forms may parse, but are not installed in this bounded runtime or admitted by CLI-v1 JSON. The supported forms are `vif`, `ar`, `overid`, classical panel `hausman`, `endogenous`, binary/count/Tobit/truncated-Gaussian continuous `margins`, and descriptive count `countprob` (described below). Generic table serialization alone does not admit additional commands. Direct `sql` SELECT/WITH queries are independently admitted as `TableResult`; `sql ... into name` returns `SqlCreateResult`, and `use name` returns `ActivateResult`. See `help sql` for supported forms and native validation/state limits.

## Classical panel FE–RE Hausman comparison

After `xtreg y x, re id(person) time(year)`, run `estat hausman`. This is a
model-based covariance-difference test of the same-sample FE and RE slope, not
`estat endogenous`'s IV Wu–Hausman test. The fit must be balanced, nonclustered
Swamy–Arora GLS with an intercept and exactly one slope. It refits FE and RE on
the owned joint-finite design; current dataset edits do not change the test.
Older retained states without that design require refitting RE. Unsupported
unbalanced, ML, robust/clustered or multiple-slope fits fail without changing
state. Nonpositive or cancellation-dominated V_FE−V_RE is an error, not repaired
by taking an absolute value or a generalized inverse.

Text and `ClassicalPanelHausmanResult` CLI-v1 JSON report both slopes and
variances, difference, group/sample counts, chi-square(1), asymptotic p-value
and model-based assumptions. Inference requires RE efficiency under the null;
remaining within-person correlation can invalidate it. It is not cluster robust,
a generic model selection tool, or evidence that exogeneity holds after failure
to reject. See the separately composed robust comparison in the Chapter 21
example and `help xtreg` for fit and sample rules.

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

## Binary continuous average partial effects

After a successful model-based, robust or one-way clustered binary fit (`logit`
or `probit`, optionally `, robust` or `, cluster(group)`), `estat margins` reports, for each
predictor column excluding the intercept, the family-specific average derivative:

- logit: `beta_j*mean(p*(1-p))`
- probit: `beta_j*mean(phi(beta_0 + beta'x))`

The average uses the **retained complete-finite estimation design**, not current
rows. Current-data edits do not rescan, refit or change the estimand.

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

These are continuous predictor-column derivatives, not discrete changes or
original-variable interaction chain rules. In particular, a generated log-price
column gives a derivative per log-price unit, not per original price unit. No
multiway-cluster/weighted effects, level option or categorical/discrete changes
are installed. Additional model-family margins are limited to the Poisson/NB2
continuous effects below. The retained fit
selects the family; with no supported fit the call keeps the established
prior-logit error wording. Refit successfully after a failed new runtime
estimation attempt; see `help logit`, `help probit`, `help poisson` and `help predict`.

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

Just-identified and clustered parent fits are rejected explicitly, as are
zero/nonfinite structural residual variance and auxiliary rank/DF failures.
Independent-row robust parent fits use the separate score test below, not Sargan.
Sargan uses the retained sample/design/residuals even after current data changes;
it does not refit 2SLS, rescan current columns or mutate dataset/model state.
CLI-v1 JSON returns `SarganResult` with test convention, assumptions, N, endogenous
and excluded-instrument names, DF, statistic and upper-tail p-value. Scripts and
`--text` expose the same owned results. Endogeneity tests are described below;
clustered overid and GMM remain separate work. See the reproducible external-data
example under `examples/cigarettes_iv/`.

### Heteroskedasticity-robust overidentification score

After `ivregress 2sls ..., robust`, `estat overid` returns
`RobustOveridScoreResult`, a Wooldridge residualized-instrument score test.
Let X be the original regressors, Z the full instrument design (both including
any intercept), Xhat=P_Z X, L=rank(Z), K=rank(Xhat), and q=L-K. Residualize Z on
Xhat and choose exactly q independent directions Q from that complement. Regress
ones on `diag(u)Q` **without an intercept**, using retained structural 2SLS
residuals u. The statistic is N-RSS, computed as the fitted projection norm to
avoid cancellation. It has an **asymptotic chi-square(q)** reference under the
null and conventional strong identification.

The uncentered score OPG is **uncorrected**: `covariance_correction=1`, not the
parent coefficient HC1 N/(N-K) adjustment. The projection removes the first-order
2SLS nuisance-estimation effect; testing all raw moments with an arbitrary weight
is not equivalent. This is not Hansen J, classical Sargan, Basmann, a finite F,
cluster/survey inference or a weak-IV-robust procedure. Independent rows, the
correctly specified maintained model and valid instruments under the null are
required. Failure to reject does not prove instrument validity. Rejection may
reflect invalid instruments or misspecification, not a named instrument's guilt.

The result reports ordered endogenous/instrument names, N, L, K, q, correction,
statistic and upper-tail p, with the same text/script/JSON reachability. The
original fit sample, design and structural residuals are retained; there is no
current-data scan, structural refit or data/model mutation. Full-rank
Z/Xhat and a rank-q score design are required. The complement uses unit-scaled
instruments and pivoted twice-reorthogonalized directions with a 1e-12 relative
norm rank bound; score columns are separately unit-scaled before QR. Rank is not
silently reduced. Just-identification, insufficient auxiliary DF, nonfinite or
zero structural residuals and singular score spaces fail explicitly. Clustered
parent fits remain rejected rather than using an independent-row reference.

See the [documented score construction, p23](https://www.stata.com/manuals/rivregresspostestimation.pdf)
and `examples/cigarettes_iv/robust_overid.td` for independent base-R method
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
Endogenous nuisance coefficients and robust/cluster confidence sets are not implemented.
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
No heteroskedastic/cluster confidence sets or endogenous-subvector/nuisance
projection are implied by this single-endogenous form. A separate projected form
is available for multi-endogenous fits below.

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
nuisance curvature. Robust/cluster projected confidence sets and a less
conservative subvector AR test are not implemented. The command uses the
retained estimation sample and does not change session state.


Links:
- `SPEC.md`

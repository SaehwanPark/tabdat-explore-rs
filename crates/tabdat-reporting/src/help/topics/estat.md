# estat

How to invoke:
`estat vif`
`estat ar, null(b1 b2 ...)`
`estat ar, ci [level(95)]`
`estat overid`
`estat endogenous`
`estat margins` (after logit or probit)

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
- Other `estat` forms may parse, but are not installed in this bounded runtime or admitted by CLI-v1 JSON. The supported forms are `vif`, `ar`, `overid`, `endogenous` and binary `margins` (described below). Generic table serialization alone does not admit additional commands. Direct `sql` SELECT/WITH queries are independently admitted as `TableResult`; `sql ... into name` returns `SqlCreateResult`, and `use name` returns `ActivateResult`. See `help sql` for supported forms and native validation/state limits.

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
multiway-cluster/weighted effects, level option, categorical/discrete-change or
other model-family margins are installed. The retained fit decides which family
runs; with no binary fit the margins call fails with the established prior-logit
message. Restore/fit a successful logit or probit after a failed new estimation
attempt; see `help logit`, `help probit` and `help predict`.

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

Just-identified and robust/cluster parent fits are rejected explicitly, as are
zero/nonfinite structural residual variance and auxiliary rank/DF failures.
Sargan uses the retained sample/design/residuals even after current data changes;
it does not refit 2SLS, rescan current columns or mutate dataset/model state.
CLI-v1 JSON returns `SarganResult` with test convention, assumptions, N, endogenous
and excluded-instrument names, DF, statistic and upper-tail p-value. Scripts and
`--text` expose the same owned results. Endogeneity tests, robust/cluster overid
and GMM remain separate work. See the reproducible external-data example under
`examples/cigarettes_iv/`.

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
Hausman, endogenous-subset, or robust/cluster variant is implied.

The conventional reference requires valid instruments, homoskedastic errors,
correct specification and standard strong-identification regularity. Exact
finite-sample F additionally requires conditional normality. Rejection of joint
exogeneity is conditional on these assumptions; invalid instruments or a
misspecified model can invalidate that interpretation. Failure to reject does
not prove exogeneity, instrument validity or weak-IV-robust coefficient inference.
Robust/cluster parent fits are rejected, not silently converted.

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

## Joint Anderson–Rubin test

After a **nonrobust** `ivregress 2sls` fit, run for example:

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
weak identification, **not** heteroskedasticity or clustering, and it does not test
instrument validity. Robust/cluster parent fits are rejected, not silently
converted to classical inference. Undefined auxiliary variance, rank or residual
DF failures are explicit errors.

The retained complete-case fit sample/design is used even after current data is
changed; AR does not refit 2SLS, reread current columns or mutate model/data state.
`--text`, scripts and CLI-v1 JSON expose the joint null, endogenous names/order,
excluded instruments, N, both DF, F, p-value and assumptions (`AndersonRubinResult`).
A small p-value rejects the **entire supplied joint null** under these assumptions;
it does not identify which endogenous coefficient differs.
Endogenous nuisance coefficients and robust/cluster AR are not implemented.
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
projection are implied. In particular, this does not provide a schooling-only
confidence set from the three-endogenous DATA66 specification.


Links:
- `SPEC.md`

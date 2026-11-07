# estimates

Installed syntax:
- `estimates store NAME`
- `estimates store NAME, covariance(working)` or `covariance(sandwich)` (Gaussian GEE only)
- `estimates table NAME [NAME ...]`
- `estimates drop NAME [NAME ...]`

Keep immutable reporting snapshots of successful current unweighted OLS, logit,
probit, Poisson, explicit-formula NLS, model-based FE/RE fits, or an explicitly
selected Gaussian GEE inference, and compare them without refitting. Example:

```
regress outcome score
estimates store linear
logit outcome score
estimates store binary
estimates table linear binary
```

Names are case-sensitive ASCII identifiers, 1–32 bytes (letter/underscore first,
then letters/digits/underscores). At most 32 reports, 64 coefficients per report,
and 1–8 unique explicit names in a table; its coefficient-name union cannot
exceed 64. No implicit all/latest, wildcards, formatting options or abbreviations.
A name is never silently replaced: deliberately `estimates drop NAME` before
reusing it. A multi-name drop validates the entire request before deleting any.

Store copies selected coefficients, SEs, test statistics/p-values, full selected
covariance/parameter order, fit-time sample positions/counts/groups and applicable
fit statistics. It does not store predictor designs, fitted values or residuals.

Panel reports retain entity name, optional key name, group count and intercept
convention in additive optional JSON `panel` metadata. FE requires model-based
covariance and available classical normalized common-intercept inference; old
states without it require refitting. It copies within slopes and uses the same
retained normalization/linear-combination inference as native FE reporting for
the common intercept: Student t(N-G-p), not pooled inference. Native `_cons`
(FE/GEE) is aligned to `intercept` in comparisons; conflicting predictor names
are rejected rather than silently merged. FE uses observation-weighted zero
effects; RE/GEE population-mean intercepts have a distinct convention.

Model-based RE reports copy native GLS/ML beta covariance and **normal z**,
never quasi-regression t. Swamy–Arora/equal-group-between/marginal Gaussian ML
are labelled separately. Only ML supplies its actual marginal Gaussian
likelihood; GLS does not inherit a transformed-OLS likelihood. Clustered FE/RE
report snapshots remain unsupported (native clustered fits/predictions still work).

Gaussian GEE always retains both inference variants, so **plain store is an
error**, not an implicit default. Choose `covariance(working)` for inverse working
bread or `covariance(sandwich)` for the uncorrected unit sandwich (multiplier1).
Both use asymptotic normal z, not G-1 t or CR1. This selector is invalid for other
estimators. It does not refit GEE, force positive working correlation or select
a covariance for subsequent active-model operations. Missing provenance,
malformed dimensions and weighted panel states fail without registry effects.

NLS reports additionally retain the explicit typed mean expression and its ordered
parameter/predictor bindings, its Gauss–Newton covariance correction, plus
centered response-scale R-squared when defined. NLS inference assumes
independent rows and the stated conditional mean; working covariance additionally
assumes homoskedasticity, while HC1 permits heteroskedasticity.
NLS `include_intercept` is false because no intercept is inserted implicitly;
an intercept-like term may still appear in the explicit formula. Transformed
responses and scale columns remain named as fitted, not relabeled as weighted
original-scale estimates. Generic WLS/diagonal-GLS/IV and unknown linear
estimator provenance are rejected, not guessed as OLS; native RE-GLS above is a
distinct supported model.
Poisson reports retain the fixed offset variable (coefficient one), count-likelihood
coefficient, null-model likelihood/convention, zero/total counts, McFadden
pseudo-R-squared and selected covariance
correction. Independent-row robust uses N/(N−1); clustered uses G/(G−1).
Both use asymptotic normal inference, not G−1 t or a few-cluster guarantee.
The offset remains fixed in the null model. Count likelihood is descriptive
under QMLE and its LR reference is not robust.
Old serialized linear states without estimator metadata remain readable but must
be refitted before storing. Ambiguous duplicate parameter names are rejected;
rename a predictor that conflicts with the intercept name and refit.

Tables preserve requested model order and align coefficients by exact name in
first-seen order. Missing terms/statistics are NA/null, never zero. Binary
and Poisson statistics are normal z; OLS statistics use Student t with residual df, or G−1
for one-way clustered OLS; NLS uses its approximate Student t with N−P df.
Selected covariance and reference are disclosed per column. No model is refitted
and no cross-model test is made; FE common-intercept inference uses its retained
normalized model, never current data.
OLS R-squared, NLS centered response-scale R-squared and binary/Poisson
McFadden pseudo-R-squared are different measures; transformed NLS R-squared refers to
the transformed response. Panel reports leave the common OLS-labelled R-squared
row unavailable rather than substituting within/conditional/quasi-GLS measures.
No panel key asserts calendar-time semantics. Gaussian marginal ML likelihood
cannot be compared as if it were GEE or FE likelihood.
Gaussian working-model RSS/N, Bernoulli and Poisson count likelihoods are descriptive, not a
cross-family or robust likelihood-ratio test. Zero/nonfinite OLS variance has
no reported likelihood or convention.

Reports survive current-data edits/reloads, refits and failed fits. Storing,
reading or dropping reports does not change the active estimator, predictions,
AME, data, labels or row order. A failed new fit still clears the active model
under existing semantics; it does not delete named reports. Names are local to
the current session and are not written into saved datasets.

Samples/outcomes may differ between columns: equal N does not prove equal rows.
Sample positions refer to candidate rows at fitting time, not current-data IDs;
no dataset fingerprint, staleness test or current-data correspondence is implied.
JSON includes full coefficients/selected covariance/sample provenance; terminal
output shows the aligned coefficient/standard-error/statistic comparison, NLS
formula and model conventions.
Both contain potentially sensitive names/sample/group metadata; review before
sharing. Terminal output is tab-separated with six-decimal display values, not
a configurable publication-layout or exact historical Stata formatting surface.

These are reports, not restorable model caches. `estimates restore`, disk
persistence, arbitrary post-estimation on a stored name and general Stata
estimates/options parity are not installed. Continue prediction or margins using
the ordinary active model, or explicitly refit it when needed.

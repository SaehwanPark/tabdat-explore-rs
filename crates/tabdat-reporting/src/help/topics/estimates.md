# estimates

Installed syntax:
- `estimates store NAME`
- `estimates table NAME [NAME ...]`
- `estimates drop NAME [NAME ...]`

Keep immutable reporting snapshots of successful current unweighted OLS, logit,
probit, or explicit-formula NLS fits, and compare them without refitting. Example:

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
NLS reports additionally retain the explicit typed mean expression and its ordered
parameter/predictor bindings, plus centered response-scale R-squared when defined.
NLS `include_intercept` is false because no intercept is inserted implicitly;
an intercept-like term may still appear in the explicit formula. Transformed
responses and scale columns remain named as fitted, not relabeled as weighted
original-scale estimates. WLS/GLS/IV and unknown estimator provenance are
rejected, not guessed as OLS.
Old serialized linear states without estimator metadata remain readable but must
be refitted before storing. Ambiguous duplicate parameter names are rejected;
rename a predictor that conflicts with the intercept name and refit.

Tables preserve requested model order and align coefficients by exact name in
first-seen order. Missing terms/statistics are NA/null, never zero. Binary
statistics are normal z; OLS statistics use Student t with residual df, or G−1
for one-way clustered OLS; NLS uses its approximate Student t with N−P df.
Selected covariance and reference are disclosed per column. No coefficient inference is recomputed, and no cross-model test is made.
OLS R-squared, NLS centered response-scale R-squared and binary McFadden
pseudo-R-squared are different measures; transformed NLS R-squared refers to
the transformed response.
Gaussian working-model RSS/N and Bernoulli likelihoods are descriptive, not a
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

# Classical IV/2SLS contract

In progress, base 03b98ac. Finite complete strong-instrument synthetic fixture:
outcome, one exogenous variable, one endogenous variable, two excluded instruments
and intercept. Exact ordered source sample and coefficient labels; owned full
covariance/SEs and R-squared. Python uses linearmodels IV2SLS classical unadjusted
covariance **without df correction**. Native tsls must explicitly select OPT_N.
Independent R base QR/projection implementation uses original structural residuals,
not second-stage fitted-X residuals, SSE/n and inverse X'PzX. Predeclared abs 1e-10
+ rel 1e-9 for all numerical fields; exact metadata and sample.

No robust/cluster/GMM/LIML/post-estimation or missing sample port. Validate both
sides of the IV list separator and all dataset indices before C; preserve opaque
owned by-value model handling, serialized native calls/destructors, errors and
recovery. No new plugin/dependency/license. Existing native limits remain recorded
while acceptance work continues.

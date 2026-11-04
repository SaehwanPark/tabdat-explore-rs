# Classical FE normalized common-intercept inference

## Preimplementation contract — in progress

Target C21-P1-FE author first-program lines201–205/Table21.2, original5,320-row
MOM panel/532 people/1979–1988, unweighted lnhr on lnwg plus person intercepts.
General native scope remains existing unweighted one-way FE with joint-finite
sample, arbitrary supported slope count, balanced/unbalanced groups/singletons.
Normalization remains sum(n_g*u_g)=0 and a=grand_mean_y-grand_mean_x' beta.

For classical iid homoskedastic disturbances conditional on retained units/design,
residual DF=N-G-p and sigma2=within_RSS/DF. Preserve the existing slope covariance
Vb; Cov(a,beta)=-mean_x' Vb, Var(a)=sigma2/N+mean_x' Vb mean_x. Retain the full
joint (a,beta) covariance as an optional existing PostEstimationModel; no new
numerical library or framework. Student-t inference uses N-G-p. Native classical
_cons combinations/restrictions become usable; existing slope-only result arrays,
DF, point predictions, effects and cache semantics are unchanged. Unit-cluster
common-intercept inference is explicitly withheld, not manufactured by applying
a slope sandwich or a classical variance to it. Preserve legacy fits if new
normalized inference is numerically unrepresentable; expose an unavailable reason.

Acceptance:
1. Independent full weighted-zero-sum dummy QR on balanced/unbalanced/multiple-
   predictor/singleton controls validates all joint covariance entries, intercept
   SE/t/p/95 CI, common+slopes lincom and joint F restrictions; original MOM gets
   the same source-bound check and readable/CLI-v1/report/prediction workflow.
2. Owned classical normalized inference survives data activation; clustered fits
   retain existing slope G-1 inference and explicitly withhold common inference.
   Existing p-by-p slope covariance/parameter names stay unchanged; new fields
   are additive. Preserve malformed/rank/DF failure state and original predictions.
3. Missingness/exact sample keys, predictor/response shifts, degenerate zero RSS,
   nonfinite normalized covariance and parameter-name collision are covered;
   no tolerance widening or hidden variance clipping. Run focused tests first,
   affected crates/reverse consumers and relevant end-to-end contracts before merge.

No weights, robust common-intercept SE, individual-effect intervals, new panel
likelihood/statistics, generic FE snapshots or cross-family restrictions. This is
not the sampled-person pairs bootstrap target. Classical exact t/F references
require normal iid errors; without that law they are working references, not
causal or arbitrary within-unit-dependence guarantees. Historical float32/output
parity and whole-table/chapter/book closure are not claimed.

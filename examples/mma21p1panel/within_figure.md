# WIP: Chapter 21 Figure 21.3 within-person source-data adaptation

## Bounded contract before implementation

Deliver Figure21.3 (§21.3 p.712) as original-source load/guard → joint-finite
sample → native fixed-effects fit → declared within-plus-overall-mean data →
linear fit/ordered-row LOWESS → diagnostics/CSV/SVG workflow. Pinned `MOM.dat`
SHA-256 `2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77`
contains 532 people × 10 years; all model fields are finite. The author uses
`xtdata, fe`: the [current Stata manual](https://www.stata.com/manuals/xtxtdata.pdf)
says this transformation adds the observation-weighted overall mean to each
person-demeaned variable. Plot `lnwg−mean_i(lnwg)+overall_mean(lnwg)` on X and
`lnhr−mean_i(lnhr)+overall_mean(lnhr)` on Y. For incomplete inputs use only
jointly finite X/Y cases to compute **both** within-person and overall means;
this differs from `xtdata` casewise deletion across all selected nonmodel fields.

As in the validated between figure, require eligible loaded X/Y values equal
two-decimal rounding and within ±1,000,000, compute source-scale sums as exact
`DECIMAL(18,2)`, divide once in float64 and perform `(raw−unit_mean)+grand_mean`
in that explicit order. Only bitwise-equal resulting X values tie; CSV loading
can already lose excess lexical precision. Define original-key stable tie
order, explicit span0.8 nonrobust centred-rank LOWESS and existing23M cap.
Historical Stata float/tie/WMF parity is unresolved, not silently equated.

Report native FE with person-cluster covariance on the same original eligible
sample; normalized FE intercept is a point only. The plotted straight line is
a separate cluster-OLS fit on transformed observations; it has the same slope
but its covariance correction treats the transformed regression's intercept as
an OLS parameter, so its inference must not be labelled identical to native FE.
After data transformation and the auxiliary fit the retained model is OLS,
not FE or LOWESS. Neither within transformation nor smoothing establishes
causality. No confidence band, new-X smoother, individual FE-intercept SE,
general `xtdata` or full chapter parity.

Independent development-only R integer-hundredth unit/overall sums, `plm`
FE and independently computed full clustered score covariances for both fits,
plus weighted local QR; Python source/transform/normal-equation and actual SVG
geometry audit. Predeclare `1e-11+1e-10*abs(reference)` numerical tolerance,
nonzero p-values relative1e-9 and QR certificates<1e-12. Check all 5,320 source
and transformed rows, 5,320 points/10,640 connected vertices. Test balanced and
unbalanced/tied synthetic input, joint missing and nonmodel missingness,
reversed order, no-clobber, collisions/precision/singular/key failures, retained
state and forged shared-wrong values/geometry. No production R/Python.

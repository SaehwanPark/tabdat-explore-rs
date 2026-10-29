# WIP: Chapter 21 Figure 21.2 between-person source-data adaptation

## Exact outcome and pre-implementation contract

Implement the Figure 21.2 (§21.3, p.712) between-person illustration from the
[March 2005 author program](https://cameron.econ.ucdavis.edu/mmabook/mma21p1panfeandre.do),
a complete prepare → person means → pooled-on-means linear fit and ordered-row
LOWESS → diagnostics → CSV/SVG/report workflow. Pinned `MOM.dat` SHA-256
`2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77` contains
532 people × 10 years (1979–88); all model variables are finite. Each person
receives one equally weighted mean of ln annual hours (Y) and ln hourly wage
(X), no other regressors. The author uses `xtdata, be` before plotting.
The current [Stata xtdata manual](https://www.stata.com/manuals/xtxtdata.pdf)
specifies between-data conversion and a casewise-deletion rule across included
variables; on this complete original source, those means coincide with jointly
finite X/Y means. Do not claim equivalence on arbitrary missing nonmodel fields.

Use existing native SQL, classical intercept OLS, native `lowess` explicitly
span 0.8, and scatter with line overlays. Stable person-ID order before LOWESS
and stable X/person-order line geometry define tie handling. Native centred-rank
local linear tricube smoothing is nonrobust, unlike an assumption that
`stats::lowess` is numerically identical; historic Stata tied order and native
float64 versus historical storage remain unresolved. Scatter points are 532
people, not 5,320 person-years. Include no confidence bands or causal claim;
OLS working covariance assumes independent homoskedastic person means and
uses t(530), while the figure itself is descriptive. Keep the retained native
model as between OLS, not LOWESS.

Independent development-only R QR fits each declared rank window and R `lm`
checks coefficients/covariance plus all source and person-mean rows. A separate
Python direct normal-equation audit verifies numerical results and all actual
SVG marker/connected-curve geometry at fixed `1e-11+1e-10*abs(reference)`;
nonzero p-values use relative `1e-9`. Do not widen tolerances to hide differences.

Acceptance: (1) all original 5,320 source rows and 532 means/fits, QR metadata,
CSV and 532 markers/1,064 vertices are checked; (2) an inspectable small fixture
with tied means agrees, including shuffled input and JSON/text byte-identical
exports; (3) test casewise missingness, invalid/duplicate IDs, name collision,
constant/singular design and no-clobber input/output invariants; (4) preserve
prior OLS state through smoothing/export/labels/plot, with script nontransactional
side effects documented; (5) reject forged shared-wrong values and SVG geometry.
Require no production R/Python. No exact historical tie/pixel parity, general
between/graph API, full chapter or other Figures21.1/21.3/21.4 claim.

## WIP finding — deterministic source-precision means

The initial floating AVG and R mean differed by up to `8.89e-16`. These tiny
differences changed 44 rank positions around mathematically tied wage means;
one LOWESS fit differed by about `0.000136`, well beyond the fixed tolerance.
This is not a tolerance problem: centred-rank LOWESS depends on exact ordering.

All original finite model values have two decimal places. The revised bounded
workflow rejects eligible values beyond ±1,000,000 or with more than two decimal
places (rather than silently rounding), computes sums exactly as fixed-scale
decimals and divides once in float64. Independent R and Python sum integer
hundredths and apply the same division; reference exports retain 17 significant
digits. The revised original-source audit passes all 5,320 exact source rows,
532 person means, ranks and QR/normal-equation fits, 532 SVG points and 1,064
vertices. Native and R person means and X rank order match **exactly**; maximum
LOWESS difference is about `2.0e-14`. This source-specific domain is not a
claim about general arbitrary-precision panel means or historical Stata tied
ordering. Synthetic/missingness/failure controls, regression tests and complete
documentation remain WIP.

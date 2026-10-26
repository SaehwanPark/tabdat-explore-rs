# WIP: PSID Epanechnikov bandwidth-sensitivity workflow

## Bounded outcome and source

Cameron–Trivedi (2005), §9.2.1 pp.295–297, Figure9.2; March2005
`mma09p1np.do` blocks for preparation/plugin bandwidth/three kernel densities.
This required Chapter9 illustration opens a missing analytical family. Reuse
native SQL/table/export and SVG line composition rather than introduce a new
estimator API, command or production dependency for a fixed descriptive analysis.
This is not installed `kdensity`, LOWESS, histogram or whole-Chapter9 support.

Sources: https://cameron.econ.ucdavis.edu/mmabook/mmaprograms.html and
https://cameron.econ.ucdavis.edu/mmabook/mmadata.html. Original assets remain
external; acquire `psidf3050.dat`, `mma09p1np.do` and `mma09p1np.txt` explicitly.
SHA256 pins:
- data: `2f24024edd599e74305924165e981790f96badb5f03048587a1b57cb74ce8346`
- program: `fdd77b2a381aa4f62c41ea570388764b4f20a54ce0c2fd91af8a83bd8d525ae4`
- log: `7f126f34b82db86480c3fe47e86e241c74271b650bd7e708757785ef559c01f8`

Original4856 fixed-width records/nine fields, females30–50; retain positive
earnings, age36, education neither missing nor0/98/99:177 rows. Topcode earnings
at100000, derive hourly wage and log wage. Preserve raw fields and source ordinals;
education controls the author's cohort despite not entering the density. Kids
are births, NOT current household children, and are not used. Marital coding is
irrelevant here and will remain raw rather than advertise a recoding workflow.

For selected finite log wages x_i, KDE f(x)=sum K((x-x_i)/h)/(N*h),
K(u)=.75*(1-u²) for |u|<1,0 otherwise. This is conventional Stata `epan2`, NOT
Stata's variance-normalized `epan`; bandwidths differ by sqrt(5).
Use sample SD (N−1), type7 linear quartiles and
s=min(SD,(q75−q25)/1.349), h=1.3643*1.7188*s/N^.2, then h/2,h,2h.
For original177 rows the quartiles also equal Stata's order statistics45/133.
No weights, conditional regression, uncertainty or density-model test is implied.
Reject insufficient/degenerate samples and nonpositive/nonfinite selected hours
or derived quantities before exports. Bound raw input10000 rows and fixed512
common grid points. Grid covers [min(x)−2h,max(x)+2h], the full three-curve support;
it is explicit, not a claim about undocumented historical Stata grid defaults.

Precision is deliberate: native double calculations give h=.5453854422894842.
Stata's two default-float generated columns give h=.5453854164300618, matching
printed .54538542. Likewise SD .8281614360257001 versus .8281614349441063.
Do not widen tolerances or force float32 into production to hide this known
generated-storage difference. Audit both paths separately and preserve source
integers exactly; source data itself needs no DTA conversion.

## Acceptance and independent reference (before implementation)

1. Actual JSON/text script produces identical selected-source/bandwidth/density
   CSVs and three-line SVG; original177 membership/source ordinals are exact.
   All1536 grid ordinates and every rendered vertex match independent R/reference
   and independent standard-library reconstruction.
2. R4.6.1 direct vectorized kernel sums, sample SD/type7 quantiles validate all
   primary values (1e-11 absolute+1e-10 relative; source integers exact).
   Analytic kernel normalization/direct piecewise-quadratic integration checks
   total mass1; `stats::density` FFT at increasing resolution is a secondary
   approximation check, not an excuse to relax primary exact-formula tolerances.
3. Source audit pins assets, reproduces original177-row selection, verifies
   original printed cohort summaries/bandwidth on a separately reconstructed
   float32 path and documents native double differences explicitly.
4. Small synthetic fixture covers excluded education/age/zero earnings, topcoding,
   NULL raw education and all three curves. Shared-R/native wrong normalization,
   source membership, bandwidth and SVG geometry forgeries must fail audit.
5. Selected invalid hours/constant or empty cohort/malformed source/duplicate
   ordinals/existing output fail with input files unchanged; script is ordered,
   not transactionally rolling back earlier commands or already-written outputs.

## Status / exclusions

Source/sample/precision feasibility resolved; implementation, R/frozen controls,
actual native/source/geometry audit and proportional regression verification are
pending. Keep PR draft until this full bounded workflow is verified. No new
native command/parser claim, general KDE API, weights, automatic bandwidth
selection beyond the declared rule, confidence bands, Figure9.1 histogram,
Figure9.3 LOWESS/Figure9.4 other kernels, Figures9.5–9.7/generated-data derivatives,
kNN/kernel regression or whole-Chapter9/book/platform completion. Those required
analyses remain separate; the full empirical inventory is still incomplete.

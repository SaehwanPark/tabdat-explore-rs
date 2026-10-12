# Classical IV/2SLS contract

Bounded fixture validated on PR #172, base 03b98ac. Finite complete strong-instrument synthetic fixture:
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

## Results / diagnosis

Native coefficients, all SEs/VCV entries and normalized structural R² match
pinned Python and independent R to maximum abs differences 6.67e-16 and
2.23e-15 respectively; exact rows 0..159 and labels intercept/x/d pass.
Initial coefficients/covariance passed, but native Gretl R² was 0.9088033291
versus structural Python/R 0.8714529581: Gretl uses a correlation diagnostic.
The owned Rust facade now computes 1-SSE/TSS from original structural data,
centering with the explicit intercept. No tolerance or reference changed.
No-intercept/implicit-constant diagnostics are not reference-validated here.

Synthetic seed 6141, n=160, endogenous error correlation .6, two strong excluded
instruments; data SHA-256
`0cb78159dfbacfbff04ecb22d5ed17d8684d75a8418f5d5345994b8c94afdc60`.
Oracle commit/tree/lock and external Python/R versions follow the
[Poisson provenance](poisson-plugin.md); Python linearmodels IV2SLS,
R 4.3.2 independent QR projection and structural variance calculation.
The probe uses actual pinned syntax `ivregress 2sls y x, endog(d) iv(z1 z2)`.
R first projects X with QR(Z), then solves X'PzX and uses original residuals;
it does not mistakenly use OLS residuals from regressing on fitted X.

Three added tests cover full nominal references, bad/missing/underidentified
inputs and recovery, and invalid low-level lists (outcome zero, wrong/multiple/
missing separator, empty side, invalid variable indices) rejected before C.
Encoded list length, dimensions and finite outputs are checked. LISTSEP=-100 is
verified in pinned gretl_list.h. By-value tsls model storage and materialized VCV
stay in the shim; one adjacent-reasoned unsafe call is confined to gretl-sys.
No borrowed dataset/model state or raw native type escapes; calls and destructors
use the existing native mutex/once lifecycle. No new license/module/dependency.

22 feature tests pass on macOS and fresh Linux x86_64 emulation; default static
suite has 17 tests. Locked checks/Clippy, audit/deny and docs validation are run
before merge. Three owner review passes: covariance/diagnostic semantics, IV
list/ownership/error boundary and policy/platform scope; no independent review.
No robust/GMM/post-estimation/product runtime or whole-phase acceptance inferred.

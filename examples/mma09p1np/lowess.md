# WIP: native centred-rank LOWESS source-data workflow

## Exact bounded outcome

Make existing `lowess y x, gen(newvar) [bandwidth=0.8]` executable end-to-end:
load/prepare → specify nonrobust row smoother → publish current smoothed column →
compare bandwidths/interpret/plot → export/report/reproduce. Source-data example:
MMA2005 §9.2.1 p297 Figure9.3's PSID177-row cohort and spans.8/.4/.1, prepared
as in the [density workflow](README.md). This delivers an explicitly defined
native smoothing method and source-data adaptation, NOT an unexplained promise
of historical Stata tied-point geometry or whole-figure/chapter completion.

Risk: **high** — new statistical/data publication and structured-result boundary.

## Algorithm and statistical decisions (before implementation)

[Stata documented methods](https://www.stata.com/manuals/rlowess.pdf), p6:
within the retained joint-finite y/X sample, stably order by numeric X then
original eligible-row order; signed zeros tie. k=floor((N*bandwidth−.5)/2), window
[max(0,i−k),min(N−1,i+k)], shrinking at boundaries. Let d be the maximum X-distance
to the window endpoints, z_j=(X_j−X_i)/d and
w_j=(1−(|z_j|/1.0001)^3)^3. Weighted local intercept/linear fit predicts at z=0.
No robust reweighting iterations or interpolation between fitted observations.

If d=0, every window X is identical: use its local arithmetic mean (degree0),
not an arbitrary slope, ridge or pseudoinverse. A supported k=0 returns the
original eligible Y exactly. Require2<=N<=10000,0<bandwidth<1,N*bandwidth>=.5
(k must be nonnegative) and N*min(N,2k+1)<=20000000. Unsupported tiny bandwidths
error, not silent clipping. Compute local centred/radius-scaled X and window-local
scaled Y with compensated sums; nonfinite local differences/unrepresentable
requested fits fail. Positive-radius windows must have positive finite centred
weighted variance. The1.0001 endpoint factor gives positive endpoint weights;
do not substitute hard-zero endpoints or an arbitrary singularity threshold.

Ties are IMPORTANT:62 of177 source rows have education12. Rank windows can give
different fits to equal X, depend on within-tie order, and change under row
permutations within ties. This is an ordered row smoother, NOT a unique
out-of-sample function of X. Preserve original row order when publishing and
explicitly report stable tie/boundary/fallback conventions. R stats::lowess uses
nearest-neighbour windows (and defaults robust iter3/delta interpolation); even
iter0/delta0 is NOT this reference algorithm. For source X3 at span.8, documented
centred-rank gives about2.096276 while R nearest-neighbour gives1.864884.

The original author's WMF confirms the same raw cohort/coordinates but its tied
curve order does not establish the old internal sort policy. Current Stata manual
is not proof of every Stata8 detail. Do not widen tolerances or claim exact
historical tied-vertex reproduction. Keep that historical detail unresolved where
relevant; deterministic native conventions and independent method validation are
separate evidence. No product-scope exclusion or full textbook closure is inferred.

## Native boundary and reference

Joint-finite numeric Y/X rows enter; excluded current rows receive NULL. Training
row smoothing requires Y, unlike ordinary current-X prediction. New typed DOUBLE
column only, never replace Y/X/another target. Calculation/validation/collision
failures preserve data/schema/order/labels and cached fit; compute all values
before existing typed staged publication. The command does not install/clear an
estimation state: later predict/test still refer to any PREVIOUS retained model,
not this smoother. No uncertainty, covariance, DF, p-values or causal meaning.

Reuse existing crates: pure scalar smoother in tabdat-stats; domain LowessResult/
LowessFailed; execution dispatch/extraction/atomic publisher; reporting text/CLI-v1/
help/admission metadata. AST spelling/effects/required active dataset are already
installed. New structured result/admission are additive; no production dependency,
new crate, generic smoother framework or independent data engine. Share existing
native PSID preparation between density and LOWESS only if the public nested-script
path supports it; explicitly replay changed density tests if refactored.

R4.6.1 `lm.wfit` independently solves each normalized weighted design by QR,
including intercept-only coincident windows; tol1e-12 on radius-normalized design,
not a ridge. Normal-equation residual certificates must pass1e-12. Native closed
weighted moments differ from QR. Feasibility on original legacy-float inputs:
k70/35/8, coincident-window counts0/0/63, normalized QR scores<3.5e-17.
Freeze only synthetic unordered/tied controls; original source stays external.
Compare all fits/sample positions/metadata with1e-11 absolute+1e-10 relative;
source integers exact, no reference replacement/tolerance widening to hide gaps.

## Acceptance and ordered delivery

1. Commit this contract/ledger and open draft PR early; add reproducible independent
   R controls before kernel implementation. Include asymmetric boundaries,
   ties/all-X-tied, k0 identity and the deliberate R-nearest-window difference.
2. Pure kernel: match all frozen QR fits/metadata; reproduce lines, positive X
   affine and Y affine transformations; preserve unsorted input order/signed-zero
   ties. Reject shape/nonfinite/N/span/work/overflow errors; no silent clipping.
3. Native runtime/current column/text/CLI-v1/help: original-position joint missing
   sample/NULLs, repeated targets/case-insensitive collision/type/unsupported numeric
   failures atomic; preserve labels/order/previous owned regression fit. Direct
   API invalid options must fail even if parser would reject them.
4. Actual scripts: synthetic and original177-row native three-span CSV/SVG in both
   CLI modes against independent QR/source/geometry audits; all531 vertices and177
   raw markers, no-clobber outputs/source preservation, shared-wrong native/R
   window/fallback/order and actual-path forgeries rejected. No production R/Python.
5. Run new/modified tests explicitly, affected crates/relevant reverse consumers,
   proportional Clippy/fmt/diff checks and bounded independent review. Update SPEC/
   roadmap/ledger honestly, then ready/merge/sync and safely remove branches.

Implement exactly this plan. Do not broaden scope. If the plan conflicts with the
codebase, stop and report the conflict instead of improvising. Report changed
files/tests/deviations/unresolved risks. Stop if independent QR/reference or typed
publication cannot support these semantics; do not disguise a fallback algorithm.

## Exclusions / status

Contract/reference feasibility only: native LOWESS remains parser-only until the
complete workflow is implemented/verified. No mean/noweight/robust/adjust/logit/
weights options, interpolation/delta acceleration, uncertainty, out-of-sample
predict/margins/estimator state, histogram/other kernels/general nonparametric
family, historical tied-sort/pixel parity, whole-book or platform/release claim.

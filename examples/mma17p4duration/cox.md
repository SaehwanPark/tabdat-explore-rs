# WIP: original Table17.8 Cox/Breslow duration column

**Status: pure Rust Cox/Breslow kernel and eight numerical controls pass. Native
stcox parser/runtime/state/reporting and original native script are NOT installed.
Independent kernel review and the complete end-to-end workflow are still required;
keep draft PR #334 open, do not merge a kernel-only slice.**

## Exact analysis and independent reference

Cameron–Trivedi (2005), §17.11 pp.603–608, Table17.8 Cox coefficient column;
March2005 mma17p4duration.do, original ema1996.dta. Same3343 spells/1073 full-time
events/20887 two-week intervals and40 regressors as the validated exponential/
Weibull columns, but NO intercept: the baseline absorbs any PH location. Keep
all102 all-zero status rows as censor1=0 and stateur from original DTA; ASCII is
not an equivalent full design. The source comment says Weibull before stcox;
executed stcox and output explicitly identify Cox/BRESLOW, not Efron/exact ties.

Development-only R4.6.1/survival3.8.6 coxph(ties='breslow',robust=TRUE) reproduces
original partial LL−7717.2334329144514/null−7981.9304249926163,24 failure knots,
reprate beta≈.5222796 and robust SE≈.5711698 after N/(N-1). Independent risk-set
information and martingale score residual reconstruction matches R full matrices
within3.6e-13 and2.2e-13 respectively; corrected Wald≈540.9797 matches author540.98.
Uncorrected R SE≈.5710844 does NOT match the author; correction is explicit, not
residual-DF HC1. Original source max raw score≈1.5e-11. The existing400-row full-
design control also admits a regular40-coefficient Cox fit. These are R-only
feasibility facts, NOT native execution/source validation.

## Bounded native behavior

Proposed installed syntax: `stcox time [xvars], failure(event) [robust]`.
Unweighted single-spell entry-zero right censoring, numeric time/exact binary
event. Joint finite time/event/X rows enter before positive-time support checks;
at least one failure. No explicit/implicit intercept, no factor expansion. An owned CoxProblem carries
time/event/design/sample: the generic location-model EstimationProblem rejects
zero predictors without an intercept. Do not fake a fitted intercept or weaken
that shared invariant to accommodate a Cox baseline-only problem.
Centered design rank must exclude constant directions, not merely raw-X rank.
N<=10000, K<=64, N>K, N*K*K<=20000000; robust additionally N>=2. Baseline-only
K0 has no coefficient tests/inference, not a fitted fake intercept.

Partial LL=sum_events X beta - sum_j d_j log(sum_{i:t_i>=t_j}exp(X_i beta)).
At a tied time, all failures AND censors stay in its risk set. This is the
Breslow tied approximation, not exact discrete-time conditional likelihood.
No duration-density/Stata-Weibull normalization, shape or cross-family AIC claim.
Use existing strict guarded concave Newton with centered/RMS X, per-N mean score
AND relative-step gates1e-10, undamped regular information, no ridge/pseudoinverse.
Time rankings/ties, not physical duration units, determine coefficient likelihood.

Model-based inverse partial-likelihood information or independent-spell FULL
martingale/efficient-score sandwich times N/(N-1), asymptotic normal95. NOT the
outer product of failure-only scores or naive X*(event-fitted_H). For risk mean
mu_j and Breslow increment a_j=d_j/S0_j, the row score is
u_i=d_i*(X_i-mu_at_exit) - exp(eta_i)*sum_{j:t_j<=t_i}a_j*(X_i-mu_j).
After time sorting/knot lookup, cumulative log-hazard and hazard-weighted risk
means permit O(N*K*K+N*K) moment work, not an unbounded N*failure_knots*K
residual loop; adaptive risk maxima prevent global-
max underflow in late risk sets. Full robust covariance/normal intervals/zero-
variance and rank-aware slope Wald rules are explicit. Null fixes all beta0 on
the SAME sample; LR p/reference withheld for robust fits, Wald separate.

Own baseline failure-time knots/risk counts/tied events/log increments/log H,
with baseline at the retained covariate-mean REFERENCE VECTOR (including dummy
means), not silently R's different default dummy-zero reference. Expose that
reference and nullable unrepresentable natural-scale baseline summaries; keep
finite log values for stable predictions. Baseline uncertainty is NOT installed.
No-covariate baseline cumulative hazard equals Nelson–Aalen, while exp(-NA) is
NOT Kaplan–Meier product survival. Pure baseline support is part of this same
baseline algorithm, not a second estimator family.

Current predict xb (bare default) uses current X only and is raw PH X beta.
Current cumhaz/survival use current X/time, never event; evaluate the owned
Breslow baseline step through t (right-continuous). Time0 is valid H0/S1; finite
negative time errors. Beyond the last failure the estimate holds its last step,
not a validated tail extrapolation. Missing/nonfinite required inputs yield NULL;
complete support/schema/target/numerical failures publish nothing and preserve
labels/order/model. Genuine survival0 allowed even if H cannot be represented.
At most10000 current rows, DOUBLE/NULL, no auxiliaries or uncertainty. No hazard
mode: baseline jumps do NOT identify an instantaneous continuous hazard. No
mean/count/binary/residual/margins meanings. New runtime fit attempts clear stale
inference only; parse failures are not executed fit attempts.

## Acceptance / failure cases

1. Full original sample/design/40 beta/all1600 covariance entries/complete normal
   and joint inference/partial full+null LL and all author coefficient/z/table
   facts against R and independently reconstructed risk/score/information.
2. Owned baseline all24 failure knots/risk/event counts/log and natural H, current
   baseline-indexed H/S/xb including coupled UI0/UI1 at observed times; every
   source/design/point cell, source restoration and JSON/text export identity.
3. Model-based/robust controls, baseline-only NA consistency, tied event/censor
   risk membership, permutation/time-unit invariance, late-risk dynamic scaling,
   finite-difference derivatives and full efficient-score covariance, not shared-
   wrong R/native agreement alone. Reference ordinary1e-9 absolute+1e-8 relative,
   positive tails1e-300+1e-8 relative; printed half final unit/source exact hex.
4. Actual script/CLI modes plus sample/missingness, current edits/drop event,
   time0/after-last points, rank/separation/no-event/unsupported-option failures,
   prediction support/overflow/schema/existing-target atomicity and stale state.
5. Complete parser→runtime→owned state→prediction→text/JSON/help/source reporting;
   focused then affected/reverse-consumer tests, quality checks, source/evidence
   docs/review and confirmed PR merge/cleanup before any next slice.

## Pure-kernel milestone (not application/source certification)

Eight focused tests pass. Independent R4.6.1/survival3.8.6 controls compare4233
model-based/robust numeric gates (including both1600-entry covariance matrices,
all28 control baseline knots/means/inference/null/LR/Wald), ALL16000 raw efficient
row-score cells and3600 unique original/UI0/UI1 point predictions exercised in
both covariance modes. Other controls cover finite-difference score/information,
adaptive late-risk scaling, baseline-only NA (not KM), tied censor/failure risk
sets, permutation/sample identity, time/affine-X units, negative last coefficient,
invalid support/weights/cluster/rank/separation/resource limits, current point
support and represented S0 despite H overflow. Unrepresentable natural baseline
summaries retain finite log values and can still give finite fitted H.

`cox_reference.R` is development-only. R hot-start polishing uses only its own
solution. Crucially, coxph(init=...) reports the INITIAL likelihood in loglik[1],
not the zero-beta null; the reference computes that null independently from tied
risk counts. Both original and control reference generators pass separate full
information/efficient-score/sandwich/baseline/point certificates. Adding score-
residual output preserves all earlier six artifacts byte-identically. Polished
original R raw score is below1e-12. Original R evidence is NOT original native
execution. Stats all-target Clippy -D warnings and changed Rust formatting/diff
checks pass; shared Newton algorithm is unchanged (only its family-support module
description now accommodates unrestricted Cox coefficients). Full affected/
reverse regression remains pending until the application workflow is complete.

## Scope and delivery order

Reuse existing crates, typed atomic column adapter, independent source acquisition
and full-design fixture; no production R/Python or new native/unsafe dependency.
Net cause-specific survival is NOT all-cause joblessness survival, competing-risk
incidence or causal UI effects. Independent-spell/model/external-censoring and PH
assumptions apply; no latent-cause independence assertion. No strata, cluster/
weights/survey, delayed/left/interval entry/censoring, frailty, PH-adequacy test,
baseline/prediction uncertainty, generic stset/sts, full Table17.9 or whole chapter.
Gompertz/heterogeneity/multiple-hazard workflows remain REQUIRED deferred work.

Order: early draft contract → independent R artifacts/kernel tests → runtime/
state/point/reporting → actual full original script/source audit → affected/
reverse tests and source review → honest capability docs/evidence → ready/merge/
synchronize/delete only this branch. Stop for substantive reference/statistical/
scope blockers rather than widening tolerances or merging parser/kernel only.

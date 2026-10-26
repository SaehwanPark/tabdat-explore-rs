# Original Table17.8 Cox/Breslow full-time re-employment column

## Exact analysis and native contract

Cameron–Trivedi (2005), §17.11 pp.603–608, Table17.8 Cox coefficient column;
March2005 `mma17p4duration.do`, original `ema1996.dta`. Same3343 spells/1073
full-time events/20887 recorded two-week intervals and40 covariates as the
[exponential](README.md)/[Weibull](weibull.md) columns, but **no intercept**:
the unspecified baseline absorbs every constant design direction. Keep all102
all-zero status rows as censor1=0 and stateur from original DTA. ASCII omits
stateur and rounds floats; it is not an equivalent full design. The author's
comment says Weibull before stcox; executed command/output identify Cox/Breslow.

```text
stcox time [xvars], failure(event) [robust]
predict index, xb
predict integrated_hazard, cumhaz
predict net_survival, survival
```

Unweighted single-spell entry-zero right censoring. Joint finite time/event/X
rows enter before positive-time and original-value exact-binary event checks;
invalid retained support errors rather than silently dropping rows. Original
DECIMAL event identity is checked after joint eligibility, before f64 rounding
can admit near-one codes. At least one failure, full centered-design rank,
N<=10000, K<=64, N>K, N*K*K<=20000000; robust also N>=2. No intercept/factors;
baseline-only K0 is supported without a fake coefficient. Its H equals
Nelson–Aalen; exp(-H) is NOT Kaplan–Meier product survival.

Partial LL=sum_events X beta - sum_j d_j log(sum_{i:t_i>=t_j}exp(X_i beta)).
Same-time failures AND censors enter each risk set: Breslow tied approximation,
not Efron/exact discrete conditional likelihood. No duration-density/Stata-Weibull
normalization or cross-family AIC claim. Centered/RMS coordinates and adaptive
per-risk-set maxima avoid late-risk underflow. Shared strict guarded concave
Newton requires per-N mean-score AND relative-step bounds1e-10 and regular final
undamped information; no ridge, pseudoinverse or clipping. Time rankings/ties,
not physical duration units, determine coefficient likelihood.

Model-based inverse partial-likelihood observed information or independent-spell
FULL efficient martingale-score sandwich times N/(N-1), asymptotic normal95.
For risk mean mu_j and increment a_j=d_j/S0_j, row score is
u_i=d_i*(X_i-mu_at_exit)-exp(eta_i)*sum_{j:t_j<=t_i}a_j*(X_i-mu_j).
Failure-only scores or X*(event-fitted_H) give the WRONG meat. Sorted risk moments
and cumulative weighted means use O(N*K*K+N*K) work, not N*failure_knots*K.
Zero-SE/rank-limited coefficient/Wald references are unavailable. Same-sample
null fixes every beta0; robust fits withhold LR p/reference, not the separate
selected-covariance Wald. Baseline-only fits have no coefficient tests.

## Owned baseline and current predictions

Baseline failure knots retain risk/tied-event counts, log increments/log H,
nullable unrepresentable natural values and exp(-H). Reference is ALL retained
predictor means, including dummy means, in predictor order—not silently R's
dummy-zero reference. Finite log baseline can still yield finite fitted H when
its natural reference value cannot be represented. Baseline uncertainty is absent.

Bare/default xb is raw current X beta and requires no time/event. Current
cumhaz/survival require X/time, NEVER event; use the right-continuous retained
Breslow step through t, including ties. Time0/before-first gives H0/S1. Beyond
the last failure holds the final step, not a validated tail extrapolation.
Required missing/nonfinite inputs yield NULL. Negative finite time, schema,
existing-target or numerical failure publishes nothing and preserves data,
labels, order and owned fit. Genuine survival0 is allowed even if requested H
would overflow. At most10000 current rows, typed DOUBLE/NULL, points only.

No instantaneous hazard from baseline jumps, residual/count/mean/binary/margins
meaning, uncertainty, weights/cluster/survey/strata, delayed entry, other
censoring/truncation, frailty, PH-adequacy test, general stset/sts, named storage,
full Table17.9 or whole chapter. Net cause-specific survival is NOT all-cause
joblessness survival, competing-risk incidence or causal UI effects. Independent-
spell/external-censoring and PH assumptions apply, not a latent-cause independence
assertion. Failed executed fits clear stale inference only; parse failures do not
execute or invalidate a fit. Gompertz/heterogeneity/multiple hazards remain required.

## Reproduce

Build `cargo build --locked` at the repository root. Source acquisition and
R/foreign conversion are explicit development-only steps, never production
requirements. Original assets are not redistributed. Numeric promoted-DTA values
are preserved; CSV does not preserve DTA storage types/value-label metadata.
Use a fresh directory (neither script nor reference producer replaces files):

```sh
repo=/absolute/path/to/tabdat-explore-rs
mkdir cox-analysis && cd cox-analysis
base=https://cameron.econ.ucdavis.edu/mmabook
curl -fsSL "$base/mma17p4duration.do" -o author.do
curl -fsSL "$base/mma17p4duration.txt" -o author.txt
curl -fsSLO "$base/ema1996.dta"
curl -fsSLO "$base/ema1996.asc"
Rscript "$repo/examples/mma17p4duration/prepare.R" ema1996.dta duration.csv
"$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma17p4duration/cox.td" > native.json
Rscript "$repo/examples/mma17p4duration/cox_reference.R" duration.csv reference ema1996.dta
python3 "$repo/examples/mma17p4duration/check_cox_source.py" \
  author.do author.txt ema1996.dta ema1996.asc . reference
```

The script guards the complete full source design, sorts source ordinals, fits
robust Cox, reports full coefficients/covariance/inference and reference baseline,
and exports observed/UI0/UI1 xb/H/S points at SAME times. Coupled UI and its two
interactions are restored before `cox_predictions.csv`:56 columns,44 source/
ordinal,three helpers,nine points. Final UI score balances are identities, not
adequacy tests. `--text` in another fresh directory containing duration.csv gives
byte-identical exports. Earlier successful script effects remain on later failure;
this is ordered execution, not whole-script rollback.

`check_cox_source.py` enforces the four original SHA256 pins, all140406 rounded
ASCII fields, full author design and printed facts. R4.6.1 can reread correct
17-digit CSV text with1ULP drift; original references read DTA directly and
native/Python source hex checks remain exact. No tolerance widening.

## Independent evidence and limits

R4.6.1/survival3.8.6 `coxph(ties='breslow',robust=TRUE)` plus independently
reconstructed information/full row scores/baseline validates both model-based and
corrected robust inference. Original partial LL−7717.2334329144514/null
−7981.9304249926163,24 failure knots; reprate beta≈.5222796, corrected robust
SE≈.5711698 and Wald≈540.9797 match the author. Uncorrected R SE≈.5710844 does
NOT match; N/(N-1) is explicit, not residual-DF HC1. R's own hot-start polishing
never uses native estimates. Crucially, coxph(init=...) loglik[1] is its INITIAL
likelihood, not the zero-beta null: compute the null independently from tied
risk counts. toler.chol<eps avoids an exploratory R warning.

- Eight pure-kernel controls:4233 model-based/robust R gates (including both1600-
  entry covariances/all28 control baseline knots/means), all16000 efficient row
  scores,3600 unique observed/UI0/UI1 point values in both covariance modes;
  finite-difference derivatives, late-risk scaling, baseline-only/ties, identity/
  permutation/time/affine invariance, negative last coefficient, rank/separation/
  support/resource failures and represented survival0 versus H overflow.
- Two parser, six runtime and three actual-script/CLI controls. Runtime checks
  all R gates/current points, original positions under four joint exclusions,
  original DECIMAL codes, owned state/labels/order/NULLs, parse-vs-executed
  invalidation, target/schema/support failures and time0/tail holds. Actual
  cox.td JSON/text exports match; guards, no-replacement, discovery, baseline-only
  JSON/rejected hazard and corrupted-reference/projection detection pass.
- Original native audit:2084 R gates/all1600 covariance entries,187208 source/
  design/projection cells,147092 exact source cells and254 printed quantities,
  including12 compact Table17.8 cells. Independent Python reconstructs risk
  information, ALL efficient scores, sandwich, baseline/null/Wald and every point.
  Final original JSON/text CSV bytes match. Only400-row synthetic controls are
  redistributed; no new production dependency. Runtime serde_json is test-only.
- One bounded independent source review confirmed the derivations and found
  near-one DECIMAL event rounding. Existing exact-integer helper fixes it AFTER
  joint eligibility; focused valid/invalid/excluded-row regression passes. No
  additional actionable issues were found in that review; reviewer did not rerun
  the supplied numerical audits.
- Local reverse-consumer regression totals1562 unique passing tests/two pre-existing
  ignored. The initial272-artifact direct run exposed six discovery expectations;
  corrected counts/sorting/native-extension checks then pass all72 affected tests.
  Prior unaffected passes are retained, not repeatedly rerun. Four zero-test
  artifacts do not count as test passes. All-target affected application and root
  Clippy with warnings denied, workspace formatting and diff checks pass.
  Broad nextest discovery timed out; a sampled original test executable stalled
  at dyld start before tests. Exact Cargo-inventory/SHA-identical fresh-copy
  standard-harness execution supplies regression evidence, not a claimed green
  nextest run. The interrupted run is not counted passing. CI is disabled;
  other platforms/release readiness are unverified.

This validates the bounded coefficient-column/source-program workflow—not the
entire survival family, Table17.9 hazard-ratio uncertainty or textbook coverage.

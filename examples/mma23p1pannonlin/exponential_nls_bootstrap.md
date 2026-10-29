# Chapter 23 patents: exponential-mean NLS whole-firm bootstrap

This bounded **SQL composition** reproduces the whole-firm pairs bootstrap
method for the §23.3/Table 23.1 NLS illustration in the [author program](https://cameron.econ.ucdavis.edu/mmabook/mma23p1pannonlin.do)
and [log](https://cameron.econ.ucdavis.edu/mmabook/mma23p1pannonlin.txt).
It is not a general bootstrap or clustered `nl` command. The script first runs
native `nl` and its mean/residual predictions, then refits the intercept and
slope on every resample. Text and CLI-v1 JSON show the same reports; an ordered
CSV contains each draw and its replicate coefficients. The bootstrap does not
create restorable estimation state. The [original NLS workflow](exponential_nls.md)
separately validates the native fit, predictions and working/firm-sandwich
covariances.

## Sample, resampling and interpretation

The pinned `patr7079.asc` SHA-256 is
`87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8`.
Firm ID is source-record position, not CUSIP. Use **all 346 firms/1,730 rows**
in 1975–79, including 22 zero-total-patent firms; missing ARDSSIC metadata do
not remove rows. The unweighted mean is `mu_it=exp(a+b*LOGR_it)` for PAT, with
no extra regressors, time effects, weights or offsets. Each of 500 replicates
draws 346 whole firms with replacement, preserving all five records per copy
and refitting both parameters. It does not resample individual rows or exclude
zero-total firms.

Replicate `r=1..500` initializes xoshiro256++ with SplitMix64 seed
`10001+r−1`; u64 rejection maps draws to firms in ascending source ID order.
This declared per-replicate stream differs from Stata and from the native
TabDat single-stream bootstrap. No historical RNG parity is claimed. The
ordered CSV records replicate, position, RNG step, firm ID, both replicate
coefficients, original coefficients and sample sizes. A step may exceed its
position if a word is rejected. The report gives bootstrap means/biases, the
full 2×2 B−1 covariance, SEs and R-type7 percentile 95% intervals. No normal
or BCa intervals or bootstrap p-values are computed.

| Original source, float64 | Intercept | LOGR slope |
| --- | ---: | ---: |
| Original NLS coefficient | 2.5287289311 | 0.5084672579 |
| Bootstrap mean | 2.4555179795 | 0.5264484372 |
| Bias | −0.0732109517 | 0.0179811793 |
| Bootstrap SE | 0.2157340370 | 0.0584172804 |
| Percentile 95% interval | [1.9258017509, 2.7541579274] | [0.4481334398, 0.6733102015] |

The covariance, intercept/slope order, is
`[[0.0465411747041, -0.0119110846966], [-0.0119110846966, 0.0034125786551]]`.
The author log reports original coefficients `2.528729`/`.5084673` and
bootstrap SEs `.198022`/`.0541264`. Different draws prevent exact bootstrap
replay; the historical percentile convention has not been reconstructed.
Whole-firm resampling assumes independent firms. It does not establish the
conditional mean, a causal effect or few-cluster reliability.

## Local solver and failure boundaries

The input requires unique integer IDs 1–346, one record in each of 1975–79,
2–346 firms, finite `|LOGR|≤20` and integer counts 0–1,000. Native `nl` also
applies its own rank, residual-variance and convergence checks.

For each original or resampled dataset and candidate slope `b`, the SQL profiles
the positive amplitude `A(b)=sum(y exp(bx))/sum(exp(2bx))`, with firm draw
multiplicities in both sums, and `a=ln A`. Its profiled score is the difference
of the `y exp(bx)`-weighted mean of x and the `exp(2bx)`-weighted mean of x.
The solver requires score signs separated from zero by `128*machine_epsilon`
on absolute-term scales at both endpoints of **[−2,2]**, then applies 80
deterministic sign-bracket bisections, moving the lower endpoint when the
midpoint score is positive and the upper endpoint otherwise. At the selected root it checks the score
at `1e-10` relative to its absolute-term scale, strictly positive observed-loss
curvature above a `128*epsilon` centered-variance scale, and positive RSS above
`(128*epsilon)^2` times its propagated residual scale. As a further local
conditioning guard, curvature times `(1e-9 + 1e-10*abs(b))` must exceed
`128*epsilon` times the absolute score scale. This requires a locally
linearized slope change of the comparison tolerance to exceed the declared
roundoff scale; it is a conservative admission rule, not a global error bound. It also requires finite
`|a|≤20` and linear predictors in `[−40,40]`. The original profiled coefficients
must match the recovered native NLS fit within `1e-9 + 1e-10*abs(native)`; its
predictions must reconstruct the native linear relation within
`1e-12*(1+max|ln(mu)|)`.

**The profiled score is not generally monotone.** On the pinned source and its
500 prescribed resamples, a 161-point scan over [−2,2] found one sign crossing
per fit, but a separate grid found positive score derivatives away from the
root in 474 of 501 cases. A finite scan does not prove uniqueness. Bisection
selects a deterministic interior root; the score and observed-Hessian checks
establish a regular **local** NLS minimum, not a globally unique optimum or
parity with all possible optimizer starts. These bounds are part of this
source-specific composition and may reject otherwise valid NLS datasets.

Invalid/nonfinite input, endpoint/cancellation-dominated roots, unidentified or
zero-residual fits, a failed replicate, exhausted RNG work or incomplete
iteration abort before export. No failed replicate is silently dropped or
redrawn. RNG rejection is limited to `G+1024` words per replicate. Some SQL
guards currently surface only `sql failed` with script location. Input files
and pre-existing exports remain unchanged, and export refuses to overwrite.
Script execution is **not transactional**: a later bootstrap failure leaves
the earlier native NLS fit and its prediction columns in session. The final
active relation after success is the draw table, not the original model data;
its repeated coefficients are not a restorable bootstrap fit. A report can
precede an export failure, so use a fresh directory and check the exit status.
The original-source run took about 95 seconds and wrote a 17 MB draw CSV on the
development machine; timing depends on hardware.

## Reproduce and independent evidence

```sh
repo=/absolute/path/to/tabdat-explore-rs
work=/absolute/path/to/empty/work-directory
curl -fsSL https://cameron.econ.ucdavis.edu/mmabook/patr7079.asc -o "$work/patr7079.asc"
shasum -a 256 "$work/patr7079.asc"  # compare with pinned hash above
(cd "$repo" && cargo build --locked --bin tabdat-explore-rs)
sh "$repo/examples/mma23p1pannonlin/prepare.sh" "$work/patr7079.asc" "$work/patents.csv"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma23p1pannonlin/exponential_nls_bootstrap.td" > result.jsonl)
python3 "$repo/examples/mma23p1pannonlin/conditional_bootstrap_draws.py" 10001 346 500 346 "$work/oracle.csv"
Rscript "$repo/examples/mma23p1pannonlin/exponential_nls_bootstrap_reference.R" "$work/patr7079.asc" "$work/oracle.csv" "$work/r-reference"
python3 "$repo/examples/mma23p1pannonlin/check_exponential_nls_bootstrap.py" "$work" "$work/oracle.csv" "$work/r-reference" "$work/result.jsonl" "$work/patr7079.asc"
python3 "$repo/examples/mma23p1pannonlin/check_nls_bootstrap_native_refits.py" "$work/patents.csv" "$work/oracle.csv" "$work/r-reference" "$repo/target/debug/tabdat-explore-rs"
```

The development-only Python oracle uses the independent xoshiro256++/SplitMix64
reference in `crates/tabdat-stats/tests/fixtures/bootstrap/draws.py`. R 4.6.1
independently reshapes the source, solves each profiled score with `uniroot`,
checks centered observed curvature and compares each root to a separate scalar
objective `optimize` result. The latter is an approximate cross-check, not the
precision oracle: observed maximum slope differences were `2.43e-8` on the
original and `5.02e-8` on the synthetic fixture. The checker pins the source
hash, verifies all 1,730 source/design and native mean/residual rows, all
**173,000 ordered draw IDs/steps and both corresponding coefficient cells**,
500 R refits, 18 summary values and native prediction summaries. Draws and
design are exact; floating-point gates use `1e-9 + 1e-10*abs(reference)`.
Maximum original replicate intercept/slope differences were `6.99e-14` and
`1.59e-14`, and the maximum summary difference was `1.29e-14`. Four selected
repeated-row native NLS refits, including each coefficient's extrema, agree
with R within `2.02e-13`. Original text/JSON exports were byte-identical.

The 12-firm R fixture validates all 6,000 ordered draws, all 500 two-parameter
fits and the complete covariance, including two zero-total firms. Tests cover
text/JSON, reversed input, no-clobber, missing/invalid/duplicate/rank failures,
exact and near ±2 endpoints, exact/near cancellation-dominated observed
curvature, a finite original fit with an all-zero-outcome replicate, an
exact-zero-RSS replicate, forced RNG/iteration exhaustion and
preserved data/prior-fit state. A separate two-firm regression reproduces a
flat-root replicate where floating-point score cancellation previously yielded
a spurious slope near `−3.5e-6` and falsely positive curvature; the local
conditioning guard now rejects it. The RNG SQL is byte-identical to the separately
tested pooled-Poisson expression, whose high-bit/forced-rejection controls are
retained. R, Python and author data are development-only.

Poisson RE, general GEE, GEE/RE firm bootstraps, Figure 23.1 and remaining
Table 23.1 outputs remain required. This workflow does not complete the
chapter, graduate-method core or book inventory.

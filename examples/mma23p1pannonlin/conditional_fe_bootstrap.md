# Chapter 23 conditional-FE Poisson: whole-firm bootstrap

This bounded **SQL composition**, not a general bootstrap or native panel
estimator, reproduces the author program's §23.3/Table 23.1 firm-pairs
bootstrap method for the conditional Poisson-FE patent slope. It provides the
same workflow in text and CLI-v1 JSON and exports ordered draws and each
replicate's slope. It does not create restorable estimation state. See the
[conditional-FE analysis](conditional_fe.md) for the model and the
[author program](https://cameron.econ.ucdavis.edu/mmabook/mma23p1pannonlin.do)
and [log](https://cameron.econ.ucdavis.edu/mmabook/mma23p1pannonlin.txt).

## Sample, inference and reproducibility

The pinned `patr7079.asc` SHA-256 is
`87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8`.
Original-record firm IDs, not CUSIP, identify 346 firms with five observations
in 1975–79. Conditioning on firm totals excludes 22 zero-total firms: the
bootstrap samples **324 retained firms and 1,620 firm-years**. No extra
regressors, year dummies or offsets are used. Each of 500 replicates samples
exactly 324 firms with replacement, retaining all five observations for each
copy. The conditional-FE slope is refitted; multiplicities weight each firm's
conditional score, not its observations as independent resamples.

The seed is 10001. In replicate `r=1..500`, the script initializes an
xoshiro256++ stream with SplitMix64 seed `10001+r-1`, and uses 64-bit rejection
mapping to sample indices in ascending original firm-ID order. This explicit
per-replicate stream differs from the existing native single-stream TabDat
bootstrap and from Stata; it makes **no historical RNG parity claim**. The CSV
records `replicate`, `position`, stream `step` (which can exceed position if a
value is rejected), firm ID, fitted slope and sample metadata. The table reports
the original slope, bootstrap mean/bias, sample standard deviation with B−1
denominator, and R-type7 percentile 95% interval. Existing files are never
overwritten; use a fresh output directory.

| Original source, float64 | Value |
| --- | ---: |
| Original slope | −0.0377642353576 |
| Bootstrap mean / bias | −0.0353021672186 / 0.0024620681390 |
| Whole-firm bootstrap SE | 0.1052064418978 |
| Percentile 95% interval | [−0.2630518832356, 0.1556901956874] |

The author log reports the same sample and B=500, with SE `0.1067039`, bias
`0.0057448` and percentile interval `[−0.2458792, 0.1454112]`; differing
pseudorandom draws prevent exact replay; the author's percentile convention has
not been independently reconstructed. Neither a
bootstrap SE nor a conditional model repairs mean misspecification, makes the
analysis causal, or guarantees reliable inference with few independent firms.
This is not the model-based information SE or a generic cluster-robust FE
sandwich, and no normal or bias-corrected bootstrap interval is claimed.

## Bounds and failure behavior

Input guards and one-slope bracket follow [the model-based workflow](conditional_fe.md):
integer IDs 1–346, unique five-year keys, counts 0–1,000, finite `|logr|≤20`,
centered deviations ≤20 and 2–346 positive-total firms. Endpoints −10 and 10
must have scale-distinct opposite scores for the original fit **and every
replicate**; 80 bisections, scale-relative final score and finite positive
information are checked. Information uses a second-pass weighted centered
variance. Zero variation, separation, boundary/near-boundary roots, numerical
failure or a replicate without a valid fit abort the analysis; replicates are
never redrawn or discarded. Each stream has at most `G+1024` steps to obtain
its G indices; exhaustion fails. Some failures currently surface only as `sql
failed` with the script location. Input and pre-existing output files remain
unchanged, and a failed fit does not export. Report output may precede an export
failure, so check the process exit status. Running the original source took
approximately 48 seconds on the development machine; the CSV was 12 MB.

## Reproduce and independently audit

```sh
repo=/absolute/path/to/tabdat-explore-rs
work=/absolute/path/to/empty/work-directory
curl -fsSL https://cameron.econ.ucdavis.edu/mmabook/patr7079.asc -o "$work/patr7079.asc"
shasum -a 256 "$work/patr7079.asc"  # compare with the pinned hash above
(cd "$repo" && cargo build --locked --bin tabdat-explore-rs)
sh "$repo/examples/mma23p1pannonlin/prepare.sh" "$work/patr7079.asc" "$work/patents.csv"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma23p1pannonlin/conditional_fe_bootstrap.td" > result.jsonl)
python3 "$repo/examples/mma23p1pannonlin/conditional_bootstrap_draws.py" 10001 324 500 324 "$work/oracle.csv"
Rscript "$repo/examples/mma23p1pannonlin/conditional_bootstrap_reference.R" "$work/patr7079.asc" "$work/oracle.csv" "$work/r-reference"
python3 "$repo/examples/mma23p1pannonlin/check_conditional_bootstrap.py" "$work/oracle.csv" "$work/r-reference" "$work/conditional_fe_bootstrap_draws.csv" "$work/result.jsonl"
```

The development-only Python oracle uses the independently implemented
xoshiro256++/SplitMix64 reference in `crates/tabdat-stats/tests/fixtures/bootstrap/draws.py`.
R 4.6.1 independently reshapes the source, verifies the sample, refits all 500
conditional scores and separately checks three replicate slopes with
frequency-weighted dummy-variable Poisson GLMs. The checker verifies all 1,730
source/design rows, all **162,000** ordered firm draws and corresponding slope
cells, and 11 report quantities. Draw identities, steps, source values and
membership are exact; slopes and summaries use absolute tolerance `1e-10`
(float64 bisection versus R root finding); the largest original-source replicate
slope difference was `2.39e-14`. A 12-firm synthetic fixture checks all 5,000
draws and 500 R fits, text/JSON, input-order invariance and no-clobber. Separate high-bit
streams with forced rejections check 24 values exactly; failure tests include a
finite original fit with a separated bootstrap replicate, empty information,
invalid keys and RNG work exhaustion. R/Python and author data are not
production dependencies.

The separate [pooled bootstrap](pooled_poisson_bootstrap.md), [exponential NLS](exponential_nls.md)
and bounded [exchangeable Poisson GEE](exchangeable_gee.md) are validated.
The [NLS firm bootstrap](exponential_nls_bootstrap.md) is also validated.
The separate [GEE bootstrap](exchangeable_gee_bootstrap.md) uses distinct copied IDs. [Gamma-RE Poisson](gamma_re_poisson.md) is separately validated; normal-RE Poisson, RE firm bootstrap, general GEE and Figure 23.1 remain
required; this composition does not complete the nonlinear panel chapter,
graduate-method target or full book inventory.

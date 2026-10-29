# Chapter 23 patents: pooled Poisson whole-firm bootstrap

This bounded **SQL composition** reproduces the pooled Poisson firm-pairs
bootstrap method in the [author program](https://cameron.econ.ucdavis.edu/mmabook/mma23p1pannonlin.do)
and [log](https://cameron.econ.ucdavis.edu/mmabook/mma23p1pannonlin.txt),
§23.3/Table 23.1. It is not a general bootstrap command. The script first
fits native `poisson` and summarizes its mean/index predictions, then refits
both intercept and slope for each firm resample. Text and CLI-v1 JSON expose
the same reports; an ordered CSV records draws and replicate coefficients. The
bootstrap composition creates no restorable estimation state. The separate
[pooled analysis](pooled_poisson.md) validates the original fit, covariance
conventions and all original prediction rows.

## Sample, inference and reproducibility

Use the pinned `patr7079.asc` SHA-256
`87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8`.
Firm ID is the source-record position, not CUSIP. All **346 firms and 1,730
firm-years** in 1975–79 are included, including 22 firms with zero total
patents; ten missing ARDSSIC metadata values do not drop observations. The
model is `mu_it=exp(a+b*LOGR_it)` for PAT, with no additional regressors,
year effects or offset. Each of 500 replicates draws exactly 346 firms with
replacement, retains all five rows per copy and refits both parameters. This
is not individual-row resampling or the conditional-FE bootstrap, whose
estimation sample excludes zero-total firms.

The seed is 10001. Replicate `r=1..500` initializes its own xoshiro256++ stream
with SplitMix64 seed `10001+r−1`; u64 rejection maps the stream to firms in
ascending original ID order. This explicitly specified stream differs from
both Stata and the native TabDat single-stream bootstrap. No historical RNG
parity is claimed. The ordered CSV includes replicate, draw position, RNG step,
firm ID, both replicate coefficients, original coefficients and sample sizes.
A step can exceed the position when an RNG word is rejected. The summary gives
bootstrap means and biases, the full 2×2 covariance with B−1 denominator, SEs,
and R-type7 percentile 95% intervals for each parameter. No normal/BCa
intervals or bootstrap p-values are claimed.

| Original source, float64 | Intercept | LOGR slope |
| --- | ---: | ---: |
| Original Poisson coefficient | 1.7115282420 | 0.6929337099 |
| Bootstrap mean | 1.6665116433 | 0.7062610802 |
| Bias | −0.0450165987 | 0.0133273703 |
| Bootstrap SE | 0.1514681772 | 0.0492489391 |
| Percentile 95% interval | [1.3305307184, 1.9202178028] | [0.6278067958, 0.8202476442] |

The bootstrap covariance, ordered intercept/slope, is
`[[0.0229426087011, -0.0069346511464], [-0.0069346511464, 0.0024254579985]]`.
The author log uses the same sample and B=500, reporting slope/intercept SEs
`.0473006`/`.141745` and percentile intervals `[.6250867,.8100113]` /
`[1.336657,1.924925]`. Different draws prevent exact replay; the historical
percentile convention has not been independently reconstructed. Whole-firm
resampling assumes independent firms. It is not a few-cluster guarantee,
proof of a correct mean, a causal effect, or the separate model-based/cluster
sandwich inference in the pooled Poisson report.

## Numerical and failure boundaries

Input requires unique integer IDs 1–346, one record in each of 1975–79,
finite `|logr|≤20`, counts that are integers in 0–1,000, and 2–346 firms.
The script's native Poisson fit applies its own rank/convergence guards. The
bootstrap profiles the intercept as `a=log(sum(y)/sum(exp(b*x)))` with
resample multiplicities and solves the monotone slope score by 80 bisections
on `[−10,10]`. Counts and predictor bounds keep exponent arguments within
±200. The total count must be positive, each endpoint score must be distinct
from zero in the required direction by more than `128*machine_epsilon` times
`1 + sum(|y*x|) + sum(y)*sum(exp(b*x)*|x|)/sum(exp(b*x))`, and the final
score must meet its declared scale-relative bound. Centered weighted variance
provides finite positive profiled information without subtracting near-equal
moments. Nonfinite, separated, unidentified, endpoint/cancellation-dominated
or nonconverged original or replicate fits fail; no replicate is redrawn or
discarded. RNG rejection is limited to `G+1024` stream steps per replicate.

A failure before export leaves the input file and any existing export unchanged;
the output refuses to overwrite. Some SQL guards surface only `sql failed` at
the script location. **Script execution is not transactional:** if a later
bootstrap step fails, the earlier native Poisson report and prediction columns
have already been produced in the session; they are not rolled back. A report
can also precede an export failure, so check the process exit status. Use a
fresh output directory. The original-source run took about 94 seconds on the
development machine and wrote a 17 MB draw CSV; times depend on hardware.

## Reproduce and independent evidence

```sh
repo=/absolute/path/to/tabdat-explore-rs
work=/absolute/path/to/empty/work-directory
curl -fsSL https://cameron.econ.ucdavis.edu/mmabook/patr7079.asc -o "$work/patr7079.asc"
shasum -a 256 "$work/patr7079.asc"  # compare with the pinned hash above
(cd "$repo" && cargo build --locked --bin tabdat-explore-rs)
sh "$repo/examples/mma23p1pannonlin/prepare.sh" "$work/patr7079.asc" "$work/patents.csv"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma23p1pannonlin/pooled_poisson_bootstrap.td" > result.jsonl)
python3 "$repo/examples/mma23p1pannonlin/conditional_bootstrap_draws.py" 10001 346 500 346 "$work/oracle.csv"
Rscript "$repo/examples/mma23p1pannonlin/pooled_bootstrap_reference.R" "$work/patr7079.asc" "$work/oracle.csv" "$work/r-reference"
python3 "$repo/examples/mma23p1pannonlin/check_pooled_bootstrap.py" "$work" "$work/oracle.csv" "$work/r-reference" "$work/result.jsonl" "$work/patr7079.asc"
```

The development-only Python oracle uses the independent xoshiro256++/SplitMix64
reference in `crates/tabdat-stats/tests/fixtures/bootstrap/draws.py`. R 4.6.1
independently reshapes source records and profiles/refits all 500 replicates;
weighted Poisson GLMs separately check the original fit and three replicates.
The checker pins the source hash, checks all 1,730 source model-design rows,
all **173,000 ordered draw IDs/steps** and both corresponding coefficient
cells, 18 report values, native Poisson coefficients and prediction summaries.
Draw identities and source design are exact; floating-point comparisons use
absolute tolerance `1e-10`. The largest observed original-source R differences
were `1.10e-13` for replicate intercept and `2.58e-14` for replicate slope.

The 12-firm synthetic fixture checks all 6,000 draws, both coefficients and
full covariance against R/Python, including draws of two zero-total firms,
text/JSON agreement, reversed input order and no-clobber. Independent R GLMs
check constructed roots near ±9.9; exact and cancellation-dominated ±10
endpoints fail. Further tests cover duplicate/missing/nonfinite input, rank
failure, prediction collisions, too few firms, RNG work exhaustion, and a
finite original fit with a zero-total failed replicate. The RNG SQL is
byte-identical to the separately tested conditional-FE bootstrap expression,
whose high-bit and forced-rejection controls are retained. R/Python and author
data are development-only.

The [conditional-FE](conditional_fe_bootstrap.md) and [NLS](exponential_nls_bootstrap.md)
bootstraps are separate; RE and GEE bootstraps, general GEE, RE Poisson, Figure 23.1 and other
Table 23.1 output remain required. The bounded [exchangeable Poisson GEE](exchangeable_gee.md)
is validated separately. Neither this composition nor the other
bounded workflows completes the chapter, graduate-method core or full book
inventory.

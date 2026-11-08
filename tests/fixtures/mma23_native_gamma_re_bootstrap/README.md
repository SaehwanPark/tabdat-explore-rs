# Native patents shared-gamma bootstrap reference

Reuse `../mma23_gamma_re_bootstrap/source.asc`:40 firms/200 five-year records,
eight zero-total firms retained (not conditional-FE exclusions). Seed10001,
B500, first-seen complete-case groups, continuous native xoshiro/u64-rejection
stream; repeated selections have separate mean-one gamma effects.

R4.6.2 generated31 scalar fields/full3x3 joint covariance and500 full vectors
in `_cons,logr,lnalpha` order (R headers intercept,slope,lnalpha), using unchanged
`examples/mma23p1pannonlin/gamma_re_bootstrap_reference.R`. Raw source and
independently checked indices/steps are the only native inputs, never native
coefficients. Base-R BFGS with analytic scores then Newton observed information
refits every vector. B−1 covariance, mean/bias/SE and type7 marginal95 limits
include nuisance lnalpha. Fixed1e-9+1e-10*abs(reference); exact labels/draws/counts.
No tolerance widening or original publisher records are included.

Reproduce after building in fresh directories:

```sh
repo=$PWD
work=$(mktemp -d)
sh examples/mma23p1pannonlin/prepare.sh \
  tests/fixtures/mma23_gamma_re_bootstrap/source.asc "$work/patents.csv"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json \
  -f "$repo/examples/mma23p1pannonlin/native_gamma_re_bootstrap.td" > result.jsonl)
python3 examples/mma23p1pannonlin/check_native_gamma_re_bootstrap.py oracle \
  "$work" tests/fixtures/mma23_gamma_re_bootstrap/source.asc "$work/draws.csv" --control
Rscript examples/mma23p1pannonlin/gamma_re_bootstrap_reference.R \
  tests/fixtures/mma23_gamma_re_bootstrap/source.asc "$work/draws.csv" "$work/reference"
python3 examples/mma23p1pannonlin/check_native_gamma_re_bootstrap.py check \
  "$work" tests/fixtures/mma23_gamma_re_bootstrap/source.asc "$work/reference" --control
```

Auditor independently checks20000 RNG draws/steps, raw three-component scores
using integer-count harmonic digamma differences, full empirical covariance/
quantiles and original marginal predictions/information. Matrix comparisons use
1e-9 times the empirical SD product (exact zero rows remain zero), supplementing
rather than widening scalar reference gates. This is neither row-level NB2,
merged-source-ID sensitivity, the separate SQL stream nor historical Stata
RNG/SE/interval parity. R/Python are optional development dependencies only.

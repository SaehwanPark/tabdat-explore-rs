# Chapter 21 pooled OLS whole-person bootstrap

## Bounded delivery contract

Reproduce the §21.3 Table 21.2 author-program pooled-OLS panel-bootstrap
analysis on the pinned original `MOM.dat`: 5,320 person-years, 532 people,
1979–1988, outcome lnhr, predictor lnwg and an intercept. Fit OLS and resample
532 complete person blocks with replacement, 500 replications and explicit local
seed 10001. Native xoshiro256++/SplitMix64 draws are deliberately **not** Stata's
historical draws; compare the algorithm and resulting estimates to an independent
reference, not literal historical bootstrap output. Report B-1 replicate
covariance/SE, bias and type-7 percentile95 intervals. The existing command is
read-only, not an automatic covariance replacement for the retained OLS fit.

Validation contract:

1. Executable load/guard → fit → whole-person bootstrap → predict/interpret →
   report/export workflow in text and CLI-v1 JSON. Independent R fits each of the
   same explicitly generated indices and checks original sample/order, full
   covariance, means, bias, SE, intervals and every exported prediction row.
2. A small inspectable independent-reference actual-script fixture checks the
   bootstrap result and sample/block metadata. Verify reproducibility and
   text/JSON output, while retaining the original OLS model for predictions.
3. Missing/invalid keys, rank-degenerate original/replicate fit and an existing
   export fail without overwriting output; document existing prior session effects
   and sample exclusions rather than implying transactional scripts.
4. Update current capability/ledger status and run affected integration tests.

No automatic FE/RE refit, iid-person-year bootstrap, finite-sample coverage or
few-cluster guarantee, survey sampling, Stata RNG/bitwise parity or whole-table
claim. Independence is across persons, with arbitrary within-person dependence;
500 replications have Monte Carlo variability.

## Reproduction and evidence

Download and prepare the pinned source as shown in the [example README](README.md).
In a fresh output directory containing `MOM.csv`:

```sh
/path/to/tabdat-explore-rs --json -f /path/to/repo/examples/mma21p1panel/pooled_bootstrap.td > pooled_bootstrap.jsonl
Rscript /path/to/repo/examples/mma21p1panel/pooled_bootstrap_reference.R /path/to/MOM.dat pooled-reference
python3 /path/to/repo/examples/mma21p1panel/check_pooled_bootstrap_reference.py . pooled-reference /path/to/MOM.dat
```

The source script checks keys before filtering and reports raw, retained and
excluded joint-finite rows explicitly. It orders retained rows by `source_row`;
the bootstrap groups by first appearance and retains within-person row order.
The source has no exclusions; for other inputs the bootstrap result's sample
metadata refers to the already filtered frame, not the raw source. Model and
predictions come from the separate original OLS fit; bootstrap does not replace it.

R 4.6.1 `lm.fit`, sample covariance and type-7 quantiles independently recompute
500 fits using the separately implemented Python reference draw stream. On the
pinned source, wage estimate is 0.08274354528, bootstrap SE 0.02881286627 and
percentile95 interval [0.02952603549, 0.14237425754]; the intercept SE is
0.07819233695. The audit checks 18 bootstrap scalars (including full covariance),
all 37,240 exported identity/design/prediction cells and all 5,320 input rows at
absolute 1e-9, plus exact cluster sizes/order metadata and all 500 sample sizes.
Original text and JSON exports are byte-identical. An independently R-checked
interleaved synthetic case includes excluded rows; tests cover duplicate/missing
keys, original and resampled rank failure, and no-clobber output. Failed scripts
leave earlier session effects intact; resampled failures are not discarded/redrawn.

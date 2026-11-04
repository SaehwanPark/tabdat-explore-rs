# Chapter 21 first-difference whole-person bootstrap

## Bounded delivery contract

Reproduce the §21.3/Table 21.2 author-program first-difference OLS clustered
bootstrap on pinned `MOM.dat`: 4,788 consecutive annual pairs, 532 people,
1979–1988 source levels, outcome Δlnhr and predictor Δlnwg with an intercept.
Construct differences within person from raw ordered levels before excluding
non-finite endpoints; retain only consecutive years. Resample all retained pairs
of each selected person together, 532 draws per replication, B=500, explicit local
seed 10001. Native xoshiro256++/SplitMix64 draws differ from historical Stata.
Report B-1 covariance/SE/bias and type-7 percentile95 intervals. This is OLS on
fixed annual differences, not an automatic panel-model refit.

Validation contract:

1. Execute load/key guard → raw annual pairs and explicit eligibility counts →
   fit → whole-person bootstrap → predictions/interpretation/export in text and
   CLI-v1 JSON. Independent R constructs original source pairs and refits every
   specified resample; audit exact membership/order, coefficients, full bootstrap
   covariance, intervals and every design/prediction row.
2. Independent small synthetic controls include irregular years, missing level
   values and unbalanced cluster sizes; verify no bridging, sample/exclusion
   counts, variable replicate counts, deterministic stream and text/JSON output.
3. Duplicate/invalid keys, singular original and replicate fits, and no-clobber
   export fail without publishing an output; prior script effects are not rolled
   back. Bootstrap does not replace the retained OLS prediction model.
4. Update capability/coverage status and run affected tests.

No Stata RNG parity, general panel lag declaration, automatic FE/RE bootstrap,
weak-IV inference, survey inference, finite-sample coverage or few-cluster
accuracy guarantee. Independence is across people; 500 draws have Monte Carlo
error.

## Reproduction and evidence

After preparing the pinned `MOM.dat` source as in the [example README](README.md),
run in a fresh output directory containing `MOM.csv`:

```sh
/path/to/tabdat-explore-rs --json -f /path/to/repo/examples/mma21p1panel/fd_bootstrap.td > fd_bootstrap.jsonl
Rscript /path/to/repo/examples/mma21p1panel/fd_bootstrap_reference.R /path/to/MOM.dat fd-reference
python3 /path/to/repo/examples/mma21p1panel/check_fd_bootstrap_reference.py . fd-reference /path/to/MOM.dat
```

The script checks raw keys and source-row uniqueness, then forms LAG on raw
levels within person before requiring consecutive years and joint-finite
first differences. It reports eligible annual-pair counts and exclusions before
filtering; bootstrap sample metadata refers to the resulting retained frame.
The original source has 4,788 retained pairs, no excluded consecutive pairs, and
532 groups of nine. Groups and within-group rows are ordered by person/year.
The bootstrap is read-only; predictions use the separately retained original OLS fit.

Independent R 4.6.1 constructs annual pairs from raw data and refits all 500
replicates using a separately implemented Python reference draw stream. Wage
slope is 0.10898514942, whole-person bootstrap SE 0.08284363321 and type-7
percentile95 interval [-0.04005699511, 0.27361196531]; intercept SE is
0.00160448717. The source audit checks 18 bootstrap quantities including full
covariance, all 52,668 pair/design/prediction cells and every original input row
at absolute 1e-9, plus exact cluster/replicate counts. Original text and JSON
exports are byte-identical; these draws are not the historical Stata draws.

A separately R-validated 27-row interleaved synthetic input has a missing year,
a missing level and a wholly excluded person: 19 consecutive candidate pairs,
14 retained pairs, six groups of unequal sizes and explicitly audited 500
variable replicate sizes. Actual-script tests verify no gap/missing-value
bridging, deterministic repeated results, singular original/replicate failure,
invalid/duplicate keys and no-clobber export. Failed scripts are ordered rather
than transactional; failed replicates are not discarded or redrawn.

# Fixed-ML-theta whole-person auxiliary bootstrap

[manual_ml_bootstrap.td](manual_ml_bootstrap.td) delivers P2-ML-BOOT, manual
source297–299 in the [pinned inventory](manual_inventory.md). It preflights MOM,
fits native balanced Gaussian RE-ML, freezes original ML theta/person means,
constructs quasi y/x, fits auxiliary OLS, resamples complete people, reports
bootstrap uncertainty and exports original ordered source/component/design/
prediction rows. Existing primitives only; no production Rust/API/dependency
change. This is **not** component-refitting ML bootstrap, a generic panel bootstrap,
a complete mixed report or historical Stata RNG/SE parity.

## Sample, bootstrap and state

Original N5320/G532/T10. Supported3..532 complete ten-year people, unique exact
source_row/id/year in ranges1..5320/1..532/1979..1988, finite abs(lnhr/lnwg)<=20
and valid identified native ML and quasi OLS fits. Missing/incomplete model samples
fail instead of deleting rows. No weights, extra slopes or time effects. The
converter prefixes physical input lines; all nine raw/key columns are retained
and exports ordered by source_row. Native population predictions precede quasi
OLS; quasi predictions after bootstrap belong to the **original** auxiliary fit,
not the last draw or a native conditional/unit-effect prediction.

The script recovers ML components/theta from native fitted population residuals
using the balanced likelihood score equations described in the
[ML covariance workflow](manual_ml_covariance.md). The independent auditor checks
recovered components/theta against native and R results; it is not an additional
runtime comparison against a separately accessible result. With theta fixed once,
quasi_y=y-theta*mean_y and quasi_x=x-theta*mean_x; quasi OLS intercept is
(1-theta) times the population intercept, slope is unchanged. Original person
means and theta remain fixed even when a person is selected multiple times; neither
ML components nor means are refitted. The
[separate marginal RE-ML bootstrap](random_effects_ml_bootstrap.md) refits
components and estimates population-scale coefficients, a different target.

Each of500 replications draws G whole complete people with replacement in first-
appearance order. Seed10001 uses `xoshiro256++/splitmix64/u64-rejection-v1`,
not a Stata draw stream. Output reports original estimates, bootstrap means,
bias, full B-1 covariance, standard errors and marginal 95% R type7 percentile
intervals; no analytic p-values or nominal finite-sample guarantee. Usual pairs
bootstrap interpretation requires independent people and adequate cluster
asymptotics; fixed-transform uncertainty does not include re-estimation of theta.
Any rank/numerical failure aborts without discarding, retrying or exporting. The
script does not offer whole-session rollback. An existing export is never replaced.

## Independent evidence and failures

The [development-only R4.6.2 reference](manual_ml_bootstrap_reference.R) fits an
independent bounded marginal-likelihood QR profile, considers the unit-variance
boundary and checks its objective by direct 10x10 covariance likelihood. An
independent Python oracle generates person indices; R refits all500 fixed-design
replicates with QR and calculates B-1 covariance/means/bias/type7 intervals.
The [auditor](check_manual_ml_bootstrap.py) checks22 native ML/aggregate bootstrap
fields and all5,320x22 source/design/prediction cells, including47,880 exact
raw/key cells. Fixed absolute tolerance1e-9; no unexplained widening. The native
CLI exposes aggregate statistics/counts, **not individual replicate coefficients or
indices**: R refits are independently recomputed, not directly compared per draw.
Original reference regeneration is byte-identical, text/JSON prediction bytes
match, and eight forged theta/covariance/cluster/replicate/raw/prediction/note/
export traces are rejected.

Original theta=0.586481000430332; quasi-intercept/slope bootstrap SEs are
0.056773636413926409 and0.051437959628468213. Two
[interleaved dyadic16-person controls](manual_gls_covariance_synthetic.py) cover
positive ML theta and constrained theta0: frozen
[references](../../tests/fixtures/mma21_manual_ml_bootstrap/) add44 fields/7,040
cells. The [three actual-script tests](../../tests/mma21_manual_ml_bootstrap_workflow.rs)
check both CLI modes, original prediction state, metadata, six invalid input
variants, source/export preservation, and a rank-deficient first replicate that
fails without retry or export. A [test-only stats fixture](../../crates/tabdat-stats/tests/fixtures/bootstrap/fixed_ml_draws.csv)
checks all8,000 control draw indices against the independent Python oracle at the
native RNG boundary; no new public RNG or replicate-output API. Dyadic inputs
avoid the documented [R decimal-parser discrepancy](manual_gls_covariance.md).

## Reproduce

From repo root with a current binary (`cargo build --locked`), locally obtained
author MOM.dat, and R/Python for independent reference only:

```sh
repo=$PWD
raw=/absolute/path/to/MOM.dat
work=$(mktemp -d)
sh examples/mma21p1panel/prepare.sh "$raw" "$work/MOM.csv"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json \
  -f "$repo/examples/mma21p1panel/manual_ml_bootstrap.td" > manual_ml_bootstrap.jsonl)
Rscript examples/mma21p1panel/manual_ml_bootstrap_reference.R "$raw" "$work/reference"
python3 examples/mma21p1panel/check_manual_ml_bootstrap.py "$work" "$work/reference" "$raw"
```

Use a separate fresh directory and `--text` for readable output. Converter/export
refuse existing targets; R requires a fresh reference directory and raw eight-column
MOM.dat, not prepared MOM.csv. Synthetic audit uses `--synthetic`, waiving only the
source hash. No publisher data/program is redistributed; R/Python are not production
dependencies. The source-labelled GLS-HC1 anomaly, full mixed report, companion
inventories and complete chapter/book coverage remain required.

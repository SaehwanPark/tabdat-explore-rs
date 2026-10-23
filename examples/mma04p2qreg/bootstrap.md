# Joint quantile pairs bootstrap

This native method counterpart advances the joint quantile bootstrap underlying
MMA §4.6.4 Figure4.1 / mma04p2qreg.do block3. It uses the original audited
5,006 complete ASCII household records and nineteen quantiles .05,.10,...,.95.
Every replication shares its N IID row draws across all quantiles. The full
joint covariance uses B-1; marginal percentile95 intervals use R type7.

From a fresh directory containing qreg.csv prepared as in [README](README.md):

```sh
/path/to/repo/target/debug/tabdat-explore-rs --json -f /path/to/repo/examples/mma04p2qreg/joint_bootstrap.td > joint_bootstrap.jsonl
```

The explicit seed10101/B200 match the author's chosen integers, with a different,
pinned native xoshiro256++/SplitMix64 draw stream. There is no claim of matching
Stata8 bootstrap draws, printed SEs or Monte Carlo output. The script first
retains a median qreg fit, computes the read-only bootstrap report, then exports
its unchanged current-row median/residual predictions and source/design columns
to quantile_bootstrap_cohort.csv. Existing exports are never replaced. Valid log
medical expenditure zero remains; only joint missing/nonfinite cases are removed.
The raw source SHA256 and all5,999 identities are checked by the audit below.

The report permits heteroskedastic conditional outcomes under IID-row sampling
and regular identified quantiles; it does not impose the original qreg command's
IID-location-error sparsity convention. It is not survey/design-based inference
for the source household survey. Independent rows/positive conditional density,
large-sample regularity and sufficient bootstrap replications are assumptions,
not coverage guarantees. B200 has Monte Carlo error. No cluster/weighted/wild
bootstrap, simultaneous coverage bands, slope-equality test, retained bootstrap
fit or native Figure4.1 is claimed. Those remaining illustration outputs need
separate work. This slice supplies joint covariance, not whole-example coverage.

Original and resampled coefficients use the existing certified LP; irrelevant
replicate IID sparsity estimates are omitted. Identification, rank, response
variation, solver or numerical-certificate failures abort with the quantile and
original/replicate index, without redraws or success-conditioned inference.
Data/labels/named reports and the retained fit are preserved on success/failure.
See help bootstrap for dimensions/work limits and exact seed semantics. Runtime
Rust requires no R/Python; the scripts below are development references only.

## Independent reference

R4.6.1/quantreg6.1 rq(method="br") fits use a separate Python implementation of
the documented row-index RNG, with common indices across quantiles. In a fresh
reference directory:

```sh
python3 /path/to/repo/crates/tabdat-stats/tests/fixtures/bootstrap/draws.py --draws 10101 5006 200 draws.csv
Rscript /path/to/repo/examples/mma04p2qreg/joint_bootstrap_reference.R qreg0902.asc draws.csv reference
python3 /path/to/repo/examples/mma04p2qreg/check_joint_bootstrap_reference.py . reference qreg0902.asc
```

The optimized native run passed the audit of1,672 original/bias/mean/SE/percentile/full-joint covariance
quantities and25,030 identity/design/retained-median prediction cells, with1e-8
absolute tolerance. Text/JSON parity is tested on synthetic workflows; the original audit uses JSON.
Run each in a fresh directory. The original audit was run with the optimized
release binary; the debug LP is substantially slower on3,800 fits. Source data are not redistributed.

A reproduced near-active numerical failure is resolved by one stricter solve
of the same LP, retaining the original certificates; it does not redraw a
replicate. A small regression checks27 additional R coefficient/covariance/normal
inference quantities at the existing tolerances.

Committed synthetic references cover477 quantities on multi-predictor,
noconstant, shifted and rescaled designs. Tests also check adding quantiles leaves
marginal draws unchanged, explicit failures and limits, exact point fits without
sparsity, model/data preservation, CLI/text/JSON and the actual household script.

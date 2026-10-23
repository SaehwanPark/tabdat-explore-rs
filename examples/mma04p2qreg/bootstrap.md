# Joint quantile pairs bootstrap

## Two-quantile coefficient contrast

For exactly two requested quantiles, `contrast(variable)` reports one
higher-minus-lower coefficient difference, its shared-pairs bootstrap standard
error, asymptotic normal zero-difference test and 95% interval. The standard
error uses shared-replicate differences directly, avoiding cancellation when
subtracting nearly equal covariance entries. Without `contrast()`, result wire
shapes remain unchanged. If contrast inference is degenerate or unrepresentable,
the marginal report remains valid and the contrast states why inference is
unavailable.

This implements the bounded asymptotic method counterpart to
`mma04p2qreg.do` block3's .25/.75 slope comparison on the audited 5,006-row
sample, B200/native seed10101. It does not replicate Stata8's draw stream or
finite-sample F convention. Figure4.1 bands and native rendering remain future
work.

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
not coverage guarantees. B200 has Monte Carlo error. The optional single
two-quantile contrast uses a normal-Wald interval and normal test; it supplies
no finite-sample guarantee, multiplicity adjustment or simultaneous coverage.
No cluster/weighted/wild bootstrap, general restriction interface, retained
bootstrap fit or native Figure4.1 is claimed. This slice supplies the bounded
comparison, not whole-example coverage or rendering.

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

### Original .25/.75 slope comparison

The independent R4.6.1/quantreg6.1 reference uses the same 200 common row draws
as the native seed10101 run. From an audit directory containing the original
ASCII `qreg0902.asc` and prepared `qreg.csv`, generate the reference, run the
native example in a fresh subdirectory, then audit it:

```sh
python3 /path/to/repo/crates/tabdat-stats/tests/fixtures/bootstrap/draws.py --draws 10101 5006 200 draws.csv
Rscript /path/to/repo/examples/mma04p2qreg/coefficient_contrast_reference.R qreg0902.asc draws.csv reference
```

In a fresh native subdirectory containing a copy of `qreg.csv`, run
`coefficient_contrast.td` with the optimized release binary to create
`coefficient_contrast.jsonl` and `quantile_contrast_cohort.csv`, then run:

```sh
python3 /path/to/repo/examples/mma04p2qreg/check_coefficient_contrast_reference.py native reference qreg0902.asc
```

The audited run passed 46 original/R common-draw bootstrap coefficient and
contrast quantities plus 25,030 cohort/retained-median prediction cells. It
pinned all5,999 raw records and retained exactly5,006 complete cases, including
valid log-medical-expenditure zeros, at absolute tolerance1e-8 (tiny normal
p-values use relative1e-7 plus1e-300). The report labels the asymptotic normal
test and normal-Wald interval explicitly. This method-counterpart check does not
claim Stata8's bootstrap stream or its finite-sample F convention.

A reproduced near-active numerical failure is resolved by one stricter solve
of the same LP, retaining the original certificates; it does not redraw a
replicate. A small regression checks27 additional R coefficient/covariance/normal
inference quantities at the existing tolerances.

Committed synthetic references cover477 joint-bootstrap quantities and 72
contrast quantities on multi-predictor, noconstant, shifted, rescaled,
cancellation and overflow cases. Tests also check adding quantiles leaves
marginal draws unchanged, explicit failures and limits, exact point fits without
sparsity, model/data preservation and CLI/text/JSON agreement.

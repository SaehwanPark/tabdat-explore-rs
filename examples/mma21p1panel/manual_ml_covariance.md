# Manual fixed-ML-theta auxiliary covariance comparison

[manual_ml_covariance.td](manual_ml_covariance.td) delivers P2-ML-IID/CL,
manual source288–293 in the [pinned inventory](manual_inventory.md). It preflights
MOM, fits native balanced Gaussian RE-ML, retains population predictions,
constructs the original ML-theta quasi design, fits ordinary and person-cluster
auxiliary OLS, reports named models/both coefficient intervals and exports ordered
source/component/mean/population/quasi predictions. Existing primitives only;
no production Rust/API/dependency change. This is not a complete mixed source
table, fixed-ML bootstrap, ML-HC1 or historical Stata parity.

## Sample, design and inference

Original N5320/G532/T10. Supported3..532 complete ten-year people, unique exact
source_row/id/year in ranges1..5320/1..532/1979..1988, finite abs(lnhr/lnwg)<=20
and a valid identified native ML fit. Missing/incomplete model samples fail
instead of deleting rows. No weights, extra slopes or time effects. The converter
prefixes physical input lines; export retains all nine raw/key columns and is
ordered by source_row. Native ML population predictions precede the auxiliary
fits; final quasi predictions belong to the last clustered auxiliary OLS.

The original ML theta is fixed across covariance choices, not refitted per
auxiliary model. To form it using existing primitives, the script takes residuals
at the native fitted population coefficients. Let sw=sum of squared within-person
centered residuals and sb=sum of squared person-mean residuals repeated over all
T10 rows. At an interior balanced ML optimum the likelihood score equations give
sigma_e²=sw/(N-G) and v=sigma_e²+10*sigma_u²=sb/G. When sb/G<=sw/(N-G), the
constrained optimum has sigma_u²=0 and sigma_e²=v=(sw+sb)/N. Then theta=1-sqrt(e/v).
The [auditor](check_manual_ml_covariance.py) compares every reconstructed
component/theta with the native result and independent likelihood reference; this
external audit is not a runtime comparison against a separately accessible result.

For X=[1,quasi_x], quasi_y=y-theta*mean_y, quasi_x=x-theta*mean_x and
B=(X'X)^-1, the auxiliary coefficients equal [(1-theta)*native intercept,
native slope]. IID uses RSS_quasi/(N-2)*B and Student-t(N-2) intervals; person
CR1 uses G/(G-1)*(N-1)/(N-2)*B*sum_g(s_g s_g')*B, with s_g the sum of row
scores in a person and Student-t(G-1) intervals. Both fit residual DF=N-2;
cluster inference DF=G-1. These are working/large-sample references, not exact
finite-sample guarantees with estimated theta; CR1 assumes independent people and
adequate cluster asymptotics. The native ML coefficient covariance comes from
joint observed information at interior solutions, constrained observed information
at the unit-variance boundary, and normal inference. It is not the auxiliary IID
matrix and must not be silently substituted. No full joint covariance inversion
is needed for the auxiliary individual intervals. Predictions are population
xb/residual, not BLUP/conditional unit effects; the quasi residual is its
person-demeaned counterpart at theta. Final cluster intervals execute **after**
reports, notes, predictions and summarize to protect owned covariance/DF.

## Reference and failure evidence

The [development-only R4.6.2 reference](manual_ml_covariance_reference.R)
independently uses dense QR across a bounded profile, considers the constrained
zero-unit-variance candidate, and cross-checks its objective against a direct
10x10 covariance likelihood. Separate QR and person-score products check66
native component/coefficient/likelihood and auxiliary full-matrix/both-interval/
DF fields. All5,320x22 exported cells are compared, including47,880 exact raw/key
cells. Fixed absolute tolerance1e-9; p-relative1e-8/exact underflow0. The auditor
also checks named model snapshots, exact samples/group labels, covariance semantics,
late inference, explicit interpretation and successful final export. Eight forged
component/covariance/DF/sample/raw/design/note/export traces are rejected.

Original native ML theta=0.586481000430332, sigma_e²=0.05425040821874512,
sigma_u²=0.026300722026614744, slope=0.11954740235725304. Auxiliary quasi slope
SE is0.01365326632169196 (IID) or0.05154743805284433 (person CR1). This theta
differs from the original GLS theta0.5847092377438329; the GLS covariance workflow
is a separate calculation, not a substitute. R and native components agree within
the fixed tolerance, including reconstructed score-equation components.

[Two interleaved dyadic16-person controls](manual_gls_covariance_synthetic.py)
cover positive ML theta and a constrained theta0 boundary. Frozen independent
[fixtures](../../tests/fixtures/mma21_manual_ml_covariance/) add132 fields/7,040
cells to the [three actual-script tests](../../tests/mma21_manual_ml_covariance_workflow.rs).
They cover exact samples/cluster labels, text/JSON prediction-byte parity, late
cluster inference, source/export ownership and six invalid duplicate/incomplete/
missing/fractional-key/rank/zero-variance variants. The interior control also
checks ML theta differs from the independent Swamy–Arora theta. Errors do not
export or overwrite source/existing export; no whole-session rollback is claimed.
The dyadic controls avoid the documented [R decimal-parser discrepancy](manual_gls_covariance.md).

## Reproduce

From repo root with a current binary (`cargo build --locked`), local author
MOM.dat, and R available for reference only:

```sh
repo=$PWD
raw=/absolute/path/to/MOM.dat
work=$(mktemp -d)
sh examples/mma21p1panel/prepare.sh "$raw" "$work/MOM.csv"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json \
  -f "$repo/examples/mma21p1panel/manual_ml_covariance.td" > manual_ml_covariance.jsonl)
Rscript examples/mma21p1panel/manual_ml_covariance_reference.R "$raw" "$work/reference"
python3 examples/mma21p1panel/check_manual_ml_covariance.py "$work" "$work/reference" "$raw"
```

Use a separate fresh directory and `--text` for readable output. Converter/export
refuse existing targets; R requires a fresh reference directory. R takes raw
8-column MOM.dat, not prepared MOM.csv. Synthetic audit uses `--synthetic`, which
waives only the source hash. No publisher data/program is redistributed; Python/R
are not production dependencies. Fixed-ML whole-person bootstrap, source-labelled
GLS-HC1 anomaly/full mixed report, companion inventories and complete chapter/book
coverage remain required.

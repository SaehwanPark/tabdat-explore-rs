# Individual pooled Poisson whole-commune bootstrap

The [actual workflow](individual_pooled_bootstrap.td) covers the first
individual bootstrap block in `mma24p2poiscluster.do`, §24.7. The author says
these results are not reported in Table24.6; the pinned program and log use
B=50 despite a comment saying website output uses B=500. This workflow uses
B=500 and a declared native RNG. The separate [conditional-FE bootstrap](individual_conditional_fe_bootstrap.md)
refits each copied FE group; individual shared-gamma-RE bootstrap, survey design
and remaining illustrations remain required.

## Sample and method

Publisher ASCII SHA256 `e876f8ee17791881c6378de2761436e1e72e1eda84347089a9dc365ed290c85d`;
DTA SHA256 `284a5dd732926fe6ee02157012b17d8751523b89f1dd2f652a6f4dd021f3dac3`.
Outcome `pharvis`; numeric regressors in order `lnhhexp insurance sex age
married illdays actdays injury illness educ`, plus an intercept. `source_row`
is an ordering key, **not time**. Of27,766 source rows the one wholly missing
row is excluded, leaving27,765 people in194 communes, sizes51–206, including
20,668 zero counts and the 94-person all-zero commune64. The total count is
14,209. The source mapping and meaning of the original numeric codes are
explained in the [pooled workflow](README.md).

The script retains a native commune-clustered Poisson fit, then calls:

```text
bootstrap pharvis lnhhexp insurance sex age married illdays actdays injury illness educ, model(poisson) cluster(commune) reps(500) seed(10001)
```

Each replication draws194 whole communes with replacement, retaining every
person in each copied block and refitting all eleven pooled log-link Poisson
coefficients. The copies have no FE/RE interpretation. Report original
coefficients, bootstrap means and bias, full B−1 covariance and SE, and marginal
R-type7 percentile95 intervals. There are no analytic t/z tests, BCa or
studentized intervals, and finite B introduces Monte Carlo error. Independent
communes and a suitable pooled conditional-mean specification are assumptions;
this is neither survey-design nor causal inference, nor a few-cluster guarantee.

The command uses a continuous xoshiro256++ stream initialized with SplitMix64
seed10001 and the fixed-u64 rejection mapping, in **first-seen complete-case
commune order**. Each replication permits at most G+1024 RNG words. This is the
same index stream as the existing OLS pairs bootstrap for the same ordered
clusters, not the per-replication FE/RE SQL composition stream and not Stata's
historical RNG. The CLI-v1 `ClusterBootstrapPoissonResult` exposes zero-based
indices into first-seen cluster labels and sizes, every replicate's coefficients
and sample size, and the complete covariance matrix. Terminal text displays all
coefficient summaries. The script's no-clobber CSV contains original selected
source fields and original-model `pooled_mu`/`pooled_xb` predictions.

## Bounds and failure/state semantics

The generic native command requires an intercept, 2..500 replications, 2..256
complete-case clusters and at most16 coefficients. It checks
`G*largest_group_size<=40000` and
`G*largest_group_size*B*K^2<=2500000000` using checked arithmetic, with K
including the intercept. These are conservative work limits, not a time or
convergence guarantee. The fixed source script additionally limits source rows
and keys to1..27766, commune keys to1..194, group size to206, counts to
0..100000 and finite model fields to magnitude1e6. It excludes wholly missing
records but rejects partial records. The underlying generic command
accepts exact nonnegative counts up to2^53−1 after joint finite-case selection,
using the same loaded-type checks as native `poisson`.

Each copy uses the existing scaled-QR Poisson solver, its full-step/score and
line-search convergence checks and its iteration limit; these are not a general
analytic separation/existence test. Original or replicate rank, zero-outcome,
numerical boundary, nonfinite or convergence failure aborts the
entire report; no sample is redrawn or discarded. The command is read-only and
does not install bootstrap state or change a prior fit, dataset or named
reports. The original retained Poisson supplies current-row predictions. A later
export collision preserves its owner and retained fit, but scripts are not
whole-script transactions: earlier predictions or other effects are not undone.

On the local Apple M3 Max, a dev binary built with only `tabdat-stats`
optimized at level3 ran the original B500 in34.83s/169MB maximum RSS for ASCII
and33.77s/165MB for precise DTA. These are local measurements, not promises for
other machines, build profiles or datasets; an ordinary unoptimized debug build
can be much slower. The analytic original fit and 500 refits reuse native Rust;
R/Python are optional preparation/validation tools, not runtime dependencies.
Offsets, weights, noconstant, general nonlinear bootstrap, FE/RE refits,
simultaneous bands and general survey design are not provided by this command.

## Independent evidence

Base R4.6.2 independently parses publisher ASCII or directly reads DTA via
`foreign`0.8.92, expands every drawn raw group and performs an actual Poisson
`glm.fit` QR refit for each copy. An independent Python xoshiro oracle checks
all97,000 ordered native draw indices. Original ASCII and exact17-digit DTA
conversion each validate all500 eleven-parameter refits and resampled N, all
121 full-covariance cells and coefficient summaries/intervals, and every27,766
source/27,765 prediction row at fixed absolute1e-9. The maximum absolute
refit/summary/prediction discrepancies are2.01e-12 / 4.68e-14 / 6.61e-13 for
ASCII and2.02e-12 / 4.68e-14 / 9.17e-13 for DTA. Replicate N ranges from
26,315 to29,410. Original text and JSON prediction CSVs are byte-identical on
both sources. The original DTA native and R coefficients each match all11
printed author bootstrap **point** facts to half the final digit; historical
B50 bias, SE and interval values are not compared.

A 21-commune synthetic control includes an all-zero singleton, a wholly missing
source row, unequal group sizes and changing replicate N. All10,500 ordered
indices, 500 R QR refits, full covariance and all240 prediction rows are
audited; maximum refit/summary/prediction discrepancies are8.82e-12 / 7.46e-13 /
1.82e-13. Focused tests exercise all original/refit fields, exact wide DECIMAL
cluster identities, joint missing/count checks, singular and all-zero sampled
fits, work limits, RNG rejection bounds, full-width JSON seeds, retained state
and source/export ownership. Twenty full-auditor forgeries and one printed-point
forgery were rejected. No independent source-code review or hosted CI is claimed.

The optional DTA converter's default retains its former15-significant-digit CSV
for reproducibility. Earlier individual DTA audits therefore did not establish
binary64 identity with the original DTA inputs; `--precise` emits17 digits and
round-trips source values here. In this dataset52,163 nonempty cells differ at
binary64 precision from the old conversion. No native DTA reader is claimed.

## Reproduce

In a fresh directory, with `repo` pointing to this checkout and paths adjusted:

```sh
python3 "$repo/examples/mma24p2poiscluster/prepare.py" vietnam_ex2.asc vietnam_individual.csv
(cd "$repo" && cargo build --locked --config 'profile.dev.package.tabdat-stats.opt-level=3' --bin tabdat-explore-rs)
"$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma24p2poiscluster/individual_pooled_bootstrap.td" > result.jsonl
python3 "$repo/examples/mma24p2poiscluster/check_individual_pooled_bootstrap.py" oracle . vietnam_ex2.asc oracle.csv
Rscript "$repo/examples/mma24p2poiscluster/individual_pooled_bootstrap_reference.R" vietnam_ex2.asc oracle.csv reference
python3 "$repo/examples/mma24p2poiscluster/check_individual_pooled_bootstrap.py" check . reference vietnam_ex2.asc
```

Use `--text` in another fresh directory for readable summaries and the same
prediction CSV. For DTA, prepare using
`Rscript "$repo/examples/mma24p2poiscluster/dta_reference.R" vietnam_ex2.dta vietnam_individual.csv --precise`,
run the native script, pass `--dta` to the oracle, R reference and final auditor,
and run `check_individual_pooled_bootstrap_author.py vietnam_ex2.dta mma24p2poiscluster.txt reference .`.
Output, oracle and reference paths must not already exist.

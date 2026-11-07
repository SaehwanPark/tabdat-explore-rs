# Chapter 24 individual conditional Poisson FE whole-commune bootstrap

This bounded workflow delivers the commune-pairs bootstrap of the individual
conditional Poisson fixed-effects column in Cameron–Trivedi (2005) §24.7/Table
24.6. It loads and audits the source, fits/retains native conditional FE, refits
500 whole-commune copies, reports slope uncertainty and exports predictions from
the original fit. It is **not** a complete table, RE/GEE bootstrap, complex
survey analysis or causal interpretation. See the [README](README.md) for
publisher provenance, hashes and numeric-column interpretations, and the
[original FE analysis](individual_conditional_fe.md) for likelihood, model-based
inference, convergence and fitted-group prediction semantics.

## Sample and bootstrap contract

Outcome `PHARVIS`; ten numeric predictors in order `LNHHEXP INSURANCE SEX AGE
MARRIED ILLDAYS ACTDAYS INJURY ILLNESS EDUC`. `COMMUNE` is both the FE ID and
the resampling unit; `source_row` is not time. The 27,766-record publisher
source includes one wholly missing record. Joint-finite selection leaves 27,765
individuals in 194 communes; all-zero commune64 (94 individuals) is then excluded
from the conditional likelihood. The original retained FE sample and bootstrap
frame are **27,671 people in 193 positive-total communes**, sizes 51–206, with
total count 14,209. The pinned author bootstrap likewise reports N=27,671 and
G=193. Zero-total communes are not drawn. Positive singleton groups, if present,
remain eligible but supply no slope information. The exported 27,765 nonmissing
rows include zero-total commune64 with NULL fitted means/residuals and explicit
inclusion/commune-total fields.

`bootstrap pharvis ... , model(conditional_poisson) cluster(commune) reps(500)
seed(10001)` samples exactly 193 of these original retained communes uniformly
with replacement per replication. Each group copy contains every one of its
people and has a distinct FE identity, even if the same commune is drawn again.
All ten conditional slopes are refit, not held fixed. The local continuous
xoshiro256++/SplitMix64 stream uses first-seen positive-total groups and 64-bit
rejection mapping, with a maximum of G+1024 words per replicate. Changing row
order can change the general command's group mapping; this workflow explicitly
sorts its cohort by `source_row`. It makes no Stata RNG parity claim.

Report original slopes, bootstrap mean/bias, B−1 full covariance/SE and type7
marginal percentile95 intervals. Terminal text and CLI-v1 JSON agree; JSON also
exposes exact group labels/sizes, all ordered zero-based draws, copied sample
sizes and all replicate coefficients. The report is read-only: it does not
replace the original FE fit, labels, named reports or data, including after
failure. Fit `xtpoisson ..., fe id(commune)` separately for prediction. Finite
B has Monte Carlo error; these intervals are not simultaneous or a few-cluster
coverage guarantee. No analytic bootstrap p-value, BCa/studentized interval,
offset, weight, survey, random-effects or causal inference is provided.

Bounds: exact integer selected counts 0..100000; 1..16 unique slopes, no common
intercept, B2..500, G<=512 with G>K, G*largest_group_size<=40000, and
G*largest_group_size*B*K²<=2500000000 using checked arithmetic. The original FE
fit also audits raw key identity and enforces its own 40000 candidate-row,
rows*K²<=20 million, convergence, scaling and information conditions. Raw keys
must be nonmissing even when y/X would otherwise omit a row. Any original or
replicate rank, numerical or convergence failure aborts without a partial report
or redraw; the bounded optimizer is not a general separation/existence test.
Structural keys are exact internal representations, e.g. DECIMAL
`9007199254740993.1` appears as `90071992547409931e-1` in JSON. Scripts are
nontransactional: an export collision leaves the owner, source, current data and
retained FE fit unchanged, but does not undo preceding script effects.

## Reproduce and independently audit

Build a native binary with `cargo build --locked --bin tabdat-explore-rs`. For
this 500-refit workflow a development build with only the numerical crate
optimized is substantially faster:

```sh
cargo build --locked --bin tabdat-explore-rs --config 'profile.dev.package.tabdat-stats.opt-level=3'
repo=/absolute/path/to/tabdat-explore-rs
work=/absolute/path/to/empty/work-directory
python3 "$repo/examples/mma24p2poiscluster/prepare.py" /path/to/vietnam_ex2.asc "$work/vietnam_individual.csv"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma24p2poiscluster/individual_conditional_fe_bootstrap.td" > result.jsonl)
python3 "$repo/examples/mma24p2poiscluster/check_individual_conditional_fe_bootstrap.py" oracle "$work" /path/to/vietnam_ex2.asc "$work/draws.csv"
Rscript "$repo/examples/mma24p2poiscluster/individual_conditional_fe_bootstrap_reference.R" /path/to/vietnam_ex2.asc "$work/draws.csv" "$work/reference"
python3 "$repo/examples/mma24p2poiscluster/check_individual_conditional_fe_bootstrap.py" check "$work" "$work/reference" /path/to/vietnam_ex2.asc
```

The Python oracle independently replays **all 96,500 ordered indices** and
verifies labels, sizes and copied Ns before emitting draws. Base R 4.6.2
independently evaluates conditional scores/information and refits all 500
replicates, using group multiplicities; a repeated identical group contributes
its conditional score once per copy. This is algebraically equivalent to
separate raw-copy group identities. On a synthetic unbalanced 18-retained-group
fixture, an independent R dummy-Poisson GLM expanded raw rows and distinct copy
IDs for replicates 1, 250 and 500, including repeated groups and a positive
singleton; maximum GLM/reference slope difference was 3.993e-13. All 9,000
synthetic draws, 500 reference fits, full covariance and 149 source/148
prediction rows are checked. Maximum native/R refit/summary/prediction absolute
differences on that fixture were 2.43e-11 / 1.06e-13 / 3.55e-15. Twenty-four
full-auditor and one author-point forgeries were rejected.

Pinned publisher ASCII and directly read DTA each check all 500 ten-slope fits,
100 covariance cells, all summaries, all source rows and all prediction/exclusion
rows at fixed absolute tolerance **1e-9** (integer indices and inclusion flags
exact). Maximum native/R absolute refit/summary/prediction differences were
**9.05e-12 / 6.93e-14 / 8.53e-14** for ASCII and **9.05e-12 / 7.00e-14 /
2.49e-14** for precise DTA. Text and JSON prediction exports are byte-identical
for both; their ten displayed summary rows match numerically. The DTA native
input was made with `dta_reference.R ... --precise` (17-digit round trip); R
reads the pinned DTA directly. The older default converter is unchanged but can
round binary64 values; do not interpret its earlier audits as exact-input checks.

The author program and log use B=50 despite a website comment mentioning 500.
On precise DTA, **9 of 10** printed bootstrap original points match both R and
native within half a printed digit. The remaining `MARRIED` point is
**0.16067924785** in independently converged R/native, versus **.1606793** in
the author's bootstrap output; notably the same author's direct FE fit prints
**.1606792**. The cause of that last-digit discrepancy is unverified. No
tolerance was widened and historical B50 bootstrap uncertainty is not compared.
To audit the explicitly recorded discrepancy and pinned author hashes:

```sh
Rscript "$repo/examples/mma24p2poiscluster/dta_reference.R" /path/to/vietnam_ex2.dta "$work/vietnam_individual.csv" --precise  # fresh work directory
# Run the same native script in that directory, then generate draws with --dta:
python3 "$repo/examples/mma24p2poiscluster/check_individual_conditional_fe_bootstrap.py" oracle "$work" /path/to/vietnam_ex2.dta "$work/draws.csv" --dta
Rscript "$repo/examples/mma24p2poiscluster/individual_conditional_fe_bootstrap_reference.R" /path/to/vietnam_ex2.dta "$work/draws.csv" "$work/reference" --dta
python3 "$repo/examples/mma24p2poiscluster/check_individual_conditional_fe_bootstrap.py" check "$work" "$work/reference" /path/to/vietnam_ex2.dta --dta
python3 "$repo/examples/mma24p2poiscluster/check_individual_conditional_fe_bootstrap_author.py" /path/to/vietnam_ex2.dta /path/to/mma24p2poiscluster.txt "$work/reference" "$work/result.jsonl"
```

On the local M3 Max, the original JSON workflow with the optimized stats crate
took **25.31 s / 174 MB peak RSS** for ASCII and **23.88 s / 160 MB** for DTA;
these are observations, not runtime or platform guarantees. Unoptimized debug
runs may be much slower. R/Python and publisher data are optional development
inputs, not production dependencies. Test fixtures and commands are documented
in `tests/fixtures/mma24_individual_conditional_fe_bootstrap/README.md`.

The separate [shared-gamma RE bootstrap](individual_gamma_re_bootstrap.md)
refits distinct copied group effects. Other GEE workflows, survey-design inference,
remaining household outputs and other chapter illustrations remain required;
this bounded slice does not complete the chapter or the roadmap.

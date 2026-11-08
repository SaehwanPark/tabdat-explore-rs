# Patents: native shared-gamma random-effects Poisson

**Implemented, independently validated bounded workflow.** Connects existing
native `xtpoisson` to Cameron–Trivedi (2005), §23.3/Table23.1. No estimator,
parser/runtime/schema/dependency change. Medium-risk source/state integration;
not whole-table/chapter/book coverage or literal historical parity.

The [script](native_gamma_re.td) gives graduate panel-count users load/audit →
ordered preparation → joint native fit/diagnostic LR → owned prediction/
interpretation → terminal/JSON report/CSV reproduction. Unlike the older
[SQL composition](gamma_re_poisson.md), whose retained model is pooled Poisson,
this workflow retains the **actual shared-gamma RE fit** for later native
prediction. The composition remains unchanged as independent diagnostic evidence.

## Source, sample and model

`patr7079.asc` SHA256
`87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8`.
Existing `prepare.sh` projects nine columns:
`id year cusip ardssic scisect logk sumpat logr pat`.
ID is source-record ordinal, not CUSIP. All **346 firms /1730 firm-years** in
1975–79 remain, including22 all-zero firms and missing nonmodel ARDSSIC.
ARDSSIC is also allowed entirely NULL despite CSV VARCHAR inference. No metadata
missingness filter; no zero-total exclusion. Preserve `sumpat` as source metadata,
not a replacement selected-five-year total. Counts imported from decimal-form
canonical CSV may be DOUBLE; validate integer values rather than round them.

Raw guards admit2..346 firms, exact positive BIGINT id<=346 and BIGINT years
1975..79, unique pairs and all five years per loaded firm. The native fit also
requires G>3 (at least four firms for its three joint parameters). Patents must be finite
integers0..1000, LOGR finite within20; the other canonical metadata are finite,
except nullable ARDSSIC. Every loaded record is retained or the workflow fails.
Sort `(id,year)` before fitting. This bounded source contract is narrower than
the existing general native estimator, not a new generic panel-data interface.

Mean `mu_it=exp(a+b*LOGR_it)`, independent firms, shared multiplicative
`u_i~Gamma(shape=1/alpha,rate=1/alpha)`, conditional Poisson observations.
`E(u_i)=1`, `Var(u_i)=alpha`. This is **not** independent-row NB2 or posterior
firm-specific prediction. No year effects, weights, offset or causal assumption
is inferred. Alpha is independent of regressors under the fitted model.

Jointly estimate intercept/LOGR/ln(alpha), retaining factorial likelihood
constants and the inverse **full joint observed information**, including all
coefficient/dispersion cross terms. Normal z/two-sided p/95% intervals,
covariance correction1; alpha delta SE and exponentiated log-Wald interval.
No cluster-robust/few-firm/survey guarantee. `lnalpha=0` tests alpha=1,
**not** zero heterogeneity. The separate alpha=0 LR refits pooled Poisson on the
same sample and uses an equal chi-square0/1 boundary mixture. A printed p=0
can be numerical tail underflow, not mathematically zero probability.

Original native float64: intercept2.312704994995631, LOGR.348783173620768,
lnalpha.545469308737375, alpha1.725417946111643; log likelihood
−5553.178728223975. Native six-step convergence reaches scaled score8.54e-15
and scaled step2.37e-14 under existing1e-10 rules. Full rank, positive joint
information and interior dispersion are required; no clipping/fallback.

## Prediction, state and files

- `predict ..., n`: population marginal mean exp(a+b*LOGR).
- `predict ..., xb`: log mean.
- `predict ..., residuals`: current PAT minus marginal mean.

An RE residual mean need not be zero: original marginal mean20.0932975585 and
residual mean14.6783787421 differ from pooled-Poisson calibration. This is not
posterior shrinkage, a count-probability distribution, prediction interval or
validation of mean adequacy/exogeneity/out-of-sample accuracy.

`native_gamma_re_predictions.csv` contains original nine columns plus
`selected_firm_total,included_in_native_re,native_re_mean,native_re_xb,native_re_residual`,
all rows in strict id/year order, including zero-total firms. The final active
relation is `native_gamma_re_cohort`; source input bytes are untouched.
Summary/scope are read-only. Existing destination columns and export files
refuse collision. The whole script is ordered, **not transactional**: successful
load/preparation/fit/predictions remain after export failure. It does not restore
an earlier data/model session or overwrite to rerun; use a fresh directory.

Established native failure policy: a failed **new fit clears stale estimation**,
but retains current data/labels/immutable stored reports. It must not leave a
failed iterate or make subsequent prediction silently use an old pooled fit.
Parser/prediction errors preserve existing state; an immutable stored Poisson
report remains readable without restoring its model. The initial contract's
contrary failed-fit preservation assumption was corrected from existing
individual_gamma_re.md/runtime contracts, not by changing production semantics.

## Reproduce

Native CSV analysis has no R/Python dependency. Use pinned Rust1.99.1 and a fresh
directory; review source-containing CSVs before sharing them.

```sh
export PATH="$HOME/.cargo/bin:$PATH"
repo="$PWD"
work=/path/to/fresh-directory
curl -fsSL https://cameron.econ.ucdavis.edu/mmabook/patr7079.asc -o "$work/patr7079.asc"
shasum -a 256 "$work/patr7079.asc"  # Must match above.
sh examples/mma23p1pannonlin/prepare.sh "$work/patr7079.asc" "$work/patents.csv"
cargo build --locked --bin tabdat-explore-rs
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma23p1pannonlin/native_gamma_re.td" > result.jsonl)
ref=$(mktemp -d)
Rscript examples/mma23p1pannonlin/gamma_re_poisson_reference.R "$work/patr7079.asc" "$ref/reference"
python3 examples/mma23p1pannonlin/check_native_gamma_re.py "$work" "$ref/reference" "$work/patr7079.asc"
```

Use `--text` in another fresh directory for readable reporting; original/control
CSV exports are byte-identical to JSON. R/base-R and standard-library Python
are optional development validation only. No publisher raw data are committed.

## Evidence and supported limits

Unchanged base-R integrated-likelihood oracle, R4.6.2: BFGS plus observed-
information score refinement, central score-Hessian check and independent
NB-total/conditional-multinomial likelihood factorization. Original max score
3.27e-12, factorization difference1.00e-11. Both original and existing inspectable
12-firm/60-row control match **37 R quantities**, full joint covariance and every
source/prediction record/key/order. Exact Decimal source identity, fixed
`1e-9+1e-10*abs(reference)` numeric gates and relative1e-9 nonzero p gates; no
reference formula change or tolerance widening.

The auditor additionally checks score/likelihood/observed-information-inverse,
normal tails, alpha/CI/LR and mean/xb/residual identities, actual zero-firm/sample
membership, convergence and metadata. Original gates match **six selected printed
author facts**, separate from R. Primary author log SHA256
`92f68c76063cf083217724f1b1b45cfaf8ae7a8c7ce0b4a1d5f62accd86dc208`.
Historical lnalpha .5454692 versus native converged .5454693087 **does not match**
half a digit; prior float32 probes are explanatory evidence, not a substitute
oracle or proof of historical internals. No whole printed-column claim.

Six new Rust cases are accounted across runs: initial **5 pass/1 fail** exposed
the wrong draft state assumption; the corrected function passes separately.
Five untouched cases were not repeated and no second full-green six-case run is
claimed. Coverage:37 fields/full matrix/every prepared/exported field, JSON/text,
permutation, all-missing nonmodel metadata, owned RE repeat predictions/labels,
no-clobber files/columns,24 invalid/singular sources, and noninterior fit with
stale-model invalidation/data-label-registry preservation and denied prediction.

`python3 examples/mma23p1pannonlin/test_check_native_gamma_re.py`: **2/2**,
real native controls/permutation/all-missing ARDSSIC and **27 rejected forgeries**.
Shared wrong native/R means, z statistics, covariance/SE inference and LR p are
rejected by additional source/equation identities, not just matching artifacts.
Source/sample/order, zero firms, prediction scope and convergence corruption
also fail. Initial nonaggregate guard/scope repetition was caught and fixed to
one-row aggregates before final evidence.

Scoped target Clippy/fmt/diff pass. Prior unchanged root-base verification is
carried forward; no production/shared helper/schema or existing Rust target
changed. No fresh full-root/workspace/doctest/release/platform claim. No Rust
documentation example changed. Remaining illustrations, native generalizations
where needed, normal-RE/bootstrap/GEE outputs, full textbook inventory, I-DETAIL/
I-FREQ/I-GRAPH, historical household ML SE and URBAN/MEDEXP/LNHHEXP reconciliation
stay required; this slice does not resolve or exclude them. The separate
[native gamma-RE bootstrap](native_gamma_re_bootstrap.md) now validates original
B500 under its512-group bound: all346 firms/22 zero totals,500 joint R refits,
full covariance/173000 draws and unchanged original predictions. That workflow
has a distinct native stream and does not establish historical bootstrap parity.

# Native normal random-intercept Poisson: patents1975–79

For applied panel-count users who need a real retained fit rather than SQL-only
reported coefficients. This is a bounded CT2005 §23.3/Table23.1 adaptation, not
historical Stata convergence/inference parity or whole-table/chapter completion.
The [SQL composition](normal_re_poisson.md) remains separate and retains its
pooled starter; this native workflow owns the actual normal-RE model.

## Reproduce

Use a fresh output directory; exports refuse to overwrite an owner file. Obtain
[`patr7079.asc` from the publisher](https://cameron.econ.ucdavis.edu/mmabook/patr7079.asc).
From the repository root, with the executable built:

```sh
repo="$PWD"
mkdir /tmp/patents-normal-native
sh examples/mma23p1pannonlin/prepare.sh /path/to/patr7079.asc /tmp/patents-normal-native/patents.csv
cd /tmp/patents-normal-native
"$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma23p1pannonlin/native_normal_re.td" > result.jsonl
Rscript "$repo/examples/mma23p1pannonlin/normal_re_poisson_reference.R" /path/to/patr7079.asc reference
python3 "$repo/examples/mma23p1pannonlin/check_native_normal_re.py" . reference /path/to/patr7079.asc
```

R/Python are development validators, not production dependencies. For readable
output use `--text` in a separate fresh directory. The script loads the prepared
canonical nine-field CSV, checks keys/counts/finite model and metadata values,
sorts a five-year cohort, fits, predicts n/xb/residuals, summarizes and exports
`native_normal_re_predictions.csv`. It preserves nullable nonmodel ARDSSIC,
all1730 observations/346 firms and22 zero-total firms. It does not use SUMPAT
as a filter. Script steps are ordered, not one rollback transaction: an export
collision leaves the actual fit and preceding data preparation intact.

## Specify, fit and interpret

```text
xtpoisson pat logr, re distribution(normal) id(id)
predict native_re_mean, n
predict native_re_xb, xb
predict native_re_residual, residuals
```

Given independent shared u_g~N(0,v), counts are conditionally independent
Poisson(exp(X beta+u_g)); u is independent of X. This is a mean-zero **log**
effect, not mean-one gamma or independent-row NB. Default `re id(id)` is still
gamma; `distribution(gamma)` is equivalent. Distribution with FE is invalid;
unsupported selectors do not fall back.

Joint beta,t=ln(v) estimates use inverse full observed information, including
all cross terms, without a DF correction. Coefficients/t have normal95
inference; sigma=exp(t/2) has delta SE=sigma*SE(t)/2 and exponentiated t-based
log-Wald CI. The t z test tests v=1, not no heterogeneity. The v=0 LR separately
refits full-X pooled Poisson on the exact selected sample, with equal
chi-square0/1 reference; slope Wald uses chi-square. These asymptotic references
assume the declared model, not robust/few-group/survey/causal inference.

Original-source (intercept,slope,t)=1.414253125154783/.2929878940875187/
.9897262362097311, sigma=1.6402736999, log likelihood=-5554.017060213401.
The historical reported solution has missing primary SEs, a materially different
likelihood and nonstationary scores for this integrated model. Its cause is
unresolved; table zeros are not validated inference.

Owned current-data predictions use xb=X beta, n=exp(xb+v/2), NOT n=exp(xb).
They require complete numeric X, no current IDs or y; residuals additionally
need finite current y. They are population predictions, not posterior effects;
population residuals need not sum to zero. Missing inputs give NULL. Overflow
or unsupported prediction/store options fail without changing model/data.
A failed **new fit** clears stale estimation, preserving data, labels and stored
reports. Normal named snapshots, posterior predictions, general test/lincom,
robust covariance and automatic bootstrap are not supported. Normal bootstrap
and remaining named comparisons are required separate work.

## Supported limits and failure recovery

Native fitting supports unbalanced groups and singletons, intercept plus1..14
unique numeric slopes, exact selected counts/group totals0..100000. All raw
keys must be valid even on omitted y/X rows, retaining typed integer/decimal/
text/boolean identity. Joint-finite selection retains all zero groups.
Candidate N<=40000, raw G<=10000, selected G>slopes+2, and
176*N*(slopes+2)^2<=20000000. The source replay adds its balanced five-year and
canonical source guards; it does not imply native fitting requires balance.

Fixed physicists'80/96 GH rules adapt to the posterior mode/curvature and check
log integrals and centered intensity/prior moment discrepancies<=1e-9 at every
evaluation. This is a diagnostic, not a certified error bound. RMS-preconditioned
undamped SPD Newton steps require nondecreasing likelihood, at most32 halvings/
100 iterations, per-group score<=1e-10 and relative step<=1e-12. No ridge,
information repair or optimizer/pooled fallback. Whitened pivots>1e-12;
t strictly(-8,8), fit X beta[-40,40], mode bracket[-60,60], quadrature u[-100,100],
marginal log means[-700,700]. Pooled LR>1e-8 numerically distinguishes an interior
variance, not statistical significance. Prediction uses N<=40000 and
N*(slopes+2)^2<=20000000 without quadrature; normal n also checks its log range.

On rank, boundary, quadrature, convergence or representability failure, inspect
source/design and diagnostics rather than increasing tolerances. Choose
`poisson` explicitly for a zero-variance model; no automatic fallback is claimed.
For export collision, choose another fresh output path, not an overwrite flag.

## Independent evidence

- Original SHA-pinned publisher data: all1730 source/design/prediction rows,
 42 direct-R improper-integration quantities/full3x3 covariance; JSON/text
 exports byte-identical.39 quantities are report-observable; three signed scores
 are independently recomputed by the auditor, not exposed native fields.
- Existing frozen40-firm/200-record reference: full workflow, source permutation/
 wholly missing metadata, default FE/gamma wire preservation, owned model/labels,
 unsupported store/prediction and export no-clobber,24 invalid/singular sources.
- [Two-slope control](../../crates/tabdat-stats/tests/fixtures/normal_poisson/multivariate/README.md):
 120 observations/40 firms/eight singletons, sizes1..5,57 direct-R quantities/
 full4x4 covariance/every prediction. Two Richardson refinements check R's
 integrated information (maximum covariance difference5.45e-12 at fixed1e-10).
 Reversed rows/opposite1e3/1e-3 units preserve sample mapping, parameter/covariance
 transformations, Wald and predictions; CSV17 digits preserves the exact design.
- Six direct-R posterior controls include zero totals,100000 counts and tiny mean.
 Native GH normalization/moments through degree25 and analytic derivatives are
 checked against independent integration/numerical differentiation.
- The auditor separately uses vector adaptive Simpson (boundary log weights
 checked below-90), complete-data derivative/centered-score covariance equations
 and a raw pooled-Poisson Newton refit.31 forged artifacts reject; coherent wrong
 native/R covariance and null/LR/p pass with the raw guard mocked, then fail
 actual equations. It reuses only generic source/CSV helpers from the gamma
 auditor, not gamma likelihood. No certified Simpson tail error is claimed.

Fixed comparisons:1e-9+1e-10*abs(reference), positive-p relative1e-9. Raw score
and observed-information-inverse gates independently defend shared native/R
errors. Simpson uses2e-13 local tolerance; mode-centered expm1/Taylor kernel
and dimensionless moment units avoid subdivision chasing of cancellation/noisy
large raw moments. Neither production numerical equations nor reference gates
were loosened. Covariance symmetry uses the declared numerical gate, not
bit-exact equality after sequential unit conversion (observed1.4e-17 roundoff).

Validation: workspace all-target nextest2138/399 binaries/two pre-existing
skips (250.245s,75726022-9c68-47a7-af05-5be929a519eb). Author review corrected
a diagnostic label from information evaluations to iterations (line-search
trials also evaluate information), without numerical changes; affected reporting58
and root workflow/help2 then pass. Scoped six-crate all-target and root bin/
new-workflow/discovery Clippy, fmt and diff checks pass. These later focused
runs are not a second workspace pass. No actionable review findings remain.
This is author review, not delegated/independent code review; numerical oracles
are independent. CI is disabled; no doctest/platform/release or whole-book
completion claim. The [normal-bootstrap extension](native_normal_re_bootstrap.md)
is WIP: exact copied-source feasibility has exposed numerical limitations,
not a validated bootstrap command. Ordinary normal model/inference/sample/
prediction/rule/bound contracts remain; numerical comparison now preserves
tiny likelihood gains with stable posterior ratios and rejects rounded no-op
trials, without relaxing convergence or accepting epsilon-sized declines.
Bootstrap and unresolved inventory entries remain required.

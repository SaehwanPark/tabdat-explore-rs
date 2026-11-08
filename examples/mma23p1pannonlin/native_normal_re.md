# Native normal random-intercept Poisson — preimplementation contract

## Selected outcome and rationale

Deliver one bounded native CT2005 §23.3/Table23.1 normal random-intercept
Poisson workflow, replacing the SQL composition's pooled retained state with
actual owned normal-RE estimation and population predictions. This adds a
missing applied panel-count method after the source-sized FE/gamma bootstrap
barriers; another cap/reporting-only change is not the next priority. Existing
[normal SQL composition](normal_re_poisson.md) remains separate, with its own
local numerical procedure and independently integrated reference.

Proposed command: `xtpoisson y xvars, re distribution(normal) id(group)`.
Existing `re id(group)` remains shared-gamma RE and preserves its command wire;
explicit `distribution(gamma)` may select the same existing gamma variant.
Distribution selection is invalid with FE; no unsupported selector falls back.
Additive normal estimator/result/state are distinct, not renamed gamma values.
No new dependency, native/FFI boundary, production Python/R or estimator framework.

## Sample, design and model

Unweighted intercept plus1..14 unique numeric slopes; K=slopes+2 joint
parameters includes ln(sigma²). All raw group keys must be valid even on rows
omitted for missing y/X; preserve exact typed key identity. Joint-finite cases
are selected, all zero-total and singleton groups remain. Exact selected counts
and group totals0..100000, candidate N<=40000, raw groups<=10000, selected G>K,
and conservative176*N*K²<=20000000 quadrature/design work bound. Collinearity,
noninterior variance and failed quadrature/information/convergence are errors,
not dropped variables, clipping, ridge, redraw or pooled fallback.

Conditional on independent shared u_g~N(0,sigma²), rows are independent
Poisson(exp(X beta+u_g)); u is independent of X. This is a mean-zero log effect,
not shared mean-one gamma or independent-row NB2. Jointly estimate beta and
t=ln(sigma²) by integrated marginal maximum likelihood. Inverse full joint
observed information supplies normal95 coefficient/log-variance inference and
coefficient/slope-Wald chi-square; sigma SE=(sigma/2)*SE(t), positive log-Wald
interval via t. t=0 tests sigma²=1, not zero heterogeneity. Same-sample refitted
full-X pooled Poisson supplies the sigma²=0 LR null, with equal chi-square0/1
mixture only under stated model/boundary regularity. No robust/few-group/causal
or global maximum guarantee.

Population mean n=exp(X beta+sigma²/2), xb=X beta (not log(n)); residual=y−n.
Owned fit supplies current-data predictions from complete X without current
IDs, and residuals additionally need current y. No posterior/group effects or
prediction uncertainty. Fit failure clears stale estimation, preserving data,
labels and named reports like current native count fits; parser/prediction/
store errors preserve existing state. Bootstrap is NOT added in this slice.

## Numerical boundaries to validate

Reuse existing pure Rust matrix/Poisson/inference support. Generate fixed
physicists'80/96 Gauss-Hermite nodes/weights from the existing development-only
R recurrence generator, preserve its tiny positive tail weights, and validate
normalization/orthogonality against independent moments. No runtime R use.
Adaptive quadrature centers at the unique strictly concave posterior mode,
bracketed[-60,60]/55 bisections with root check, scale from posterior curvature.
Log-sum-exp weights and centered posterior covariance moments avoid raw-second-
moment cancellation. Compare both levels for every firm's log integral and
required posterior means/centered covariances in intensity/prior-score units
(M*EexpU,Eu²/v,M²*VarExpU,M*CovExpUu²/v,VarU²/v²,Eu/v,M*CovExpUu/v)
at every evaluated fit; reject
relative discrepancy>1e-9, not silently choose one rule as exact.

RMS-preconditioned beta plus t, full joint observed-information Newton steps,
likelihood-monotone bounded halving (at most32 trials), at most100 iterations;
scaled relative step<=1e-12 and per-group scaled score<=1e-10. Require positive
scale-aware undamped information pivots, t strictly(-8,8), fixed X beta[-40,40],
quadrature u[-100,100] and marginal prediction log-mean[-700,700]. An unusable
start/step or no ascending step fails without covariance/optimizer fallback.
These are conservative supported limits, not universal numerical guarantees.

## Predeclared acceptance (five cases)

1. **Trusted kernel:** existing40-firm/200-row source (eight zero-total firms),
   independent direct R adaptive improper integration/BFGS/Newton reference,
   all42 report fields/full joint covariance/every marginal prediction. Add
   multivariate/rescaling/unbalanced/singleton and typed-key controls; compare
   raw group scores/full information to numerical differentiation and R
   integration, not only another Gauss-Hermite implementation. Fixed scalar
   tolerance1e-9+1e-10*abs(reference), nonzero p relative1e-9; diagnose differences,
   never widen tolerances to conceal them.
2. **Original workflow:** pinned `patr7079.asc` SHA256
   `87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8`,
   all1730 1975–79 records/346 firms including22 zero-total firms, source-row
   ordinal ID (not CUSIP), intercept/LOGR only. Reuse exact source guards and
   independent42-field direct-R reference, verify full sample/design/order and
   all original population predictions, readable/CLI-v1/export reproduction.
3. **Failures:** exact rank deficiency, invalid counts/keys/weights/options,
   no positive-interior variance, nonconvergence/information/mode/quadrature/
   work failures reject honestly. Preserve stale-fit clearing/data/labels/
   named reports, prediction column/no-clobber files and failed-store ownership.
   Reproduce the documented high-variance80/96 discrepancy, no tolerance repair.
4. **Compatibility/state:** explicit/default gamma and FE still execute with
   unchanged wire/formulas/samples/inference/failure behavior. New normal state
   survives read-only diagnostics and predicts after reload/subset/no-ID data;
   xb vs population mean includes the declared variance shift. Missing/all-null
   nonmodel metadata/permutation do not change model sample or coefficients.
5. **Interpretation/evidence:** additive family-specific reports/JSON/help,
   owned state and exact independent evidence reject shared forged coefficient,
   covariance, variance, LR, quadrature and prediction artifacts through raw
   identities/integration. New/modified focused tests first, affected crates/
   reverse consumers before completion; SPEC/roadmap/ledger/inventory reconciled,
   author review, PR verification/merge/tree identity/branch cleanup. No kernel-
   only or parser-only merge advertised as a complete analysis.

## Independent reference and exclusions

Reuse `normal_re_poisson_reference.R`: direct improper integration, analytic
integrated scores/information, BFGS/Newton and Richardson differentiation;
R/Python remain development-only. Original targets: intercept1.4142531252,
slope.2929878941, t.9897262362, sigma1.6402736999, loglik−5554.0170602134.
The existing SQL result is a secondary cross-check, not the sole reference.

Historical output slope.815977/intercept1.156293/t−1.310299/loglik−6261.9825 has
missing SEs and is not stationary for the declared integrated likelihood.
Its cause remains unestablished; later0.000 SE placeholders do not prove zero
uncertainty. No historical or whole-table/chapter/book parity is claimed.
Normal-RE bootstrap, robust/cluster covariance, offsets/weights/factors,
posterior effects, named snapshots/general restrictions, new links/distributions,
source named comparisons and full illustration inventory remain required where
applicable, not silently excluded. If numerical evidence contradicts a declared
bound/algorithm, document and validate the smallest explicit design revision;
do not silently change statistical semantics or thresholds.

## Verified pre-estimator checkpoint (not merge-ready)

Test-only adaptive quadrature primitive now has independent R4.6.2 improper-
integration controls: six artificial groups/six posterior quantities each,
including zero totals and100000 counts, pass at unchanged scalar gates. Both
fixed80/96 rules have positive/ordered weights and exact Gaussian-moment checks
through degree25. A real high-variance t=2 pooled-R-GLM start on the existing
40-firm source exposes paired-rule disagreement and is rejected; invalid
parameters/unbracketed modes fail. Focused four tests passed under locked
Rust1.99.1 (f620e2a6-e676-493c-974c-b1b7088ee3f8,0.024s;77 unrelated tests
filtered out), with stats lib/tests Clippy. After adding explicit six-case/
nine-column fixture-count assertions, that modified case separately passes1/1
(ee1ecbef-172d-49d8-8c30-70c5a4cd72e3); this is not a new broad/final4 run.
No broad baseline rerun claimed.

Further test-only joint-likelihood checkpoint: independent R controls now
include E(u)/Cov(exp(u),u), eight quantities per group. Integration by parts
supplies count-total score Eu/v and curvature Cov(lambda,u)/v, avoiding the
large-count E(lambda)−Var(lambda) subtraction without changing the likelihood.
General multivariate integrated scores/full joint information are implemented
and checked against Richardson likelihood/score derivatives on unbalanced/
singleton groups; malformed partitions/designs fail. Seven focused tests pass
(d761f3c9-55aa-4620-ae39-8273f075ffe1,0.024s),77 unrelated filtered.
Clippy exposed a needless-index loop in information normalization; iterator/
split-borrow symmetry construction replaces it without changing mathematics.
The two affected evaluator cases pass separately2/2
(41cdb9af-efb6-4f4e-87b2-1c3cc1abb95b), and final stats lib/tests Clippy passes.
Earlier other five passing tests remain applicable; no fresh final7/broad run
is claimed. No fitted inference/source-native workflow is claimed at that checkpoint.

A bounded test-only Newton optimizer now uses diagonal-whitened undamped SPD
information (existing inverse with dimensionless pivots>1e-12),100-iteration/
32-halving limits, nondecreasing likelihood and the declared joint score/
relative-step criteria; no ridge/fallback. Existing200-row/40-firm direct-R
control validates all three MLE parameters, six covariance entries and log
likelihood (ten scalar quantities), plus all200 population mean/residual/key/
design records. Iteration limits, high-variance quadrature start and indefinite/
near-singular information fail; diagonal whitening handles disparate units.
The two new optimizer cases pass2/2 (8afad200-7398-4b8a-81ae-69092d49f917,
0.039s), with final stats lib/tests Clippy. Earlier seven mathematical checks
remain applicable; no new final9/broad pass is claimed. This is not yet a public
fit API, sample adapter, full42-field inference or original1730-source proof.

## Public statistics API checkpoint (still not a delivered command)

`fit_normal_poisson`/`NormalPoissonOptions`/`NormalPoissonResult` now exist in
production Rust statistics, with validated sample provenance, complete-case
count/group limits, RMS/QR preprocessing, detached owned pooled initialization/
same-sample null, full joint observed covariance, normal/log-Wald inference,
sigma delta SE/interval and boundary-mixture LR. Positive variance must have
pooled LR>1e-8 to distinguish it numerically from the boundary (not a significance
cutoff); undamped information uses dimensionless whitened pivots>1e-12.
The ln-variance coefficient
name is `lnsigma2`; all parameter names are unique. Population means include
the variance shift; native residual/prediction dispatch is not yet integrated.
Invalid candidate/sample/work/keys alignment/weights/rank/options reject.

New API2/2 (8cca57ca-c29a-4bde-882b-fc32ceb1a158) validates all42 existing
independent R fields/full covariance on200 records/40 firms, input/sample
immutability and serde round-trip; rescaling1e-3/1e3 preserves predictions/
transformed covariance, with ten explicit invalid contracts. Fresh affected
stats320 pass/58 binaries/one pre-existing skip,12.591s,
59af8cf8-104e-4a46-af1d-ca87e20145e4; stats-all-target Clippy passes. The
candidate-row failure control was then hardened to keep sample mapping valid
so it reaches the actual row bound; that modified case separately passes1/1
(b4ec09ec-9394-4bb4-a70f-c41d6e100ece), not a new final320 run.

A temporary locked Rust API probe on original1730/346/22-zero-firm data and
fresh R4.6.2 direct integration validates39 observable R fields/full covariance
and all1730 population mean/residual/key/design records. Three signed scores
are not API-exposed and not compared by that probe; control unit tests compare
them internally. Original point is(1.414253125154783,.2929878940875187,
.9897262362097311), loglik−5554.017060213401, seven evaluations; score/relative
step/final paired-rule difference6.74e-16/1.09e-15/3.36e-11. A first temporary
probe wrongly assumed25 tokens per physical line; publisher records wrap.
Corrected to25-token records, with no statistical code/tolerance change. No
publisher data or temporary example is committed.

Independent multivariate control now validates120 observations/40 firms/eight
singletons, group sizes1..5 and two slopes. The separate direct-improper-R
oracle checks integrated information with two Richardson refinements (maximum
covariance difference5.45e-12), not native GH or its Stein decomposition. All57
R quantities/full4x4 covariance and all120 exact key/design/count/prediction
records pass, including nonzero p relative1e-9. Source/covariate CSV uses17
digits to preserve the exact fitted design. Reversed rows/opposite1e3/1e-3
units preserve sample mapping, transformed coefficients/full covariance,
Wald and predictions. Modified/new case passes1/1
(84c1d558-3469-4be4-b327-fab82bdc8e2a), stats-all-target Clippy passes.
The retained320-test baseline plus this new case is not a fresh321 run.

No parser/execution/owned state/application reports, full original-source audit
or completed slice is claimed. Next typed execution/prediction/reporting and
actual-source workflow. Do not merge this kernel-only WIP.
Artifacts `/tmp/tabdat-ch23-native-normal/`.

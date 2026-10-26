# Original Table17.8 Weibull full-time re-employment column

## Native contract

Cameron–Trivedi (2005), §17.11 pp.603–608, March2005 `mma17p4duration.do`,
Table17.8 Weibull coefficient column. This is one complete native analysis,
not all Chapter17 models/tables/figures. The [exponential companion](README.md)
uses existing offset-Poisson composition; Weibull needs native duration ML.

Original DTA:3343 one-record spells,1073 full-time events,20887 recorded two-week
intervals, all40 author regressors+intercept. `censor1=1` marks full-time
re-employment. Other outcomes, including102 rows with all four status flags0,
remain non-events for that cause. Flags are nonoverlapping, NOT exhaustive.
ASCII rounds floats and omits required stateur: never silently substitute a
reduced model. Source ordinals and all44 source/ordinal columns are preserved.
Recorded integer durations are treated as exact continuous times, as by the
author—not an interval-censored/grouped-time likelihood.

Installed syntax:

```text
streg time xvars, failure(event) dist(weibull) [robust noconstant]
predict index, xb
predict current_hazard, hazard
predict integrated_hazard, cumhaz
predict net_survival, survival
```

PH eta=X beta, shape a>0, H(t)=exp(eta)*t^a, h(t)=a*exp(eta)*t^(a-1),
S(t)=exp(-H). Optimize jointly concave natural(beta,a) with RMS-coordinate
conditioning, strict mean-score AND relative-step convergence; report full
beta/lnshape covariance. No shape clipping, ridge or pseudoinverse. Bounds:
N<=10000, P=K+shape<=64, N>P, N*P*P<=20000000, regular full-rank information.
All unweighted joint-finite time/event/X rows enter before positive-time/exact
binary-event validation; require at least one failure. Original DECIMAL event
identity is checked before f64 rounding, after joint finite eligibility: near-one
codes error on eligible rows but do not reject already-excluded rows. The focused
runtime/CLI regression preserves valid DECIMAL0/1 R-reference results and checks
failed-fit stale-inference clearing, labels/data preservation and script stopping.
Original script uses a stronger complete full-source guard and never silently
drops an original spell.

Model-based inverse observed information or independent-spell robust full
observed-information/score sandwich times N/(N-1), NOT residual-DF HC1. PH beta
has normal95 inference; shape has delta SE and positive exponentiated-logshape95
bounds. Logshape z tests a=1 (constant hazard within Weibull), NOT shape=0 or
overall model adequacy. Full joint beta/lnshape cross terms are indispensable.

The same-sample null refits intercept+shape, or shape only with PH index fixed0
for noconstant. Robust LR references are withheld; slope-only rank-guarded Wald
is separate. Density LL and Stata-normalized LL differ by fixed sum(event*log t).
No-intercept models are not generally invariant to duration-unit changes.

Prediction uses CURRENT rows: xb (bare default) requires current X only; timed
hazard/cumhaz/survival require current X/time, NEVER event. Any required missing/
nonfinite input yields NULL. Complete finite nonpositive time or unrepresentable
requested calculations fail atomically; genuine survival0 remains valid even if
H cannot be represented. At most10000 current rows, typed DOUBLE/NULL, points
without parameter uncertainty. Prediction failure preserves order, labels,
targets and owned fit; failed new runtime fits clear stale inference only.

## Reproduce

Build `cargo build --locked` at the repository root. The explicit development-only
conversion uses foreign0.8.91; the application requires neither R nor Python,
and no production/native dependency was added. CSV preserves numeric values,
not original DTA storage types/value-label metadata. Use a fresh directory:

```sh
repo=/absolute/path/to/tabdat-explore-rs
mkdir weibull-analysis && cd weibull-analysis
base=https://cameron.econ.ucdavis.edu/mmabook
curl -fsSLO "$base/mma17p4duration.do"
curl -fsSLO "$base/mma17p4duration.txt"
curl -fsSLO "$base/ema1996.dta"
curl -fsSLO "$base/ema1996.asc"
Rscript "$repo/examples/mma17p4duration/prepare.R" ema1996.dta duration.csv
"$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma17p4duration/weibull.td" > native.json
Rscript "$repo/examples/mma17p4duration/weibull_reference.R" duration.csv reference ema1996.dta
python3 "$repo/examples/mma17p4duration/check_weibull_source.py" \
  mma17p4duration.do mma17p4duration.txt ema1996.dta ema1996.asc . reference
```

The script fits/reports full robust inference, exports original/UI0/UI1 PH index,
hazard, cumulative hazard and net survival at SAME observed times. UI and both
UI interactions change together and are restored before `weibull_predictions.csv`
(59 columns:44 source/ordinal,three helpers,twelve points). No existing file is
replaced. Final UI-group score balances follow included intercept/UI equations:
NOT adequacy/noninformative-censoring tests. Readable `--text` uses the same script
in another fresh directory containing duration.csv. This is ordered script
execution, NOT transactional rollback of preceding in-memory commands.

The four original SHA256 pins are enforced by check_weibull_source.py; original
DTA MD5 is cc37709732a1d5b65f73536563ebbf3e. Original assets are acquired by
contributors, not redistributed. The reference reads DTA directly: R4.6.1 can
reread correct17-digit CSV text with1ULP drift. Native/Python source values are
checked against original promoted DTA hex; never loosen exact source/numerical
tolerances to conceal a decimal-parser artifact.

## Independent evidence

R4.6.1/survival3.8.6 `survreg` independently fits AFT. For AFT(b,ell=log sigma),
beta=-b/sigma and q=lnshape=-ell. Full Jacobian d beta/d b=-a I,
d beta/d ell=-beta, d q/d ell=-1 transforms ALL covariance entries. Merely
rescaling the beta block loses shape uncertainty/cross terms. The reference also
reconstructs natural-shape observed information/sandwich separately.

Original native robust shape1.1292248659060844, density LL−3933.489879652094 and
normalized LL−2687.5995196287336 agree with R and author's−2687.5995. Audits pass:

- 2031 R gates, including ALL1764 joint covariance entries, full normal/shape/CI/
  null/LR/Wald and coupled current points;
- 197237 native source/design/projection cells and147092 EXACT source/ordinal cells;
- 276 author printed quantities, including14 compact Table17.8 coefficient/z cells,
  plus all140406 rounded ASCII fields/stateur omission. Not separate page typography;
- original JSON/text CSV exports byte-identical.

Python standard-library audit independently reconstructs actual logshape score/
observed Hessian/full sandwich, normal/shape inference, slope Wald/exact even-df
gamma tail, all current points, and a scalar PROFILE null solved by bisection
rather than native joint Newton. Ordinary tolerance1e-9 absolute+1e-8 relative;
positive probability/projection tails1e-300+1e-8 relative, exact source values
without tolerance, author numbers half final displayed unit. No unexplained
numerical difference/tolerance widening. Independent score bound1e-9,
information-inverse identity1e-8; no comparison-only covariance acceptance.

Permanent controls:7 kernel tests/7948 R gates span model-based/robust and
intercept/no-intercept, full covariance,4800 unique point predictions, finite-
difference score/cross-Hessian, unit covariance Jacobian, row permutations,
invalid support/rank/options and exponential/probability limits. Five runtime
tests cover retained positions/missingness, all inference modes/current edits,
unnecessary time/event columns, atomic target/schema/support/overflow failures,
labels/owned fit and stale-state clearing. Two parser and four actual-script
root tests include both CLI modes/23600 control cells/source-file guards and
coherent shared-wrong native/R beta-shape cross covariance, normal/shape/Wald
tails and survival mutations. The existing count parser regression also passes.

An independent read-only model review found no concrete kernel/state/reporting
issues; it inspected source, did not reproduce executed evidence, and is NOT
human statistical approval. PR #333 records full affected/reverse-consumer
regression, validation methods/limits and exact-head quality evidence. Local
macOS checks are not hosted CI, other-platform or release certification.

## Interpretation and limits

This is full-time CAUSE-SPECIFIC net survival, NOT observed all-cause joblessness
survival, competing-risk incidence, or causal UI effects. UI hazard ratio at
replacement r/disregard d is exp(beta_ui+r*beta_rr_ui+d*beta_dr_ui), not merely
exp(beta_ui). Independent-spell/model/external-censoring assumptions apply;
no latent-cause independence is asserted. No prediction/effect intervals.

Native exponential/cluster parse but explicitly fail; composed exponential is
unchanged. No weights/survey, delayed entry/left/interval censoring/truncation,
frailty/Cox/Gompertz, generic stset/sts, margins/general tests/named-model storage,
full hazard-ratio Table17.9 or whole-Chapter17/family/book completion claim.

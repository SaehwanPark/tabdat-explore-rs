# Original Table17.8 Gompertz full-time re-employment column

## Bounded native analysis

Cameron–Trivedi (2005), §17.11 pp.603–608, March2005 `mma17p4duration.do`,
Gompertz coefficient column. Original3343 one-record spells/1073 full-time events/
20887 recorded two-week intervals, all40 author covariates+intercept. Other
causes and102 all-zero status rows remain non-events. ASCII omits stateur and
rounds source values; do not substitute a reduced design. All44 source/ordinal
columns are preserved numerically, not as original DTA storage/label metadata.
Recorded durations are exact continuous times here, not an interval-censor model.

```text
streg time xvars, failure(event) dist(gompertz) [robust noconstant]
predict index, xb
predict current_hazard, hazard
predict integrated_hazard, cumhaz
predict net_survival, survival
```

At least one explicit predictor is required; intercept-only is additionally a
kernel/API control. PH eta=X beta, h=exp(eta+gamma*t),
H=exp(eta)*t*exprel(gamma*t), S=exp(-H), exprel(z)=expm1(z)/z, exprel(0)=1.
Gamma is unrestricted, including zero/negative: NOT log gamma or positive scale.
[Established Gompertz documentation](https://chjackson.github.io/flexsurv/reference/Gompertz.html)
uses the same Stata-compatible parameterization. Gamma0 is exponential within
this family, not overall adequacy. Gamma<0 implies H(infinity)=exp(eta)/(-gamma)
and positive survival mass at infinity: do not force a proper tail or claim
identified causal cure. Gamma/hazard units are inverse recorded time units.

Joint finite time/event/X eligibility precedes positive-time/original-cell exact
binary-event checks. Eligible near-one DECIMAL codes error, excluded rows do not;
require a failure. N<=10000, P=location coefficients+gamma<=64, N>P,
N*P*P<=20000000, full-rank regular information. Shared guarded concave Newton on
RMS X and linearly rescaled natural gamma requires mean-score AND relative-step
bounds1e-10, undamped regular information and bounded iterations. No sign/shape
clipping, ridge, pseudoinverse or nonconverged publication.

Full beta/natural-gamma observed inverse or independent-spell FULL row-score
sandwich N/(N-1), including cross terms, fixed normal95. Gamma z/CI test gamma0;
CIs may cross zero. Zero/unrepresentable inference is unavailable, not exact.
The same-sample null refits intercept+gamma, or gamma only at index0 for
noconstant. Slope Wald is rank-aware; robust LR p/reference is withheld.
Density LL=sum[d*(eta+gamma*t)-H]; Stata-normalized LL adds fixed sum(d*log t).

Current xb (also bare default) needs only current X; timed modes need current
X/time, never event. Required missing/nonfinite cells give NULL. Time0 gives
h=exp(eta), H0/S1; negative finite time/unrepresentable required calculations fail
before publication. Represented survival0 remains valid when H overflows but
log H is represented. At most10000 current rows, atomic DOUBLE/NULL columns,
no replacement/auxiliaries/uncertainty. Failure preserves data/order/labels/fit;
executed new-fit failures clear stale inference only. Parse failures do not execute.

## Reproduce original source

Acquire the same four assets as the [exponential](README.md)/[Weibull](weibull.md)
companions from `https://cameron.econ.ucdavis.edu/mmabook`: `mma17p4duration.do`,
`mma17p4duration.txt`, `ema1996.dta`, `ema1996.asc`. No originals are redistributed.
Build `cargo build --locked` at repository root; in a FRESH analysis directory:

```sh
repo=/absolute/path/to/tabdat-explore-rs
Rscript "$repo/examples/mma17p4duration/prepare.R" ema1996.dta duration.csv
"$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma17p4duration/gompertz.td" > native.json
Rscript "$repo/examples/mma17p4duration/gompertz_reference.R" duration.csv reference ema1996.dta
python3 "$repo/examples/mma17p4duration/check_gompertz_source.py" \
  mma17p4duration.do mma17p4duration.txt ema1996.dta ema1996.asc . reference
```

`gompertz.td` guards complete original input, fits/reports robust inference,
scores observed/UI0/UI1 xb/h/H/S at SAME times, changes UI and BOTH interactions
together, restores all source values and exports `gompertz_predictions.csv`.
59 columns:44 source/ordinal,three helpers,twelve points. Existing files are not
replaced. Final UI score balances are included-regressor equations, not adequacy
or censoring tests. Use `--text` in another fresh directory for readable output;
ordered script execution does not roll back earlier in-memory commands.
Original SHA256 pins are enforced by the source checker. R reference reads DTA
for exact promoted source doubles: do not loosen hex equality for CSV-parser drift.

## Independent evidence

R4.6.1/foreign0.8.91: numerical time-moment integration, conditional Poisson-GLM
beta optimization and bracketed scalar gamma profile-score root, NOT native joint
Newton/closed moments. Conditional GLM is an optimization identity, not a Poisson
duration DGP. Full observed curvature, row-score sandwich and `optimHess` numerical
certificates are checked; no Gompertz package/production R/Python dependency.
[Controls/reproduction](../../tests/fixtures/gompertz/README.md) include400-row
full design/model/robust/no-intercept and direct-integral negative-gamma matrices.

Original native gamma=.002658034540134388, robust SE=.006775913960645321,
density LL=-3946.4953351777904, normalized LL=-2700.60497515443,
refitted-null normalized LL=-3002.026000460346, Wald529.7502954600209.
Mean-score/relative-step bounds1.58e-16/7.33e-16. Audits pass:
- 2029 R gates, ALL1764 joint covariance entries, normal/gamma/null/LR/Wald;
- 197237 source/design/current-projection cells,147092 exact source/ordinal cells;
- 268 detailed/scalar/compact author quantities, including14 Table17.8 cells,
  plus140406 rounded ASCII fields/stateur omission;
- original JSON/text exports byte-identical, coupled UI source values restored.

The standard-library Python audit independently uses segmented Gauss–Legendre16
quadrature, all natural information/score/sandwich blocks, erfc/df40 tails and a
scalar conditional-baseline null. Its offline |gamma*time|<=1024 bound is NOT a
production constraint. Quadrature agrees with all15 independent R controls through
zero/both signs/±1000; shared-native/R wrong tail/cross-covariance/survival forgeries
are rejected. Ordinary tolerance1e-9 absolute+1e-8 relative; positive tails
1e-300+1e-8 relative; original hex exact; printed half-last-unit. Independent score
bound1e-9/inverse identity1e-8; no unexplained differences/tolerance widening.

Eleven kernel,seven runtime,two parser,three CLI andthree actual-script controls
cover full references, derivatives, gamma0 full-information exponential and
negative-tail fits, time/X covariance transformations, identities, invalid support/
resource/convergence, DECIMAL eligibility, current edits, time0, NULLs, state/
label/data atomicity and no-clobber export. Independent read-only kernel/oracle
and integration/source-audit reviews found no issues. Broader local evidence:
1231 distinct passes/two pre-existing ignored; affected134-artifact regression
uses current Cargo inventory/SHA256-identical fresh-copy standard harness for the
previously diagnosed host dyld-start issue, not a broad nextest claim. The one
pinned Python diagnostic-list extension is explicit MIG-0022; original corpus
unchanged. streg schema required predictors now matches its pre-existing parser.

## Limits

Unweighted independent entry-zero/single-spell right censoring only: no cluster/
survey/weights, other entry/censoring/truncation, frailty, margins/general tests,
named survival storage, formal adequacy or point uncertainty. Net cause-specific
survival is not all-cause joblessness survival/competing-risk incidence; UI points
are observational, not causal. Native exponential remains unsupported; use its
separate composition. No full Table17.9/HR uncertainty, joint table layout,
whole survival/chapter/book completion or platform/release certification. GitHub
Actions is disabled; local evidence is not CI success. Required gaps remain.

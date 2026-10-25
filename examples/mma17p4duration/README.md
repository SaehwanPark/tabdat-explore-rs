# Empirical exponential full-time re-employment hazard

The separate [native Weibull Table17.8 column](weibull.md) shares this exact source
and preparation, but has its own duration likelihood/retained family, current-time
point modes, full beta/lnshape inference, script and independent source audit.
The exclusions below describe this exponential composition, not all app features.

## Implemented contract

Cameron–Trivedi (2005) §17.11 pp.603–608, `mma17p4duration.do`,
Table17.8 exponential coefficient column. Original `ema1996.dta`:3343 one-record
spells,1073 full-time events,20887 observed two-week intervals, time1..28;
all40 author regressors plus intercept, complete finite sample. `censor1=1`
is full-time re-employment; part-time/unknown-status re-employment and continued
joblessness are non-events for this cause. Also retain the102 original rows
whose four status flags are all zero: the author still treats censor1=0 as
censored. Flags are nonoverlapping, not exhaustive; do not impose one-hot
membership or invent a status. Original source ordinals retained.

Use native `poisson censor1 ... , offset(log_spell) robust` for the EXACT
exponential cause-specific likelihood parameter estimates and robust covariance:
log mean=log(spell)+X beta. This is event/exposure likelihood algebra, **not a
Poisson count data-generating assumption**. The duration-density log likelihood
is sum[d Xbeta - t exp(Xbeta)]; adding sum[d log(t)] gives native Poisson/Stata's
reported normalized likelihood for binary d. R survival exponential AFT beta has
the opposite sign; its covariance maps unchanged. Native independent-spell
sandwich uses observed bread and N/(N-1), matching author Stata, normal z/95%.
No robust LR p-value or Poisson Pearson/deviance survival-adequacy claim.

The author ASCII outfile omits required `stateur` and rounds other variables.
Use a development-only R/foreign conversion of the original DTA, not an invented
stateur value or silently reduced model. No native DTA/R/Python dependency is
added to the application. Aliases RR/DR/UI/LOGWAGE map to native original names
reprate/disrate/ui/logwage; explicitly generated interactions remain coupled to UI.

Acceptance:
1. Original3343/1073 sample/design, all41 coefficients, full covariance, robust
   normal inference/omnibus and both likelihood conventions match independent
   R survival plus source printed values with explicit tolerances.
2. Native script fits/reports the full robust model, exports observed-time
   cumulative hazard, instantaneous hazard, net survival and mean duration,
   plus conditional UI0/UI1 projections and fixed-horizon net survival points.
   Every source/design/prediction cell is independently audited.
3. Actual-script synthetic full-design control validates both CLI modes, sample
   preservation, UI/interactions restoration and point predictions; malformed
   source/design and output collisions stop safely without overwriting files.
4. DTA conversion rejects wrong schema/source, missingness and existing outputs,
   publishing only a complete new CSV; native sample guards do not silently
   delete incomplete or invalid durations/flags/covariates.

Interpretation: constant full-time cause-specific hazard conditional on covariates;
net exp(-hazard*t) is NOT all-cause joblessness survival or the full-time cumulative
incidence in the presence of competing exits. Independent external censoring,
independent spells and model assumptions apply; UI filing associations/projections
are not causal effects. No latent-competing-risk independence is asserted.

Exclusions: Weibull/Gompertz/Cox, nonconstant hazards, full hazard-ratio table17.9,
prediction/effect uncertainty, survey/cluster/weights, generic survival syntax,
formal exponential adequacy tests, all table/figure/chapter/family coverage.

## Reproduce the original analysis

Build the application with `cargo build --locked` at the repository root, then
use a fresh directory. Original assets are downloaded by contributors and are
not redistributed in this repository:

```sh
repo=/absolute/path/to/tabdat-explore-rs
mkdir duration-analysis && cd duration-analysis
base=https://cameron.econ.ucdavis.edu/mmabook
curl -fsSLO "$base/mma17p4duration.do"
curl -fsSLO "$base/mma17p4duration.txt"
curl -fsSLO "$base/ema1996.dta"
curl -fsSLO "$base/ema1996.asc"
Rscript "$repo/examples/mma17p4duration/prepare.R" ema1996.dta duration.csv
"$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma17p4duration/exponential.td" > native.json
Rscript "$repo/examples/mma17p4duration/reference.R" duration.csv reference ema1996.dta
python3 "$repo/examples/mma17p4duration/check_source.py" \
  mma17p4duration.do mma17p4duration.txt ema1996.dta ema1996.asc . reference
```

`prepare.R` is **development-only source acquisition/conversion**, not native DTA
IO or an application dependency. It uses foreign0.8.91, pins original DTA identity
with built-in MD5, checks exact schema/sample/numeric values, writes17-significant-
digit CSV strings and publishes a completed file through a no-overwrite hard link.
The full source audit separately pins SHA256 for all four assets. Users may supply
an equivalent numeric CSV converted by another tool; the application itself needs
neither R nor Python. DTA storage types/value labels are not native CSV metadata.

For a readable coefficient/diagnostic report, run the same script with `--text`
in a fresh directory containing `duration.csv`. Keep the JSON run for numerical
source auditing. The only published native output file is
`exponential_predictions.csv`: all44 source/ordinal columns plus derived design
and18 total derived/prediction columns,62 columns overall. Neither input nor an
existing output file is replaced. Invalid full designs fail before publication;
output failure stops the subsequent UI score-summary query. This is ordered script
execution, not transactional rollback of the in-memory session.

Native names reprate/disrate/ui/logwage avoid case-insensitive collisions with
Stata's uppercase aliases. `rr_ui` and `dr_ui` are explicit products. The script
changes UI/interactions together for UI0/UI1 point projections and restores every
source value before export. The retained estimator remains the native Poisson
primitive, not a registered survival fit. Its `predict n` is integrated hazard H,
while `xb` includes log exposure and is log H, not log instantaneous hazard.
Generic count PMF/CDF/margins and Pearson/deviance output must **not** be interpreted
as survival probabilities, total interaction effects or exponential adequacy tests.

## Interpretation and source quirks

- Hazard is per two-week interval; fixed horizon10 means20 calendar weeks. Recorded
  integer spell lengths are treated as exact continuous times, as in the author
  model—not an interval-censored/grouped-time likelihood.
- Exported net survival is exp(-H); mean_net_duration=1/hazard is a hypothetical
  net-cause functional, not observed mean joblessness/return time with competing
  exits. No full-time cumulative-incidence claim is made.
- UI hazard ratio at replacement r and disregard d is
  exp(beta_ui+r*beta_rr_ui+d*beta_dr_ui), not merely exp(beta_ui). All UI projections
  hold other predictors fixed and are observational/noncausal. They have no
  uncertainty intervals; the coefficient model has full robust inference.
- Final UI-group event-score balances follow included intercept/UI score equations;
  they do not test exponential adequacy or validate noninformative censoring.
- The42-field ASCII export omits stateur and rounds floats. The source audit
  verifies all140406 ASCII fields at their printed precision, but NEVER substitutes
  that reduced design for the43-field DTA model. All102 all-zero status rows stay
  in the sample as censor1=0, with no invented alternative status.
- On development R4.6.1, decimal CSV parsing can differ by1ULP from the DTA's exact
  promoted float even for a correct17-digit string (reprate row3 is
  `0x1.a1cacp-3`; its CSV string is `0.20399999618530273`). Consequently preparation
  checks completed CSV text, and the audit checks native/Python IEEE values against
  R/foreign **hexadecimal original cells**. The original reference command reads
  DTA directly. Do not weaken exact source checks or widen statistical tolerances
  to conceal a decimal-parser artifact.

The native normalized LL is−2700.6902704985096 (author−2700.6903), whereas R's
ordinary duration-density LL is−3946.5806305218698. They differ ONLY by the fixed
sum of event log times. Both full/null conventions, the LR numerical difference
and robust Wald are independently checked. Robust LR reference p-values remain
withheld. Robust covariance is the full observed-information score sandwich
multiplied by3343/3342, not residual-DF HC1 or uncorrected R survival covariance.
The robust slope Wald is565.2369454886683 on40 df (p≈6.0003899929e-94).

## Observed validation

Independent reference: R4.6.1/survival3.8.6 exponential `survreg` with robust
covariance, explicit tight convergence and sign/covariance mapping. Native output
passes1936 R gates including all1681 covariance entries, every207266 source/design/
point-projection cell, all147092 exact original source/ordinal cells,262 author
printed quantities including14 compact Table17.8 coefficient/z cells, and every
rounded ASCII field. This validates the exact author-program exponential column;
it is not separate book-page typography/transcription or other model columns.
Original JSON/text exports are byte-identical. Original raw SHA256 fingerprints
are enforced in `check_source.py`; original DTA MD5 is
`cc37709732a1d5b65f73536563ebbf3e`.

Ordinary numerical tolerance is1e-9 absolute+1e-8 relative; positive probability
and positive projection tails use1e-300+1e-8 relative (no zero-tail substitution).
Exact source hex comparisons have no tolerance. Printed numbers use only half
their last displayed unit. The standard-library audit independently reconstructs
full original-X score/information/inverse/sandwich, normal/CI inference, full-rank
Wald/its exact even-df gamma tail, full/null density/normalized likelihoods and
all coupled UI projections. Independent score certificate is bounded by1e-9 and
information-inverse identity by1e-8; no covariance/reference comparison-only pass.

Permanent `mma17_exponential_duration_workflow` tests run the actual script on a
separate400-row, full41-parameter R control (1936 gates/24800 cells), both CLI
modes, all-zero status retention, input/source restoration, duration-unit scaling,
invalid/missing/absent-stateur/rank-deficient/overlapping-status/zero-event inputs
and existing-file preservation. Corruption controls reject shared wrong native/R
covariance, normal/Wald zero tails and net-survival projections. Four original-DTA
preparation failure checks (existing output, wrong/missing source, missing output
parent) preserve input/output hashes and publish no partial file. Regenerate only
the synthetic fixtures with:

```sh
Rscript examples/mma17p4duration/control_reference.R /tmp/duration-control
```

Current root coverage reconciles376 passing tests across88 artifacts:86 unchanged
passing SHA256 identities plus the two current four-test targets rerun. Root CLI
main is unchanged. Root all-target Clippy `-D warnings`, changed Rust formatting
and diff checks pass. Unaffected non-root tests/doctests were not rerun; no hosted
Actions, other-platform or release certification is claimed. Remaining native
survival declarations, Weibull/Gompertz/Cox, survey/cluster inference, full Table17.9,
formal adequacy checks and remaining textbook illustrations stay required.

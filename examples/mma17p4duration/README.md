# Empirical exponential full-time re-employment hazard

## Contract (implementation in progress)

Cameron–Trivedi (2005) §17.11 pp.603–608, `mma17p4duration.do`,
Table17.8 exponential coefficient column. Original `ema1996.dta`:3343 one-record
spells,1073 full-time events,20887 observed two-week intervals, time1..28;
all40 author regressors plus intercept, complete finite sample. `censor1=1`
is full-time re-employment; part-time/unknown-status re-employment and continued
joblessness are non-events for this cause. Original source ordinals retained.

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

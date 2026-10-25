# Exponential full-time cause-specific hazard via exact event/exposure likelihood.
# Poisson labels/diagnostics are primitive output, not a Poisson survival DGP claim.
use duration.csv
isid source_row
sql """
SELECT CASE WHEN COUNT(*)<=41 OR SUM(censor1)<=0 OR COUNT(*) FILTER (WHERE NOT coalesce(
  isfinite(source_row) AND source_row>0 AND source_row=floor(source_row)
  AND isfinite(spell) AND spell>0
  AND isfinite(censor1) AND isfinite(censor2) AND isfinite(censor3) AND isfinite(censor4)
  AND censor1 IN (0,1) AND censor2 IN (0,1) AND censor3 IN (0,1) AND censor4 IN (0,1)
  AND censor1+censor2+censor3+censor4<=1 AND ui IN (0,1)
  AND isfinite(ui) AND isfinite(reprate) AND isfinite(disrate) AND isfinite(logwage)
  AND isfinite(tenure) AND isfinite(slack) AND isfinite(abolpos) AND isfinite(explose)
  AND isfinite(stateur) AND isfinite(houshead) AND isfinite(married) AND isfinite(female)
  AND isfinite(child) AND isfinite(ychild) AND isfinite(nonwhite) AND isfinite(age)
  AND isfinite(schlt12) AND isfinite(schgt12) AND isfinite(smsa) AND isfinite(bluecoll)
  AND isfinite(mining) AND isfinite(constr) AND isfinite(transp) AND isfinite(trade)
  AND isfinite(fire) AND isfinite(services) AND isfinite(pubadmin)
  AND isfinite(year85) AND isfinite(year87) AND isfinite(year89) AND isfinite(midatl)
  AND isfinite(encen) AND isfinite(wncen) AND isfinite(southatl) AND isfinite(escen)
  AND isfinite(wscen) AND isfinite(mountain) AND isfinite(pacific),FALSE))>0
THEN error('Exponential workflow requires N>41, positive complete finite times, finite full design including stateur, valid source_row/ui, nonoverlapping censor1..4 (all-zero allowed) and observed full-time events')
ELSE COUNT(*) END AS spells,SUM(censor1) AS full_time_events,SUM(spell) AS intervals_at_risk,
  'Exponential full-time cause-specific hazard, not all-cause survival or causal UI effect' AS analysis
FROM active
"""
sql """SELECT *, reprate*ui AS rr_ui,disrate*ui AS dr_ui,ln(spell) AS log_spell,ui AS original_ui FROM active ORDER BY source_row""" into exponential_design
poisson censor1 reprate disrate ui rr_ui dr_ui logwage tenure slack abolpos explose stateur houshead married female child ychild nonwhite age schlt12 schgt12 smsa bluecoll mining constr transp trade fire services pubadmin year85 year87 year89 midatl encen wncen southatl escen wscen mountain pacific, offset(log_spell) robust
predict cumulative_hazard, n
predict log_cumulative_hazard, xb
# Conditional UI projections must update both interactions, then restore source.
replace ui = 0
replace rr_ui = reprate*ui
replace dr_ui = disrate*ui
predict ui0_cumulative_hazard, n
replace ui = 1
replace rr_ui = reprate*ui
replace dr_ui = disrate*ui
predict ui1_cumulative_hazard, n
replace ui = original_ui
replace rr_ui = reprate*ui
replace dr_ui = disrate*ui
sql """
SELECT *,cumulative_hazard/spell AS hazard,exp(-cumulative_hazard) AS net_survival,
  spell/cumulative_hazard AS mean_net_duration,
  ui0_cumulative_hazard/spell AS ui0_hazard,ui1_cumulative_hazard/spell AS ui1_hazard,
  ui1_cumulative_hazard/ui0_cumulative_hazard AS ui_hazard_ratio,
  exp(-10*cumulative_hazard/spell) AS net_survival_10,
  exp(-10*ui0_cumulative_hazard/spell) AS ui0_net_survival_10,
  exp(-10*ui1_cumulative_hazard/spell) AS ui1_net_survival_10,
  censor1-cumulative_hazard AS event_score_residual
FROM active ORDER BY source_row
""" into exponential_predictions
export exponential_predictions.csv
# These score balances follow included intercept/UI, not an adequacy test.
sql """SELECT ui,COUNT(*) AS spells,SUM(censor1) AS full_time_events,SUM(spell) AS intervals_at_risk,SUM(cumulative_hazard) AS fitted_integrated_events,SUM(event_score_residual) AS score_balance,AVG(hazard) AS mean_hazard,AVG(net_survival_10) AS mean_net_survival_10 FROM active GROUP BY ui ORDER BY ui"""

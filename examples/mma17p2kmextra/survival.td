# Explicit failed=1 is failure; failed=0 is right-censoring, not an event.
use censored.csv
isid source_row
sql """SELECT CASE WHEN COUNT(*)=0 OR COUNT(*) FILTER (WHERE duration IS NULL OR NOT isfinite(duration) OR duration<=0 OR failed IS NULL OR NOT isfinite(failed) OR failed NOT IN (0,1) OR source_row IS NULL OR NOT isfinite(source_row) OR source_row!=floor(source_row) OR source_row<1)>0 OR SUM(failed)=0 OR COUNT(DISTINCT duration)>4999 THEN error('Censored workflow requires positive finite complete times, exact 0/1 failed with at least one failure, positive integer source_row and at most4999 distinct times') ELSE COUNT(*) END AS spells, SUM(failed) AS failures, COUNT(*)-SUM(failed) AS censored, SUM(duration) AS days_at_risk FROM active"""
export censored_source.csv
sql """
WITH exits AS (
  SELECT duration AS time, SUM(failed)::BIGINT AS events,
    COUNT(*)-SUM(failed)::BIGINT AS censored FROM active GROUP BY duration
), risks AS (
  SELECT *, SUM(events+censored) OVER (ORDER BY time ROWS BETWEEN CURRENT ROW AND UNBOUNDED FOLLOWING) AS at_risk
  FROM exits
), estimates AS (
  SELECT *, PRODUCT((at_risk-events)::DOUBLE/at_risk) OVER (ORDER BY time ROWS UNBOUNDED PRECEDING) AS survival,
    SUM(CASE WHEN at_risk>events THEN events::DOUBLE/(at_risk*(at_risk-events)) END)
      OVER (ORDER BY time ROWS UNBOUNDED PRECEDING) AS greenwood,
    SUM(events::DOUBLE/at_risk) OVER (ORDER BY time ROWS UNBOUNDED PRECEDING) AS cumhaz,
    SUM(events::DOUBLE/(at_risk*at_risk)) OVER (ORDER BY time ROWS UNBOUNDED PRECEDING) AS cumhaz_variance
  FROM risks
), uncertainty AS (
  SELECT time, at_risk, events, censored, survival,
    CASE WHEN survival>0 THEN survival*sqrt(greenwood) END AS survival_se,
    CASE WHEN survival=1 THEN 1.0 WHEN survival>0 THEN exp(-exp(ln(-ln(survival))+1.959963984540054*sqrt(greenwood)/abs(ln(survival)))) END AS survival_lower,
    CASE WHEN survival=1 THEN 1.0 WHEN survival>0 THEN exp(-exp(ln(-ln(survival))-1.959963984540054*sqrt(greenwood)/abs(ln(survival)))) END AS survival_upper,
    cumhaz, sqrt(cumhaz_variance) AS cumhaz_se
  FROM estimates
)
SELECT *, CASE WHEN cumhaz=0 THEN 0.0 ELSE cumhaz*exp(-1.959963984540054*cumhaz_se/cumhaz) END AS cumhaz_lower,
  CASE WHEN cumhaz=0 THEN 0.0 ELSE cumhaz*exp(1.959963984540054*cumhaz_se/cumhaz) END AS cumhaz_upper
FROM uncertainty ORDER BY time
""" into censored_survival
head 200
export censored_survival.csv
# Pre/post vertices preserve simultaneous jumps and horizontal censor-only holds.
sql """
WITH steps AS (
  SELECT time, 0 AS phase,
    lag(survival,1,1.0) OVER (ORDER BY time) AS survival,
    lag(survival_lower,1,1.0) OVER (ORDER BY time) AS survival_lower,
    lag(survival_upper,1,1.0) OVER (ORDER BY time) AS survival_upper,
    lag(cumhaz,1,0.0) OVER (ORDER BY time) AS cumhaz,
    lag(cumhaz_lower,1,0.0) OVER (ORDER BY time) AS cumhaz_lower,
    lag(cumhaz_upper,1,0.0) OVER (ORDER BY time) AS cumhaz_upper
  FROM active
  UNION ALL
  SELECT time,1,survival,survival_lower,survival_upper,cumhaz,cumhaz_lower,cumhaz_upper FROM active
  UNION ALL
  SELECT 0.0,0,1.0,1.0,1.0,0.0,0.0,0.0
)
SELECT * FROM steps ORDER BY time,phase
""" into censored_steps
export censored_steps.csv
label variable time "Time since entry at zero"
label variable survival "Kaplan-Meier survival"
label variable survival_lower "Lower pointwise 95% band"
label variable survival_upper "Upper pointwise 95% band"
label variable cumhaz "Nelson-Aalen cumulative hazard"
label variable cumhaz_lower "Lower pointwise 95% band"
label variable cumhaz_upper "Upper pointwise 95% band"
scatter survival time, lines(survival survival_lower survival_upper) title("Right-censored KM: pointwise 95% log-log intervals") saving("censored_survival.svg") noopen
scatter cumhaz time, lines(cumhaz cumhaz_lower cumhaz_upper) title("Right-censored NA: pointwise 95% log-Wald intervals") saving("censored_cumhaz.svg") noopen

# Original source has no censoring: every duration is an observed failure.
use strikes.csv
isid source_row
sql """SELECT CASE WHEN COUNT(*)=0 OR COUNT(*) FILTER (WHERE dur IS NULL OR NOT isfinite(dur) OR dur<=0 OR source_row IS NULL OR NOT isfinite(source_row) OR source_row!=floor(source_row) OR source_row<1)>0 THEN error('Strike workflow requires nonempty positive finite complete durations and positive integer source_row') ELSE COUNT(*) END AS spells, SUM(dur) AS days_at_risk, MIN(dur) AS first_failure, MAX(dur) AS last_failure FROM active"""
export strikes_source.csv
sql """
WITH events AS (
  SELECT dur AS time, COUNT(*) AS events FROM active GROUP BY dur
), risks AS (
  SELECT *, SUM(events) OVER () AS spells,
    SUM(events) OVER (ORDER BY time ROWS BETWEEN CURRENT ROW AND UNBOUNDED FOLLOWING) AS at_risk
  FROM events
), estimates AS (
  SELECT *, (at_risk-events)::DOUBLE/spells AS survival,
    SUM(CASE WHEN at_risk>events THEN events::DOUBLE/(at_risk*(at_risk-events)) END)
      OVER (ORDER BY time ROWS UNBOUNDED PRECEDING) AS greenwood,
    SUM(events::DOUBLE/at_risk) OVER (ORDER BY time ROWS UNBOUNDED PRECEDING) AS cumhaz,
    SUM(events::DOUBLE/(at_risk*at_risk)) OVER (ORDER BY time ROWS UNBOUNDED PRECEDING) AS cumhaz_variance
  FROM risks
), uncertainty AS (
  SELECT time, at_risk, events, survival,
    CASE WHEN survival>0 THEN survival*sqrt(greenwood) END AS survival_se,
    CASE WHEN survival>0 THEN exp(-exp(ln(-ln(survival))+1.959963984540054*sqrt(greenwood)/abs(ln(survival)))) END AS survival_lower,
    CASE WHEN survival>0 THEN exp(-exp(ln(-ln(survival))-1.959963984540054*sqrt(greenwood)/abs(ln(survival)))) END AS survival_upper,
    cumhaz, sqrt(cumhaz_variance) AS cumhaz_se
  FROM estimates
)
SELECT * FROM uncertainty ORDER BY time
""" into strike_survival
head 200
export strikes_survival.csv
# Two vertices at each event time encode vertical jumps, with horizontal holds.
# Upper/lower NULL at the terminal failure break those paths, not zero bands.
sql """
WITH steps AS (
  SELECT time, 0 AS phase,
    lag(survival,1,1.0) OVER (ORDER BY time) AS survival,
    lag(survival_lower,1,1.0) OVER (ORDER BY time) AS survival_lower,
    lag(survival_upper,1,1.0) OVER (ORDER BY time) AS survival_upper,
    lag(cumhaz,1,0.0) OVER (ORDER BY time) AS cumhaz
  FROM active
  UNION ALL
  SELECT time, 1, survival, survival_lower, survival_upper, cumhaz FROM active
  UNION ALL
  SELECT 0.0, 0, 1.0, 1.0, 1.0, 0.0
)
SELECT * FROM steps ORDER BY time, phase
""" into strike_steps
export strikes_steps.csv
label variable time "Strike duration in days"
label variable survival "Kaplan-Meier survival"
label variable survival_lower "Lower pointwise 95% band"
label variable survival_upper "Upper pointwise 95% band"
label variable cumhaz "Nelson-Aalen cumulative hazard"
scatter survival time, lines(survival survival_lower survival_upper) title("Strike survival: pointwise 95% log-log intervals") saving("strikes_survival.svg") noopen
scatter cumhaz time, lines(cumhaz) title("Strike Nelson-Aalen cumulative hazard") saving("strikes_cumhaz.svg") noopen

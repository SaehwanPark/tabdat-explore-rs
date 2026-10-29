# Table 23.1 pooled Poisson only; cluster inference assumes independent firms.
use patents.csv
isid id year
sql """SELECT CASE WHEN COUNT(*) = 0 OR COUNT(*) FILTER (WHERE id IS NULL OR year IS NULL OR pat IS NULL OR logr IS NULL OR NOT isfinite(id) OR NOT isfinite(year) OR NOT isfinite(pat) OR NOT isfinite(logr) OR id != floor(id) OR id < 1 OR year != floor(year) OR year < 1975 OR year > 1979 OR pat < 0 OR pat != floor(pat)) > 0 THEN error('invalid patents firm/year/count/model data') ELSE COUNT(*) END AS observations, COUNT(DISTINCT id) AS firms FROM active"""
sql """SELECT CASE WHEN COUNT(*) = 0 OR MIN(n) != 5 OR MAX(n) != 5 THEN error('each firm requires all five years 1975–79') ELSE COUNT(*) END AS firms FROM (SELECT id, COUNT(*) AS n FROM active GROUP BY id)"""
summarize pat logr
poisson pat logr
estat margins
poisson pat logr, robust
estat margins
poisson pat logr, cluster(id)
estat margins
predict poisson_mu, n
predict poisson_xb, xb
summarize poisson_mu poisson_xb
export pooled_poisson_predictions.csv

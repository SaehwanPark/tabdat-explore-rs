# Table 5.7 auxiliary NLS covariance under Var(y|x)=fitted mean squared.
# This is a plug-in variance model, not an assumption-free robust sandwich.
use nls.csv
isid source_row
sql """
SELECT CASE WHEN COUNT(*)<=2 OR COUNT(*)>10000
  OR MIN(source_row)<>1 OR MAX(source_row)<>COUNT(*)
  OR COUNT(*) FILTER (WHERE NOT coalesce(
    isfinite(source_row) AND source_row=floor(source_row) AND source_row>0
    AND isfinite(x) AND isfinite(y) AND y>0,FALSE))>0
  OR NOT coalesce(isfinite(stddev_samp(x)) AND stddev_samp(x)>0,FALSE)
THEN error('NLS model-variance workflow requires 3..10000 unique dense rows, positive finite y and finite nonconstant x')
ELSE COUNT(*) END AS observations FROM active
"""
sql """SELECT 'Plug-in covariance assumes the fitted conditional mean and Var(y|x)=mean squared; no finite-sample adjustment' AS interpretation FROM active LIMIT 1"""
nl y = exp(-a-b*x), params(a b) start(1 0)
predict fitted_mean, mean
sql """
SELECT CASE WHEN COUNT(*) FILTER (WHERE NOT coalesce(
  isfinite(fitted_mean) AND fitted_mean>0
  AND isfinite(fitted_mean*fitted_mean) AND fitted_mean*fitted_mean>0
  AND isfinite(fitted_mean*fitted_mean*fitted_mean*fitted_mean)
  AND fitted_mean*fitted_mean*fitted_mean*fitted_mean>0,FALSE))>0
THEN error('NLS model-variance workflow requires representable positive mean powers')
ELSE COUNT(*) END AS valid_means FROM active
"""
# J=(-mu,-x*mu); B=J'J and M=J'diag(mu^2)J. Signs cancel in both matrices.
sql """
SELECT count(*) AS n,
  sum(fitted_mean*fitted_mean) AS b00,
  sum(x*fitted_mean*fitted_mean) AS b01,
  sum(x*x*fitted_mean*fitted_mean) AS b11,
  sum(fitted_mean*fitted_mean*fitted_mean*fitted_mean) AS m00,
  sum(x*fitted_mean*fitted_mean*fitted_mean*fitted_mean) AS m01,
  sum(x*x*fitted_mean*fitted_mean*fitted_mean*fitted_mean) AS m11
FROM active
""" into nls_modelvar_moments
sql """
SELECT CASE WHEN COUNT(*)<>1 OR COUNT(*) FILTER (WHERE NOT coalesce(
  isfinite(b00) AND b00>0 AND isfinite(b01) AND isfinite(b11) AND b11>0
  AND isfinite(m00) AND m00>0 AND isfinite(m01) AND isfinite(m11) AND m11>0
  AND isfinite(b00*b11-b01*b01) AND b00*b11-b01*b01>1e-12*b00*b11,FALSE))>0
THEN error('NLS model-variance information is ill-conditioned or nonrepresentable')
ELSE COUNT(*) END AS valid_information FROM active
"""
sql """
WITH base AS (SELECT *, b00*b11-b01*b01 AS det FROM active)
SELECT n, b00, b01, b11, m00, m01, m11,
  (b11*b11*m00-2*b11*b01*m01+b01*b01*m11)/(det*det) AS cov_aa,
  (-b11*b01*m00+(b11*b00+b01*b01)*m01-b01*b00*m11)/(det*det) AS cov_ab,
  (b01*b01*m00-2*b01*b00*m01+b00*b00*m11)/(det*det) AS cov_bb
FROM base
""" into nls_modelvar_covariance
sql """
SELECT CASE WHEN COUNT(*)<>1 OR COUNT(*) FILTER (WHERE NOT coalesce(
  isfinite(cov_aa) AND cov_aa>0 AND isfinite(cov_ab)
  AND isfinite(cov_bb) AND cov_bb>0
  AND isfinite(cov_aa*cov_bb-cov_ab*cov_ab)
  AND cov_aa*cov_bb-cov_ab*cov_ab>0,FALSE))>0
THEN error('NLS model-variance covariance is nonpositive or nonrepresentable')
ELSE COUNT(*) END AS valid_covariance FROM active
"""
sql """
SELECT n, b00, b01, b11, m00, m01, m11, cov_aa, cov_ab, cov_bb,
  sqrt(cov_aa) AS se_a, sqrt(cov_bb) AS se_b,
  'variance-model plug-in, no HC correction; parameter order a,b' AS inference
FROM active
""" into nls_modelvar_result
sql """SELECT * FROM active"""
export nls_modelvar.csv

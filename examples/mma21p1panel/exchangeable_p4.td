# Same-session continuation: first run gaussian_pa.td; its retained model remains pooled OLS.
use gaussian_pa_fit
sql """SELECT 'P4 exchangeable Gaussian PA: the working covariance assumes the specified exchangeable second moments. The G/(G-1) person sandwich assumes independent people and a correct marginal mean, not independent person-years. The preceding pooled OLS t/F and uncorrected GEE sandwich are distinct. This is not a causal or formal correlation test.' AS inference_scope"""
sql """
WITH checked AS MATERIALIZED (
 SELECT *,CASE WHEN people<3 OR observations!=10*people OR years_per_person!=10
 OR convention!='legacy_pearson_exchangeable_rss_over_n' OR unit_covariance!='uncorrected_unit_score_sandwich'
 OR NOT isfinite(working_covariance_aa) OR working_covariance_aa<=0 OR NOT isfinite(working_covariance_ab)
 OR NOT isfinite(working_covariance_bb) OR working_covariance_bb<=0
 OR NOT isfinite(unit_covariance_aa) OR unit_covariance_aa<=2.842170943040401e-14*working_covariance_aa
 OR NOT isfinite(unit_covariance_ab) OR NOT isfinite(unit_covariance_bb) OR unit_covariance_bb<=2.842170943040401e-14*working_covariance_bb
 THEN error('P4 exchangeable GEE requires a valid fit and resolved unit-score covariance') ELSE unit_covariance_aa END AS checked_unit_aa
 FROM active
)
SELECT observations,people,years_per_person,iterations,intercept,slope,scale,working_correlation,
 working_covariance_aa,working_covariance_ab,working_covariance_bb,
 checked_unit_aa AS raw_unit_covariance_aa,unit_covariance_ab AS raw_unit_covariance_ab,unit_covariance_bb AS raw_unit_covariance_bb,
 people/(people-1.0) AS unit_multiplier,checked_unit_aa*people/(people-1.0) AS adjusted_unit_covariance_aa,
 unit_covariance_ab*people/(people-1.0) AS adjusted_unit_covariance_ab,unit_covariance_bb*people/(people-1.0) AS adjusted_unit_covariance_bb,
 intercept_step,slope_step,'legacy_pearson_exchangeable_rss_over_n' AS convention,
 'g_over_g_minus_1_unit_sandwich' AS unit_covariance,'normal_z_chi2_1' AS inference
FROM checked
""" into exchangeable_p4_fit
export panel_exchangeable_p4_fit.csv
sql """
WITH methods AS (
 SELECT 'working_model' AS covariance,intercept,slope,working_covariance_aa AS va,working_covariance_bb AS vb FROM active UNION ALL
 SELECT 'unit_sandwich_g_adjusted',intercept,slope,adjusted_unit_covariance_aa,adjusted_unit_covariance_bb FROM active
), parameters AS (
 SELECT covariance,'_cons' AS parameter,intercept AS estimate,SQRT(va) AS standard_error FROM methods UNION ALL
 SELECT covariance,'lnwg',slope,SQRT(vb) FROM methods
), statistics AS (SELECT *,estimate/standard_error AS z FROM parameters)
SELECT covariance,parameter,estimate,standard_error,z,2*tabdat_gauss_phi(-ABS(z)) AS p_value,
 estimate-1.959963984540054*standard_error AS normal95_lower,estimate+1.959963984540054*standard_error AS normal95_upper,
 CASE WHEN parameter='lnwg' THEN z*z ELSE NULL END AS slope_wald_chi2_1 FROM statistics ORDER BY covariance,parameter
""" into exchangeable_p4_coefficients
sql """SELECT * FROM active"""
export panel_exchangeable_p4_coefficients.csv
use gaussian_pa_predictions
status

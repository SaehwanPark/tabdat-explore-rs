"""Development-only original-person Poisson cluster reference.

python3 check_poisson_cluster_reference.py R_reference.csv R_predictions.csv native.jsonl poisson_person_predictions.csv
"""
import csv
from decimal import Decimal
import json
import math
import sys

PREDICTORS = 'LC IDP LPI FMDE PHYSLIM NDISEASE HLTHG HLTHF HLTHP LINC LFAM EDUCDEC AGE FEMALE CHILD FEMCHILD BLACK'.split()
TOTALS = {'log_likelihood', 'null_log_likelihood', 'likelihood_ratio', 'residual_deviance', 'pearson_chi_square'}


def read_csv(path):
  with open(path, encoding='utf-8', newline='') as source:
    return list(csv.DictReader(source))


def close(actual, expected, quantity=''):
  if expected == 'NA':
    assert actual is None, (quantity, actual)
    return
  actual, expected = float(actual), float(expected)
  tolerance = (abs(expected)*1e-10+sys.float_info.min if quantity.endswith('p_value') else
    1e-7 if quantity in TOTALS else 1e-12 if quantity == 'design' else 1e-10)
  assert math.isfinite(actual) and abs(actual-expected) <= tolerance, (quantity, actual, expected, tolerance)


def check(reference, predictions, native, native_predictions):
  with open(native, encoding='utf-8') as source:
    records = [json.loads(line) for line in source]
  assert all(e['schema_version'] == 1 and e.get('error') is None for e in records)
  assert [e['result_type'] for e in records] == [
    'LoadResult', 'IsidResult', 'IsidResult', 'IsidResult', 'SummarizeResult',
    'PoissonResult', 'PoissonMarginalEffectsResult', 'PredictionResult', 'PredictionResult', 'ExportResult']
  fit, effects = records[5]['data'], records[6]['data']
  assert fit['estimator'] == effects['estimator'] == 'poisson'
  assert fit['outcome'] == 'MDU' and fit['predictors'] == PREDICTORS and fit['include_intercept'] is True
  assert fit['observation_count'] == effects['observation_count'] == 20186
  assert fit['count_sum'] == 57746 and fit['zero_count'] == 6308 and fit['df_residual'] == 20168
  assert fit['covariance'] == effects['covariance'] == 'cluster_sandwich_g_over_g_minus_1'
  assert fit['reference_distribution'] == effects['reference_distribution'] == 'asymptotic_standard_normal'
  assert fit['confidence_level'] == effects['confidence_level'] == 95
  assert [c['name'] for c in fit['coefficients']] == fit['covariance_parameter_names'] == ['intercept', *PREDICTORS]
  assert [e['name'] for e in effects['effects']] == PREDICTORS
  assert effects['effect'] == 'average_continuous_predictor_column_derivative'
  assert effects['sample'] == 'retained_estimation_design'
  assert effects['uncertainty'] == 'conditional_on_observed_design_delta_method'
  assert fit['cluster'] == effects['cluster'] == {'variable': 'person_id', 'group_count': 5908,
    'finite_sample_multiplier': 5908/5907, 'independence_assumption': 'across_clusters'}
  assert fit['likelihood_ratio_p_value'] is None and fit['likelihood_ratio_reference_distribution'] is None
  assert fit['likelihood_ratio_df'] == fit['omnibus_wald']['degrees_of_freedom'] == 17
  assert fit['omnibus_wald']['reference_distribution'] == 'asymptotic_chi_square'
  assert fit['omnibus_wald']['unavailable_reason'] is None
  assert fit['log_likelihood_convention'] == 'poisson_count_pmf'
  assert fit['null_model'] == 'intercept_only_estimated_mean'
  assert fit['dispersion_inference'] == 'descriptive_not_a_panel_or_specification_test'
  assert fit['converged'] is True and 1 <= fit['iterations'] <= 100
  assert fit['convergence_coordinate_system'] == 'unit_column_norm' and fit['convergence_absolute_tolerance'] == 1e-10
  assert 0 <= fit['max_abs_scaled_score'] <= 1e-10 and 0 <= fit['max_abs_scaled_newton_step'] <= 1e-10
  refs = read_csv(reference)
  assert len(refs) == 551
  for r in refs:
    assert r['case'] == 'mma20p1count_person'
    q, i, j = r['quantity'], int(r['i']), int(r['j'])
    if q.startswith('coefficient_'):
      actual = fit['coefficients'][i][q.removeprefix('coefficient_')]
    elif q.startswith('effect_'):
      actual = effects['effects'][i][q.removeprefix('effect_')]
    elif q == 'covariance':
      actual = fit['covariance_matrix'][i][j]
    elif q == 'cluster_group_count':
      actual = fit['cluster']['group_count']
    elif q == 'cluster_correction':
      actual = fit['cluster']['finite_sample_multiplier']
    elif q.startswith('wald_'):
      actual = fit['omnibus_wald'][q.removeprefix('wald_')]
    else:
      actual = fit[q]
    close(actual, r['value'], q)
  expected, actual = read_csv(predictions), read_csv(native_predictions)
  assert len(expected) == len(actual) == 20186
  assert list(actual[0]) == ['prepared_row', 'original_row', 'person_id', 'study_year', 'MDU', *PREDICTORS,
    'poisson_person_mu', 'poisson_person_xb']
  groups = [str(int(Decimal(r['person_id']))) for r in expected]
  assert len(set(groups)) == 5908
  assert fit['sample'] == {'retained_indices': list(range(20186)), 'total_observations': 20186,
    'dropped_observations': 0, 'weights': None, 'cluster_groups': groups}
  for index, (r, n) in enumerate(zip(expected, actual), 1):
    assert Decimal(r['prepared_row']) == Decimal(n['prepared_row']) == index
    for field in ['original_row', 'person_id', 'study_year', 'MDU']:
      assert Decimal(r[field]) == Decimal(n[field]), (index, field)
    for field in PREDICTORS:
      close(n[field], r[field], 'design')
    close(n['poisson_person_mu'], r['mu'])
    close(n['poisson_person_xb'], r['xb'])
    close(n['poisson_person_mu'], math.exp(float(n['poisson_person_xb'])))
  assert max(float(r['poisson_person_mu']) for r in actual) > 1
  print('Matched all551 independently clustered Poisson quantities and every source/original-row/person/year/design/mean/xb field of all20,186 records;5,908 original people.')
  print('G/(G-1), normal95 not G-1 t; no HC1, row-robust substitution, LR reference, fabricated prepared-position ID, household/site/multiway or few-cluster guarantee.')


if __name__ == '__main__':
  assert len(sys.argv) == 5, __doc__
  check(*sys.argv[1:])

"""Development-only retained count-frequency comparison against independent R.

python3 check_count_probability_reference.py R_reference.csv R_predictions.csv native.jsonl count_probability_predictions.csv [native_probability_rows.csv]

The optional final CSV is a development probe of count_bin_probabilities, not
an installed rowwise probability-export command. Without it only aggregate
probabilities and every source/mean/index record are compared.
"""
import csv
from decimal import Decimal
import json
import math
import sys

PREDICTORS = 'LC IDP LPI FMDE PHYSLIM NDISEASE HLTHG HLTHF HLTHP LINC LFAM EDUCDEC AGE FEMALE CHILD FEMCHILD BLACK'.split()


def read_csv(path):
  with open(path, encoding='utf-8', newline='') as source:
    return list(csv.DictReader(source))


def close(actual, expected, quantity=''):
  actual, expected = float(actual), float(expected)
  tolerance = (abs(expected)*1e-10+sys.float_info.min if quantity == 'probability' else
    1e-12 if quantity == 'design' else 1e-7 if quantity == 'log_likelihood' else 1e-10)
  assert math.isfinite(actual) and abs(actual-expected) <= tolerance, (quantity, actual, expected, tolerance)


def check(reference, predictions, native, native_predictions, probability_rows=None):
  with open(native, encoding='utf-8') as source:
    records = [json.loads(line) for line in source]
  assert all(r['schema_version'] == 1 and r.get('error') is None for r in records)
  assert [r['result_type'] for r in records] == [
    'LoadResult', 'IsidResult', 'SummarizeResult', 'PoissonResult', 'CountProbabilityResult',
    'PredictionResult', 'PredictionResult', 'NbregResult', 'CountProbabilityResult',
    'PredictionResult', 'PredictionResult', 'ExportResult']
  fits = {'poisson': records[3]['data'], 'nb2': records[7]['data']}
  tables = {'poisson': records[4]['data'], 'nb2': records[8]['data']}
  expected_counts = [6308, 3815, 2795, 1884, 1345, 968, 689, 531, 408, 287, 1156]
  for family, t in tables.items():
    fit = fits[family]
    assert fit['outcome'] == 'MDU' and fit['predictors'] == PREDICTORS and fit['include_intercept'] is True
    assert fit['observation_count'] == 20186 and fit['converged'] is True
    assert fit['count_sum'] == 57746 and fit['zero_count'] == 6308
    assert t['estimator'] == family and t['outcome'] == 'MDU'
    assert t['distribution'] == ({'family': 'poisson'} if family == 'poisson' else
      {'family': 'nb2', 'alpha': fit['dispersion']['alpha']})
    assert t['observation_count'] == 20186 and t['tail_from'] == 10
    assert t['sample_role'] == 'retained_estimation_sample'
    assert t['probability_role'] == 'mean_of_individual_plugin_conditional_probabilities'
    assert t['covariance_role'] == 'does_not_establish_distribution'
    assert t['uncertainty'] == 'not_computed' and t['goodness_of_fit_test'] == 'not_performed'
    assert 'p_value' not in t and 'standard_error' not in t
    assert t['sample'] == fit['sample'] == {'retained_indices': list(range(20186)),
      'total_observations': 20186, 'dropped_observations': 0, 'weights': None, 'cluster_groups': None}
    assert len(t['bins']) == 11
    for i, b in enumerate(t['bins']):
      assert b['lower_bound'] == i and b['upper_bound'] == (i if i < 10 else None)
      assert b['observed_count'] == expected_counts[i]
      close(b['observed_fraction'], expected_counts[i]/20186)
      p = b['fitted_fraction']
      assert math.isfinite(p) and 0 <= p <= 1
      close(b['expected_count'], p*20186)
      assert 'p_value' not in b and 'standard_error' not in b
    close(math.fsum(b['fitted_fraction'] for b in t['bins']), 1)
    close(math.fsum(b['observed_fraction'] for b in t['bins']), 1)
  refs = read_csv(reference)
  assert len(refs) == 95
  for r in refs:
    family, q, i, j = r['case'], r['quantity'], int(r['i']), int(r['j'])
    assert family in fits and j == 0
    t, f = tables[family], fits[family]
    if q in {'observed_count', 'observed_fraction', 'fitted_fraction', 'expected_count'}:
      actual = t['bins'][i][q]
    elif q == 'N':
      actual = t['observation_count']
    elif q == 'tail_from':
      actual = t['tail_from']
    elif q == 'alpha':
      actual = t['distribution']['alpha']
    elif q == 'log_likelihood':
      actual = f[q]
    else:
      raise AssertionError(q)
    close(actual, r['value'], q)
  expected, actual = read_csv(predictions), read_csv(native_predictions)
  assert len(expected) == len(actual) == 20186
  mean_fields = ['poisson_mu', 'poisson_xb', 'nb2_mu', 'nb2_xb']
  source_fields = ['prepared_row', 'MDU', *PREDICTORS]
  assert list(actual[0]) == [*source_fields, *mean_fields]
  probability_fields = [f'{family}_{bin}' for family in ['poisson', 'nb2'] for bin in
    [*[f'p{i}' for i in range(10)], 'ge10']]
  assert set(expected[0]) == set([*source_fields, *mean_fields, *probability_fields])
  for index, (r, n) in enumerate(zip(expected, actual), 1):
    assert Decimal(r['prepared_row']) == Decimal(n['prepared_row']) == index
    assert Decimal(r['MDU']) == Decimal(n['MDU'])
    for field in PREDICTORS:
      close(n[field], r[field], 'design')
    for field in mean_fields:
      close(n[field], r[field])
    for family in ['poisson', 'nb2']:
      close(n[family+'_mu'], math.exp(float(n[family+'_xb'])))
  if probability_rows:
    probabilities = read_csv(probability_rows)
    assert len(probabilities) == 20186 and list(probabilities[0]) == ['prepared_row', *probability_fields]
    for index, (r, n) in enumerate(zip(expected, probabilities), 1):
      assert Decimal(r['prepared_row']) == Decimal(n['prepared_row']) == index
      for field in probability_fields:
        close(n[field], r[field], 'probability')
      for family in ['poisson', 'nb2']:
        values = [float(n[f'{family}_{field}']) for field in [*[f'p{i}' for i in range(10)], 'ge10']]
        assert all(math.isfinite(p) and 0 <= p <= 1 for p in values)
        assert abs(math.fsum(values)-1) <= 1e-12
    for family in ['poisson', 'nb2']:
      for i, field in enumerate([*[f'p{i}' for i in range(10)], 'ge10']):
        close(math.fsum(float(n[f'{family}_{field}']) for n in probabilities)/20186,
          tables[family]['bins'][i]['fitted_fraction'])
    print('Additional native scalar-helper probe: all444,092 individual PMF/survival probabilities match with relative-tail tolerance; no installed rowwise probability export is implied.')
  print('Matched all95 independent R aggregate/model quantities and every source/design/mean/xb field of all20,186 records; exact0..9 plus >=10.')
  print('Mean of individual conditional probabilities, descriptive full-law assumptions, no uncertainty/formal test. Printed Table20.6 NB2 row remains unresolved; not literal whole-table replication.')


if __name__ == '__main__':
  assert len(sys.argv) in (5, 6), __doc__
  check(*sys.argv[1:])

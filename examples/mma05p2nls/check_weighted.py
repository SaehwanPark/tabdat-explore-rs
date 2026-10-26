#!/usr/bin/env python3
"""Audit the original-source weighted CLI results against independent R output."""
import argparse
import json
import math
import os
from pathlib import Path
import tempfile

from check_reference import HASHES, rows, sha


def audit(native_json, native_csv, reference, source, receipt):
  reference, source = Path(reference), Path(source)
  for name, expected in HASHES.items():
    assert sha(source / name) == expected, f'original source hash: {name}'
  leaves = [json.loads(line) for line in Path(native_json).read_text().splitlines()]
  assert leaves and all(v['schema_version'] == 1 for v in leaves)
  fits = [v['data'] for v in leaves if v.get('result_type') == 'NlsResult']
  assert len(fits) == 5
  initial = fits[0]
  selected = {}
  for method, offset in [('wnls', 1), ('fgnls', 3)]:
    for mode, i in [('classical', offset), ('hc1', offset + 1)]:
      fit = fits[i]
      selected[method, mode] = fit
      assert fit['model']['outcome'] == method + '_y'
      assert fit['model']['predictor_names'] == ['x', method + '_scale']
      expr = fit['model']['expression']
      assert expr == {'kind': 'divide', 'arguments': [
        {'kind': 'exp', 'arguments': {'kind': 'subtract', 'arguments': [
          {'kind': 'negate', 'arguments': {'kind': 'parameter', 'arguments': 0}},
          {'kind': 'multiply', 'arguments': [{'kind': 'parameter', 'arguments': 1},
                                             {'kind': 'predictor', 'arguments': 0}]}]}},
        {'kind': 'predictor', 'arguments': 1}]}
      assert fit['robust'] is (mode == 'hc1')
      assert fit['covariance'] == ('hc1_gauss_newton' if mode == 'hc1' else 'working_homoskedastic_gauss_newton')
  assert initial['model']['outcome'] == 'y'
  assert initial['model']['predictor_names'] == ['x']
  for fit in fits:
    assert fit['model']['parameter_names'] == ['a', 'b']
    assert fit['start_values'] == [1, 0]
    assert fit['observation_count'] == 10000 and fit['residual_degrees_of_freedom'] == 9998
    sample = fit['sample']
    assert sample['total_observations'] == 10000 and sample['dropped_observations'] == 0
    assert sample['retained_indices'] == list(range(10000))
    assert all(type(i) is int for i in sample['retained_indices'])
    assert sample['weights'] is None and sample['cluster_groups'] is None
    assert fit['inference']['reference_distribution'] == 'approximate_student_t'
    assert fit['inference']['degrees_of_freedom'] == 9998 and fit['inference']['confidence_level'] == .95
    assert fit['covariance_bread'] == 'inverse_jacobian_cross_product'
    assert all(fit[k] is None for k in ['log_likelihood', 'likelihood_ratio', 'omnibus_f'])
    assert 0 <= fit['normalized_score'] <= 1e-12 and 0 <= fit['relative_full_gauss_newton_step'] <= 1e-12
    assert 0 < fit['iterations'] <= 200 and 0 < fit['evaluation_work'] <= 500000000
    assert [c['name'] for c in fit['coefficients']] == ['a', 'b']
    assert len(fit['covariance_matrix']) == 2 and all(len(row) == 2 for row in fit['covariance_matrix'])
    assert len(fit['fitted_values']) == len(fit['residuals']) == 10000
  exported = rows(native_csv)
  columns = ['source_row', 'y', 'x', 'preliminary_mean', 'wnls_scale', 'fgnls_scale',
    'wnls_weight', 'fgnls_weight', 'wnls_y', 'fgnls_y',
    'wnls_mean_classical', 'wnls_residual_classical', 'wnls_mean_hc1', 'wnls_residual_hc1',
    'fgnls_mean_classical', 'fgnls_residual_classical', 'fgnls_mean_hc1', 'fgnls_residual_hc1',
    'wnls_level_mean', 'wnls_level_residual', 'wnls_rate',
    'fgnls_level_mean', 'fgnls_level_residual', 'fgnls_rate']
  assert len(exported) == 10000 and list(exported[0]) == columns
  original = [list(map(float, line.split())) for line in (source/'mma05data.asc').read_text().splitlines()]
  projections = rows(reference/'projections.csv')
  gates = 0
  maximum = {}
  def check(name, actual, expected):
    nonlocal gates
    actual, expected = float(actual), float(expected)
    assert math.isfinite(actual) and math.isfinite(expected), (name, actual, expected)
    error = abs(actual - expected)
    assert error <= 1e-11 + 1e-10 * abs(expected), (name, actual, expected)
    maximum[name] = max(maximum.get(name, 0), error)
    gates += 1
  index = {'a': 0, 'b': 1}
  for ref in rows(reference/'parameters.csv'):
    if ref['case'] != 'original': continue
    coefficient = selected[ref['method'], ref['mode']]['coefficients'][index[ref['parameter']]]
    for native, rfield in [('value', 'estimate'), ('standard_error', 'standard_error'),
        ('statistic', 't'), ('p_value', 'p_value'), ('confidence_lower', 'ci_lower'), ('confidence_upper', 'ci_upper')]:
      check(rfield, coefficient[native], ref[rfield])
      if rfield == 'p_value' and float(ref[rfield]) == 0:
        assert coefficient[native] == 0
  for ref in rows(reference/'covariance.csv'):
    if ref['case'] == 'original':
      fit = selected[ref['method'], ref['mode']]
      check('covariance', fit['covariance_matrix'][index[ref['row_parameter']]][index[ref['column_parameter']]], ref['value'])
  for method in ['wnls', 'fgnls']:
    refs = [r for r in projections if r['case'] == 'original' and r['method'] == method]
    assert len(refs) == len(original) == 10000
    for i, ((y, x), ref, actual) in enumerate(zip(original, refs, exported)):
      assert ref['row'] == actual['source_row'] == str(i + 1)
      assert y == float(ref['y']) == float(actual['y']) and x == float(ref['x']) == float(actual['x'])
      check('preliminary_mean', actual['preliminary_mean'], ref['preliminary_mean'])
      check('initial_json_mean', initial['fitted_values'][i], ref['preliminary_mean'])
      check('initial_json_residual', initial['residuals'][i], y - float(ref['preliminary_mean']))
      for column, field in [(method+'_weight', 'weight'), (method+'_scale', 'scale'),
          (method+'_y', 'transformed_response'), (method+'_level_mean', 'level_mean'),
          (method+'_level_residual', 'level_residual'), (method+'_rate', 'rate')]:
        check(column, actual[column], ref[field])
      for mode in ['classical', 'hc1']:
        fit = selected[method, mode]
        for field, column, rfield in [('fitted_values', method+'_mean_'+mode, 'transformed_mean'),
                                      ('residuals', method+'_residual_'+mode, 'transformed_residual')]:
          check('json_'+rfield, fit[field][i], ref[rfield])
          check('csv_'+rfield, actual[column], ref[rfield])
  for ref in rows(reference/'summaries.csv'):
    if ref['case'] != 'original': continue
    fit = selected[ref['method'], ref['mode']]
    for field, expected in [('residual_sum_of_squares', 'scaled_rss'),
        ('root_mean_squared_error', 'scaled_rmse'), ('centered_r_squared', 'scaled_r_squared')]:
      check(field, fit[field], ref[expected])
    check('residual_variance', fit['residual_variance'], float(ref['scaled_rss'])/9998)
    check('covariance_correction', fit['covariance_correction'], 10000/9998 if fit['robust'] else 1)
  headers = ['observations', 'wnls_scaled_rss', 'wnls_level_rss', 'wnls_level_r_squared',
             'fgnls_scaled_rss', 'fgnls_level_rss', 'fgnls_level_r_squared', 'scope']
  diagnostics = [v['data'] for v in leaves if v.get('result_type') == 'TableResult' and v['data']['headers'] == headers]
  assert len(diagnostics) == 1 and len(diagnostics[0]['rows']) == 1
  values = diagnostics[0]['rows'][0]
  assert values[0] == 10000 and values[-1] == 'fixed first-fit weights; scaled and original-level diagnostics are distinct'
  for j, method in [(1, 'wnls'), (4, 'fgnls')]:
    ref = next(r for r in rows(reference/'summaries.csv') if r['case'] == 'original' and r['method'] == method)
    for k, field in enumerate(['scaled_rss', 'level_rss', 'level_r_squared']):
      check('diagnostic_'+field, values[j+k], ref[field])
  result = {'scope': 'original-source weighted CLI JSON and CSV versus independent R',
    'numeric_gates': gates, 'exact_source_rows_per_method': 10000, 'max_absolute_differences': maximum,
    'source_sha256': HASHES, 'native_json_sha256': sha(native_json), 'native_csv_sha256': sha(native_csv),
    'reference_R_sha256': sha(Path(__file__).with_name('weighted_reference.R'))}
  staged = None
  try:
    with tempfile.NamedTemporaryFile(mode='w', dir=Path(receipt).parent, prefix='.weighted-audit-', delete=False) as stream:
      staged = Path(stream.name)
      json.dump(result, stream, indent=2); stream.write('\n')
    os.link(staged, receipt)
  finally:
    if staged is not None:
      staged.unlink()
  print(json.dumps(result, indent=2))


if __name__ == '__main__':
  parser = argparse.ArgumentParser(description=__doc__)
  for arg in ['native_json', 'native_csv', 'reference', 'source', 'receipt']:
    parser.add_argument(arg, type=Path)
  audit(**vars(parser.parse_args()))

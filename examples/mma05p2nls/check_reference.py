#!/usr/bin/env python3
"""Development-only full actual-CLI/source/R audit; never create receipt on failure."""
import argparse
import csv
import hashlib
import json
import math
import os
from pathlib import Path
import tempfile

HASHES = {
  'mma05data.asc': '460f5d731f1848c8695d78a1fbf0db348d0daced4b8b5b0c2734bffa20b3e31c',
  'mma05p2nls.do': '61a47826604f21594c2c2c96e44ef7233caa7b82df2b63566f49eecf0586554d',
  'mma05p2nls.txt': 'ca79b44886ee861f16a5e617f9e5f88c79fa5ad0fdb3276261010e9708a1d557',
}


def sha(path):
  return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def rows(path):
  with Path(path).open(newline='') as stream:
    return list(csv.DictReader(stream))


def audit(native_json, native_csv, reference, source, receipt):
  reference, source = Path(reference), Path(source)
  for name, expected in HASHES.items():
    assert sha(source / name) == expected, f'original source hash: {name}'
  leaves = [json.loads(line) for line in Path(native_json).read_text().splitlines()]
  assert leaves and all(v['schema_version'] == 1 for v in leaves)
  fits = [v['data'] for v in leaves if v.get('result_type') == 'NlsResult']
  assert len(fits) == 2
  modes = {'classical': 'working_homoskedastic_gauss_newton', 'hc1': 'hc1_gauss_newton'}
  fitted = {}
  for mode, covariance in modes.items():
    selected = [r for r in fits if r['covariance'] == covariance]
    assert len(selected) == 1
    fit = selected[0]; fitted[mode] = fit
    assert fit['model']['outcome'] == 'y'
    assert fit['model']['parameter_names'] == ['a', 'b']
    assert fit['model']['predictor_names'] == ['x']
    assert fit['model']['expression'] == {'kind':'exp', 'arguments':
      {'kind':'subtract', 'arguments':[
        {'kind':'negate', 'arguments':{'kind':'parameter', 'arguments':0}},
        {'kind':'multiply', 'arguments':[{'kind':'parameter', 'arguments':1}, {'kind':'predictor', 'arguments':0}]}]}}
    assert fit['start_values'] == [1, 0] and fit['robust'] is (mode == 'hc1')
    assert fit['observation_count'] == 10000 and fit['residual_degrees_of_freedom'] == 9998
    assert len(fit['coefficients']) == len(fit['covariance_matrix']) == 2
    assert [c['name'] for c in fit['coefficients']] == ['a', 'b']
    assert all(len(row) == 2 for row in fit['covariance_matrix'])
    assert fit['sample']['total_observations'] == 10000 and fit['sample']['dropped_observations'] == 0
    assert fit['sample']['retained_indices'] == list(range(10000))
    assert all(type(v) is int for v in fit['sample']['retained_indices'])
    assert fit['sample']['weights'] is None and fit['sample']['cluster_groups'] is None
    assert fit['inference']['reference_distribution'] == 'approximate_student_t'
    assert fit['inference']['degrees_of_freedom'] == 9998 and fit['inference']['confidence_level'] == .95
    assert fit['covariance_bread'] == 'inverse_jacobian_cross_product'
    assert all(fit[k] is None for k in ['log_likelihood', 'likelihood_ratio', 'omnibus_f'])
    assert 0 <= fit['normalized_score'] <= 1e-12 and 0 <= fit['relative_full_gauss_newton_step'] <= 1e-12
    assert 0 < fit['iterations'] <= 200 and 0 < fit['evaluation_work'] <= 500000000
    assert len(fit['fitted_values']) == len(fit['residuals']) == 10000
  exported = rows(native_csv)
  expected_columns = ['source_row', 'y', 'x', 'mean_classical', 'residual_classical',
                      'mean_hc1', 'residual_hc1', 'fitted_rate', 'squared_residual']
  assert len(exported) == 10000 and list(exported[0]) == expected_columns
  original = [list(map(float, line.split())) for line in (source/'mma05data.asc').read_text().splitlines()]
  inputs = [r for r in rows(reference/'inputs.csv') if r['case'] == 'original']
  assert len(original) == len(inputs) == 10000
  for i, ((y, x), ref, actual) in enumerate(zip(original, inputs, exported), 1):
    assert ref['row'] == actual['source_row'] == str(i)
    assert y == float(ref['y']) == float(actual['y']) and x == float(ref['x']) == float(actual['x'])
  gates, maximum = 0, {}
  def check(quantity, actual, expected):
    nonlocal gates
    actual, expected = float(actual), float(expected)
    assert math.isfinite(actual) and math.isfinite(expected)
    error = abs(actual - expected)
    assert error <= 1e-11 + 1e-10 * abs(expected), (quantity, actual, expected)
    maximum[quantity] = max(maximum.get(quantity, 0), error)
    gates += 1
  index = {'a': 0, 'b': 1}
  tail_relative_errors = []
  for ref in rows(reference/'parameters.csv'):
    if ref['case'] != 'original': continue
    coefficient = fitted[ref['mode']]['coefficients'][index[ref['parameter']]]
    for native, rfield in [('value','estimate'), ('standard_error','standard_error'),
        ('statistic','t'), ('p_value','p_value'), ('confidence_lower','ci_lower'), ('confidence_upper','ci_upper')]:
      check(rfield, coefficient[native], ref[rfield])
      if rfield == 'p_value':
        expected = float(ref[rfield])
        if expected == 0:
          assert coefficient[native] == 0
        else:
          # Additional, STRICTER tail gate: the primary absolute tolerance alone
          # would not detect zero substituted for these ~1e-40/1e-51 probabilities.
          # Allow the propagated converged-t difference, not a new primary tolerance.
          relative = abs(coefficient[native] / expected - 1)
          assert relative <= 1e-9, ('nonzero Student-t tail', relative)
          tail_relative_errors.append(relative)
  for ref in rows(reference/'covariance.csv'):
    if ref['case'] == 'original':
      check('covariance', fitted[ref['mode']]['covariance_matrix'][index[ref['row_parameter']]][index[ref['column_parameter']]], ref['value'])
  projection = [r for r in rows(reference/'projections.csv') if r['case'] == 'original']
  assert len(projection) == 10000
  for i, ref in enumerate(projection):
    assert ref['row'] == str(i + 1)
    actual = exported[i]
    for mode, fit in fitted.items():
      for quantity, field in [('mean','fitted_values'), ('residual','residuals')]:
        check('json_' + quantity, fit[field][i], ref[quantity])
        check('csv_' + quantity, actual[quantity + '_' + mode], ref[quantity])
    check('rate', actual['fitted_rate'], 1 / float(ref['mean']))
    check('squared_residual', actual['squared_residual'], float(ref['residual']) ** 2)
    assert fitted['classical']['fitted_values'][i] == fitted['hc1']['fitted_values'][i]
    assert fitted['classical']['residuals'][i] == fitted['hc1']['residuals'][i]
    assert actual['mean_classical'] == actual['mean_hc1'] and actual['residual_classical'] == actual['residual_hc1']
  for ref in rows(reference/'summaries.csv'):
    if ref['case'] != 'original': continue
    fit = fitted[ref['mode']]
    for native, rfield in [('observation_count','n'), ('residual_degrees_of_freedom','df'),
        ('residual_sum_of_squares','rss'), ('root_mean_squared_error','rmse'), ('centered_r_squared','r_squared')]:
      check(rfield, fit[native], ref[rfield])
    check('p', len(fit['coefficients']), ref['p'])
    assert fit['observation_count'] == int(ref['n']) and fit['residual_degrees_of_freedom'] == int(ref['df'])
    check('covariance_correction', fit['covariance_correction'], 10000 / 9998 if fit['robust'] else 1)
    check('residual_variance', fit['residual_variance'], float(ref['rss']) / 9998)
    a, b = [c['value'] for c in fit['coefficients']]
    assert a == fitted['classical']['coefficients'][0]['value'] and b == fitted['classical']['coefficients'][1]['value']
    for (y, x), mu in zip(original, fit['fitted_values']):
      check('formula_mean', mu, math.exp(-a-b*x))
  headers = ['observations', 'observed_mean', 'mean_fitted_response', 'residual_sum_of_squares',
             'mean_residual', 'covariance_mode_mean_difference', 'covariance_mode_residual_difference', 'scope']
  diagnostics = [v['data'] for v in leaves if v.get('result_type') == 'TableResult' and v['data']['headers'] == headers]
  assert len(diagnostics) == 1 and len(diagnostics[0]['rows']) == 1
  diagnostic = diagnostics[0]['rows'][0]
  assert diagnostic[-1] == 'descriptive residual summaries, not an adequacy test; covariance assumes correct mean'
  expected = [10000, math.fsum(y for y, x in original)/10000,
              math.fsum(float(r['mean']) for r in projection)/10000,
              math.fsum(float(r['residual'])**2 for r in projection),
              math.fsum(float(r['residual']) for r in projection)/10000, 0, 0]
  for name, actual, value in zip(headers, diagnostic, expected):
    check('diagnostic_' + name, actual, value)
  # Author compact precision is a separate historical gate, never the primary R tolerance.
  author = [('classical',0,'value',1.8876,.00005), ('classical',1,'value',-.9575,.00005),
    ('classical',0,'standard_error',.0307,.00005), ('classical',1,'standard_error',.0097,.00005),
    ('hc1',0,'standard_error',.1421,.00005), ('hc1',1,'standard_error',.0612,.00005),
    ('classical',0,'statistic',61.52,.005), ('classical',1,'statistic',-98.28,.005)]
  for mode, i, field, expected, half_unit in author:
    assert abs(fitted[mode]['coefficients'][i][field] - expected) <= half_unit
  assert gates == 120055, gates
  result = {'scope': 'actual original-source CLI JSON and exported CSV versus independent R',
    'numeric_gates': gates, 'exact_exported_source_fields': 30000,
    'exact_reference_source_fields': 20000, 'compact_author_gates': len(author)+1,
    'max_absolute_differences': maximum, 'source_sha256': HASHES,
    'additional_nonzero_tail_gates': len(tail_relative_errors),
    'max_nonzero_tail_relative_error': max(tail_relative_errors),
    'native_json_sha256': sha(native_json), 'native_csv_sha256': sha(native_csv),
    'reference_R_sha256': sha(Path(__file__).with_name('reference.R'))}
  staged = None
  try:
    with tempfile.NamedTemporaryFile(mode='w', dir=Path(receipt).parent,
                                    prefix='.nls-audit-', delete=False) as stream:
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

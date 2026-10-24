"""Permanent adversarial two-step audit controls; mutate temporary copies only.
Usage: python3 check_heckman_failures.py VALIDATED_WORK_DIRECTORY
"""
import contextlib
import hashlib
import io
import json
import math
import pathlib
import shutil
import sys
import tempfile
from check_heckman import audit
from check_tobit import FIELDS, NORMAL_95
from check_tobit_failures import change_csv, original_change

FILES = ['mma16p1tobit.asc', 'mma16p1tobit.csv', 'heckman.jsonl', 'heckman_predictions.csv', 'R_heckman_predictions.csv', 'R_heckman_quantities.csv']


def records(work):
  return [json.loads(line) for line in (work/'heckman.jsonl').read_text().splitlines()]


def publish(work, data):
  (work/'heckman.jsonl').write_text(''.join(json.dumps(row)+'\n' for row in data))


def wrong_order(work):
  change_csv(work, 'heckman_predictions.csv', lambda rows: rows.reverse())


def wrong_conditional(work):
  change_csv(work, 'heckman_predictions.csv', lambda rows: rows[0].update(conditional_selected_mean=rows[0]['latent']))


def incomplete_reference(work):
  change_csv(work, 'R_heckman_quantities.csv', lambda rows: rows.pop())


def shared_zero_tail(work):
  data = records(work);data[1]['data']['selection_coefficients'][0]['p_value'] = 0;publish(work, data)
  change_csv(work, 'R_heckman_quantities.csv', lambda rows: [r.update(value='0') for r in rows if r['quantity'] == 'selection_p_value' and r['i'] == '1'])


def shared_covariance(work, stage):
  data = records(work);key = 'covariance_matrix' if stage == 'outcome' else 'selection_covariance_matrix'
  for i, j in [(0, 1), (1, 0)]:
    data[1]['data'][key][i][j] *= 1.01
  publish(work, data)
  change_csv(work, 'R_heckman_quantities.csv', lambda rows: [r.update(value=str(float(r['value'])*1.01)) for r in rows if r['quantity'] == stage+'_covariance' and (r['i'], r['j']) in [('1', '2'), ('2', '1')]])


def false_stationary_control(work):
  data = records(work);fit = data[1]['data'];row = fit['selection_coefficients'][0]
  row['value'] += .001;row['statistic'] = row['value']/row['standard_error']
  row['p_value'] = math.erfc(abs(row['statistic'])/math.sqrt(2))
  row['confidence_lower'] = row['value']-NORMAL_95*row['standard_error']
  row['confidence_upper'] = row['value']+NORMAL_95*row['standard_error']
  for name in ['selection_max_abs_scaled_score', 'selection_max_abs_scaled_newton_step', 'max_abs_scaled_outcome_normal_equation']:
    fit[name] = 0
  publish(work, data)
  def quantities(rows):
    for r in rows:
      if r['quantity'] in ['selection_scaled_score', 'selection_scaled_step', 'outcome_scaled_normal_equation']:
        r['value'] = '0'
      elif r['quantity'].startswith('selection_') and r['i'] == '1' and r['j'] == '0':
        field = r['quantity'].removeprefix('selection_')
        if field in FIELDS:
          r['value'] = str(row[field])
  change_csv(work, 'R_heckman_quantities.csv', quantities)
  # Keep both exported/reference predictions coherent with the forged point.
  # A cross-tool equality-only checker would accept the forged zero certificate.
  gamma = [c['value'] for c in fit['selection_coefficients']]
  beta = [c['value'] for c in fit['coefficients']]
  def predictions(rows, native):
    for r in rows:
      t = gamma[0]+gamma[1]*float(r['lnwage'])
      density = math.exp(-t*t/2)/math.sqrt(2*math.pi);p = .5*math.erfc(-t/math.sqrt(2));mills = density/p
      latent = beta[0]+beta[1]*float(r['lnwage'])
      r['selection_probability'] = str(p)
      r['inverse_mills' if native else 'mills'] = str(mills)
      r['conditional_selected_mean'] = str(latent+beta[2]*mills)
      r['observed_contribution_mean'] = str(p*latent+beta[2]*density)
      if not native:
        r['selection_index'] = str(t)
  change_csv(work, 'heckman_predictions.csv', lambda rows: predictions(rows, True))
  change_csv(work, 'R_heckman_predictions.csv', lambda rows: predictions(rows, False))


def wrong_reference_cell(work):
  change_csv(work, 'R_heckman_predictions.csv', lambda rows: rows[0].update(e=str(float(rows[0]['e'])+1)))


def wrong_selected_identity(work):
  data = records(work);indices = data[1]['data']['selected_sample']['retained_indices'];indices[0], indices[1] = indices[1], indices[0];publish(work, data)


def main(work):
  before = {name: hashlib.sha256((work/name).read_bytes()).hexdigest() for name in FILES}
  with contextlib.redirect_stdout(io.StringIO()):
    audit(work)
  cases = [original_change, wrong_order, wrong_conditional, incomplete_reference, shared_zero_tail,
    lambda p: shared_covariance(p, 'outcome'), lambda p: shared_covariance(p, 'selection'),
    false_stationary_control, wrong_reference_cell, wrong_selected_identity]
  for i, change in enumerate(cases, 1):
    with tempfile.TemporaryDirectory(prefix='tabdat-heckman-corruption-') as tmp:
      target = pathlib.Path(tmp)
      for name in FILES:
        shutil.copy2(work/name, target/name)
      change(target)
      try:
        with contextlib.redirect_stdout(io.StringIO()):
          audit(target)
      except AssertionError as error:
        if change is false_stationary_control:
          assert 'independent selection score' in str(error), error
      else:
        raise AssertionError(f'corruption{i} accepted')
      print(f'corruption{i} rejected')
  assert before == {name: hashlib.sha256((work/name).read_bytes()).hexdigest() for name in FILES}
  print('PASS:10 adversarial audit controls; all6 input fingerprints unchanged')


if __name__ == '__main__':
  assert len(sys.argv) == 2
  main(pathlib.Path(sys.argv[1]))

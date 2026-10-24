"""Development-only original-source two-step audit; standard library, no runtime dependency.
Usage: python3 check_heckman.py WORK_DIRECTORY
Scalar gates1e-9 abs+1e-8 rel; positive normal tails1e-8 rel; probabilities1e-10 abs.
Coordinate-dependent certificates are bounded separately, never equated.
"""
import hashlib
import json
import math
import pathlib
import sys
from check_tobit import FIELDS, SOURCE, close, finite, inference, probability, read_csv


def transpose(a):
  return list(map(list, zip(*a)))


def multiply(a, b):
  return [[sum(x*y for x, y in zip(row, col)) for col in zip(*b)] for row in a]


def inverse(a):
  n = len(a)
  rows = [list(row)+[float(i == j) for j in range(n)] for i, row in enumerate(a)]
  for j in range(n):
    pivot = max(range(j, n), key=lambda i: abs(rows[i][j]))
    rows[j], rows[pivot] = rows[pivot], rows[j]
    divisor = rows[j][j]
    assert math.isfinite(divisor) and divisor != 0, 'singular independent design'
    rows[j] = [v/divisor for v in rows[j]]
    for i in range(n):
      if i != j:
        factor = rows[i][j]
        rows[i] = [v-factor*w for v, w in zip(rows[i], rows[j])]
  return [row[n:] for row in rows]


def audit(work):
  raw = (work/'mma16p1tobit.asc').read_bytes()
  assert hashlib.sha256(raw).hexdigest() == '8ec7ba9cf3da6854217f873441e47fd5492e777904057c8d9ca8f8ee00244ba6', 'original source SHA256'
  source = [line.split() for line in raw.decode().splitlines() if line.strip()]
  assert len(source) == 200 and all(len(row) == 6 for row in source)
  events = [json.loads(line) for line in (work/'heckman.jsonl').read_text().splitlines()]
  assert len(events) == 8 and all(e['schema_version'] == 1 for e in events)
  assert [e['result_type'] for e in events] == ['LoadResult', 'HeckmanTwoStepResult']+['PredictionResult']*5+['ExportResult']
  fit = events[1]['data']
  assert fit['estimator'] == 'heckman' and fit['method'] == 'twostep'
  assert fit['covariance'] == 'model_based_heckman_two_step_generated_regressor_correction'
  assert fit['selection_covariance'] == 'model_based_observed_probit_information'
  assert fit['covariance_blocks'] == 'separate_selection_and_augmented_outcome_no_cross_equation_inference'
  assert fit['reference_distribution'] == 'asymptotic_standard_normal' and fit['covariance_correction'] == 1
  assert fit['confidence_level'] == 95 and fit['scale_correlation_inference'] == 'points_only'
  assert fit['joint_log_likelihood'] is None and fit['excluded_selection_predictors'] == []
  assert 'functional form' in fit['identification_warning']
  assert 'scale_standard_error' not in fit and 'rho_standard_error' not in fit
  assert fit['selection_outcome'] == 'dy' and fit['outcome'] == 'ytrunc'
  assert fit['predictors'] == fit['selection_predictors'] == ['lnwage'] and fit['include_intercept']
  assert [c['name'] for c in fit['coefficients']] == ['intercept', 'lnwage', 'mills_coefficient']
  assert [c['name'] for c in fit['selection_coefficients']] == ['intercept', 'lnwage']
  ref_rows = read_csv(work, 'R_heckman_quantities.csv')
  ref = {(r['quantity'], int(r['i']), int(r['j'])): finite(r['value']) for r in ref_rows}
  expected = {(stage+'_'+field, i, 0) for stage, n in [('selection', 2), ('outcome', 3)] for field in FIELDS for i in range(1, n+1)}
  expected |= {(stage+'_covariance', i, j) for stage, n in [('selection', 2), ('outcome', 3)] for i in range(1, n+1) for j in range(1, n+1)}
  summary = ['n', 'selected_count', 'unselected_count', 'selection_log_likelihood', 'sigma', 'rho', 'mills_coefficient', 'outcome_sse', 'mean_mills', 'mean_selected_delta', 'regressor_mills_correlation', 'excluded_selection_predictor_count', 'selection_scaled_score', 'selection_scaled_step', 'outcome_scaled_normal_equation']
  expected |= {(name, 0, 0) for name in summary}
  assert len(ref_rows) == len(ref) == 58 and set(ref) == expected, 'exact reference inventory'
  for stage, coefs, covariance in [('selection', fit['selection_coefficients'], fit['selection_covariance_matrix']), ('outcome', fit['coefficients'], fit['covariance_matrix'])]:
    assert len(covariance) == len(coefs) and all(len(row) == len(coefs) for row in covariance)
    for i, c in enumerate(coefs, 1):
      inference(c)
      for field in FIELDS:
        close(c[field], ref[stage+'_'+field, i, 0], stage+'_'+field)
      close(c['standard_error']**2, covariance[i-1][i-1], 'coefficient variance')
      for j, value in enumerate(covariance[i-1], 1):
        close(value, ref[stage+'_covariance', i, j], stage+' covariance')
  beta = [c['value'] for c in fit['coefficients']]
  gamma = [c['value'] for c in fit['selection_coefficients']]
  sigma, rho, alpha = finite(fit['sigma']), finite(fit['rho']), beta[-1]
  assert sigma > 0 and abs(rho) < 1
  close(alpha, rho*sigma, 'alpha=rho*sigma')
  output = read_csv(work, 'heckman_predictions.csv')
  prepared = read_csv(work, 'mma16p1tobit.csv')
  gold = read_csv(work, 'R_heckman_predictions.csv')
  assert len(output) == len(prepared) == len(gold) == len(source)
  information = [[0.0]*2 for _ in range(2)]
  score = [0.0]*2
  w, z1, delta, residuals, all_mills, wages, selected = [], [], [], [], [], [], []
  loglik = 0.0
  pairs = [('latent', 'latent_mean'), ('selection_probability', 'selection_probability'), ('inverse_mills', 'mills'), ('conditional_selected_mean', 'conditional_selected_mean'), ('observed_contribution_mean', 'observed_contribution_mean')]
  for i, (raw_row, row, prep, r) in enumerate(zip(source, output, prepared, gold)):
    for table in [row, prep, r]:
      assert int(table['source_row']) == i+1, 'original row order'
      for name, token in zip(SOURCE, raw_row):
        assert table[name] == '' if token == '.' else finite(table[name]) == finite(token), ('original cell', i, name)
    wage = finite(raw_row[1]);d = finite(raw_row[5]);assert d in (0, 1)
    t = gamma[0]+gamma[1]*wage
    p = .5*math.erfc(-t/math.sqrt(2));q = .5*math.erfc(t/math.sqrt(2))
    density = math.exp(-t*t/2)/math.sqrt(2*math.pi)
    mills, complement = density/p, density/q
    curvature = mills*(mills+t) if d else complement*(complement-t)
    g = mills if d else -complement
    zi = [1.0, wage]
    for j in range(2):
      score[j] += zi[j]*g
      for k in range(2):
        information[j][k] += curvature*zi[j]*zi[k]
    loglik += math.log(p if d else q)
    latent = beta[0]+beta[1]*wage
    values = [latent, p, mills, latent+alpha*mills, p*latent+alpha*density]
    for (name, reference_name), value in zip(pairs, values):
      if name == 'selection_probability':
        probability(row[name], value);probability(row[name], r[reference_name])
      else:
        close(row[name], value, name);close(row[name], r[reference_name], name)
    close(r['selection_index'], t, 'reference selection index')
    wages.append(wage);all_mills.append(mills)
    if d:
      selected.append(i);w.append([1.0, wage, mills]);z1.append(zi)
      delta.append(mills*(mills+t));residuals.append(finite(raw_row[3])-values[3])
  assert len(selected) == 130
  for key, indices in [('selection_sample', list(range(200))), ('selected_sample', selected)]:
    sample = fit[key]
    assert sample['retained_indices'] == indices and sample['total_observations'] == 200
    assert sample['dropped_observations'] == 200-len(indices) and sample['weights'] is None and sample['cluster_groups'] is None
  assert max(abs(s)*max(1, abs(g)) for s, g in zip(score, gamma))/200 <= 1e-10, 'independent selection score'
  identity = multiply(information, fit['selection_covariance_matrix'])
  assert max(abs(value-float(i == j)) for i, row in enumerate(identity) for j, value in enumerate(row)) <= 1e-8, 'independent observed-information inverse'
  close(loglik, fit['selection_log_likelihood'], 'selection-only likelihood')
  sse = sum(v*v for v in residuals)
  close(sse, fit['outcome_sse'], 'selected SSE')
  close(sigma*sigma, sse/130+alpha*alpha*sum(delta)/130, 'two-step scale moment')
  for j in range(3):
    bound = abs(sum(row[j]*e for row, e in zip(w, residuals)))/(130*max(abs(float(source[i][3])) for i in selected)*math.sqrt(sum(row[j]**2 for row in w)/130))
    assert bound <= 1e-10, 'independent augmented outcome normal equation'
  a_inv = inverse(multiply(transpose(w), w))
  f = multiply(transpose(w), [[d*v for v in row] for d, row in zip(delta, z1)])
  generated = multiply(multiply(f, fit['selection_covariance_matrix']), transpose(f))
  meat = multiply(transpose(w), [[(sigma*sigma-alpha*alpha*d)*v for v in row] for d, row in zip(delta, w)])
  for i in range(3):
    for j in range(3):
      meat[i][j] += alpha*alpha*generated[i][j]
  corrected = multiply(multiply(a_inv, meat), a_inv)
  for i in range(3):
    for j in range(3):
      close(fit['covariance_matrix'][i][j], corrected[i][j], 'independent generated-regressor covariance')
  mean_wage, mean_mills = sum(wages)/200, sum(all_mills)/200
  corr = sum((x-mean_wage)*(v-mean_mills) for x, v in zip(wages, all_mills))/math.sqrt(sum((x-mean_wage)**2 for x in wages)*sum((v-mean_mills)**2 for v in all_mills))
  values = [200, 130, 70, loglik, sigma, rho, alpha, sse, mean_mills, sum(delta)/130, corr, 0]
  for name, value in zip(summary[:12], values):
    close(value, ref[name, 0, 0], name)
  for reference_name, native_name in [('selection_scaled_score', 'selection_max_abs_scaled_score'), ('selection_scaled_step', 'selection_max_abs_scaled_newton_step'), ('outcome_scaled_normal_equation', 'max_abs_scaled_outcome_normal_equation')]:
    assert 0 <= finite(ref[reference_name, 0, 0]) <= 1e-10
    assert 0 <= finite(fit[native_name]) <= 1e-10
  print('PASS:55 numerical comparisons+3 separately bounded certificates,13 covariance entries,1000 native predictions+200 reference indices,1400 original cells/ordinals; independently certified selection score/information, two-step covariance/scale and normal inference')


if __name__ == '__main__':
  assert len(sys.argv) == 2, 'usage: check_heckman.py WORK_DIRECTORY'
  audit(pathlib.Path(sys.argv[1]))

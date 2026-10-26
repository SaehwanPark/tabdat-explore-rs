#!/usr/bin/env python3
"""Development-only Cox/Breslow audit: exact source, full scores/information/baseline."""
import json
import math
from pathlib import Path
import sys
from check_reference import rows, close, inverse, multiply


def audit(input_csv, native_json, native_predictions, reference_directory):
  ref = Path(reference_directory)
  raw, pred = rows(input_csv), rows(native_predictions)
  expected, hexes = rows(ref/'predictions.csv'), rows(ref/'source_hex.csv')
  assert len(raw) == len(pred) == len(expected) == len(hexes) > 40
  assert list(raw[0]) == list(hexes[0])
  assert set(pred[0]) == set(expected[0]) | {'rr_ui', 'dr_ui', 'original_ui'}
  for source, actual, hexcells in zip(raw, pred, hexes):
    for name, value in hexcells.items():
      assert float(source[name]) == float(actual[name]) == float.fromhex(value), (name, source[name], actual[name], value)
  events = [json.loads(line) for line in Path(native_json).read_text().splitlines()]
  assert all(e.get('error') is None for e in events)
  fits = [e['data'] for e in events if e['result_type'] == 'CoxResult']
  assert len(fits) == 1
  fit = fits[0]
  names = (ref/'parameter_names.txt').read_text().splitlines()
  n, k = len(raw), len(names)
  assert k == 40 and fit['predictors'] == names == fit['covariance_parameter_names']
  assert [c['name'] for c in fit['coefficients']] == names
  assert len(fit['covariance_matrix']) == k and all(len(row) == k for row in fit['covariance_matrix'])
  assert fit['estimator'] == 'stcox_breslow' and fit['ties'] == 'breslow'
  assert fit['robust'] and not fit['include_intercept']
  assert fit['time_variable'] == 'spell' and fit['failure_variable'] == 'censor1'
  assert fit['covariance'] == 'full_efficient_martingale_score_sandwich'
  assert fit['reference_distribution'] == 'asymptotic_standard_normal'
  assert fit['null_model'] == 'all_beta_zero_same_sample'
  assert fit['likelihood_ratio_p_value'] is None and fit['likelihood_ratio_reference_distribution'] is None
  assert fit['baseline_reference'] == 'all_retained_predictor_means_in_predictor_order_including_dummy_means'
  assert fit['baseline_uncertainty'] == 'unavailable'
  assert fit['confidence_level'] == 95 and fit['converged']
  assert fit['max_abs_scaled_score'] <= 1e-10 and fit['max_abs_scaled_newton_step'] <= 1e-10
  assert not {'fitted_xb', 'fitted_cumulative_hazard', 'fitted_survival', 'fitted_martingale'} & set(fit)
  sample = fit['sample']
  assert sample['total_observations'] == n and sample['dropped_observations'] == 0
  assert sample['retained_indices'] == list(range(n)) and sample['weights'] is None and sample['cluster_groups'] is None
  gates = rows(ref/'robust.csv')
  assert len(gates) == 7*k + k*k + 8*len(fit['baseline']) + 12
  for gate in gates:
    value = fit
    for part in gate['path'].strip('/').split('/'):
      value = value[int(part)] if isinstance(value, list) else value[part]
    close(float(value), float(gate['value']), gate['path'].endswith('p_value'))
  for actual, expected_row in zip(pred, expected):
    for name, value in expected_row.items():
      positive = ('hazard' in name or 'survival' in name) and float(value) > 0
      close(float(actual[name]), float(value), positive)
  b = [c['value'] for c in fit['coefficients']]
  X, time, event = [], [], []
  for row in pred:
    r = {name: float(value) for name, value in row.items()}
    assert r['ui'] == r['original_ui'] and r['rr_ui'] == r['reprate']*r['ui'] and r['dr_ui'] == r['disrate']*r['ui']
    X.append([r[name] for name in names]); time.append(r['spell']); event.append(r['censor1'])
  center = [math.fsum(row[j]/n for row in X) for j in range(k)]
  for a, value in zip(fit['baseline_reference_values'], center): close(a, value)
  scales = [math.sqrt(math.fsum((row[j]-center[j])**2/n for row in X)) for j in range(k)]
  Z = [[(x-m)/s for x,m,s in zip(row,center,scales)] for row in X]
  eta = [math.fsum(z*s*v for z,s,v in zip(row,scales,b)) for row in Z]
  scores, information = [[0.0]*k for _ in raw], [[0.0]*k for _ in range(k)]
  ll, null_ll, H = 0.0, 0.0, 0.0
  knots = sorted(set(t for t,d in zip(time,event) if d == 1))
  assert len(knots) == len(fit['baseline'])
  independent_baseline = []
  for t, baseline in zip(knots, fit['baseline']):
    risk = [i for i in range(n) if time[i] >= t]
    failures = [i for i in risk if time[i] == t and event[i] == 1]
    d = len(failures)
    maximum = max(eta[i] for i in risk)
    weights = [math.exp(eta[i]-maximum) for i in risk]
    S0 = math.fsum(weights)
    mean = [math.fsum(Z[i][j]*w for i,w in zip(risk,weights))/S0 for j in range(k)]
    ll += math.fsum(eta[i]-maximum for i in failures) - d*math.log(S0)
    null_ll -= d*math.log(len(risk))
    log_increment = math.log(d)-maximum-math.log(S0)
    increment = math.exp(log_increment); H += increment
    assert baseline['time'] == t and baseline['risk_count'] == len(risk) and baseline['event_count'] == d
    for field, value in [('log_increment',log_increment),('increment',increment),('log_cumulative_hazard',math.log(H)),('cumulative_hazard',H),('survival',math.exp(-H))]:
      close(baseline[field],value,field in ('increment','cumulative_hazard','survival'))
    independent_baseline.append(H)
    for j in range(k):
      for l in range(j+1):
        value = d*math.fsum(w*(Z[i][j]-mean[j])*(Z[i][l]-mean[l]) for i,w in zip(risk,weights))/S0
        information[j][l] += value
        information[l][j] = information[j][l]
    for i in failures:
      for j in range(k): scores[i][j] += Z[i][j]-mean[j]
    for i,w in zip(risk,weights):
      for j in range(k): scores[i][j] -= w*d/S0*(Z[i][j]-mean[j])
  assert max(abs(math.fsum(row[j] for row in scores)) for j in range(k)) < 1e-7
  reference_scores = rows(ref/'score_residuals.csv')
  assert len(reference_scores) == n and len(reference_scores[0]) == k
  for actual, expected_row in zip(scores, reference_scores):
    for value, scale, expected_value in zip(actual, scales, expected_row.values()): close(value*scale,float(expected_value))
  bread = inverse(information)
  identity = multiply(information,bread)
  assert max(abs(v-float(i==j)) for i,row in enumerate(identity) for j,v in enumerate(row)) < 1e-8
  # Efficient-score influences form the FULL meat, including all censored rows.
  influence = [[math.fsum(a*u for a,u in zip(row,score)) for row in bread] for score in scores]
  correction = n/(n-1)
  for j in range(k):
    for l in range(k):
      value = math.fsum(row[j]*row[l] for row in influence)*correction/scales[j]/scales[l]
      close(fit['covariance_matrix'][j][l],value)
  close(fit['covariance_correction'],correction)
  for i,c in enumerate(fit['coefficients']):
    se = math.sqrt(fit['covariance_matrix'][i][i]); z = b[i]/se
    for field,value in [('standard_error',se),('statistic',z),('confidence_lower',b[i]-1.959963984540054*se),('confidence_upper',b[i]+1.959963984540054*se)]: close(c[field],value)
    close(c['p_value'],math.erfc(abs(z)/math.sqrt(2)),True)
  covariance = [[fit['covariance_matrix'][i][j]*scales[i]*scales[j] for j in range(k)] for i in range(k)]
  restricted = inverse(covariance); theta = [v*s for v,s in zip(b,scales)]
  wald = math.fsum(theta[i]*restricted[i][j]*theta[j] for i in range(k) for j in range(k))
  close(fit['omnibus_wald']['statistic'],wald)
  assert fit['omnibus_wald']['degrees_of_freedom'] == k
  y = wald/2; terms = [j*math.log(y)-math.lgamma(j+1) for j in range(k//2)]; maximum = max(terms)
  close(fit['omnibus_wald']['p_value'],math.exp(-y+maximum+math.log(math.fsum(math.exp(t-maximum) for t in terms))),True)
  close(fit['log_likelihood'],ll); close(fit['null_log_likelihood'],null_ll)
  close(fit['likelihood_ratio'],2*(ll-null_ll)); assert fit['likelihood_ratio_df'] == k
  assert abs(math.fsum(float(r['censor1'])-float(r['cumulative_hazard']) for r in pred)) < 1e-7
  for ui in (0,1): assert abs(math.fsum(float(r['censor1'])-float(r['cumulative_hazard']) for r in pred if float(r['ui']) == ui)) < 1e-7
  for r in pred:
    r = {name: float(value) for name,value in r.items()}
    baseline = independent_baseline[max(i for i,t in enumerate(knots) if t <= r['spell'])] if r['spell'] >= knots[0] else 0.0
    for prefix,ui in [('',r['ui']),('ui0_',0),('ui1_',1)]:
      candidate = dict(r,ui=ui,rr_ui=r['reprate']*ui,dr_ui=r['disrate']*ui)
      xb = math.fsum(candidate[name]*v for name,v in zip(names,b))
      centered = math.fsum((candidate[name]-m)*v for name,m,v in zip(names,center,b))
      cumulative = baseline*math.exp(centered)
      for field,value in [('xb',xb),('cumulative_hazard',cumulative),('net_survival',math.exp(-cumulative))]: close(r[prefix+field],value,field != 'xb' and value > 0)
  return len(gates), len(pred)*len(pred[0])


if __name__ == '__main__':
  assert len(sys.argv) == 5, 'usage: check_cox.py duration.csv native.json cox_predictions.csv reference_directory'
  gates,cells = audit(*sys.argv[1:])
  print(f'Validated {gates} R gates/all1600 covariance entries, {cells} source/design/projection cells, exact source hex, all efficient row scores and independent risk information/sandwich/baseline/null/points')

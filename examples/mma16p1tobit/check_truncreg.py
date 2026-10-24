"""Development-only source-intended truncated Gaussian audit; standard library.
Usage: python3 check_truncreg.py WORK_DIRECTORY
45 scalar comparisons/4 separately bounded R certificates, full observed inverse,
independent raw-coordinate full/null stationarity and closed-moment null fit.
"""
import hashlib
import json
import math
import pathlib
import sys
from check_tobit import FIELDS, SOURCE, NORMAL_95, close, finite, inference, probability, read_csv
from check_heckman import inverse
PREDICTIONS = ['latent', 'retention_probability', 'conditional_mean']


def tail(t):
  p = .5*math.erfc(-t/math.sqrt(2))
  assert p > 0
  m = math.exp(-.5*t*t)/math.sqrt(2*math.pi)/p
  d = m*(m+t)
  return p, m, d, 1-d, d*(2*m+t)-m


def likelihood(beta, sigma, selected):
  score = [0.0]*3
  info = [[0.0]*3 for _ in range(3)]
  ll = 0.0
  for row in selected:
    x, y = float(row[1]), float(row[3])
    design = [1, x]
    mu = beta[0]+beta[1]*x
    t, z = mu/sigma, (y-mu)/sigma
    p, m, d, h, hp = tail(t)
    ll += -.5*z*z-math.log(sigma)-.5*math.log(2*math.pi)-math.log(p)
    for a in range(2):
      score[a] += design[a]*(z-m)/sigma
      for b in range(2):
        info[a][b] += design[a]*design[b]*h/sigma**2
      info[a][2] += design[a]*(2*z-m+t*d)/sigma
      info[2][a] = info[a][2]
    score[2] += z*z-1+t*m
    info[2][2] += 2*z*z+t*m-t*t*d
  return ll, score, info


def certify(theta, score, info, n):
  # Correlation-scaled Cholesky: positive FULL information, not just diagonals.
  p = len(theta)
  diagonal = [math.sqrt(finite(info[i][i])) for i in range(p)]
  assert all(d > 0 for d in diagonal)
  lower = [[0.0]*p for _ in range(p)]
  for i in range(p):
    for j in range(i+1):
      v = info[i][j]/diagonal[i]/diagonal[j]-sum(lower[i][k]*lower[j][k] for k in range(j))
      if i == j:
        assert v > 0
        lower[i][j] = math.sqrt(v)
      else:
        lower[i][j] = v/lower[j][j]
  covariance = inverse(info)
  step = [sum(row[j]*score[j] for j in range(p)) for row in covariance]
  assert max(abs(g)*max(1, abs(t)) for g, t in zip(score, theta))/n <= 1e-10, 'independent observed score certificate'
  assert max(abs(g)/max(1, abs(t)) for g, t in zip(step, theta)) <= 1e-10, 'independent observed Newton certificate'
  return covariance


def null_fit(selected):
  # With known lower0, the normal conditional mean/variance determine t uniquely
  # in the regular interior. Bisection is independent of both NR implementations.
  values = [float(row[3]) for row in selected]
  mean = sum(values)/len(values)
  variance = sum((v-mean)**2 for v in values)/len(values)
  target = variance/mean**2
  def cv(t):
    p, m, d, h, hp = tail(t)
    return h/(t+m)**2
  lo, hi = -8.0, 12.0
  assert cv(lo) > target > cv(hi)
  for _ in range(100):
    mid = (lo+hi)/2
    if cv(mid) > target:
      lo = mid
    else:
      hi = mid
  t = (lo+hi)/2
  sigma = mean/(t+tail(t)[1])
  beta = [sigma*t, 0.0]
  ll, score, information = likelihood(beta, sigma, selected)
  certify([beta[0], math.log(sigma)], [score[0], score[2]], [[information[i][j] for j in [0, 2]] for i in [0, 2]], len(values))
  return ll


def audit(work):
  raw = (work/'mma16p1tobit.asc').read_bytes()
  assert hashlib.sha256(raw).hexdigest() == '8ec7ba9cf3da6854217f873441e47fd5492e777904057c8d9ca8f8ee00244ba6', 'original source SHA256'
  source = [line.split() for line in raw.decode().splitlines() if line.strip()]
  assert len(source) == 200 and all(len(row) == 6 for row in source)
  indices = [i for i, row in enumerate(source) if row[3] != '.']
  selected = [source[i] for i in indices]
  assert len(selected) == 130 and all(float(row[3]) > 0 and row[5] == '1' for row in selected)
  records = [json.loads(line) for line in (work/'truncreg.jsonl').read_text().splitlines()]
  assert [r['result_type'] for r in records] == ['LoadResult','TruncatedGaussianResult','TruncatedGaussianMarginalEffectsResult','PredictionResult','PredictionResult','PredictionResult','ExportResult']
  assert all(r.get('schema_version') == 1 and 'error' not in r for r in records)
  fit, effects = records[1]['data'], records[2]['data']
  assert fit['estimator'] == effects['estimator'] == 'truncreg'
  assert fit['include_intercept'] and fit['predictors'] == ['lnwage'] and fit['lower_limit'] == 0
  assert fit['sample'] == {'retained_indices':indices,'total_observations':200,'dropped_observations':70,'weights':None,'cluster_groups':None}
  assert fit['observation_count'] == 130 and effects['observation_count'] == 130
  assert fit['covariance_correction'] == effects['covariance_correction'] == 1
  assert fit['covariance_parameter_names'] == ['intercept','lnwage','lnsigma']
  assert fit['converged'] and fit['likelihood_ratio_df'] == 1
  assert effects['sample'] == 'retained_estimation_design'
  for key in ['max_abs_scaled_score','max_abs_scaled_newton_step']:
    assert abs(finite(fit[key])) <= 1e-10, key
  values = {}
  def emit(q, value, i=0, j=0):
    key = (q, i, j)
    assert key not in values
    values[key] = finite(value)
  for prefix, rows in [('coefficient',fit['coefficients']),('ame',effects['effects'])]:
    for i, row in enumerate(rows, 1):
      inference(row)
      for field in FIELDS:
        emit(prefix+'_'+field,row[field],i)
  for i, row in enumerate(fit['covariance_matrix'], 1):
    for j, value in enumerate(row, 1):
      emit('joint_covariance',value,i,j)
  for i, value in enumerate(fit['mean_conditional_derivative_gradient'], 1):
    emit('mean_conditional_derivative_gradient',value,i)
  for key in ['log_likelihood','null_log_likelihood','likelihood_ratio','likelihood_ratio_df','likelihood_ratio_p_value','mean_conditional_derivative']:
    emit(key,fit[key])
  for q, field in [('sigma','value'),('log_sigma','log_value'),('log_sigma_standard_error','log_standard_error'),('sigma_standard_error','standard_error'),('sigma_confidence_lower','confidence_lower'),('sigma_confidence_upper','confidence_upper')]:
    emit(q,fit['scale'][field])
  emit('n',fit['observation_count']);emit('dropped_count',fit['sample']['dropped_observations']);emit('lower_limit',fit['lower_limit'])
  reference = {}
  for row in read_csv(work,'R_truncreg_quantities.csv'):
    key = (row['quantity'],int(row['i']),int(row['j']))
    assert key not in reference
    reference[key] = finite(row['value'])
  certificates = {(q,0,0) for q in ['scaled_score','scaled_step','null_scaled_score','null_scaled_step']}
  assert len(values) == 45 and len(reference) == 49 and set(reference) == set(values)|certificates, 'exact reference inventory'
  for key in certificates:
    assert abs(reference[key]) <= 1e-10, key
  for key, value in values.items():
    close(value,reference[key],key[0])
  beta = [r['value'] for r in fit['coefficients']]
  sigma = fit['scale']['value']
  ll, score, information = likelihood(beta,sigma,selected)
  covariance = certify(beta+[math.log(sigma)],score,information,len(selected))
  close(ll,fit['log_likelihood'],'independent full likelihood')
  for i in range(3):
    for j in range(3):
      close(covariance[i][j],fit['covariance_matrix'][i][j],'independent full observed inverse')
  # Shared native/R SE/statistic/tail/CI corruption can be internally Wald-
  # consistent. Anchor each SE to the independently rebuilt observed inverse.
  for i, row in enumerate(fit['coefficients']):
    close(row['standard_error'],math.sqrt(covariance[i][i]),'independent coefficient observed SE')
  null_ll = null_fit(selected)
  close(null_ll,fit['null_log_likelihood'],'independent same-sample moment null')
  lr = 2*(ll-null_ll)
  close(lr,fit['likelihood_ratio'],'independent LR')
  close(math.erfc(math.sqrt(lr/2)),fit['likelihood_ratio_p_value'],'independent LR p_value')
  scale = fit['scale']
  close(math.log(sigma),scale['log_value'],'log sigma')
  close(math.sqrt(covariance[2][2]),scale['log_standard_error'],'log sigma SE')
  close(sigma*scale['log_standard_error'],scale['standard_error'],'sigma delta SE')
  close(math.exp(math.log(sigma)-NORMAL_95*scale['log_standard_error']),scale['confidence_lower'],'sigma log-Wald lower')
  close(math.exp(math.log(sigma)+NORMAL_95*scale['log_standard_error']),scale['confidence_upper'],'sigma log-Wald upper')
  hs, gs = [], [[],[],[]]
  for row in selected:
    x = float(row[1]);t = (beta[0]+beta[1]*x)/sigma;p,m,d,h,hp = tail(t)
    hs.append(h);gs[0].append(hp/sigma);gs[1].append(hp*x/sigma);gs[2].append(-t*hp)
  mean_h = sum(hs)/130;gradient = [sum(g)/130 for g in gs]
  close(mean_h,fit['mean_conditional_derivative'],'independent mean conditional derivative')
  for a,b in zip(gradient,fit['mean_conditional_derivative_gradient']):
    close(a,b,'independent retained joint gradient')
  g = [beta[1]*v for v in gradient];g[1] += mean_h
  variance = sum(g[i]*covariance[i][j]*g[j] for i in range(3) for j in range(3))
  close(beta[1]*mean_h,effects['effects'][0]['value'],'independent conditional AME')
  close(math.sqrt(variance),effects['effects'][0]['standard_error'],'independent full-joint AME SE')
  native, predicted = read_csv(work,'truncreg_predictions.csv'),read_csv(work,'R_truncreg_predictions.csv')
  converted = read_csv(work,'mma16p1tobit.csv')
  assert len(native) == len(predicted) == len(converted) == 200
  for i, (row, gold, original, prepared) in enumerate(zip(native,predicted,source,converted),1):
    for bound in [row,gold,prepared]:
      assert int(bound['source_row']) == i
      for key, token in zip(SOURCE,original):
        if token == '.':
          assert bound[key] == ''
        else:
          assert float(bound[key]) == float(token), (i,key)
    x = float(original[1]);mu = beta[0]+beta[1]*x;p,m,d,h,hp = tail(mu/sigma)
    for key, independent in zip(PREDICTIONS,[mu,p,mu+sigma*m]):
      if key == 'retention_probability':
        probability(row[key],gold[key]);probability(row[key],independent)
      else:
        close(row[key],gold[key],key);close(row[key],independent,'independent '+key)
  print('Validated45 numerical comparisons+4 bounded R certificates,9 covariance entries,600 predictions,1400 original cells/ordinals and exact130-row membership; independent full/null likelihood/score/information, moment-root null, LR, normal/log-Wald and full-joint conditional-mean delta gates pass.')


if __name__ == '__main__':
  audit(pathlib.Path(sys.argv[1]))

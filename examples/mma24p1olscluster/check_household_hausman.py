#!/usr/bin/env python3
"""Audit original source, separate-scale joint Hausman and owned RE predictions."""
from decimal import Decimal
import hashlib
import json
import math
from pathlib import Path
import re
import shlex
import sys

from check_household_bp_lm import rows, near


def audit(native, reference, raw, mode='--ascii', author=None):
  hashes = {'--ascii':'74cb37e8c14609e382d334e368a54f63bccd4e952d02bcbf535baf19115d4061',
            '--dta':'0ea6639f93a5bc3f53bcbd26b6865ad6908a04f41ab04d90e7c9aa4b7e9787bf'}
  assert mode in ('--ascii','--dta','--csv')
  if mode in hashes:
    assert hashlib.sha256(raw.read_bytes()).hexdigest() == hashes[mode]
  source, original = rows(native/'vietnam.csv'), rows(reference/'source.csv')
  keys = ['source_row','lnexp12m','age','sex','hhsize','farm','educ','lnhhexp','commune']
  assert len(source) == len(original) > 7
  if mode in hashes:
    assert len(source) == 5999
  if mode == '--csv':
    assert rows(raw) == source
  tokens = shlex.split(raw.read_text(encoding='ascii')) if mode == '--ascii' else None
  farms = {'non farm':0,'farm':1}
  educ = {'Never':0,'Nhatre':1,'<cap I':2,'Cap I':3,'Cap II':4,'Cap III':5,'Nghe SC':6,
          'THCN':7,'DHDC':8,'DHCD':9,'Thac sy':10,'PTS':11,'TS':12}
  if tokens is not None:
    assert len(tokens) == len(source)*8
  for i,(a,b) in enumerate(zip(source,original)):
    assert list(a) == list(b) == keys
    for key in keys:
      if a[key] == '':
        assert b[key] == ''
      else:
        assert float(a[key]) == float(b[key]), (i,key)
    if tokens is not None:
      y,age,sex,size,farm,education,x,group = tokens[i*8:i*8+8]
      values = [i+1,'' if y == '.' else y,age,sex,size,farms[farm],educ[education],x,group]
      for key,value in zip(keys,values):
        if value == '':
          assert a[key] == ''
        else:
          assert Decimal(a[key]) == Decimal(str(value)), (i,key)
  selected = sorted((r for r in source if r['lnexp12m'] != ''),key=lambda r:int(r['source_row']))
  groups = {}
  for row in selected:
    groups[row['commune']] = groups.get(row['commune'],0)+1
  n,g = len(selected),len(groups)
  events = [json.loads(line) for line in (native/'result.jsonl').read_text().splitlines()]
  assert all(e['schema_version'] == 1 and 'error' not in e for e in events)
  expected_types = ['LoadResult','IsidResult','TableResult','SqlCreateResult',
    'RandomEffectsResult' if min(groups.values()) == max(groups.values()) else 'UnbalancedRandomEffectsGlsResult',
    'JointPanelHausmanResult','PredictionResult','PredictionResult','SummarizeResult','TableResult','ExportResult']
  assert [e['result_type'] for e in events] == expected_types
  fit,h = events[4]['data'],events[5]['data']
  names = ['lnhhexp','age','sex','hhsize','farm','educ']
  assert h['predictor_names'] == names and h['outcome'] == 'lnexp12m' and h['entity_variable'] == 'commune'
  assert h['re_component_method'] == 'equal_group_between'
  assert h['test'] == 'joint_panel_fe_re_covariance_difference'
  assert h['covariance'] == 'model_based_nonrobust' and h['reference_distribution'] == 'chi_square'
  assert h['covariance_scale_convention'] == 'separate_native_fe_and_re_residual_scales'
  assert 'RE efficiency' in h['assumptions'] and 'not cluster-robust' in h['assumptions']
  assert 'Non-rejection does not establish exogeneity' in h['assumptions']
  assert (h['observation_count'],h['candidate_observation_count'],h['dropped_observation_count']) == (n,n,0)
  assert (h['group_count'],h['minimum_group_size'],h['maximum_group_size'],h['singleton_groups']) == (
    g,min(groups.values()),max(groups.values()),sum(v == 1 for v in groups.values()))
  assert h['df'] == 6 and h['fe_residual_df'] == n-g-6 and h['re_residual_df'] == n-7
  assert fit['variance_component_method'] == 'equal_group_between' and fit['covariance'] == 'nonrobust'
  assert fit['estimator'] == 'equal_group_between_random_effects_gls'
  assert fit['time_variable'] == 'source_row'
  assert [c['name'] for c in fit['coefficients']] == ['intercept']+names
  assert h['re_coefficients'] == [c['value'] for c in fit['coefficients'][1:]]
  assert h['re_covariance'] == [r[1:] for r in fit['covariance_matrix'][1:]]
  near(h['fe_residual_scale'],fit['sigma_e_squared'])
  near(h['re_residual_scale'],fit['transformed_rss']/fit['df_residual'])
  references = rows(reference/'reference.csv')
  assert len(references) == len({r['path'] for r in references}) == 139
  for r in references:
    value = h
    for key in r['path'].strip('/').split('/'):
      value = value[int(key)] if isinstance(value,list) else value[key]
    near(value,r['value'],r['path'] == '/p_value')
  scales = [math.sqrt(max(h['fe_covariance'][i][i],h['re_covariance'][i][i])) for i in range(6)]
  lower = [[0.0]*6 for _ in range(6)]
  whitened = []
  for i in range(6):
    near(h['differences'][i],h['fe_coefficients'][i]-h['re_coefficients'][i])
    for j in range(6):
      near(h['covariance_difference'][i][j],h['fe_covariance'][i][j]-h['re_covariance'][i][j])
      near(h['covariance_difference'][i][j],h['covariance_difference'][j][i])
    for j in range(i+1):
      value = h['covariance_difference'][i][j]/scales[i]/scales[j]-math.fsum(lower[i][s]*lower[j][s] for s in range(j))
      if i == j:
        assert value > 128*sys.float_info.epsilon
        lower[i][j] = math.sqrt(value)
      else:
        lower[i][j] = value/lower[j][j]
    whitened.append((h['differences'][i]/scales[i]-math.fsum(lower[i][j]*whitened[j] for j in range(i)))/lower[i][i])
  near(h['statistic'],math.fsum(v*v for v in whitened))
  actual,expected = rows(native/'household_hausman_predictions.csv'),rows(reference/'predictions.csv')
  assert len(actual) == len(expected) == n
  for a,b,s in zip(actual,expected,selected):
    assert list(a) == list(b) == keys+['re_xb','re_residual']
    for key in keys:
      assert float(a[key]) == float(b[key]) == float(s[key]), (s['source_row'],key)
    for key in ('re_xb','re_residual'):
      near(a[key],b[key])
    xb = fit['coefficients'][0]['value']+math.fsum(c['value']*float(s[name]) for c,name in zip(fit['coefficients'][1:],names))
    near(a['re_xb'],xb)
    near(float(a['re_xb'])+float(a['re_residual']),s['lnexp12m'])
  author_facts = 0
  if author is not None:
    assert mode == '--dta'
    assert hashlib.sha256(author.read_bytes()).hexdigest() == '5e74c8de58eaf833b0070d68b289d61fad43831f3eb8b57c06818a6bf549837e'
    block = author.read_text().split('Hausman specification test')[1].split('. * Alternative')[0]
    refvalues = {r['path']:float(r['value']) for r in references}
    def printed_gate(value,token):
      printed = Decimal(token)
      assert abs(Decimal(str(value))-printed) <= Decimal(10)**printed.as_tuple().exponent/2, (value,token)
    for i,name in enumerate(names):
      match = re.search(r'^\s*'+name.upper()+r'\s*\|\s*([-0-9.]+)\s+([-0-9.]+)\s+([-0-9.]+)\s*$',block,re.M)
      assert match is not None
      for field,token in zip(('fe_coefficients','re_coefficients','differences'),match.groups()):
        printed_gate(h[field][i],token)
        printed_gate(refvalues[f'/{field}/{i}'],token)
        author_facts += 1
    for key,pattern in [('statistic',r'\n\s*=\s*([0-9.]+)'),('p_value',r'Prob>chi2\s*=\s*([0-9.]+)')]:
      token = re.search(pattern,block)[1]
      printed_gate(h[key],token)
      printed_gate(refvalues['/'+key],token)
      author_facts += 1
  assert events[-1]['data']['dataset']['row_count'] == n
  return n,g,author_facts


def main():
  if len(sys.argv) not in (4,5,6):
    raise SystemExit('usage: check_household_hausman.py native-dir reference-dir source [--ascii|--dta|--csv] [author-log]')
  mode = sys.argv[4] if len(sys.argv)>4 else '--ascii'
  n,g,facts = audit(*map(Path,sys.argv[1:4]),mode,Path(sys.argv[5]) if len(sys.argv)==6 else None)
  print('validated139 independent R quantities, all source rows and',n,'prediction rows;',g,'groups;',facts,'author facts')


if __name__ == '__main__':
  main()

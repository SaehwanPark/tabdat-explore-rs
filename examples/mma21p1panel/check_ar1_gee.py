#!/usr/bin/env python3
"""Source-bound AR1 GEE audit against independent dense/R-gee reference."""
import argparse
import hashlib
import math
from pathlib import Path
from check_independent_gee import compare, rows

FIELDS=('lnhr','lnwg','kids','ageh','agesq','disab','id','year')
SHA='2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77'


def main():
  parser=argparse.ArgumentParser()
  parser.add_argument('native',type=Path);parser.add_argument('reference',type=Path);parser.add_argument('source',type=Path)
  parser.add_argument('--synthetic',action='store_true');args=parser.parse_args()
  data=args.source.read_bytes()
  if not args.synthetic and hashlib.sha256(data).hexdigest()!=SHA:
    raise AssertionError('MOM.dat hash mismatch')
  tokens=data.split()
  if not tokens or len(tokens)%8:
    raise AssertionError('source shape mismatch')
  original=[dict(zip(FIELDS,map(float,tokens[i:i+8]))) for i in range(0,len(tokens),8)]
  if not args.synthetic and len(original)!=5320:
    raise AssertionError('original source row count mismatch')
  native={}
  for name in ('fit','coefficients','predictions','unit_scores'):
    a=rows(args.native/f'panel_ar1_gee_{name}.csv');b=rows(args.reference/f'{name}.csv')
    compare(a,b,name);native[name]=a
  if len(native['fit'])!=1:
    raise AssertionError('fit must have one row')
  f=native['fit'][0]
  assert f['convention']=='pearson_ar1_rss_over_n' and f['unit_covariance']=='g_over_g_minus_1_unit_sandwich'
  assert f['inference']=='normal_z_chi2_1' and f['gee_join_key']=='1'
  assert 1<=int(f['iterations'])<100
  g=int(f['people']);n=int(f['observations']);assert n==len(original)==10*g and 3<=g<=532
  assert abs(float(f['unit_multiplier'])-g/(g-1))<1e-14
  predictions=native['predictions'];assert len(predictions)==n
  groups={}
  for index,(r,src) in enumerate(zip(predictions,original),1):
    assert int(r['source_row'])==index
    for key in FIELDS:
      if float(r[key])!=src[key]:raise AssertionError(('source field mismatch',index,key,r[key],src[key]))
    groups.setdefault(int(float(r['id'])),[]).append((int(float(r['year'])),float(r['gee_residual'])))
  assert len(groups)==g and all(sorted(year for year,_ in values)==list(range(1979,1989)) for values in groups.values())
  residual_groups=[sorted(v) for v in groups.values()]
  rss=math.fsum(e*e for group in residual_groups for _,e in group)
  lag=math.fsum(group[j][1]*group[j+1][1] for group in residual_groups for j in range(9))
  phi=rss/n;rho=lag/(9*g*phi)
  assert abs(float(f['rss'])-rss)<=1e-9 and abs(float(f['scale'])-phi)<=1e-12
  assert abs(float(f['working_correlation'])-rho)<=1e-12 and 1-rho*rho>1e-10
  scores=native['unit_scores'];assert len(scores)==g
  assert [int(float(u['id'])) for u in scores]==sorted(groups)
  w00=float(f['working_covariance_aa']);w01=float(f['working_covariance_ab']);w11=float(f['working_covariance_bb'])
  for u in scores:
    assert int(u['observations'])==10
    u0=float(u['score_intercept']);u1=float(u['score_slope'])
    assert abs(float(u['influence_intercept'])-(w00*u0+w01*u1))<=1e-12
    assert abs(float(u['influence_slope'])-(w01*u0+w11*u1))<=1e-12
  for left,right,field in [('influence_intercept','influence_intercept','unit_covariance_aa'),
    ('influence_intercept','influence_slope','unit_covariance_ab'),('influence_slope','influence_slope','unit_covariance_bb')]:
    covariance=g/(g-1)*math.fsum(float(u[left])*float(u[right]) for u in scores)
    assert abs(covariance-float(f[field]))<=1e-12
  print(f'source: {n} ordered rows/{8*n} exact raw fields, {9*g} keyed annual lag pairs; full influence/covariance identities pass')
  if not args.synthetic:
    c={(r['covariance'],r['parameter']):r for r in native['coefficients']}
    facts=[
      ('working_model','lnwg','estimate',.0843777,.00000005),
      ('working_model','lnwg','standard_error',.0123428,.00000005),
      ('working_model','lnwg','z',6.84,.005),
      ('working_model','lnwg','normal95_lower',.0601862,.00000005),
      ('working_model','lnwg','normal95_upper',.1085691,.00000005),
      ('working_model','lnwg','slope_wald_chi2_1',46.73,.005),
      ('working_model','_cons','estimate',7.439893,.0000005),
      ('working_model','_cons','standard_error',.0327698,.00000005),
      ('working_model','_cons','z',227.04,.005),
      ('working_model','_cons','normal95_lower',7.375665,.0000005),
      ('working_model','_cons','normal95_upper',7.50412,.000005),
      ('unit_sandwich_g_adjusted','lnwg','estimate',.0843777,.00000005),
      ('unit_sandwich_g_adjusted','lnwg','standard_error',.0371764,.00000005),
      ('unit_sandwich_g_adjusted','lnwg','z',2.27,.005),
      ('unit_sandwich_g_adjusted','lnwg','p_value',.023,.0005),
      ('unit_sandwich_g_adjusted','lnwg','p_value',.0232,.00005),
      ('unit_sandwich_g_adjusted','lnwg','normal95_lower',.0115133,.00000005),
      ('unit_sandwich_g_adjusted','lnwg','normal95_upper',.1572421,.00000005),
      ('unit_sandwich_g_adjusted','lnwg','slope_wald_chi2_1',5.15,.005),
      ('unit_sandwich_g_adjusted','_cons','estimate',7.439893,.0000005),
      ('unit_sandwich_g_adjusted','_cons','standard_error',.100308,.0000005),
      ('unit_sandwich_g_adjusted','_cons','z',74.17,.005),
      ('unit_sandwich_g_adjusted','_cons','normal95_lower',7.243293,.0000005),
      ('unit_sandwich_g_adjusted','_cons','normal95_upper',7.636493,.0000005),
    ]
    for method,parameter,key,target,half in facts:
      value=float(c[(method,parameter)][key])
      if abs(value-target)>half:raise AssertionError(('author printed',method,parameter,key,value,target,half))
    assert abs(phi-.0803129)<=.00000005
    print(f'author log: {len(facts)+1} facts match printed precision')


if __name__=='__main__':main()

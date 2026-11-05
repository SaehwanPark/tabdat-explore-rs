#!/usr/bin/env python3
"""Audit the P4 report and its entire source-bound Gaussian PA prerequisite."""
import argparse
import math
from pathlib import Path
from check_gaussian_pa import audit as audit_baseline
from check_independent_gee import compare, rows


def main():
  parser = argparse.ArgumentParser()
  parser.add_argument('native', type=Path)
  parser.add_argument('gaussian_reference', type=Path)
  parser.add_argument('p4_reference', type=Path)
  parser.add_argument('source', type=Path)
  parser.add_argument('--synthetic', action='store_true')
  args = parser.parse_args()
  audit_baseline(args.native, args.gaussian_reference, args.source, synthetic=args.synthetic)
  fit = rows(args.native/'panel_exchangeable_p4_fit.csv')
  reference_fit = rows(args.p4_reference/'fit.csv')
  assert len(fit)==len(reference_fit)==1
  assert fit[0]['convention']=='legacy_pearson_exchangeable_rss_over_n'
  assert fit[0]['unit_covariance']=='g_over_g_minus_1_unit_sandwich'
  assert fit[0]['inference']=='normal_z_chi2_1'
  assert 1<=int(fit[0]['iterations'])<100
  compare(fit, reference_fit, 'P4 fit')
  old = rows(args.native/'panel_gaussian_pa_fit.csv')[0]
  g = float(fit[0]['people']);mult = g/(g-1)
  assert abs(float(fit[0]['unit_multiplier'])-mult)<1e-14
  for suffix in ('aa','ab','bb'):
    raw=float(old['unit_covariance_'+suffix]);adj=float(fit[0]['adjusted_unit_covariance_'+suffix])
    assert math.isclose(float(fit[0]['raw_unit_covariance_'+suffix]),raw,rel_tol=1e-14,abs_tol=1e-14)
    assert math.isclose(adj,raw*mult,rel_tol=1e-14,abs_tol=1e-14)
  coefficients=rows(args.native/'panel_exchangeable_p4_coefficients.csv')
  expected=rows(args.p4_reference/'coefficients.csv')
  compare(coefficients,expected,'P4 inference')
  if not args.synthetic:
    values={(r['covariance'],r['parameter']):r for r in coefficients}
    facts=[
      ('working_model','lnwg','estimate',.1195474,.00000005),
      ('working_model','lnwg','standard_error',.0136507,.00000005),
      ('working_model','lnwg','z',8.76,.005),
      ('working_model','lnwg','normal95_lower',.0927925,.00000005),
      ('working_model','lnwg','normal95_upper',.1463023,.00000005),
      ('working_model','lnwg','slope_wald_chi2_1',76.70,.005),
      ('working_model','_cons','estimate',7.345479,.0000005),
      ('working_model','_cons','standard_error',.0364481,.00000005),
      ('working_model','_cons','z',201.53,.005),
      ('working_model','_cons','normal95_lower',7.274042,.0000005),
      ('working_model','_cons','normal95_upper',7.416916,.0000005),
      ('unit_sandwich_g_adjusted','lnwg','estimate',.1195474,.00000005),
      ('unit_sandwich_g_adjusted','lnwg','standard_error',.0515426,.00000005),
      ('unit_sandwich_g_adjusted','lnwg','z',2.32,.005),
      ('unit_sandwich_g_adjusted','lnwg','p_value',.020,.0005),
      ('unit_sandwich_g_adjusted','lnwg','p_value',.0204,.00005),
      ('unit_sandwich_g_adjusted','lnwg','normal95_lower',.0185258,.00000005),
      ('unit_sandwich_g_adjusted','lnwg','normal95_upper',.220569,.0000005),
      ('unit_sandwich_g_adjusted','lnwg','slope_wald_chi2_1',5.38,.005),
      ('unit_sandwich_g_adjusted','_cons','estimate',7.345479,.0000005),
      ('unit_sandwich_g_adjusted','_cons','standard_error',.1379494,.00000005),
      ('unit_sandwich_g_adjusted','_cons','z',53.25,.005),
      ('unit_sandwich_g_adjusted','_cons','normal95_lower',7.075103,.0000005),
      ('unit_sandwich_g_adjusted','_cons','normal95_upper',7.615855,.0000005),
    ]
    for method,parameter,key,target,half in facts:
      value=float(values[(method,parameter)][key])
      if abs(value-target)>half:
        raise AssertionError(('author printed',method,parameter,key,value,target,half))
    assert abs(float(fit[0]['scale'])-.0805511)<=.00000005
    print(f'P4 author output: {len(facts)+1} facts match printed precision')


if __name__=='__main__':
  main()

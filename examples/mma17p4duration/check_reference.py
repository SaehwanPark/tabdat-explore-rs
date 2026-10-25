#!/usr/bin/env python3
"""Independent full-design/sample/likelihood/sandwich and point-projection audit."""
import csv
import json
import math
from pathlib import Path
import sys


def rows(path):
  with open(path, newline="") as stream:
    return list(csv.DictReader(stream))


def close(a, b, relative_only=False):
  assert math.isfinite(a) and math.isfinite(b), (a, b)
  tolerance = 1e-300 + 1e-8*abs(b) if relative_only else 1e-9 + 1e-8*abs(b)
  assert abs(a-b) <= tolerance, (a, b, tolerance)


def inverse(matrix):
  n = len(matrix)
  a = [list(row)+[float(i==j) for j in range(n)] for i,row in enumerate(matrix)]
  for j in range(n):
    pivot = max(range(j,n),key=lambda i: abs(a[i][j]))
    assert abs(a[pivot][j]) > 1e-14, "singular independent information"
    a[j],a[pivot] = a[pivot],a[j]
    value = a[j][j]
    a[j] = [x/value for x in a[j]]
    for i in range(n):
      if i != j:
        factor = a[i][j]
        a[i] = [x-factor*y for x,y in zip(a[i],a[j])]
  return [row[n:] for row in a]


def multiply(a,b):
  return [[math.fsum(x*y for x,y in zip(row,column)) for column in zip(*b)] for row in a]


def audit(input_csv, native_json, native_predictions, reference_directory):
  ref = Path(reference_directory)
  original, predictions, expected = rows(input_csv), rows(native_predictions), rows(ref/"predictions.csv")
  hex_cells = rows(ref/"source_hex.csv")
  assert len(original)==len(predictions)==len(expected)==len(hex_cells)>41
  assert list(original[0])==list(hex_cells[0])
  assert list(predictions[0])==list(expected[0])
  for raw,pred,hexcells in zip(original,predictions,hex_cells):
    for name,value in hexcells.items():
      # Exact original promoted-DTA numbers, independent of R's decimal parser.
      assert float(raw[name])==float(pred[name])==float.fromhex(value), (name,raw[name],pred[name],value)
  events = [json.loads(line) for line in Path(native_json).read_text().splitlines()]
  assert all(e.get("error") is None for e in events)
  fits = [e["data"] for e in events if e["result_type"]=="PoissonResult"]
  assert len(fits)==1
  fit = fits[0]
  names = (ref/"parameter_names.txt").read_text().splitlines()
  assert fit["covariance_parameter_names"]==names and len(names)==41
  assert fit["predictors"]==names[1:]
  assert [c["name"] for c in fit["coefficients"]]==names
  assert fit["covariance"]=="robust_sandwich_n_over_n_minus_1"
  assert fit["reference_distribution"]=="asymptotic_standard_normal"
  assert fit["offset_variable"]=="log_spell" and fit["offset_coefficient"]==1
  sample=fit["sample"]
  assert sample["total_observations"]==len(original) and sample["dropped_observations"]==0
  assert sample["retained_indices"]==list(range(len(original)))
  assert sample["cluster_groups"] is None and sample["weights"] is None
  assert fit["zero_count"]==len(original)-sum(float(r["censor1"]) for r in original)
  assert fit["df_residual"]==len(original)-41
  assert fit["likelihood_ratio_p_value"] is None
  assert fit["likelihood_ratio_reference_distribution"] is None
  assert fit["converged"] and fit["confidence_level"]==95
  fit["duration_density_log_likelihood"] = fit["log_likelihood"]-math.fsum(float(r["censor1"])*math.log(float(r["spell"])) for r in original)
  gates = rows(ref/"reference.csv")
  assert len(gates)==1936
  for gate in gates:
    value = fit
    for part in gate["path"].strip("/").split("/"):
      value = value[int(part)] if isinstance(value,list) else value[part]
    close(float(value),float(gate["value"]),gate["path"].endswith("/p_value"))
  for actual,expected_row in zip(predictions,expected):
    for name,value in expected_row.items():
      positive = "survival" in name or "hazard" in name or name=="mean_net_duration"
      close(float(actual[name]),float(value),positive and float(value)>0)
  beta = [c["value"] for c in fit["coefficients"]]
  X=[];H=[];residual=[]
  point_fields = ["hazard","net_survival","mean_net_duration","ui0_hazard","ui1_hazard","ui_hazard_ratio",
    "net_survival_10","ui0_net_survival_10","ui1_net_survival_10","event_score_residual"]
  for row in predictions:
    r = {key:float(value) for key,value in row.items()}
    assert r["ui"]==r["original_ui"]
    assert r["rr_ui"]==r["reprate"]*r["ui"] and r["dr_ui"]==r["disrate"]*r["ui"]
    close(r["log_spell"],math.log(r["spell"]))
    x = [1.0]+[r[name] for name in names[1:]]
    h = math.exp(math.fsum(a*b for a,b in zip(x,beta)))
    projected=[]
    for ui in (0,1):
      candidate = dict(r,ui=ui,rr_ui=r["reprate"]*ui,dr_ui=r["disrate"]*ui)
      projected.append(math.exp(beta[0]+math.fsum(candidate[name]*b for name,b in zip(names[1:],beta[1:]))))
    h0,h1 = projected
    values = [h,math.exp(-h*r["spell"]),1/h,h0,h1,h1/h0,math.exp(-10*h),math.exp(-10*h0),math.exp(-10*h1),r["censor1"]-h*r["spell"]]
    for field,value in zip(point_fields,values): close(r[field],value,value>0 and field!="event_score_residual")
    for field,value in [("cumulative_hazard",h*r["spell"]),("log_cumulative_hazard",math.log(h*r["spell"])),
        ("ui0_cumulative_hazard",h0*r["spell"]),("ui1_cumulative_hazard",h1*r["spell"])]: close(r[field],value)
    X.append(x);H.append(h*r["spell"]);residual.append(r["censor1"]-h*r["spell"])
  # Standardized original-X score/information and full sandwich reconstruction.
  n,k=len(X),len(names)
  scales=[math.sqrt(math.fsum(row[j]**2 for row in X)) for j in range(k)]
  Z=[[x/s for x,s in zip(row,scales)] for row in X]
  score=[math.fsum(row[j]*r for row,r in zip(Z,residual)) for j in range(k)]
  assert max(map(abs,score))<1e-9, "independent source score certificate failed"
  info=[[math.fsum(h*row[i]*row[j] for row,h in zip(Z,H)) for j in range(k)] for i in range(k)]
  meat=[[math.fsum(r*r*row[i]*row[j] for row,r in zip(Z,residual)) for j in range(k)] for i in range(k)]
  bread=inverse(info)
  identity=multiply(info,bread)
  assert max(abs(value-float(i==j)) for i,row in enumerate(identity) for j,value in enumerate(row))<1e-8
  sandwich=multiply(multiply(bread,meat),bread)
  for i in range(k):
    for j in range(k): close(fit["covariance_matrix"][i][j],sandwich[i][j]*n/(n-1)/scales[i]/scales[j])
  # Independently reconstruct all normal inference and the full-rank Wald.
  for coefficient in fit["coefficients"]:
    i=names.index(coefficient["name"])
    se=math.sqrt(fit["covariance_matrix"][i][i]);z=beta[i]/se
    for field,value in [("standard_error",se),("statistic",z),
        ("confidence_lower",beta[i]-1.959963984540054*se),("confidence_upper",beta[i]+1.959963984540054*se)]: close(coefficient[field],value)
    close(coefficient["p_value"],math.erfc(abs(z)/math.sqrt(2)),True)
  slope_inverse=inverse([row[1:] for row in fit["covariance_matrix"][1:]])
  wald=math.fsum(beta[i+1]*slope_inverse[i][j]*beta[j+1] for i in range(40) for j in range(40))
  close(fit["omnibus_wald"]["statistic"],wald)
  # df40 is even: chi-square survival is an exact finite gamma sum.
  x=wald/2
  terms=[j*math.log(x)-math.lgamma(j+1) for j in range(20)]
  maximum=max(terms)
  p=math.exp(-x+maximum+math.log(math.fsum(math.exp(t-maximum) for t in terms)))
  close(fit["omnibus_wald"]["p_value"],p,True)
  density=math.fsum(float(row["censor1"])*math.log(h/float(row["spell"]))-h for row,h in zip(predictions,H))
  constant=math.fsum(float(row["censor1"])*math.log(float(row["spell"])) for row in predictions)
  null_rate=sum(float(row["censor1"]) for row in predictions)/sum(float(row["spell"]) for row in predictions)
  null_density=math.fsum(float(row["censor1"])*math.log(null_rate)-float(row["spell"])*null_rate for row in predictions)
  close(fit["duration_density_log_likelihood"],density)
  close(fit["log_likelihood"],density+constant)
  close(fit["null_log_likelihood"],null_density+constant)
  close(fit["likelihood_ratio"],2*(density-null_density))
  # Exact included intercept/UI score identities, not a specification test.
  for ui in (0,1):
    balance=math.fsum(r for row,r in zip(predictions,residual) if float(row["ui"])==ui)
    assert abs(balance)<1e-7
  return len(gates),len(predictions)*len(predictions[0])


if __name__=="__main__":
  assert len(sys.argv)==5, "usage: check_reference.py duration.csv native.json native_predictions.csv reference_directory"
  gates,cells=audit(*sys.argv[1:])
  print(f"Validated {gates} R gates/all1681 covariance entries, {cells} source/design/projection cells, exact source hex, full independent sandwich and score")

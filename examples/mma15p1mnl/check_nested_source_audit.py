"""Development-only original-source binding and independent likelihood/score audit."""
import csv
import hashlib
import itertools
import math
import pathlib
import re
import struct
import sys

work = pathlib.Path(sys.argv[1])
hashes = {
  "Nldata.asc": "1fb6f7d5abaf790d4df2c6a13a499ca42c0db4f950d941ba92565c30b4ab6c22",
  "mma15p2gev.do": "0dff4b93103da9d075967c1517f04fbe44a1b08543a34421e44f0cd717e50e27",
  "mma15p2gev.txt": "b1c2d7bf4008fae64ed659d046debd846ff59c6aa2115ea518ce8f0dc0037726",
  "mma15p4gev.lim": "56327afb5d709b26d88465850a273376ed94e51aa9280f1e997ebd1613cf146b",
  "mma15p4gev.out": "1a72c351216114d3c49f14ab6f1e45b02721a8278a20c5c47d2aadeb9b5e63de",
}
for name, expected in hashes.items():
  assert hashlib.sha256((work / name).read_bytes()).hexdigest() == expected, name

def read_csv(name):
  with (work / name).open() as source:
    return list(csv.DictReader(source))

tokens = [float(t) for t in (work / "Nldata.asc").read_text().split()]
assert len(tokens) == 1182*16
names = ["mode", "price", "crate", "dbeach", "dpier", "dprivate", "dcharter", "pbeach", "ppier", "pprivate", "pcharter", "qbeach", "qpier", "qprivate", "qcharter", "income"]
wide = read_csv("Nldata.csv")
assert len(wide) == 1182 and list(wide[0]) == ["source_row"] + names
for i, row in enumerate(wide):
  assert float(row["source_row"]) == i+1
  for j, name in enumerate(names):
    assert float(row[name]) == tokens[16*i+j], (i, name)

rows = read_csv("R_nested_source_audit.csv")
assert list(rows[0]) == ["section", "quantity", "i", "j", "value"]
values = {}
for row in rows:
  key = (row["section"], row["quantity"], int(row["i"]), int(row["j"]))
  value = float(row["value"])
  assert key not in values and math.isfinite(value), key
  values[key] = value
used = set()
def get(section, quantity, i=0, j=0):
  key = (section, quantity, i, j)
  used.add(key)
  return values[key]

def close(actual, expected, tolerance=1e-9):
  assert math.isfinite(actual) and math.isfinite(expected)
  assert abs(actual-expected) < tolerance, (actual, expected)

assert get("sample", "decisions") == 1182 and get("sample", "long_rows") == 4728
for i, count in enumerate([134, 178, 418, 452], 1):
  assert get("sample", "chosen_count", i) == count
for i in range(1, 5):
  assert 0 <= get("package_control", "ll", i) < 1e-8
  assert 0 <= get("package_control", "score", i) < 1e-6
  assert 0 <= get("package_control", "probability", i) < 1e-12

def fit(section, i, j=0):
  result = {q: get(section, q, i, j) for q in ["log_likelihood", "ks", "kb", "active_shore", "active_boat", "free_score", "free_step", "raw_lbfgs_code"]}
  assert result["ks"] >= 0 and result["kb"] >= 0
  assert result["active_shore"] in [0, 1] and result["active_boat"] == (result["kb"] == 0)
  assert 0 <= result["free_score"] < 1e-11 and 0 <= result["free_step"] < 1e-10
  # A raw line-search code52 is not called success: the independent polish
  # score/step/curvature/KKT gates are required by the R reference too.
  assert result["raw_lbfgs_code"] in [0, 52]
  return result

starts = [fit("multistart", i) for i in range(1, 12)]
boundary_ll = max(f["log_likelihood"] for f in starts)
for result in starts:
  assert result["ks"] == 0 and result["kb"] > 0 and result["active_shore"] == 1
  close(result["log_likelihood"], boundary_ll)
previous = math.inf
for i, ks in enumerate([0, .0001, .001, .005, .01, .02, .04, .1, .25, .5, 1, 2], 1):
  candidates = [fit("profile_candidate", i, j) for j in range(1, 4)]
  result = fit("profile", i)
  assert result["ks"] == ks and all(f["ks"] == ks for f in candidates)
  assert result["active_shore"] == 0  # Fixed profile role, not an estimated bound.
  close(result["log_likelihood"], max(f["log_likelihood"] for f in candidates))
  assert result["log_likelihood"] <= boundary_ll+1e-9 and result["log_likelihood"] < previous
  assert get("profile", "shore_score", i) < 0
  previous = result["log_likelihood"]

theta = [get("boundary", "theta", i) for i in range(1, 7)]
assert theta[4] == 0 and theta[5] > 0
assert 0 <= get("boundary", "ll_difference") < 1e-9
assert 0 <= get("boundary", "score_difference") < 1e-9
assert get("boundary", "minimum_free_information") > 0
assert 0 <= get("boundary", "free_step") < 1e-10
assert get("boundary", "shore_score") < -2
close(get("boundary", "boat_lambda"), 1/(.05*theta[5]))
for i, epsilon in enumerate([1e-3, 1e-4, 1e-5], 1):
  assert get("boundary_hessian", "step", i) == epsilon
  assert 0 <= get("boundary_hessian", "relative_error", i) < 2e-5

def sigmoid(x):
  e = math.exp(-abs(x))
  return 1/(1+e) if x >= 0 else e/(1+e)

def logcosh(x):
  a = abs(x)
  return math.log1p(2*math.sinh(x/2)**2) if a <= 20 else a+math.log1p(math.exp(-2*a))-math.log(2)

def f32(x):
  return struct.unpack("f", struct.pack("f", x))[0]

def data(single=False):
  for source in wide:
    row = {name: float(source[name]) for name in names}
    if single:
      row = {name: f32(value) for name, value in row.items()}
    p = [row["p"+mode] for mode in ["beach", "pier", "private", "charter"]]
    q = [row["q"+mode] for mode in ["beach", "pier", "private", "charter"]]
    y = row["income"]/1000
    yield int(row["mode"])-1, p, q, f32(y) if single else y

# Independent scalar likelihood and full score at the reciprocal-shore boundary.
bp, bq, a, b, ks, boat_scaled = theta
bp /= 100
kb = .05*boat_scaled
loglik = []
scores = [[] for _ in range(6)]
for chosen, p, q, y in data():
  shore = chosen < 2
  ds = bp*(p[0]-p[1])+bq*(q[0]-q[1])
  db = bp*(p[2]-p[3])+bq*(q[2]-q[3])
  dmp = (p[0]+p[1]-p[2]-p[3])/2
  dmq = (q[0]+q[1]-q[2]-q[3])/2
  x = kb*db/2
  eta = a+b*y+bp*dmp+bq*dmq-logcosh(x)/kb
  ps = sigmoid(eta)
  res = int(shore)-ps
  within = -math.log(2) if shore else (1 if chosen == 2 else -1)*x-logcosh(x)-math.log(2)
  loglik.append(int(shore)*eta-max(eta, 0)-math.log1p(math.exp(-abs(eta)))+within)
  tb = math.tanh(x)
  for k, (dm, delta) in enumerate([(dmp, p[2]-p[3]), (dmq, q[2]-q[3])]):
    score = (0 if shore else kb/2*((1 if chosen == 2 else -1)-tb)*delta)+res*(dm-tb*delta/2)
    scores[k].append(score/(100 if k == 0 else 1))
  scores[2].append(res)
  scores[3].append(res*y)
  scores[4].append((ds/2*(1 if chosen == 0 else -1) if shore else 0)+res*ds**2/8)
  derivative = (x*math.tanh(x)-logcosh(x))/kb**2
  scores[5].append(((0 if shore else db/2*((1 if chosen == 2 else -1)-tb))-res*derivative)*.05)
close(math.fsum(loglik), boundary_ll)
gradient = [math.fsum(s) for s in scores]
assert max(abs(gradient[k]) for k in [0, 1, 2, 3, 5])/1182 < 1e-11
close(gradient[4], get("boundary", "shore_score"))

# Parse the original published points, not transcriptions in the audit program.
number = r"[-+]?(?:\d*\.)?\d+(?:[Ee][-+]?\d+)?"
stata_text = (work / "mma15p2gev.txt").read_bytes().decode("latin1")
block = re.split(r"Log likelihood\s*=\s*-1180\.0156", stata_text, maxsplit=1)[1]
stata_strings = [re.search(r"^\s*"+re.escape(name)+r"\s*\|\s*("+number+r")", block, re.M).group(1) for name in ["p", "q", "dshore", "dshorey", "/shore", "/boat"]]
stata = [float(s) for s in stata_strings]
half_units = [0.5*10**(-len(s.split(".")[1])) for s in stata_strings]
limdep_text = (work / "mma15p4gev.out").read_bytes().decode("latin1")
block = limdep_text.split("FIML: Nested Multinomial Logit Model", 1)[1]
l = [float(re.search(r"^\s*"+name+r"\s+("+number+r")", block, re.M).group(1)) for name in ["P", "Q", "DSHORE", "DSHOREY", "SHORE", "BOAT"]]
limdep = l[:2] + [l[4]*l[2]+(l[4]-l[5])*l[3], l[5]*l[3], l[4], l[5]]
assert "create; dshorey = 1 - (d3y + d4y)" in (work / "mma15p4gev.lim").read_text()
for i in range(6):
  close(get("historical_stata", "parameter", i+1), stata[i], 1e-12)
  close(get("historical_limdep", "parameter", i+1), limdep[i], 1e-12)
old = read_csv("R_nested_quantities.csv")
stationary = [float(r["value"]) for r in old if r["model"] == "nested" and r["quantity"] == "parameter_value"]
assert len(stationary) == 6

def earlier(par, single):
  bp, bq, a, b, ls, lb = par
  ll, score = [], [[] for _ in range(6)]
  for chosen, p, q, y in data(single):
    utility = [bp*x+bq*z for x, z in zip(p, q)]
    within, logsums, means = [], [], []
    for k in [0, 2]:
      u, v = utility[k:k+2]
      weight = sigmoid(u-v)
      within.extend([weight, 1-weight])
      logsums.append(max(u, v)+math.log1p(math.exp(-abs(u-v))))
      means.append([weight*p[k]+(1-weight)*p[k+1], weight*q[k]+(1-weight)*q[k+1]])
    shore = int(chosen < 2)
    eta = a+b*y+ls*logsums[0]-lb*logsums[1]
    res = shore-sigmoid(eta)
    nest = 0 if shore else 1
    ll.append(utility[chosen]-logsums[nest]+shore*eta-max(eta, 0)-math.log1p(math.exp(-abs(eta))))
    for k, x in enumerate([p, q]):
      score[k].append(x[chosen]-means[nest][k]+res*(ls*means[0][k]-lb*means[1][k]))
    for k, value in enumerate([res, res*y, res*logsums[0], -res*logsums[1]], 2):
      score[k].append(value)
  return math.fsum(ll), max(abs(math.fsum(s)*d) for s, d in zip(score, [.01, 1, 1, 1, 30, 15]))/1182

signs = list(itertools.product([-1, 1], repeat=6))
# R expand.grid varies its first column fastest; match it explicitly.
signs = [tuple(reversed(s)) for s in signs]
for single in [False, True]:
  j = int(single)
  for i, par in enumerate([stationary, stata, limdep], 1):
    ll, score = earlier(par, single)
    close(get("historical_point", "log_likelihood", i, j), ll)
    close(get("historical_point", "scaled_score", i, j), score, 1e-10)
    assert get("historical_point", "scaled_step", i, j) >= 0
    assert get("historical_point", "minimum_information", i, j) > 0
    if i == 1 and not single:
      assert score < 1e-10 and get("historical_point", "scaled_step", i, j) < 1e-10
    if i == 2:
      assert round(ll, 4) == -1180.0156 and score > 1e-8
      assert get("historical_point", "scaled_step", i, j) > 1e-3
    if i == 3 and not single:
      close(ll, get("historical_point", "log_likelihood", 1, 0), 1e-8)
  for i, direction in enumerate(signs, 1):
    par = [p+s*h for p, s, h in zip(stata, direction, half_units)]
    ll, score = earlier(par, single)
    close(get("historical_rounding_vertex", "log_likelihood", i, j), ll)
    close(get("historical_rounding_vertex", "scaled_score", i, j), score, 1e-10)
    assert get("historical_rounding_vertex", "scaled_step", i, j) > 1e-3
    assert get("historical_rounding_vertex", "minimum_information", i, j) > 0
assert used == values.keys() and len(values) == 1068
print("Validated1068 diagnostics: source-bound1182 decisions, preferred boundary/12 profiles/11 starts, independent scores,128 historical rounding vertices and source parameter mapping. No finite unrestricted MLE, global-maximum proof or ordinary boundary inference is claimed.")

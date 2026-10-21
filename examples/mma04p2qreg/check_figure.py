"""Offline development audit of Figure4.2 SVG against independent R predictions."""
import csv
import math
import re
import sys
import xml.etree.ElementTree as ET

reference, svg = sys.argv[1:]
with open(reference, newline='') as f:
  rows = list(csv.DictReader(f))
n = len(rows)
assert n in (120, 5006)
ns = '{http://www.w3.org/2000/svg}'
root = ET.parse(svg).getroot()
assert root.attrib['data-candidate-rows'] == str(n)
assert root.attrib['width'] == '900' and root.attrib['height'] == '660'
assert float(root.attrib['data-jitter-y']) == 0
assert root.attrib['data-coordinate-precision'] == 'float64'
def close(a, b, label):
  assert math.isfinite(float(a)) and abs(float(a)-b) <= 1e-8, (label, a, b)
x = [float(r['lntotal']) for r in rows]
y = [float(r['lnmed']) for r in rows]
variables = ['q90', 'q50', 'q10']
values = y + [float(r[v]) for r in rows for v in variables]
lo_x, hi_x, lo_y, hi_y = min(x), max(x), min(values), max(values)
dx, dy = hi_x-lo_x, hi_y-lo_y
lo_x -= .05*dx; hi_x += .05*dx; lo_y -= .05*dy; hi_y += .05*dy
for name, value in [('x-min',lo_x),('x-max',hi_x),('y-min',lo_y),('y-max',hi_y)]:
  close(root.attrib['data-'+name],value,name)
def position(x,y):
  return 90+730*(x-lo_x)/(hi_x-lo_x),75+400*(hi_y-y)/(hi_y-lo_y)
assert root.find(ns+'title').text == 'Regression Lines as Quantile Varies'
texts = [e.text for e in root.iter(ns+'text')]
for label in ['Log total household expenditure', 'Log household medical expenditure', '90th percentile', 'Median', '10th percentile']:
  assert label in texts
points = [e for e in root.iter(ns+'circle') if e.attrib.get('class') == 'scatter-point']
assert len(points) == n
for i, point in enumerate(points):
  a = point.attrib
  assert int(a['data-row-index']) == i
  assert a['data-x-variable'] == 'lntotal' and a['data-y-variable'] == 'lnmed'
  for key, value in [('data-x',x[i]),('data-y',y[i]),('data-display-y',y[i])]: close(a[key],value,key)
  px,py = position(x[i],y[i]);close(a['cx'],px,'scatter x');close(a['cy'],py,'scatter y')
lines = [e for e in root.iter(ns+'path') if e.attrib.get('class') == 'line-series']
assert [e.attrib['data-variable'] for e in lines] == variables
assert [e.attrib['stroke-dasharray'] for e in lines] == ['none','8 4','3 3']
order = sorted(range(n),key=lambda i:(x[i],i))
for line, v in zip(lines,variables):
  a = line.attrib
  assert list(map(int,a['data-row-indices'].split(','))) == order
  xs = list(map(float,a['data-x-values'].split(',')))
  ys = list(map(float,a['data-y-values'].split(',')))
  tokens = re.findall(r'([ML])([^ML\s]+)\s+([^ML\s]+)',a['d'])
  assert len(xs) == len(ys) == len(tokens) == n
  for j,(i,(mode,px,py)) in enumerate(zip(order,tokens)):
    assert mode == ('M' if j == 0 else 'L')
    value = float(rows[i][v]);close(xs[j],x[i],v);close(ys[j],value,v)
    ex,ey = position(x[i],value);close(px,ex,'line x');close(py,ey,'line y')
print(f'PASS:{n} original-order scatter marks,{3*n} independent-R line vertices, stable x/row ordering, axes/legends and actual SVG geometry (1e-8 absolute)')

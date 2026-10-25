#!/usr/bin/env python3
"""Pin author assets, executable input (not prose), and both printed sts tables."""
import hashlib
from pathlib import Path
import re
import sys
from check_censored import check, close, rows

assert len(sys.argv)==5, "usage: check_source.py author.do author.txt output_directory reference.csv"
program, log, output, reference = map(Path, sys.argv[1:])
for path, digest in [(program,"947e32d5c91bcc590be975be284fa4b97200a419b12a3e2c63cab520dd86cf90"),
    (log,"bae7c6334c103c25a69a66f35cc99633c4e2f15aaedceb011be4755348853f80")]:
  assert hashlib.sha256(path.read_bytes()).hexdigest()==digest, f"Unexpected source bytes: {path}"
text = program.read_text()
block = text.split("input duration failed\n",1)[1].split("\nend",1)[0]
original = [line.split() for line in block.splitlines() if line.strip()]
converted = rows(output / "censored.csv")
assert len(original)==len(converted)==80
for i,(a,b) in enumerate(zip(original,converted),1):
  assert float(b["source_row"])==i
  assert [float(b[k]) for k in ["duration","failed"]]==list(map(float,a))
assert sum(float(a[0]) for a in original)==3170
assert sum(float(a[1]) for a in original)==40
checks,vertices = check(output,output/"censored.csv",reference)
printed=[]
for line in log.read_text().splitlines():
  fields=line.split()
  if len(fields)==8 and all(re.fullmatch(r"\d+",f) for f in fields[:4]):
    if all(re.fullmatch(r"\d+\.\d+|\.",f) for f in fields[4:]):
      printed.append(fields)
assert len(printed)==18
native=rows(output/"censored_survival.csv")
assert len(native)==9
facts=0
for table,variables in [(printed[:9],["survival","survival_se","survival_lower","survival_upper"]),
    (printed[9:],["cumhaz","cumhaz_se","cumhaz_lower","cumhaz_upper"])]:
  for a,b in zip(native,table):
    assert [int(float(a[k])) for k in ["time","at_risk","events","censored"]]==list(map(int,b[:4]))
    for name,value in zip(variables,b[4:]):
      if value==".":
        assert a[name]==""
      else:
        assert abs(float(a[name])-float(value))<=.000050000001,(name,a[name],value)
    facts+=8
# The program's explicit first-three-time arithmetic independently fixes d/n.
bytime={float(r["time"]):r for r in native}
for t,h,s in [(10,6/80,1-6/80),(20,6/80+5/70,(1-6/80)*(1-5/70)),
    (30,6/80+5/70+2/62,(1-6/80)*(1-5/70)*(1-2/62))]:
  close(float(bytime[t]["cumhaz"]),h)
  close(float(bytime[t]["survival"]),s)
print(f"Pinned80-spell/40-failure source audit: {checks} R cells, {facts} printed facts, six manual gates, {vertices} SVG vertices")

#!/usr/bin/env python3
"""Finite interleaved T10 controls; boundary means fit exactly, not within scores."""
import argparse
from pathlib import Path


def generate(mode, output):
  output.mkdir()
  raw, csv = [], ["source_row,lnhr,lnwg,kids,ageh,agesq,disab,id,year"]
  for t in reversed(range(10)):
    for g in range(1, 17):
      z = t-4.5
      wx = z/4
      x = 2+g/8+wx
      effect = ((g % 4)-1.5)/2 if mode == "interior" else 0
      # Coarse dyadics avoid the reproduced R decimal-parser ULP drift;
      # the independent raw-cell audit must remain exact, not tolerance-based.
      y = 7+x/4+effect+(g-8.5)/16*wx+(z*z-8.25)/16
      values = [y, x, 0, 30, 900, 0, g, 1979+t]
      fields = [str(v) for v in values]
      raw.append(" ".join(fields))
      csv.append(",".join([str(len(raw))]+fields))
  (output/"MOM.dat").write_text("\n".join(raw)+"\n")
  (output/"MOM.csv").write_text("\n".join(csv)+"\n")


if __name__ == "__main__":
  parser = argparse.ArgumentParser(description=__doc__)
  parser.add_argument("mode", choices=["interior", "boundary"])
  parser.add_argument("output", type=Path)
  args = parser.parse_args()
  generate(args.mode, args.output)

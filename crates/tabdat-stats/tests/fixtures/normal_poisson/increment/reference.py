"""Development-only closed-form oracles at actual IEEE binary64 inputs.

Decimal uses 400 digits so subtracting the first-order term at x=1e-150 still
leaves ~100 reliable digits in its order-1e-300 remainder. No Rust output inputs.
"""
import csv
from decimal import Decimal, localcontext
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def binary(value):
  return Decimal.from_float(float(value))


def main():
  common = [0, 1e-150, -1e-150, 1e-12, -1e-12, 1e-6, -1e-6,
            .01, -.01, .124, -.124, .125, -.125, .49, -.49, .5, -.5]
  with localcontext() as context:
    context.prec = 400
    with (ROOT / "remainders.csv").open("w", newline="") as output:
      writer = csv.writer(output)
      writer.writerow(["kind", "x", "expected"])
      for kind, values in [("exp", common + [1, -1, 40, -40, 700]),
                           ("log", common + [1, 40, 700, -.99, -.999999])]:
        for value in values:
          x = binary(value)
          expected = x.exp() - 1 - x if kind == "exp" else (1 + x).ln() - x
          writer.writerow([kind, repr(float(value)), repr(float(expected))])
    cases = [(0, 0, 0), (1e-12, 0, 0), (1e-12, 0, -1e-24),
             (0, 1e-12, -1e-12), (0, -1e-12, 1e-12),
             (1e-12, -2e-13, 2e-13), (1e-12, 0, -.5e-24),
             (0, 0, 1e-12), (0, 0, -1e-12),
             (.025, .005, -.002), (-.025, -.005, .002), (.03, .01, -.011)]
    with (ROOT / "gaussian.csv").open("w", newline="") as output:
      writer = csv.writer(output)
      writer.writerow(["a", "b", "c", "log_ratio"])
      for aa, bb, cc in cases:
        a, b, c = map(binary, (aa, bb, cc))
        denominator = 1 - 2*b
        expected = c - denominator.ln()/2 + a*a/(2*denominator)
        writer.writerow([repr(float(v)) for v in (aa, bb, cc)] + [repr(float(expected))])


if __name__ == "__main__":
  main()

"""External pinned TabDat binary-MLE probe; Python is not a Rust dependency."""
import csv
import sys
import tempfile
from pathlib import Path
import numpy as np
import pandas as pd
from tabdat.executor import Executor
from tabdat.models import UseCommand
from tabdat.parser import parse_command

root = Path(__file__).parent
if sys.argv[1:] == ["--generate-data"]:
  rng = np.random.default_rng(4276)
  x1, x2 = rng.normal(size=(2, 200))
  pr = 1 / (1 + np.exp(-(-0.4 + 0.7 * x1 - 0.3 * x2)))
  y = rng.binomial(1, pr)
  with (root / "binary.csv").open("w", newline="") as f:
    writer = csv.writer(f, lineterminator="\n")
    writer.writerow(["y", "x1", "x2"])
    writer.writerows(zip(y, x1, x2, strict=True))

with tempfile.TemporaryDirectory() as d:
  path = Path(d) / "input.parquet"
  pd.read_csv(root / "binary.csv").to_parquet(path, index=False)
  executor = Executor()
  try:
    executor.execute(UseCommand(path=path))
    for mode in ["logit", "probit"]:
      result = executor.execute(parse_command(mode + " y x1 x2"))
      model = executor.state.binary_regression.fitted_model
      values = [c.value for c in result.coefficients]
      values += [c.standard_error for c in result.coefficients]
      values += list(np.asarray(model.cov_params()).reshape(-1))
      values += [model.llf]
      print("python," + mode + "," + ",".join(format(float(v), ".17g") for v in values))
      print(f"{mode} n={model.nobs} names={[c.name for c in result.coefficients]} converged={model.mle_retvals['converged']}", file=sys.stderr)
  finally:
    executor.close()

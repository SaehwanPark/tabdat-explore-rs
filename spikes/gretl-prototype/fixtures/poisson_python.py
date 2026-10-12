"""External pinned TabDat Poisson probe; not a Rust runtime dependency."""
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
  rng = np.random.default_rng(7193)
  x1, x2 = rng.normal(size=(2, 240))
  y = rng.poisson(np.exp(0.3 + 0.4*x1 - 0.25*x2))
  with (root / "poisson.csv").open("w", newline="") as f:
    writer = csv.writer(f, lineterminator="\n")
    writer.writerow(["y", "x1", "x2"])
    writer.writerows(zip(y, x1, x2, strict=True))

with tempfile.TemporaryDirectory() as d:
  path = Path(d) / "input.parquet"
  pd.read_csv(root / "poisson.csv").to_parquet(path, index=False)
  executor = Executor()
  try:
    executor.execute(UseCommand(path=path))
    result = executor.execute(parse_command("poisson y x1 x2"))
    model = executor.state.poisson_regression.fitted_model
    assert model.mle_retvals["converged"]
    assert model.nobs == 240
    assert [c.name for c in result.coefficients] == ["intercept", "x1", "x2"]
    # Input is complete and consumed in exact source order; verify stored arrays.
    original = pd.read_csv(root / "poisson.csv")
    np.testing.assert_array_equal(model.model.endog, original.y.to_numpy())
    np.testing.assert_array_equal(model.model.exog[:, 1:], original[["x1", "x2"]].to_numpy())
    values = [c.value for c in result.coefficients]
    values += [c.standard_error for c in result.coefficients]
    values += list(np.asarray(model.cov_params()).reshape(-1))
    values += [model.llf]
    print("python,poisson," + ",".join(format(float(v), ".17g") for v in values))
    print(f"n={model.nobs} names={[c.name for c in result.coefficients]} converged=True", file=sys.stderr)
  finally:
    executor.close()

"""External pinned TabDat probe, not a runtime dependency.
Run with PYTHONPATH pointing at the verified clean oracle src directory.
"""
import csv
import sys
import tempfile
from pathlib import Path
import pandas as pd
from tabdat.executor import Executor
from tabdat.models import UseCommand
from tabdat.parser import parse_command

root = Path(__file__).parent
if sys.argv[1:] == ["--generate-data"]:
  with (root / "covariance.csv").open("w", newline="") as f:
    writer = csv.writer(f)
    writer.writerow(["y", "x1", "x2", "cid"])
    for i in range(40):
      x1 = i % 7 - 3
      x2 = (i * i % 11) / 3
      y = 1 + 2 * x1 - 0.5 * x2 + (i % 5 - 2) * (0.2 + abs(x1) * 0.1) + (i // 4 % 3) * 0.3
      writer.writerow([y, x1, x2, i // 4])

with tempfile.TemporaryDirectory() as d:
  path = Path(d) / "input.parquet"
  pd.read_csv(root / "covariance.csv").to_parquet(path, index=False)
  executor = Executor()
  try:
    executor.execute(UseCommand(path=path))
    for mode, option in [("classical", ""), ("hc1", ", robust"), ("cluster", ", cluster(cid)")]:
      result = executor.execute(parse_command("regress y x1 x2" + option))
      model = executor.state.regression.fitted_model
      values = [c.value for c in result.coefficients]
      values += [c.standard_error for c in result.coefficients]
      values += list(model.cov_params().reshape(-1))
      print("python," + mode + "," + ",".join(format(float(v), ".17g") for v in values))
      print(f"mode={mode} n={result.observation_count} names={[c.name for c in result.coefficients]} df_resid={model.df_resid} df_inference={getattr(model, 'df_resid_inference', model.df_resid)}", file=sys.stderr)
  finally:
    executor.close()

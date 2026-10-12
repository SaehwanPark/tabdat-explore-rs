"""Pinned TabDat NB2 oracle; raw alpha is incorrectly named lnalpha by executor."""
import csv, sys, tempfile
from pathlib import Path
import numpy as np
import pandas as pd
from tabdat.executor import Executor
from tabdat.models import UseCommand
from tabdat.parser import parse_command
root = Path(__file__).parent
if sys.argv[1:] == ["--generate-data"]:
  rng = np.random.default_rng(1267)
  x1, x2 = rng.normal(size=(2, 320))
  mu = np.exp(0.3 + 0.4*x1 - 0.25*x2)
  y = rng.negative_binomial(1/0.65, 1/(1+0.65*mu))
  with (root / "nb2.csv").open("w", newline="") as f:
    w = csv.writer(f, lineterminator="\n"); w.writerow(["y", "x1", "x2"])
    w.writerows(zip(y, x1, x2, strict=True))
with tempfile.TemporaryDirectory() as d:
  data = pd.read_csv(root / "nb2.csv")
  path = Path(d) / "input.parquet"; data.to_parquet(path, index=False)
  executor = Executor()
  try:
    executor.execute(UseCommand(path=path))
    result = executor.execute(parse_command("nbreg y x1 x2"))
    model = executor.state.nbreg_regression.fitted_model
    assert model.mle_retvals["converged"] and model.nobs == 320
    np.testing.assert_array_equal(model.model.endog, data.y.to_numpy())
    np.testing.assert_array_equal(model.model.exog[:, 1:], data[["x1", "x2"]].to_numpy())
    assert result.coefficients[-1].name == "lnalpha"
    assert result.coefficients[-1].value == model.params[-1]
    assert np.isclose(model.lnalpha, np.log(model.params[-1]))
    values = list(model.params) + list(model.bse) + list(np.asarray(model.cov_params()).reshape(-1)) + [model.llf]
    original = "python-default,nb2," + ",".join(format(float(v), ".17g") for v in values) + "\n"
    (root / "nb2-python-default.csv").write_text(original)
    print(f"names={[c.name for c in result.coefficients]} raw_alpha={model.params[-1]} lnalpha={model.lnalpha} score_max={np.max(np.abs(model.model.score(model.params)))}", file=sys.stderr)
    # Preserve the executor result above; refine only external validation tooling.
    refined = model.model.fit(method="newton", start_params=model.params, tol=1e-12, maxiter=100, disp=0)
    assert refined.mle_retvals["converged"]
    assert np.max(np.abs(refined.model.score(refined.params))) < 1e-7
    values = list(refined.params) + list(refined.bse) + list(np.asarray(refined.cov_params()).reshape(-1)) + [refined.llf]
    print("python-refined,nb2," + ",".join(format(float(v), ".17g") for v in values))
  finally:
    executor.close()

"""Reproduce the pinned TabDat and statsmodels/SciPy reference table."""

from __future__ import annotations

import hashlib
import importlib.metadata
import json
import platform
import tempfile
from datetime import datetime, timezone
from pathlib import Path

import duckdb
import numpy as np
import pandas as pd
import scipy
import statsmodels.api as sm
from scipy.stats import f as f_distribution

from tabdat.executor import Executor
from tabdat.models import (
  BinaryExpression,
  IdentifierExpression,
  NumberExpression,
  RegressCommand,
  TestCommand,
  UseCommand,
)


ROWS = [
  (1.0, 10.0, 5.0, 1.0, 1.0, "a"),
  (2.0, 12.0, 6.0, 2.0, 2.0, "a"),
  (3.0, 15.0, 8.0, 3.0, 1.5, "b"),
  (4.0, 18.0, 9.0, 1.0, 1.0, "c"),
  (5.0, 20.0, 11.0, 4.0, 3.0, "c"),
  (6.0, 25.0, 12.0, 2.0, 2.0, "d"),
]


def write_fixture(path: Path, outcome_scale: float = 1.0) -> None:
  with duckdb.connect(":memory:") as connection:
    connection.execute(
      "create table fixture (x1 double, x2 double, y double, w double, sigma double, grp varchar)"
    )
    connection.executemany(
      "insert into fixture values (?, ?, ?, ?, ?, ?)",
      [(x1, x2, y * outcome_scale, w, sigma, group) for x1, x2, y, w, sigma, group in ROWS],
    )
    connection.execute("copy fixture to ? (format parquet)", [str(path)])


def difference(left: IdentifierExpression, right: IdentifierExpression | NumberExpression):
  return BinaryExpression(left, "-", right)


CASES = [
  (
    "ols_joint",
    RegressCommand(outcome="y", predictors=("x1", "x2")),
    (IdentifierExpression("x1"), IdentifierExpression("x2")),
    np.array([[0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]),
    np.array([0.0, 0.0]),
  ),
  (
    "ols_equality",
    RegressCommand(outcome="y", predictors=("x1", "x2")),
    (difference(IdentifierExpression("x1"), IdentifierExpression("x2")),),
    np.array([[0.0, 1.0, -1.0]]),
    np.array([0.0]),
  ),
  (
    "ols_multiple",
    RegressCommand(outcome="y", predictors=("x1", "x2")),
    (
      difference(IdentifierExpression("x1"), IdentifierExpression("x2")),
      difference(IdentifierExpression("x2"), NumberExpression(2)),
    ),
    np.array([[0.0, 1.0, -1.0], [0.0, 0.0, 1.0]]),
    np.array([0.0, 2.0]),
  ),
  (
    "ols_intercept",
    RegressCommand(outcome="y", predictors=("x1", "x2")),
    (IdentifierExpression("intercept"),),
    np.array([[1.0, 0.0, 0.0]]),
    np.array([0.0]),
  ),
  (
    "wls_equality",
    RegressCommand(outcome="y", predictors=("x1", "x2"), estimator="wls", weight_variable="w"),
    (difference(IdentifierExpression("x1"), IdentifierExpression("x2")),),
    np.array([[0.0, 1.0, -1.0]]),
    np.array([0.0]),
  ),
  (
    "gls_equality",
    RegressCommand(
      outcome="y", predictors=("x1", "x2"), estimator="gls", weight_variable="sigma"
    ),
    (difference(IdentifierExpression("x1"), IdentifierExpression("x2")),),
    np.array([[0.0, 1.0, -1.0]]),
    np.array([0.0]),
  ),
  (
    "hc1_equality",
    RegressCommand(outcome="y", predictors=("x1", "x2"), robust=True),
    (difference(IdentifierExpression("x1"), IdentifierExpression("x2")),),
    np.array([[0.0, 1.0, -1.0]]),
    np.array([0.0]),
  ),
  (
    "cluster_equality",
    RegressCommand(outcome="y", predictors=("x1", "x2"), cluster_variable="grp"),
    (difference(IdentifierExpression("x1"), IdentifierExpression("x2")),),
    np.array([[0.0, 1.0, -1.0]]),
    np.array([0.0]),
  ),
  (
    "no_intercept_equality",
    RegressCommand(outcome="y", predictors=("x1", "x2"), include_intercept=False),
    (difference(IdentifierExpression("x1"), IdentifierExpression("x2")),),
    np.array([[1.0, -1.0]]),
    np.array([0.0]),
  ),
]


def result_fields(result) -> dict[str, object]:
  return {
    "constraints": list(result.constraints),
    "statistic": float(result.statistic),
    "p_value": float(result.p_value),
    "df": int(result.df),
    "df_residual": result.df_residual,
    "is_chi2": bool(result.is_chi2),
  }


def direct_reference(frame: pd.DataFrame, command: RegressCommand, matrix, rhs):
  design = frame[["x1", "x2"]].to_numpy()
  if command.include_intercept:
    design = sm.add_constant(design, has_constant="add")
  outcome = frame["y"].to_numpy()
  if command.estimator == "wls":
    fitted = sm.WLS(outcome, design, weights=frame["w"].to_numpy()).fit()
  elif command.estimator == "gls":
    fitted = sm.GLS(outcome, design, sigma=frame["sigma"].to_numpy()).fit()
  else:
    fitted = sm.OLS(outcome, design).fit()
  if command.robust:
    fitted = fitted.get_robustcov_results(cov_type="HC1")
  if command.cluster_variable:
    fitted = fitted.get_robustcov_results(cov_type="cluster", groups=frame["grp"].to_numpy())
  test = fitted.f_test((matrix, rhs))
  statistic = float(test.fvalue)
  df_num = int(matrix.shape[0])
  df_denom = int(fitted.df_resid)
  return {
    "statistic": statistic,
    "p_value_statsmodels": float(test.pvalue),
    "p_value_scipy": float(f_distribution.sf(statistic, df_num, df_denom)),
    "df": df_num,
    "df_residual": df_denom,
  }


def main() -> None:
  output: dict[str, object] = {
    "environment": {
      "python": platform.python_version(),
      "platform": platform.platform(),
      "run_date_utc": datetime.now(timezone.utc).isoformat(),
      "tabdat_revision": "16b45d9b66b0d80f32d4d220e84d81bc5180bdbe",
      "uv_lock_sha256": hashlib.sha256(Path("uv.lock").read_bytes()).hexdigest(),
      "statsmodels": importlib.metadata.version("statsmodels"),
      "scipy": scipy.__version__,
    },
    "python_executor": {},
    "statsmodels_scipy": {},
  }
  with tempfile.TemporaryDirectory() as directory:
    fixture_path = Path(directory) / "fixture.parquet"
    write_fixture(fixture_path)
    frame = pd.DataFrame(ROWS, columns=("x1", "x2", "y", "w", "sigma", "grp"))
    executor = Executor()
    try:
      executor.execute(UseCommand(fixture_path))
      for name, command, constraints, matrix, rhs in CASES:
        executor.execute(command)
        python_result = executor.execute(TestCommand(constraints))
        output["python_executor"][name] = result_fields(python_result)
        output["statsmodels_scipy"][name] = direct_reference(frame, command, matrix, rhs)

      scaled_path = Path(directory) / "scaled_fixture.parquet"
      write_fixture(scaled_path, outcome_scale=1e-8)
      scaled_frame = frame.copy()
      scaled_frame["y"] *= 1e-8
      scaled_command = RegressCommand(
        outcome="y", predictors=("x1", "x2"), cluster_variable="grp"
      )
      executor.execute(UseCommand(scaled_path))
      executor.execute(scaled_command)
      scaled_result = executor.execute(
        TestCommand((difference(IdentifierExpression("x1"), IdentifierExpression("x2")),))
      )
      output["python_executor"]["cluster_equality_small_outcome"] = result_fields(scaled_result)
      output["statsmodels_scipy"]["cluster_equality_small_outcome"] = direct_reference(
        scaled_frame,
        scaled_command,
        np.array([[0.0, 1.0, -1.0]]),
        np.array([0.0]),
      )

      executor.execute(RegressCommand(outcome="y", predictors=("x1", "x2")))
      try:
        executor.execute(RegressCommand(outcome="missing", predictors=("x1",)))
      except Exception:
        pass
      try:
        executor.execute(TestCommand((IdentifierExpression("x1"),)))
      except Exception as error:
        output["test_after_failed_regress_error"] = str(error)
      else:
        raise AssertionError("test unexpectedly succeeded after failed regression")
    finally:
      executor.close()

  print(json.dumps(output, indent=2, sort_keys=True))


if __name__ == "__main__":
  main()

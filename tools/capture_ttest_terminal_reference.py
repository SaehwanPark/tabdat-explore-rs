"""Generate the independent t-test display fixture (development only).

Run with NumPy/SciPy installed:
  PYTHONDONTWRITEBYTECODE=1 <python> tools/capture_ttest_terminal_reference.py
"""
import json
from pathlib import Path
import platform

import numpy as np
import scipy
from scipy import stats

x = np.array([1., 2., 3., 4., 5., 6., np.nan, 8.])
y = np.array([10., 12., 15., 18., 20., 25., 9., np.nan])
groups = np.array(["group1", "group1", "group2", "group2", "group1", "group2", "group1", None])


def summary(name, values):
    mean = float(np.mean(values))
    se = float(stats.sem(values))
    interval = stats.t.interval(.95, len(values) - 1, loc=mean, scale=se)
    return {"name": name, "n": len(values), "values": [mean, se, float(np.std(values, ddof=1)), *map(float, interval)]}


def case(mode, sample_indices, rows, test):
    t, df = float(test.statistic), float(test.df)
    return {"mode": mode, "retained_indices": sample_indices, "rows": rows,
            "t": t, "df": df, "probabilities": [float(stats.t.cdf(t, df)), float(test.pvalue), float(stats.t.sf(t, df))]}


one_indices = np.flatnonzero(np.isfinite(x)).tolist()
one_values = x[one_indices]
one_summary = summary("x1", one_values)
# The owned one-sample result has both a variable summary and tested-mean row.
one_mean = {**one_summary, "name": "mean"}
one = case("one-sample", one_indices, [one_summary, one_mean], stats.ttest_1samp(one_values, 3.))
paired_indices = np.flatnonzero(np.isfinite(x) & np.isfinite(y)).tolist()
paired = case("paired", paired_indices,
              [summary("x1", x[paired_indices]), summary("x2", y[paired_indices]), summary("diff", x[paired_indices] - y[paired_indices])],
              stats.ttest_rel(x[paired_indices], y[paired_indices]))
first_indices = np.flatnonzero(np.isfinite(x) & (groups == "group1")).tolist()
second_indices = np.flatnonzero(np.isfinite(x) & (groups == "group2")).tolist()
first, second = x[first_indices], x[second_indices]
independent = {}
for name, equal_var in [("pooled equal-variance", True), ("Welch unequal-variance", False)]:
    test = stats.ttest_ind(first, second, equal_var=equal_var)
    interval = test.confidence_interval(.95)
    # Public t-test output supplies the interval/DF; SE is the classical formula.
    if equal_var:
        variance = ((len(first) - 1) * np.var(first, ddof=1) + (len(second) - 1) * np.var(second, ddof=1)) / (len(first) + len(second) - 2)
        se = np.sqrt(variance * (1 / len(first) + 1 / len(second)))
    else:
        se = np.sqrt(np.var(first, ddof=1) / len(first) + np.var(second, ddof=1) / len(second))
    diff = {"name": "diff", "n": None, "values": [float(np.mean(first) - np.mean(second)), float(se), None, float(interval.low), float(interval.high)]}
    independent[name] = case(name, [first_indices, second_indices],
                             [summary("group1", first), summary("group2", second), summary("combined", np.concatenate([first, second])), diff], test)
reference = {
    "provenance": {"python": platform.python_version(), "numpy": np.__version__, "scipy": scipy.__version__,
                   "method": "SciPy ttest_1samp/ttest_rel/ttest_ind and Student-t 95% intervals",
                   "sample": "one: finite x; paired: joint finite x/y; independent: finite x and nonmissing group, lexical group1-group2",
                   "tolerance": "max(1e-10 absolute, 1e-9 * abs(expected)); no display rounding"},
    "cases": [one, paired, independent["pooled equal-variance"], independent["Welch unequal-variance"]],
}
path = Path(__file__).resolve().parents[1] / "tests/fixtures/ttest-terminal-reference.json"
path.write_text(json.dumps(reference, indent=2, allow_nan=False) + "\n")
print(path)

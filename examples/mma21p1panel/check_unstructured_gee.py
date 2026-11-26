#!/usr/bin/env python3
"""Compare the native unstructured GEE run with its independent Base-R fit."""
import csv
import hashlib
import json
import math
import sys
from pathlib import Path


def read_csv(path):
    with path.open(newline="") as handle:
        return list(csv.DictReader(handle))


def close(actual, expected, label, *, relative=1e-10, absolute=1e-9):
    actual = float(actual)
    expected = float(expected)
    tolerance = absolute + relative * abs(expected)
    if not math.isfinite(actual) or not math.isfinite(expected) or abs(actual - expected) > tolerance:
        raise SystemExit(f"{label}: {actual:.17g} != {expected:.17g}; tolerance {tolerance:.3g}")


def main():
    if len(sys.argv) != 5:
        raise SystemExit("usage: check_unstructured_gee.py output.jsonl unstructured_gee_predictions.csv reference_directory MOM.dat")
    jsonl, native_predictions, reference_dir, source_path = map(Path, sys.argv[1:])
    events = [json.loads(line) for line in jsonl.read_text().splitlines() if line.strip()]
    fits = [event["data"] for event in events if event.get("result_type") == "UnstructuredGaussianGeeResult"]
    if len(fits) != 1:
        raise SystemExit(f"expected one unstructured fit result, found {len(fits)}")
    fit = fits[0]
    if fit.get("estimator") != "gaussian_identity_unstructured_gee":
        raise SystemExit(f"unexpected estimator identity: {fit.get('estimator')}")
    if fit.get("correlation") != "balanced_unstructured_period_standardized_residual_moments":
        raise SystemExit(f"unexpected working-correlation method: {fit.get('correlation')}")
    reference_fit = read_csv(reference_dir / "fit.csv")[0]
    for native_key, reference_key in [
        ("observation_count", "observations"),
        ("group_count", "entities"),
        ("period_count", "periods"),
        ("phi", "scale"),
        ("iterations", "iterations"),
        ("max_abs_scaled_score", "max_abs_scaled_score"),
    ]:
        close(fit[native_key], reference_fit[reference_key], f"fit.{native_key}")
    close(fit["working"]["coefficients"][0]["value"], reference_fit["intercept"], "intercept")
    close(fit["working"]["coefficients"][1]["value"], reference_fit["slope"], "slope")

    matrix_rows = read_csv(reference_dir / "correlation.csv")
    checked_correlation = 0
    for row in matrix_rows:
        i = int(row["period_row"]) - 1
        j = int(row["period_column"]) - 1
        close(fit["correlation_matrix"][i][j], row["value"], f"correlation[{i},{j}]")
        checked_correlation += 1

    coefficient_rows = read_csv(reference_dir / "coefficients.csv")
    covariance_rows = read_csv(reference_dir / "covariance.csv")
    inference_keys = {"working": "working", "sandwich": "sandwich"}
    checked_covariance = 0
    for row in covariance_rows:
        name = row["kind"]
        parameters = ["_cons", "lnwg"]
        i = parameters.index(row["row"])
        j = parameters.index(row["column"])
        close(fit[inference_keys[name]]["covariance"][i][j], row["value"], f"{name}.covariance[{i},{j}]")
        checked_covariance += 1
    checked_inference = 0
    for row in coefficient_rows:
        kind = row["covariance"]
        parameter = row["parameter"]
        index = 0 if parameter == "_cons" else 1
        native = fit[kind]["coefficients"][index]
        for field in ["estimate", "standard_error", "statistic", "p_value", "confidence_lower", "confidence_upper"]:
            close(native["value" if field == "estimate" else field], row[field], f"{kind}.{parameter}.{field}", relative=1e-8 if field == "p_value" else 1e-10)
            checked_inference += 1

    reference_predictions = read_csv(reference_dir / "predictions.csv")
    predictions = read_csv(native_predictions)
    if len(predictions) != 5320 or len(predictions) != len(reference_predictions):
        raise SystemExit(f"prediction row count mismatch: native {len(predictions)}, R {len(reference_predictions)}")
    checked_predictions = 0
    for index, (native, expected) in enumerate(zip(predictions, reference_predictions), start=1):
        if int(native["source_row"]) != index or int(expected["source_row"]) != index:
            raise SystemExit(f"source order mismatch at row {index}")
        for native_key, reference_key in [
            ("lnhr", "lnhr"),
            ("lnwg", "lnwg"),
            ("id", "entity"),
            ("year", "time"),
            ("unstructured_mean", "fitted_mean"),
            ("unstructured_residual", "residual"),
        ]:
            close(native[native_key], expected[reference_key], f"prediction[{index}].{native_key}")
            checked_predictions += 1

    digest = hashlib.md5(source_path.read_bytes()).hexdigest()
    recorded_digest = (reference_dir / "MOM.md5").read_text().strip()
    if digest != recorded_digest:
        raise SystemExit(f"MOM.dat digest differs: native input {digest}, R reference {recorded_digest}")
    if fit["sample"] != {"total_observations": 5320, "dropped_observations": 0}:
        raise SystemExit(f"unexpected sample metadata: {fit['sample']}")

    historical = {
        "slope": (float(fit["working"]["coefficients"][1]["value"]), 0.0910023),
        "working_se": (float(fit["working"]["coefficients"][1]["standard_error"]), 0.0137712),
        "panel_robust_se": (float(fit["sandwich"]["coefficients"][1]["standard_error"]), 0.0501344),
    }
    print(
        f"PASS native/Base-R: {checked_correlation} R entries, {checked_covariance} covariance entries, "
        f"{checked_inference} inference quantities, {checked_predictions} ordered prediction values."
    )
    print("Historical Stata author-log values (diagnostic; not a parity assertion):")
    for label, (actual, expected) in historical.items():
        print(f"  {label}: native {actual:.10f}; author {expected:.10f}; difference {actual - expected:+.3g}")


if __name__ == "__main__":
    main()

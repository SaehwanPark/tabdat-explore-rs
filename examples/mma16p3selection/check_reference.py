#!/usr/bin/env python3
"""Validate the pinned MMA §16.6 source, base-R gates and native outputs."""

from __future__ import annotations

import csv
import hashlib
import json
import math
import sys
from pathlib import Path


SOURCE_HASHES = {
    "mma16p3selection.do": "f7a37a6f67c0a9e9ed812d92cad19750d57bf09928995fda0536308dd78af2be",
    "mma16p3selection.txt": "8de79b87b7667f68ca762e611bd5a7382aa20c9997a921e29ea81419dfb84be6",
    "mma16p3selection.asc": "2fde2032f8f6319731ee56b40e87f901abcca9f2072211b61cedd6dab00c37e8",
}
FIELDS = [
    "DMED", "MED", "LNMED", "LC", "IDP", "LPI", "FMDE", "PHYSLIM",
    "NDISEASE", "HLTHG", "HLTHF", "HLTHP", "LINC", "LFAM", "EDUCDEC",
    "AGE", "FEMALE", "CHILD", "FEMCHILD", "BLACK",
]
PREDICTIONS = {
    "twopart_probability": "probit_probability",
    "twopart_lnmed_xb": "twopart_lnmed_xb",
    "ml_lnmed_xb": "ml_xb",
    "ml_selection_probability": "ml_probability",
    "ml_inverse_mills": "ml_mills",
    "ml_lnmed_ycond": "ml_ycond",
    "twostep_lnmed_xb": "twostep_xb",
    "twostep_selection_probability": "twostep_probability",
    "twostep_inverse_mills": "twostep_mills",
    "twostep_lnmed_ycond": "twostep_ycond",
}


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL: {message}")


def read_csv(path: Path) -> list[dict[str, str]]:
    with path.open(newline="", encoding="utf-8") as source:
        return list(csv.DictReader(source))


def close(actual: float, expected: float, tolerance: float, label: str) -> None:
    difference = abs(actual - expected)
    require(
        math.isfinite(actual) and math.isfinite(expected) and difference <= tolerance,
        f"{label}: {actual:.16g} differs from {expected:.16g} by {difference:.3g} "
        f"(limit {tolerance:g})",
    )


def quantity(gates: dict[tuple[str, int, int], float], name: str, i: int = 0, j: int = 0) -> float:
    try:
        return gates[(name, i, j)]
    except KeyError:
        raise SystemExit(f"FAIL: reference quantity missing: {name}[{i},{j}]") from None


def compare_vector(
    actual: list[dict[str, object]],
    gates: dict[tuple[str, int, int], float],
    value_name: str,
    se_name: str | None = None,
    tolerance: float = 1e-8,
) -> None:
    for index, estimate in enumerate(actual, 1):
        close(float(estimate["value"]), quantity(gates, value_name, index), tolerance, f"{value_name}[{index}]")
        if se_name is not None:
            close(
                float(estimate["standard_error"]),
                quantity(gates, se_name, index),
                tolerance,
                f"{se_name}[{index}]",
            )


def compare_matrix(
    actual: list[list[float]],
    gates: dict[tuple[str, int, int], float],
    name: str,
    size: int,
    tolerance: float,
) -> int:
    require(len(actual) == size and all(len(row) == size for row in actual), f"{name} dimension")
    for i in range(1, size + 1):
        for j in range(1, size + 1):
            close(float(actual[i - 1][j - 1]), quantity(gates, name, i, j), tolerance, f"{name}[{i},{j}]")
    return size * size


def main(directory: Path) -> None:
    for name, expected in SOURCE_HASHES.items():
        actual = hashlib.sha256((directory / name).read_bytes()).hexdigest()
        require(actual == expected, f"{name} fingerprint {actual} != {expected}")

    source_tokens = (directory / "mma16p3selection.asc").read_text(encoding="ascii").split()
    require(len(source_tokens) == 111_480, f"source token count {len(source_tokens)}")
    require(len(source_tokens) % len(FIELDS) == 0, "source record width")
    source_matrix = [source_tokens[i : i + len(FIELDS)] for i in range(0, len(source_tokens), len(FIELDS))]
    require(len(source_matrix) == 5_574, "source row count")

    prepared = read_csv(directory / "mma16p3selection.csv")
    require(len(prepared) == 5_574, f"prepared row count {len(prepared)}")
    require(list(prepared[0]) == ["source_row", *FIELDS], "prepared field order")
    selected_rows: list[int] = []
    log_rounding_error = 0.0
    for row_number, (record, source_values) in enumerate(zip(prepared, source_matrix, strict=True), 1):
        require(record["source_row"] == str(row_number), f"prepared source row {row_number}")
        for name, token in zip(FIELDS, source_values, strict=True):
            require(record[name] == ("" if token == "." else token), f"source value {row_number}:{name}")
        dmed = float(record["DMED"])
        med = float(record["MED"])
        require(dmed in (0.0, 1.0), f"nonbinary DMED at row {row_number}")
        require((dmed == 1.0) == (med > 0.0), f"DMED/MED mismatch at row {row_number}")
        if dmed == 1.0:
            selected_rows.append(row_number - 1)
            require(record["LNMED"] != "", f"selected LNMED missing at row {row_number}")
            log_rounding_error = max(log_rounding_error, abs(float(record["LNMED"]) - math.log(med)))
        else:
            require(record["LNMED"] == "", f"unselected LNMED present at row {row_number}")
    require(len(selected_rows) == 4_281, f"selected row count {len(selected_rows)}")
    require(log_rounding_error <= 3.3e-6, f"source LNMED rounding difference {log_rounding_error:g}")

    quantities = read_csv(directory / "reference_quantities.csv")
    gates = {
        (row["quantity"], int(row["i"]), int(row["j"])): float(row["value"])
        for row in quantities
    }
    reference_predictions = read_csv(directory / "reference_predictions.csv")
    native_predictions = read_csv(directory / "mma16p3selection_predictions.csv")
    require(len(reference_predictions) == 5_574, "R reference prediction row count")
    require(len(native_predictions) == 5_574, "native prediction row count")

    envelopes = [json.loads(line) for line in (directory / "native.jsonl").read_text().splitlines() if line.strip()]
    results = {
        envelope["result_type"]: envelope["data"]
        for envelope in envelopes
        if envelope.get("result_type") in {
            "BinaryProbitResult", "RegressionResult", "HeckmanTwoStepResult", "HeckmanMlResult"
        }
    }
    require(set(results) == {"BinaryProbitResult", "RegressionResult", "HeckmanTwoStepResult", "HeckmanMlResult"}, "native JSON result types")
    probit = results["BinaryProbitResult"]
    ols = results["RegressionResult"]
    two_step = results["HeckmanTwoStepResult"]
    ml = results["HeckmanMlResult"]
    require(probit["observation_count"] == 5_574 and probit["event_count"] == 4_281, "probit samples")
    require(ols["observation_count"] == 4_281, "selected OLS sample")
    require(two_step["observation_count"] == 5_574 and two_step["selected_count"] == 4_281, "two-step samples")
    require(ml["observation_count"] == 5_574 and ml["selected_count"] == 4_281, "ML samples")
    require(ml["samples"]["selection"]["retained_indices"] == list(range(5_574)), "ML selection row identity")
    require(ml["samples"]["outcome"]["retained_indices"] == selected_rows, "ML outcome row identity")
    require(ml["samples"]["selected_selection_indices"] == selected_rows, "ML selected-subset identity")

    compare_vector(probit["coefficients"], gates, "probit_value", "probit_standard_error")
    compare_matrix(probit["covariance_matrix"], gates, "probit_covariance", 18, 1e-8)
    close(float(probit["log_likelihood"]), quantity(gates, "probit_loglikelihood"), 1e-8, "probit log likelihood")
    compare_vector(ols["coefficients"], gates, "twopart_ols_value", "twopart_ols_standard_error")
    compare_matrix(ols["covariance_matrix"], gates, "twopart_ols_covariance", 18, 1e-8)
    close(float(ols["log_likelihood"]), quantity(gates, "twopart_ols_loglikelihood"), 1e-8, "selected OLS log likelihood")

    compare_vector(two_step["selection_coefficients"], gates, "heckman_twostep_selection_value", "heckman_twostep_selection_standard_error")
    compare_matrix(two_step["selection_covariance_matrix"], gates, "probit_covariance", 18, 1e-8)
    compare_vector(two_step["coefficients"], gates, "heckman_twostep_outcome_value", "heckman_twostep_outcome_standard_error")
    compare_matrix(two_step["covariance_matrix"], gates, "heckman_twostep_outcome_covariance", 19, 1e-8)
    close(float(two_step["sigma"]), quantity(gates, "heckman_twostep_sigma"), 1e-8, "two-step sigma")
    close(float(two_step["rho"]), quantity(gates, "heckman_twostep_rho"), 1e-8, "two-step rho")
    require(two_step["joint_log_likelihood"] is None, "two-step must not report joint ML likelihood")

    compare_vector(ml["estimates"]["outcome"], gates, "heckman_ml_outcome_value", "heckman_ml_outcome_standard_error", 1e-5)
    compare_vector(ml["estimates"]["selection"], gates, "heckman_ml_selection_value", "heckman_ml_selection_standard_error", 1e-5)
    dimension = 38
    covariance_checks = compare_matrix(ml["covariance_matrix"], gates, "heckman_ml_covariance", dimension, 2e-8)
    parameter_names = ml["covariance_parameter_names"]
    require(len(parameter_names) == dimension, "ML covariance parameter count")
    require(parameter_names[-2:] == ["lnsigma", "athrho"], "ML covariance ancillary order")
    close(float(ml["estimates"]["ancillary"][0]["value"]), quantity(gates, "heckman_ml_lnsigma"), 1e-5, "ML lnsigma")
    close(float(ml["estimates"]["ancillary"][1]["value"]), quantity(gates, "heckman_ml_athrho"), 1e-5, "ML atanh(rho)")
    close(float(ml["likelihood"]["joint"]), quantity(gates, "heckman_ml_loglikelihood"), 1e-7, "joint log likelihood")
    close(float(ml["likelihood"]["independent"]), quantity(gates, "heckman_ml_independent_loglikelihood"), 1e-7, "independence log likelihood")
    close(float(ml["likelihood"]["independence_lr_chi_square"]), quantity(gates, "heckman_ml_independence_lr"), 1e-7, "independence LR")
    close(float(ml["sigma"]["value"]), quantity(gates, "heckman_ml_sigma"), 1e-5, "ML sigma")
    close(float(ml["rho"]["value"]), quantity(gates, "heckman_ml_rho"), 1e-5, "ML rho")
    close(float(ml["lambda"]["value"]), quantity(gates, "heckman_ml_lambda"), 1e-5, "ML lambda")
    require(ml["covariance"] == "model_based_joint_observed_information", "ML covariance label")
    require(ml["identification_warning"] is not None, "no-exclusion warning")
    require(ml["optimizer"]["converged"] is True, "ML convergence")

    for native_row, reference_row in zip(native_predictions, reference_predictions, strict=True):
        require(native_row["source_row"] == reference_row["source_row"], "prediction source-row order")
        for field in ["DMED", "MED", "LNMED"]:
            require((native_row[field] == "") == (reference_row[field] == ""), f"prediction source missingness {field}")
        for native_name, reference_name in PREDICTIONS.items():
            close(float(native_row[native_name]), float(reference_row[reference_name]), 1e-5, f"prediction {native_name} row {native_row['source_row']}")

    text = (directory / "native.txt").read_text(encoding="utf-8")
    for marker in [
        "Heckman sample selection (joint IID Gaussian maximum likelihood)",
        "Joint log likelihood = -10170.110",
        "independence LR chi2(1) = 27.930",
        "No excluded selection predictor",
        "Heckman two-step sample selection",
    ]:
        require(marker in text, f"native text output missing {marker!r}")

    print(
        "PASS: three pinned sources, 5,574 source rows, 4,281 selected rows, "
        "probit/OLS/two-step/ML JSON, 1,444 full joint covariance cells, "
        "and 55,740 native-versus-R projection values."
    )
    print(f"Source LNMED/log(MED) maximum rounding difference: {log_rounding_error:.7g}")
    print(f"Checked full ML covariance entries: {covariance_checks}; prediction modes: {len(PREDICTIONS)}")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit("usage: check_reference.py WORK_DIRECTORY")
    main(Path(sys.argv[1]).resolve())

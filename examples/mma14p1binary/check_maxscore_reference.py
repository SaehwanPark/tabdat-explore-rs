#!/usr/bin/env python3
"""Audit the native §14.7.5 point-only maximum-score workflow against base R."""

import csv
import json
import math
import sys


def close(actual, expected, tolerance=1e-10):
    if not math.isfinite(float(actual)) or abs(float(actual) - float(expected)) > tolerance:
        raise SystemExit(f"{actual!r} differs from {expected!r} (tol={tolerance})")


def rows(path):
    with open(path, newline="", encoding="utf-8") as stream:
        return list(csv.DictReader(stream))


def main():
    if len(sys.argv) != 6:
        raise SystemExit(
            "usage: check_maxscore_reference.py regions.csv reference_predictions.csv "
            "native.jsonl native_predictions.csv Nldata.csv"
        )
    region_path, reference_path, json_path, native_path, source_path = sys.argv[1:]
    regions = rows(region_path)
    reference = rows(reference_path)
    source = rows(source_path)
    native_predictions = rows(native_path)
    envelopes = [json.loads(line) for line in open(json_path, encoding="utf-8") if line.strip()]
    fits = [item["data"] for item in envelopes if item.get("result_type") == "MaximumScoreResult"]
    if len(fits) != 1:
        raise SystemExit(f"expected one MaximumScoreResult, found {len(fits)}")
    fit = fits[0]
    if len(source) != 1182 or len(reference) != 630 or len(native_predictions) != 630:
        raise SystemExit("expected 1,182 source rows and 630 prepared/prediction rows")
    if fit["outcome"] != "charter" or fit["predictor"] != "lnrelp":
        raise SystemExit("unexpected outcome or predictor")
    if fit["estimator"] != "binary_maximum_score_exhaustive_one_predictor":
        raise SystemExit("unexpected estimator label")
    if fit["coefficient_normalization"] != "euclidean_unit_norm":
        raise SystemExit("unexpected coefficient normalization")
    if fit["inference"] != "not_provided" or fit["probability_predictions"] is not False:
        raise SystemExit("point-only output must not imply formal inference or probabilities")

    result = fit["result"]
    if fit["sample"] != {
        "retained_observations": 630,
        "total_observations": 630,
        "dropped_observations": 0,
    }:
        raise SystemExit(f"unexpected sample summary: {fit['sample']}")
    if result["observation_count"] != 630 or result["event_count"] != 452:
        raise SystemExit("incorrect sample/event count")
    if result["correct_count"] != 536:
        raise SystemExit("incorrect maximum correct-classification count")
    close(result["empirical_score"], 536 / 630)
    expected_classification = {
        "true_positive": 421,
        "false_positive": 63,
        "false_negative": 31,
        "true_negative": 115,
    }
    if result["classification"] != expected_classification:
        raise SystemExit(f"incorrect confusion counts: {result['classification']}")
    if len(regions) != 2 or len(result["optimal_regions"]) != 2:
        raise SystemExit("expected the two tied empirical-score maximizing regions")

    for index, (actual, expected) in enumerate(zip(result["optimal_regions"], regions)):
        rule = actual["rule"]
        if rule["kind"] != "threshold" or rule["direction"] != expected["direction"]:
            raise SystemExit(f"region {index + 1}: direction or rule kind differs")
        for key in ("lower", "upper", "threshold"):
            close(rule[key], expected[key])
        if actual["correct_count"] != int(expected["correct_count"]):
            raise SystemExit(f"region {index + 1}: correct count differs")
    for key, expected in (("intercept", regions[0]["intercept"]), ("slope", regions[0]["slope"])):
        close(result[key], expected)
    if result["predicted_classes"] != [row["selected_class"] == "1" for row in reference]:
        raise SystemExit("retained-sample classes differ from the R reference")

    for index, (actual, expected) in enumerate(zip(native_predictions, reference)):
        for key in ("source_row", "mode", "charter"):
            if int(float(actual[key])) != int(float(expected[key])):
                raise SystemExit(f"prediction row {index + 1}: {key} differs")
        close(actual["lnrelp"], expected["lnrelp"], 1e-12)
        if int(float(actual["predicted_charter"])) != int(expected["selected_class"]):
            raise SystemExit(f"prediction row {index + 1}: class differs")

    source_ids = [int(row["source_row"]) for row in source]
    expected_ids = [int(row["source_row"]) for row in reference]
    if source_ids != list(range(1, 1183)):
        raise SystemExit("source row IDs or order differ")
    if expected_ids != [int(row["source_row"]) for row in native_predictions]:
        raise SystemExit("native and reference prediction row order differs")
    print(
        "PASS: 1,182 pinned source rows, 630 charter/pier cases, both maximizing "
        "regions, unit-norm point estimates, confusion counts, and all 630 current-row classes."
    )


if __name__ == "__main__":
    main()

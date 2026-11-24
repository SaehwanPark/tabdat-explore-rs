use rand_xoshiro::{
  Xoshiro256PlusPlus,
  rand_core::{RngCore, SeedableRng},
};
use tabdat_stats::{EstimationProblem, EstimationSample, bootstrap_quantiles_clustered};

const REPS: usize = 80;
const SEED: u64 = 918_273;
const QUANTILES: &[f64] = &[0.25, 0.5, 0.75];

fn problem() -> EstimationProblem {
  let mut retained = Vec::new();
  let mut response = Vec::new();
  let mut design = Vec::new();
  let mut groups = Vec::new();
  for line in include_str!("fixtures/cluster-quantile-bootstrap/inputs.csv")
    .lines()
    .skip(1)
  {
    let fields = line.split(',').collect::<Vec<_>>();
    if fields[1..].contains(&"") {
      continue;
    }
    retained.push(fields[0].parse::<usize>().unwrap() - 1);
    response.push(fields[1].parse().unwrap());
    design.push(vec![fields[2].parse().unwrap(), fields[3].parse().unwrap()]);
    groups.push(fields[4].to_owned());
  }
  let n = response.len();
  EstimationProblem::new("y", vec!["x".into(), "z".into()], response, design)
    .unwrap()
    .with_sample(EstimationSample::new(retained, n + 3, 3, None, Some(groups)).unwrap())
}

fn draw_index(rng: &mut impl RngCore, bound: usize) -> usize {
  let bound = bound as u64;
  let zone = u64::MAX - u64::MAX % bound;
  loop {
    let value = rng.next_u64();
    if value < zone {
      return (value % bound) as usize;
    }
  }
}

fn reference_value(result: &tabdat_stats::QuantileBootstrapResult, row: &[&str]) -> f64 {
  match row[0] {
    "coefficient" => {
      let coefficient = result
        .coefficients
        .iter()
        .find(|coefficient| coefficient.name == row[1])
        .unwrap();
      match row[2] {
        "estimate" => coefficient.estimate,
        "bootstrap_mean" => coefficient.bootstrap_mean,
        "bias" => coefficient.bias,
        "standard_error" => coefficient.standard_error,
        "ci_lower" => coefficient.ci_lower,
        "ci_upper" => coefficient.ci_upper,
        _ => unreachable!(),
      }
    }
    "covariance" => {
      let i = result
        .covariance_parameter_names
        .iter()
        .position(|name| name == row[1])
        .unwrap();
      let j = result
        .covariance_parameter_names
        .iter()
        .position(|name| name == row[2])
        .unwrap();
      result.covariance_matrix[i][j]
    }
    "meta" => match row[1] {
      "observation_count" => result.observation_count as f64,
      "dropped_observations" => result.sample.as_ref().unwrap().dropped_observations as f64,
      "cluster_count" => result.cluster_count.unwrap() as f64,
      "replications" => result.replications as f64,
      "min_replicate_n" => *result
        .replicate_observation_counts
        .as_ref()
        .unwrap()
        .iter()
        .min()
        .unwrap() as f64,
      "max_replicate_n" => *result
        .replicate_observation_counts
        .as_ref()
        .unwrap()
        .iter()
        .max()
        .unwrap() as f64,
      _ => unreachable!(),
    },
    "contrast" => {
      let contrast = result.contrast.as_ref().unwrap();
      match row[2] {
        "estimate" => contrast.estimate,
        "standard_error" => contrast.standard_error.unwrap(),
        "statistic" => contrast.statistic.unwrap(),
        "p_value" => contrast.p_value.unwrap(),
        "ci_lower" => contrast.ci_lower.unwrap(),
        "ci_upper" => contrast.ci_upper.unwrap(),
        _ => unreachable!(),
      }
    }
    _ => unreachable!(),
  }
}

#[test]
fn whole_cluster_draws_share_quantiles_and_match_independent_quantreg_reference() {
  let problem = problem();
  let result = bootstrap_quantiles_clustered(&problem, QUANTILES, REPS, SEED).unwrap();
  assert_eq!(
    result,
    bootstrap_quantiles_clustered(&problem, QUANTILES, REPS, SEED).unwrap()
  );
  assert_eq!(result.observation_count, 100);
  assert_eq!(result.cluster_count, Some(20));
  assert_eq!(
    result.cluster_sizes.as_ref().unwrap(),
    &[4, 5, 6, 5].repeat(5)
  );
  let replicate_counts = result.replicate_observation_counts.as_ref().unwrap();
  assert_eq!(replicate_counts.len(), REPS);
  assert!(replicate_counts.iter().any(|count| *count != 100));
  assert_eq!(result.cluster_variable, None);
  assert_eq!(problem.sample.as_ref().unwrap().dropped_observations, 3);

  let mut rng = Xoshiro256PlusPlus::seed_from_u64(SEED);
  let draws = include_str!("fixtures/cluster-quantile-bootstrap/cluster_draws.csv")
    .lines()
    .skip(1)
    .map(|line| {
      line
        .split(',')
        .skip(1)
        .map(|value| value.parse::<usize>().unwrap() - 1)
        .collect::<Vec<_>>()
    })
    .collect::<Vec<_>>();
  assert_eq!(draws.len(), REPS);
  for replicate in draws {
    assert_eq!(replicate.len(), 20);
    for expected in replicate {
      assert_eq!(expected, draw_index(&mut rng, 20));
    }
  }

  let mut comparisons = 0;
  for line in include_str!("fixtures/cluster-quantile-bootstrap/reference.csv")
    .lines()
    .skip(1)
  {
    let fields = line.split(',').collect::<Vec<_>>();
    if fields[0] == "contrast" {
      continue;
    }
    let actual = reference_value(&result, &fields);
    let expected = fields[3].parse::<f64>().unwrap();
    assert!(
      (actual - expected).abs() <= 2e-8,
      "{fields:?}: {actual} != {expected}"
    );
    comparisons += 1;
  }
  assert_eq!(comparisons, 9 * 6 + 9 * 9 + 6);
}

#[test]
fn clustered_two_quantile_contrast_matches_independent_common_draw_reference() {
  let result = tabdat_stats::bootstrap_quantiles_clustered_with_contrast(
    &problem(),
    &[0.25, 0.75],
    REPS,
    SEED,
    "x",
  )
  .unwrap();
  assert_eq!(result.contrast.as_ref().unwrap().parameter, "x");
  assert_eq!(result.contrast.as_ref().unwrap().lower_quantile, 0.25);
  assert_eq!(result.contrast.as_ref().unwrap().upper_quantile, 0.75);
  let mut comparisons = 0;
  for line in include_str!("fixtures/cluster-quantile-bootstrap/reference.csv")
    .lines()
    .skip(1)
  {
    let fields = line.split(',').collect::<Vec<_>>();
    if fields[0] != "contrast" {
      continue;
    }
    let actual = reference_value(&result, &fields);
    let expected = fields[3].parse::<f64>().unwrap();
    assert!(
      (actual - expected).abs() <= 2e-8,
      "{fields:?}: {actual} != {expected}"
    );
    comparisons += 1;
  }
  assert_eq!(comparisons, 6);
}

#[test]
fn clustered_quantile_bootstrap_rejects_missing_or_small_cluster_samples_and_work_overflow() {
  let mut no_groups = problem();
  no_groups.sample.as_mut().unwrap().cluster_groups = None;
  assert!(
    bootstrap_quantiles_clustered(&no_groups, QUANTILES, REPS, SEED)
      .unwrap_err()
      .to_string()
      .contains("retained cluster identifiers")
  );

  let mut two_groups = problem();
  let groups = two_groups
    .sample
    .as_mut()
    .unwrap()
    .cluster_groups
    .as_mut()
    .unwrap();
  for (index, group) in groups.iter_mut().enumerate() {
    *group = (if index % 2 == 0 { "a" } else { "b" }).into();
  }
  assert!(
    bootstrap_quantiles_clustered(&two_groups, QUANTILES, REPS, SEED)
      .unwrap_err()
      .to_string()
      .contains("at least three retained clusters")
  );

  assert!(
    bootstrap_quantiles_clustered(
      &problem(),
      &(1..=19).map(|i| i as f64 / 20.0).collect::<Vec<_>>(),
      5000,
      SEED
    )
    .unwrap_err()
    .to_string()
    .contains("must be at most 100000000")
  );
}

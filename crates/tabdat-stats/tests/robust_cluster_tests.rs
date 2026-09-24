#![forbid(unsafe_code)]

use tabdat_stats::estimates::CovarianceType;
use tabdat_stats::least_squares::{LeastSquaresOptions, fit_least_squares_with_options};
use tabdat_stats::problem::EstimationProblem;
use tabdat_stats::sample::EstimationSample;

#[test]
fn test_robust_hc1_and_clustered_covariance() {
  let x1_raw = vec![
    4.095824194223853,
    2.7555137590082093,
    4.43439167964553,
    3.7894721162374556,
    1.3767093915505981,
    4.902489406547024,
    4.044558807961412,
    4.144257221107815,
    1.5124545307021835,
    2.8015437515822685,
    2.483192096930325,
    4.707059955394407,
    3.575460480322658,
    4.29104645308332,
    2.7736567953093245,
    1.9089548871391075,
    3.218339148063339,
    1.2552690244167013,
    4.310524687970329,
    3.5266575964882594,
    4.032350960341495,
    2.4181038725194735,
    4.882792097579613,
    4.572484485288791,
    4.113533988295048,
    1.7785548314078703,
    2.8668840149081367,
    1.175215063148915,
    1.6171579682701913,
    3.7321958129698185,
  ];

  let y_raw = vec![
    10.762472188882937,
    6.807820009824111,
    10.11266199475529,
    8.672057868350972,
    4.561408494388944,
    11.869464959454493,
    9.532143887095387,
    9.368436203734367,
    4.112668453558747,
    7.428383897076888,
    6.838011279462371,
    11.185697044941412,
    8.318166107000968,
    10.1981735677,
    7.105656495189013,
    5.427254072642722,
    8.372392685100774,
    4.122335823220744,
    10.460506157476605,
    8.587104727720966,
    9.709261620027982,
    6.651851857958217,
    10.537006285231392,
    10.485133362398932,
    9.491881649443698,
    4.73767073869407,
    7.096196904202931,
    4.597900781915028,
    4.301400378693761,
    9.448530803235377,
  ];

  let weights = vec![
    1.1956652745368321,
    1.3354992001912178,
    1.2443028240004135,
    1.1875055804605972,
    1.1914450125217895,
    1.0127650686438285,
    0.8215724841975576,
    1.1057021724626537,
    0.9502092709736705,
    1.0859700506072454,
    1.3973821512877163,
    0.9637576401057386,
    0.8408119191823462,
    0.9969687244153976,
    1.0055156304366786,
    1.2633415603088265,
    1.1899225066388948,
    1.3487287463744895,
    1.2650194782291713,
    1.0844708030080494,
    1.3698142692662243,
    0.9168810439353928,
    0.8158984511937024,
    0.8630335025429493,
    1.3056515454175153,
    1.123314061175971,
    0.9128902453235213,
    1.1507313425723544,
    0.906618471899218,
    1.2874242625544152,
  ];

  let mut cluster_ids = Vec::with_capacity(30);
  for _ in 0..10 {
    cluster_ids.push("c0".to_string());
  }
  for _ in 0..10 {
    cluster_ids.push("c1".to_string());
  }
  for _ in 0..10 {
    cluster_ids.push("c2".to_string());
  }

  let n = x1_raw.len();
  let design_matrix: Vec<Vec<f64>> = x1_raw.iter().map(|&x| vec![x]).collect();

  let sample = EstimationSample::new(
    (0..n).collect(),
    n,
    0,
    Some(weights.clone()),
    Some(cluster_ids.clone()),
  )
  .unwrap();

  let problem = EstimationProblem::new("y", vec!["x1".into()], y_raw.clone(), design_matrix)
    .unwrap()
    .with_sample(sample);

  // 1. OLS Classical NonRobust
  let res_ols = fit_least_squares_with_options(
    &problem,
    &LeastSquaresOptions {
      covariance_type: CovarianceType::NonRobust,
    },
  )
  .unwrap();
  assert!((res_ols.coefficients[0].value - 1.616492059061).abs() < 1e-9);
  assert!((res_ols.coefficients[1].value - 1.981581139553).abs() < 1e-9);
  assert!((res_ols.coefficients[0].standard_error.unwrap() - 0.220633485739).abs() < 1e-9);
  assert!((res_ols.coefficients[1].standard_error.unwrap() - 0.064148712282).abs() < 1e-9);

  // 2. OLS Robust HC1
  let res_hc1 = fit_least_squares_with_options(
    &problem,
    &LeastSquaresOptions {
      covariance_type: CovarianceType::RobustHc1,
    },
  )
  .unwrap();
  assert!((res_hc1.coefficients[0].value - 1.616492059061).abs() < 1e-9);
  assert!((res_hc1.coefficients[1].value - 1.981581139553).abs() < 1e-9);
  assert!((res_hc1.coefficients[0].standard_error.unwrap() - 0.231325052778).abs() < 1e-9);
  assert!((res_hc1.coefficients[1].standard_error.unwrap() - 0.071302904077).abs() < 1e-9);
  assert!((res_hc1.coefficients[0].p_value.unwrap() - 1.341495009093e-07).abs() < 1e-12);
  assert!((res_hc1.coefficients[1].p_value.unwrap() - 6.252486673766e-22).abs() < 1e-25);

  // 3. OLS Clustered Covariance
  let res_clu = fit_least_squares_with_options(
    &problem,
    &LeastSquaresOptions {
      covariance_type: CovarianceType::Cluster("cid".into()),
    },
  )
  .unwrap();
  assert!((res_clu.coefficients[0].value - 1.616492059061).abs() < 1e-9);
  assert!((res_clu.coefficients[1].value - 1.981581139553).abs() < 1e-9);
  assert!((res_clu.coefficients[0].standard_error.unwrap() - 0.160036793687).abs() < 1e-9);
  assert!((res_clu.coefficients[1].standard_error.unwrap() - 0.058679166405).abs() < 1e-9);
  assert!((res_clu.coefficients[0].p_value.unwrap() - 9.659710021942e-03).abs() < 1e-8);
  assert!((res_clu.coefficients[1].p_value.unwrap() - 8.757363331674e-04).abs() < 1e-8);

  // 4. WLS Robust HC1
  let problem_wls = problem.clone().with_weights(weights);
  let res_wls_hc1 = fit_least_squares_with_options(
    &problem_wls,
    &LeastSquaresOptions {
      covariance_type: CovarianceType::RobustHc1,
    },
  )
  .unwrap();
  assert!((res_wls_hc1.coefficients[0].value - 1.635870925897).abs() < 1e-9);
  assert!((res_wls_hc1.coefficients[1].value - 1.982073509616).abs() < 1e-9);
  assert!((res_wls_hc1.coefficients[0].standard_error.unwrap() - 0.218060528403).abs() < 1e-9);
  assert!((res_wls_hc1.coefficients[1].standard_error.unwrap() - 0.067977697154).abs() < 1e-9);

  // 5. WLS Clustered Covariance
  let res_wls_clu = fit_least_squares_with_options(
    &problem_wls,
    &LeastSquaresOptions {
      covariance_type: CovarianceType::Cluster("cid".into()),
    },
  )
  .unwrap();
  assert!((res_wls_clu.coefficients[0].value - 1.635870925897).abs() < 1e-9);
  assert!((res_wls_clu.coefficients[1].value - 1.982073509616).abs() < 1e-9);
  assert!((res_wls_clu.coefficients[0].standard_error.unwrap() - 0.141853022976).abs() < 1e-9);
  assert!((res_wls_clu.coefficients[1].standard_error.unwrap() - 0.050368672401).abs() < 1e-9);
}

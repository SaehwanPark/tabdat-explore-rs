use serde_json::json;
use tabdat_domain::{Command, EstimationFamily, XtPoissonEstimator};
#[test]
fn normal_selector_is_additive_and_legacy_fe_gamma_wires_remain_exact() {
  for (selector, expected) in [
    ("ConditionalFe", XtPoissonEstimator::ConditionalFe),
    ("SharedGammaRe", XtPoissonEstimator::SharedGammaRe),
    ("NormalRe", XtPoissonEstimator::NormalRe),
  ] {
    let wire = json!({"kind":"xt_poisson","payload":{"command":{"estimator":selector,"outcome":"y","predictors":["x"],"entity_variable":"g"}}});
    let c: Command = serde_json::from_value(wire.clone()).unwrap();
    let Command::XtPoisson { ref command } = c else {
      panic!()
    };
    assert_eq!(command.estimator, expected);
    assert_eq!(serde_json::to_value(c).unwrap(), wire);
  }
  let c:Command=serde_json::from_value(json!({"kind":"xt_poisson","payload":{"command":{"outcome":"y","predictors":["x"],"entity_variable":"g"}}})).unwrap();
  let Command::XtPoisson { command } = c else {
    panic!()
  };
  assert_eq!(command.estimator, XtPoissonEstimator::ConditionalFe);
  assert_eq!(
    serde_json::to_value(EstimationFamily::NormalPoisson).unwrap(),
    json!("normal_poisson")
  );
  assert_eq!(
    serde_json::to_value(EstimationFamily::GammaPoisson).unwrap(),
    json!("gamma_poisson")
  );
}

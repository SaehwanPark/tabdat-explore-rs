use serde_json::json;
use tabdat_domain::{Command, XtGeeCommand};
#[test]
fn gaussian_command_wire_is_frozen_and_poisson_is_additive() {
  let payload = json!({"command":{"outcome":"y","predictors":["x"],"entity_variable":"id"}});
  let gaussian = json!({"kind":"xt_gee","payload":payload});
  let old: Command = serde_json::from_value(gaussian.clone()).unwrap();
  assert!(matches!(old, Command::XtGee { .. }));
  assert_eq!(serde_json::to_value(&old).unwrap(), gaussian);
  assert_eq!(old.name(), "xtgee");
  let unstructured = Command::XtGee {
    command: XtGeeCommand {
      outcome: "y".into(),
      predictors: vec!["x".into()],
      entity_variable: "id".into(),
      independent: false,
      time_variable: Some("year".into()),
    },
  };
  let unstructured_wire = json!({
    "kind":"xt_gee",
    "payload":{"command":{"outcome":"y","predictors":["x"],"entity_variable":"id","time_variable":"year"}}
  });
  assert_eq!(
    serde_json::to_value(&unstructured).unwrap(),
    unstructured_wire
  );
  assert_eq!(
    serde_json::from_value::<Command>(unstructured_wire).unwrap(),
    unstructured
  );
  let independent = Command::XtGee {
    command: XtGeeCommand {
      outcome: "y".into(),
      predictors: vec!["x".into()],
      entity_variable: "id".into(),
      independent: true,
      time_variable: None,
    },
  };
  let independent_wire = json!({
    "kind":"xt_gee",
    "payload":{"command":{"outcome":"y","predictors":["x"],"entity_variable":"id","independent":true}}
  });
  assert_eq!(
    serde_json::to_value(&independent).unwrap(),
    independent_wire
  );
  assert_eq!(
    serde_json::from_value::<Command>(independent_wire).unwrap(),
    independent
  );
  let new = Command::XtGeePoisson {
    command: XtGeeCommand {
      outcome: "y".into(),
      predictors: vec!["x".into()],
      entity_variable: "id".into(),
      independent: false,
      time_variable: None,
    },
  };
  let wire = json!({"kind":"xt_gee_poisson","payload":payload});
  assert_eq!(serde_json::to_value(&new).unwrap(), wire);
  assert_eq!(serde_json::from_value::<Command>(wire).unwrap(), new);
  assert_eq!(new.name(), "xtgee");
  assert_eq!(new.requirements(), old.requirements());
  assert_eq!(
    serde_json::to_value(tabdat_domain::EstimationFamily::GaussianGee).unwrap(),
    json!("gaussian_gee")
  );
  assert_eq!(
    serde_json::to_value(tabdat_domain::EstimationFamily::PoissonGee).unwrap(),
    json!("poisson_gee")
  );
}

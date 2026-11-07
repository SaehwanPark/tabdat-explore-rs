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
  let new = Command::XtGeePoisson {
    command: XtGeeCommand {
      outcome: "y".into(),
      predictors: vec!["x".into()],
      entity_variable: "id".into(),
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

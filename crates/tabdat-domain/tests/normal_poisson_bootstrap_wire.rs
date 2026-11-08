use serde_json::json;
use tabdat_domain::{Command, CommandEffect};
#[test]
fn additive_normal_bootstrap_wire_and_readonly_effect_preserve_legacy_variants() {
  for kind in [
    "poisson_bootstrap",
    "conditional_poisson_bootstrap",
    "gamma_poisson_bootstrap",
    "normal_poisson_bootstrap",
  ] {
    let wire = json!({"kind":kind,"payload":{"command":{"outcome":"y","predictors":["x"],"cluster_variable":"g","replications":2,"seed":10001}}});
    let command: Command = serde_json::from_value(wire.clone()).unwrap();
    assert_eq!(serde_json::to_value(&command).unwrap(), wire);
    assert_eq!(command.name(), "bootstrap");
    assert_eq!(command.effects(), &[CommandEffect::Read]);
    if kind == "normal_poisson_bootstrap" {
      assert!(matches!(command, Command::NormalPoissonBootstrap { .. }));
    }
  }
}

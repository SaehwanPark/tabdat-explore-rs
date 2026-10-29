mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{
  fs,
  path::PathBuf,
  process::{Command, Output},
};
use tabdat_language::parse_command;
use tabdat_runtime::Session;

fn root() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
fn prepare(d: &TestDir) {
  fs::write(
    d.path().join("source.asc"),
    include_str!("fixtures/mma23_gamma_re_bootstrap/source.asc"),
  )
  .unwrap();
  let out = Command::new("sh")
    .current_dir(d.path())
    .arg(root().join("examples/mma23p1pannonlin/prepare.sh"))
    .args(["source.asc", "patents.csv"])
    .output()
    .unwrap();
  assert!(
    out.status.success(),
    "{}",
    String::from_utf8_lossy(&out.stderr)
  );
}
fn run(d: &TestDir, mode: &str) -> Output {
  tabdat_bin()
    .current_dir(d.path())
    .args([mode, "-f"])
    .arg(root().join("examples/mma23p1pannonlin/linear_panel.td"))
    .output()
    .unwrap()
}
fn events(out: &Output) -> Vec<Value> {
  String::from_utf8_lossy(&out.stdout)
    .lines()
    .map(|s| serde_json::from_str(s).unwrap())
    .collect()
}
fn close(a: f64, b: f64) {
  assert!(
    a.is_finite() && (a - b).abs() <= 1e-9 + 1e-10 * b.abs(),
    "{a} != {b}"
  );
}
fn check_results(ev: &[Value]) {
  let data = |kind: &str| {
    ev.iter()
      .filter(|e| e["result_type"] == kind)
      .map(|e| &e["data"])
      .collect::<Vec<_>>()
  };
  let pools = data("RegressionResult");
  let fes = data("FixedEffectsResult");
  let res = data("RandomEffectsResult");
  let ls = data("LincomResult");
  let ts = data("TestResult");
  assert_eq!(
    (pools.len(), fes.len(), res.len(), ls.len(), ts.len()),
    (1, 1, 1, 2, 2)
  );
  assert_eq!(pools[0]["covariance"], "cluster(id)");
  assert_eq!(fes[0]["covariance"], "nonrobust");
  assert_eq!(
    fes[0]["common_intercept_inference"],
    "normalization_point_only"
  );
  assert_eq!(
    res[0]["estimator"],
    "balanced_swamy_arora_random_effects_gls"
  );
  assert_eq!(res[0]["reference_distribution"], "normal");
  for model in [pools[0], fes[0], res[0]] {
    assert_eq!(model["sample"]["dropped_observations"], 0);
  }
  let mut n = 0;
  for line in include_str!("fixtures/mma23_linear_panel/reference.csv")
    .lines()
    .skip(1)
  {
    let mut row = line.split(',');
    let model = row.next().unwrap();
    let path = row.next().unwrap();
    let expected: f64 = row.next().unwrap().parse().unwrap();
    let value = match model {
      "pooled" => pools[0],
      "fe" => fes[0],
      "re" => res[0],
      "pooled_lincom" => ls[0],
      "fe_lincom" => ls[1],
      "pooled_test" => ts[0],
      "fe_test" => ts[1],
      _ => panic!("unknown model"),
    }
    .pointer(path)
    .unwrap()
    .as_f64()
    .unwrap();
    if path.ends_with("p_value") && expected > 0.0 {
      assert!(
        value.is_finite() && (value - expected).abs() <= 1e-9 * expected,
        "{model}{path}: {value} != {expected}"
      );
    } else {
      close(value, expected);
    }
    n += 1;
  }
  assert_eq!(n, 91);
}
fn check_predictions(text: &str, input: &str) {
  let mut rows = text.lines();
  assert_eq!(
    rows.next().unwrap(),
    "id,year,cusip,ardssic,scisect,logk,sumpat,logr,pat,lpat,pooled_xb,pooled_residual,fe_xb,fe_xbu,fe_u,fe_residual,re_xb,re_residual"
  );
  let mut source: Vec<Vec<&str>> = input
    .lines()
    .skip(1)
    .map(|s| s.split(',').collect())
    .collect();
  source.sort_by_key(|v| {
    (
      v[0].parse::<usize>().unwrap(),
      v[1].parse::<usize>().unwrap(),
    )
  });
  let refs = include_str!("fixtures/mma23_linear_panel/predictions.csv")
    .lines()
    .skip(1);
  let mut n = 0;
  for (line, (src, reference)) in rows.by_ref().zip(source.iter().zip(refs)) {
    let a: Vec<_> = line.split(',').collect();
    let b: Vec<_> = reference.split(',').collect();
    assert_eq!(a.len(), 18);
    assert_eq!(b.len(), 18);
    for i in 0..18 {
      if b[i].is_empty() {
        assert!(a[i].is_empty());
      } else {
        close(a[i].parse().unwrap(), b[i].parse().unwrap());
      }
      if i < 9 {
        if src[i].is_empty() {
          assert!(a[i].is_empty())
        } else {
          assert_eq!(a[i].parse::<f64>().unwrap(), src[i].parse::<f64>().unwrap());
        }
      }
    }
    n += 1;
  }
  assert_eq!(n, 200);
  assert!(rows.next().is_none());
}
#[test]
fn independent_panel_models_predictions_text_json_order_and_no_clobber() {
  let d = TestDir::new();
  prepare(&d);
  let input = fs::read_to_string(d.path().join("patents.csv")).unwrap();
  let out = run(&d, "--json");
  assert!(
    out.status.success(),
    "{}",
    String::from_utf8_lossy(&out.stderr)
  );
  check_results(&events(&out));
  let path = d.path().join("linear_panel_predictions.csv");
  let bytes = fs::read(&path).unwrap();
  check_predictions(std::str::from_utf8(&bytes).unwrap(), &input);
  assert_eq!(
    fs::read_to_string(d.path().join("patents.csv")).unwrap(),
    input
  );
  let out = run(&d, "--json");
  assert!(!out.status.success());
  assert!(String::from_utf8_lossy(&out.stderr).contains("export target already exists"));
  assert_eq!(fs::read(&path).unwrap(), bytes);
  fs::remove_file(&path).unwrap();
  let out = run(&d, "--text");
  assert!(
    out.status.success(),
    "{}",
    String::from_utf8_lossy(&out.stderr)
  );
  let text = String::from_utf8(out.stdout).unwrap();
  assert!(text.contains("fixed") || text.contains("Fixed"));
  assert_eq!(fs::read(&path).unwrap(), bytes);
  check_predictions(&fs::read_to_string(&path).unwrap(), &input);
  fs::remove_file(&path).unwrap();
  let mut lines = input.lines();
  let header = lines.next().unwrap();
  let mut data: Vec<_> = lines.collect();
  data.reverse();
  fs::write(
    d.path().join("patents.csv"),
    format!("{header}\n{}\n", data.join("\n")),
  )
  .unwrap();
  let out = run(&d, "--json");
  assert!(
    out.status.success(),
    "{}",
    String::from_utf8_lossy(&out.stderr)
  );
  check_results(&events(&out));
  check_predictions(&fs::read_to_string(&path).unwrap(), &input);
}
fn rewrite<F: FnMut(&mut Vec<String>)>(input: &str, mut f: F) -> String {
  let mut lines = input.lines();
  let mut out = lines.next().unwrap().to_owned() + "\n";
  for line in lines {
    let mut row: Vec<_> = line.split(',').map(str::to_owned).collect();
    f(&mut row);
    out.push_str(&row.join(","));
    out.push('\n');
  }
  out
}
fn exec(s: &mut Session, text: &str) -> tabdat_domain::ExecutionResult {
  s.execute(parse_command(text).unwrap()).unwrap()
}
#[test]
fn malformed_input_model_failure_and_file_state() {
  let d = TestDir::new();
  prepare(&d);
  let path = d.path().join("patents.csv");
  let input = fs::read_to_string(&path).unwrap();
  let export = d.path().join("linear_panel_predictions.csv");
  fs::write(&export, b"sentinel").unwrap();
  let mut dup: Vec<_> = input.lines().map(str::to_owned).collect();
  dup[2] = dup[1].clone();
  let cases = [
    dup.join("\n") + "\n",
    input.lines().take(6).collect::<Vec<_>>().join("\n") + "\n",
    input
      .lines()
      .enumerate()
      .map(|(i, line)| format!("{line},{}", if i == 0 { "lpat" } else { "999" }))
      .collect::<Vec<_>>()
      .join("\n")
      + "\n",
    input
      .lines()
      .enumerate()
      .map(|(i, line)| format!("{line},{}", if i == 0 { "LPAT" } else { "999" }))
      .collect::<Vec<_>>()
      .join("\n")
      + "\n",
    input
      .lines()
      .enumerate()
      .map(|(i, line)| format!("{line},{}", if i == 0 { "RE_XB" } else { "999" }))
      .collect::<Vec<_>>()
      .join("\n")
      + "\n",
    rewrite(&input, |r| r[7] = "1".into()),
    rewrite(&input, |r| {
      if r[0] == "1" && r[1] == "1975" {
        r[7] = "".into()
      }
    }),
    rewrite(&input, |r| {
      if r[0] == "1" && r[1] == "1975" {
        r[7] = "NaN".into()
      }
    }),
    rewrite(&input, |r| {
      if r[0] == "1" && r[1] == "1975" {
        r[7] = "21".into()
      }
    }),
    rewrite(&input, |r| {
      if r[0] == "1" {
        r[0] = "347".into()
      }
    }),
    rewrite(&input, |r| {
      if r[0] == "1" && r[1] == "1975" {
        r[8] = "-1".into()
      }
    }),
    rewrite(&input, |r| {
      if r[0] == "1" && r[1] == "1975" {
        r[8] = "1001".into()
      }
    }),
    rewrite(&input, |r| {
      if r[0] == "1" && r[1] == "1975" {
        r[8] = "1.5".into()
      }
    }),
    input.replacen(",1975,", ",1980,", 1),
  ];
  for case in cases {
    fs::write(&path, &case).unwrap();
    let out = run(&d, "--json");
    assert!(!out.status.success());
    assert!(!String::from_utf8_lossy(&out.stderr).contains("export target already exists"));
    assert!(
      !events(&out)
        .iter()
        .any(|e| e["result_type"] == "RegressionResult")
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), case);
    assert_eq!(fs::read(&export).unwrap(), b"sentinel");
  }
  // Pooled slope is identified but time-invariant x makes the within design singular.
  let invariant = rewrite(&input, |r| {
    r[7] = (r[0].parse::<f64>().unwrap() / 20.0).to_string()
  });
  fs::write(&path, &invariant).unwrap();
  let out = run(&d, "--json");
  assert!(!out.status.success());
  let ev = events(&out);
  assert!(ev.iter().any(|e| e["result_type"] == "RegressionResult"));
  assert!(!ev.iter().any(|e| e["result_type"] == "FixedEffectsResult"));
  assert_eq!(fs::read_to_string(&path).unwrap(), invariant);
  assert_eq!(fs::read(&export).unwrap(), b"sentinel");
  let mut s = Session::new();
  exec(&mut s, &format!("use \"{}\"", path.display()));
  exec(&mut s, "generate lpat = 0");
  exec(
    &mut s,
    "sql \"\"\"SELECT * EXCLUDE(lpat),LN(CASE WHEN pat=0 THEN 0.5 ELSE pat END) AS lpat FROM active\"\"\" into panel",
  );
  exec(&mut s, "regress lpat logr, cluster(id)");
  exec(&mut s, "predict pooled_xb, xb");
  let rows = exec(&mut s, "head 100");
  assert!(
    s.execute(parse_command("xtreg lpat logr, fe id(id) time(year)").unwrap())
      .is_err()
  );
  assert_eq!(exec(&mut s, "head 100"), rows);
  assert!(s.state().estimation.is_none());

  // Identical panels identify pooled and within fits, but the RE between design is singular.
  let between_singular = rewrite(&input, |r| {
    let t = r[1].parse::<usize>().unwrap() - 1975;
    r[7] = ((t as f64 - 2.0) * 0.2).to_string();
    r[8] = [1, 2, 2, 3, 4][t].to_string();
  });
  fs::write(&path, &between_singular).unwrap();
  let out = run(&d, "--json");
  assert!(!out.status.success());
  let ev = events(&out);
  assert!(ev.iter().any(|e| e["result_type"] == "RegressionResult"));
  assert!(ev.iter().any(|e| e["result_type"] == "FixedEffectsResult"));
  assert!(!ev.iter().any(|e| e["result_type"] == "RandomEffectsResult"));
  assert_eq!(fs::read_to_string(&path).unwrap(), between_singular);
  assert_eq!(fs::read(&export).unwrap(), b"sentinel");
  let mut s = Session::new();
  exec(&mut s, &format!("use \"{}\"", path.display()));
  exec(&mut s, "generate lpat = 0");
  exec(
    &mut s,
    "sql \"\"\"SELECT * EXCLUDE(lpat),LN(CASE WHEN pat=0 THEN 0.5 ELSE pat END) AS lpat FROM active\"\"\" into panel",
  );
  exec(&mut s, "regress lpat logr, cluster(id)");
  exec(&mut s, "predict pooled_xb, xb");
  exec(&mut s, "xtreg lpat logr, fe id(id) time(year)");
  exec(&mut s, "predict fe_xbu, xbu");
  let rows = exec(&mut s, "head 100");
  assert!(
    s.execute(parse_command("xtreg lpat logr, re id(id) time(year)").unwrap())
      .is_err()
  );
  assert_eq!(exec(&mut s, "head 100"), rows);
  assert!(s.state().estimation.is_none());
}
#[test]
fn successful_workflow_retains_re_fit_and_predictions() {
  let d = TestDir::new();
  prepare(&d);
  let mut s = Session::new();
  exec(
    &mut s,
    &format!("use \"{}\"", d.path().join("patents.csv").display()),
  );
  exec(&mut s, "generate lpat = 0");
  exec(
    &mut s,
    "sql \"\"\"SELECT * EXCLUDE(lpat),LN(CASE WHEN pat=0 THEN 0.5 ELSE pat END) AS lpat FROM active\"\"\" into panel",
  );
  exec(&mut s, "regress lpat logr, cluster(id)");
  exec(&mut s, "predict pooled_xb, xb");
  exec(&mut s, "xtreg lpat logr, fe id(id) time(year)");
  exec(&mut s, "predict fe_xbu, xbu");
  exec(&mut s, "xtreg lpat logr, re id(id) time(year)");
  let fit = s.state().estimation;
  exec(&mut s, "predict re_xb, xb");
  exec(&mut s, "predict re_after, xb");
  exec(&mut s, "summarize pooled_xb fe_xbu re_xb");
  exec(&mut s, "sort id year");
  assert_eq!(s.state().estimation, fit);
  exec(
    &mut s,
    "sql \"\"\"SELECT CASE WHEN MAX(ABS(re_xb-re_after))>1e-12 OR MAX(ABS(re_xb-fe_xbu))<1e-6 THEN error('wrong retained RE state') ELSE 1 END AS valid FROM active\"\"\"",
  );
}

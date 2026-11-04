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
fn example(name: &str) -> PathBuf {
  root().join("examples/mma21p1panel").join(name)
}
fn ok(o: &Output) {
  assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
}
fn references(d: &TestDir, missing: bool) {
  let full = [
    (
      "source.csv",
      include_str!("fixtures/mma21_within_figure_full/source.csv"),
    ),
    (
      "rows.csv",
      include_str!("fixtures/mma21_within_figure_full/rows.csv"),
    ),
    (
      "meta.csv",
      include_str!("fixtures/mma21_within_figure_full/meta.csv"),
    ),
  ];
  let incomplete = [
    (
      "source.csv",
      include_str!("fixtures/mma21_within_figure_missing/source.csv"),
    ),
    (
      "rows.csv",
      include_str!("fixtures/mma21_within_figure_missing/rows.csv"),
    ),
    (
      "meta.csv",
      include_str!("fixtures/mma21_within_figure_missing/meta.csv"),
    ),
  ];
  fs::create_dir_all(d.path().join("reference")).unwrap();
  for (name, text) in if missing { incomplete } else { full } {
    fs::write(d.path().join("reference").join(name), text).unwrap();
  }
}
fn prepare(d: &TestDir) {
  fs::write(
    d.path().join("source.dat"),
    include_str!("fixtures/mma21_between_figure/source.dat"),
  )
  .unwrap();
  let o = Command::new("sh")
    .current_dir(d.path())
    .arg(example("prepare.sh"))
    .args(["source.dat", "MOM.csv"])
    .output()
    .unwrap();
  ok(&o);
  references(d, false);
}
fn run(d: &TestDir, mode: &str) -> Output {
  tabdat_bin()
    .current_dir(d.path())
    .args([mode, "-f"])
    .arg(example("within_figure.td"))
    .output()
    .unwrap()
}
fn audit(d: &TestDir) -> Output {
  Command::new("python3")
    .arg(example("check_within_figure.py"))
    .arg(d.path())
    .arg(d.path().join("reference"))
    .arg(d.path().join("out.jsonl"))
    .output()
    .unwrap()
}
fn svg_audit(d: &TestDir) -> Output {
  Command::new("python3").args(["-c","import sys;sys.path.insert(0,sys.argv[1]);from check_within_figure import check_svg,rows;check_svg(sys.argv[2],rows(sys.argv[3]))"]).arg(example("")).arg(d.path().join("within_figure.svg")).arg(d.path().join("within_figure.csv")).output().unwrap()
}
fn events(o: &Output) -> Vec<Value> {
  String::from_utf8_lossy(&o.stdout)
    .lines()
    .map(|s| serde_json::from_str(s).unwrap())
    .collect()
}
fn clear(d: &TestDir) {
  for f in ["within_figure.csv", "within_figure.svg"] {
    fs::remove_file(d.path().join(f)).unwrap();
  }
}
fn rewrite(input: &str, mut f: impl FnMut(usize, &mut Vec<String>)) -> String {
  let mut lines = input.lines();
  let header = lines.next().unwrap();
  let records = lines
    .enumerate()
    .map(|(i, l)| {
      let mut fields = l.split(',').map(str::to_owned).collect();
      f(i, &mut fields);
      fields.join(",")
    })
    .collect::<Vec<_>>();
  format!("{header}\n{}\n", records.join("\n"))
}
#[test]
fn independent_cluster_ols_lowess_svg_modes_order_and_no_clobber() {
  let d = TestDir::new();
  prepare(&d);
  let p = d.path().join("MOM.csv");
  let input = fs::read_to_string(&p).unwrap();
  let o = run(&d, "--json");
  ok(&o);
  fs::write(d.path().join("out.jsonl"), &o.stdout).unwrap();
  ok(&audit(&d));
  let csv = fs::read(d.path().join("within_figure.csv")).unwrap();
  let svg = fs::read(d.path().join("within_figure.svg")).unwrap();
  assert_eq!(fs::read_to_string(&p).unwrap(), input);
  let o = run(&d, "--json");
  assert!(!o.status.success());
  assert!(String::from_utf8_lossy(&o.stderr).contains("export target already exists"));
  assert_eq!(fs::read(d.path().join("within_figure.csv")).unwrap(), csv);
  assert_eq!(fs::read(d.path().join("within_figure.svg")).unwrap(), svg);
  clear(&d);
  ok(&run(&d, "--text"));
  assert_eq!(fs::read(d.path().join("within_figure.csv")).unwrap(), csv);
  assert_eq!(fs::read(d.path().join("within_figure.svg")).unwrap(), svg);
  clear(&d);
  let mut lines = input.lines();
  let header = lines.next().unwrap();
  let mut records: Vec<_> = lines.collect();
  records.reverse();
  fs::write(&p, format!("{header}\n{}\n", records.join("\n"))).unwrap();
  let o = run(&d, "--json");
  ok(&o);
  fs::write(d.path().join("out.jsonl"), &o.stdout).unwrap();
  ok(&audit(&d));
  assert_eq!(fs::read(d.path().join("within_figure.csv")).unwrap(), csv);
  assert_eq!(fs::read(d.path().join("within_figure.svg")).unwrap(), svg);
  clear(&d);
  fs::write(d.path().join("within_figure.svg"), b"sentinel").unwrap();
  let o = run(&d, "--json");
  assert!(!o.status.success());
  assert_eq!(
    fs::read(d.path().join("within_figure.svg")).unwrap(),
    b"sentinel"
  );
  assert_eq!(fs::read(d.path().join("within_figure.csv")).unwrap(), csv);
}
#[test]
fn joint_finite_model_cohort_matches_independent_r_with_missing_nonmodel_values() {
  let d = TestDir::new();
  prepare(&d);
  let p = d.path().join("MOM.csv");
  let input = fs::read_to_string(&p).unwrap();
  let changed = rewrite(&input, |i, r| match i {
    0 => r[1] = "".into(),
    5 => r[2] = "".into(),
    8 => r[3] = "".into(),
    92.. => r[2] = "".into(),
    _ => {}
  });
  fs::write(&p, &changed).unwrap();
  references(&d, true);
  let o = run(&d, "--json");
  ok(&o);
  fs::write(d.path().join("out.jsonl"), &o.stdout).unwrap();
  ok(&audit(&d));
  assert_eq!(fs::read_to_string(p).unwrap(), changed);
  let records = fs::read_to_string(d.path().join("within_figure.csv")).unwrap();
  assert_eq!(records.lines().count() - 1, 90);
}
#[test]
fn invalid_keys_collision_and_singular_design_fail_without_files_or_source_changes() {
  let d = TestDir::new();
  prepare(&d);
  let p = d.path().join("MOM.csv");
  let input = fs::read_to_string(&p).unwrap();
  let mut lines = input.lines();
  let header = lines.next().unwrap();
  let rows: Vec<_> = lines.collect();
  let svg = d.path().join("within_figure.svg");
  fs::write(&svg, b"sentinel").unwrap();
  let cases = [
    format!("{header}\n{}\n{}\n", rows.join("\n"), rows[0]),
    rewrite(&input, |i, r| {
      if i == 0 {
        r[7] = "1.5".into()
      }
    }),
    rewrite(&input, |i, r| {
      if i == 0 {
        r[2] = "1.001".into()
      }
    }),
    rewrite(&input, |i, r| {
      if i == 0 {
        r[2] = "1000000.01".into()
      }
    }),
    format!(
      "{header},FIGURE_LOWESS\n{}\n",
      rows
        .iter()
        .map(|r| format!("{r},999"))
        .collect::<Vec<_>>()
        .join("\n")
    ),
    rewrite(&input, |_, r| r[2] = "1.00".into()),
  ];
  for case in cases {
    fs::write(&p, &case).unwrap();
    let o = run(&d, "--json");
    assert!(!o.status.success());
    assert!(
      !events(&o).iter().any(|e|
        e["result_type"] == "RegressionResult" || e["result_type"] == "FixedEffectsResult")
    );
    assert_eq!(fs::read_to_string(&p).unwrap(), case);
    assert!(!d.path().join("within_figure.csv").exists());
    assert_eq!(fs::read(&svg).unwrap(), b"sentinel");
  }
}
#[test]
fn checker_rejects_shared_wrong_values_and_svg_forgeries() {
  let d = TestDir::new();
  prepare(&d);
  let o = run(&d, "--json");
  ok(&o);
  fs::write(d.path().join("out.jsonl"), &o.stdout).unwrap();
  ok(&audit(&d));
  let p = d.path().join("within_figure.svg");
  let svg = fs::read_to_string(&p).unwrap();
  let start = svg.find("class=\"line-series\"").unwrap();
  let pos = start + svg[start..].find(" d=\"").unwrap() + 4;
  let first_l = pos + svg[pos..].find('L').unwrap();
  let mut bad = svg.clone();
  bad.replace_range(first_l..first_l + 1, "M");
  fs::write(&p, bad).unwrap();
  assert!(!svg_audit(&d).status.success());
  fs::write(&p, &svg).unwrap();
  let start = svg.find("class=\"scatter-point\"").unwrap();
  let pos = start + svg[start..].find(" cx=\"").unwrap() + 5;
  let end = pos + svg[pos..].find('"').unwrap();
  let mut bad = svg.clone();
  bad.replace_range(pos..end, "0");
  fs::write(&p, bad).unwrap();
  assert!(!svg_audit(&d).status.success());
  fs::write(&p, svg).unwrap();
  for f in ["within_figure.csv", "reference/rows.csv"] {
    let p = d.path().join(f);
    let text = fs::read_to_string(&p).unwrap();
    let mut lines = text.lines();
    let header = lines.next().unwrap();
    let data = lines
      .enumerate()
      .map(|(i, l)| {
        if i == 0 {
          let mut v: Vec<_> = l.split(',').collect();
          v[16] = "999";
          v.join(",")
        } else {
          l.to_owned()
        }
      })
      .collect::<Vec<_>>()
      .join("\n");
    fs::write(p, format!("{header}\n{data}\n")).unwrap();
  }
  let o = audit(&d);
  assert!(!o.status.success());
  assert!(String::from_utf8_lossy(&o.stderr).contains("equation"));
}
#[test]
fn transformation_replaces_fe_with_auxiliary_ols_and_smoothing_preserves_it() {
  fn exec(s: &mut Session, command: &str) {
    s.execute(parse_command(command).unwrap()).unwrap();
  }
  let d = TestDir::new();
  prepare(&d);
  let mut s = Session::new();
  exec(
    &mut s,
    &format!("use \"{}\"", d.path().join("MOM.csv").display()),
  );
  let mut fe = None;
  let mut ols = None;
  for line in include_str!("../examples/mma21p1panel/within_figure.td").lines() {
    if line.starts_with('#')
      || line.starts_with("use ")
      || line.starts_with("export ")
      || line.starts_with("scatter ")
      || line.trim().is_empty()
    {
      continue;
    }
    exec(&mut s, line);
    if line.starts_with("xtreg ") {
      fe = s.state().estimation.clone();
      assert!(fe.is_some());
    }
    if line.contains("into within_figure_data") {
      assert_eq!(s.state().estimation, fe);
    }
    if line.starts_with("regress ") {
      ols = s.state().estimation.clone();
      assert!(ols.is_some());
      assert_ne!(ols, fe);
    }
  }
  assert_eq!(s.state().estimation, ols);
  exec(
    &mut s,
    &format!("export \"{}\"", d.path().join("state.csv").display()),
  );
  exec(
    &mut s,
    &format!(
      "scatter within_lnhr within_lnwg, lines(figure_lowess figure_linear) saving(\"{}\") noopen",
      d.path().join("state.svg").display()
    ),
  );
  assert_eq!(s.state().estimation, ols);
  exec(&mut s, "predict after_lowess, xb");
  assert_eq!(s.state().estimation, ols);
}

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
  let o = Command::new("sh")
    .current_dir(d.path())
    .arg(root().join("examples/mma23p1pannonlin/prepare.sh"))
    .args(["source.asc", "patents.csv"])
    .output()
    .unwrap();
  assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
  fs::create_dir(d.path().join("reference")).unwrap();
  fs::write(
    d.path().join("reference/rows.csv"),
    include_str!("fixtures/mma23_figure/rows.csv"),
  )
  .unwrap();
  fs::write(
    d.path().join("reference/meta.csv"),
    include_str!("fixtures/mma23_figure/meta.csv"),
  )
  .unwrap();
}
fn run(d: &TestDir, mode: &str) -> Output {
  tabdat_bin()
    .current_dir(d.path())
    .args([mode, "-f"])
    .arg(root().join("examples/mma23p1pannonlin/figure.td"))
    .output()
    .unwrap()
}
fn events(o: &Output) -> Vec<Value> {
  String::from_utf8_lossy(&o.stdout)
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect()
}
fn audit(d: &TestDir) -> Output {
  Command::new("python3")
    .arg(root().join("examples/mma23p1pannonlin/check_figure.py"))
    .arg(d.path())
    .arg(d.path().join("reference"))
    .arg(d.path().join("out.jsonl"))
    .output()
    .unwrap()
}
fn svg_audit(d: &TestDir) -> Output {
  Command::new("python3").args(["-c", "import sys;sys.path.insert(0,sys.argv[1]);from check_figure import check_svg,rows;check_svg(sys.argv[2],rows(sys.argv[3]))"])
    .arg(root().join("examples/mma23p1pannonlin")).arg(d.path().join("figure.svg")).arg(d.path().join("figure_data.csv")).output().unwrap()
}
fn ok(o: &Output) {
  assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
}
fn clear(d: &TestDir) {
  for name in ["figure.svg", "figure_data.csv"] {
    fs::remove_file(d.path().join(name)).unwrap();
  }
}

#[test]
fn source_figure_reference_text_json_order_and_no_clobber() {
  let d = TestDir::new();
  prepare(&d);
  let input = fs::read_to_string(d.path().join("patents.csv")).unwrap();
  let o = run(&d, "--json");
  ok(&o);
  fs::write(d.path().join("out.jsonl"), &o.stdout).unwrap();
  ok(&audit(&d));
  let table = fs::read(d.path().join("figure_data.csv")).unwrap();
  let svg = fs::read(d.path().join("figure.svg")).unwrap();
  assert_eq!(
    fs::read_to_string(d.path().join("patents.csv")).unwrap(),
    input
  );
  let o = run(&d, "--json");
  assert!(!o.status.success());
  assert!(String::from_utf8_lossy(&o.stderr).contains("export target already exists"));
  assert_eq!(fs::read(d.path().join("figure_data.csv")).unwrap(), table);
  assert_eq!(fs::read(d.path().join("figure.svg")).unwrap(), svg);
  clear(&d);
  let o = run(&d, "--text");
  ok(&o);
  assert_eq!(fs::read(d.path().join("figure_data.csv")).unwrap(), table);
  assert_eq!(fs::read(d.path().join("figure.svg")).unwrap(), svg);
  clear(&d);
  let mut lines = input.lines();
  let header = lines.next().unwrap();
  let mut records: Vec<_> = lines.collect();
  records.reverse();
  fs::write(
    d.path().join("patents.csv"),
    format!("{header}\n{}\n", records.join("\n")),
  )
  .unwrap();
  let o = run(&d, "--json");
  ok(&o);
  assert_eq!(fs::read(d.path().join("figure_data.csv")).unwrap(), table);
  assert_eq!(fs::read(d.path().join("figure.svg")).unwrap(), svg);
  fs::write(d.path().join("out.jsonl"), &o.stdout).unwrap();
  ok(&audit(&d));
  clear(&d);
  fs::write(d.path().join("figure.svg"), b"sentinel").unwrap();
  let o = run(&d, "--json");
  assert!(!o.status.success());
  assert_eq!(fs::read(d.path().join("figure.svg")).unwrap(), b"sentinel");
  assert_eq!(fs::read(d.path().join("figure_data.csv")).unwrap(), table);
}

#[test]
fn invalid_data_and_case_insensitive_output_collision_fail_before_fit() {
  let d = TestDir::new();
  prepare(&d);
  let path = d.path().join("patents.csv");
  let original = fs::read_to_string(&path).unwrap();
  let mut lines = original.lines();
  let header = lines.next().unwrap();
  let records: Vec<_> = lines.collect();
  let cases = [
    format!("{header}\n{}\n{}\n", records.join("\n"), records[0]),
    format!(
      "{header},FIGURE_LOWESS\n{}\n",
      records
        .iter()
        .map(|r| format!("{r},999"))
        .collect::<Vec<_>>()
        .join("\n")
    ),
    format!(
      "{header}\n{}\n",
      records
        .iter()
        .enumerate()
        .map(|(i, r)| if i == 0 {
          let mut v: Vec<_> = r.split(',').collect();
          v[8] = "-1";
          v.join(",")
        } else {
          r.to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
    ),
    format!(
      "{header}\n{}\n",
      records
        .iter()
        .enumerate()
        .map(|(i, r)| if i == 0 {
          let mut v: Vec<_> = r.split(',').collect();
          v[7] = "";
          v.join(",")
        } else {
          r.to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
    ),
  ];
  for input in cases {
    fs::write(&path, &input).unwrap();
    let o = run(&d, "--json");
    assert!(!o.status.success());
    assert!(
      !events(&o)
        .iter()
        .any(|e| e["result_type"] == "RegressionResult")
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), input);
    assert!(!d.path().join("figure_data.csv").exists());
    assert!(!d.path().join("figure.svg").exists());
  }
}

#[test]
fn svg_geometry_and_shared_wrong_numeric_forgeries_are_rejected() {
  let d = TestDir::new();
  prepare(&d);
  let o = run(&d, "--json");
  ok(&o);
  fs::write(d.path().join("out.jsonl"), &o.stdout).unwrap();
  ok(&audit(&d));
  let path = d.path().join("figure.svg");
  let svg = fs::read_to_string(&path).unwrap();
  let line_start = svg.find("class=\"line-series\"").unwrap();
  let d_pos = line_start + svg[line_start..].find(" d=\"").unwrap() + 4;
  let first_l = d_pos + svg[d_pos..].find('L').unwrap();
  let mut bad = svg.clone();
  bad.replace_range(first_l..first_l + 1, "M");
  fs::write(&path, &bad).unwrap();
  assert!(!svg_audit(&d).status.success());
  fs::write(&path, &svg).unwrap();
  let marker = svg.find("class=\"scatter-point\"").unwrap();
  let cx = marker + svg[marker..].find(" cx=\"").unwrap() + 5;
  let end = cx + svg[cx..].find('"').unwrap();
  let mut bad = svg.clone();
  bad.replace_range(cx..end, "0");
  fs::write(&path, bad).unwrap();
  assert!(!svg_audit(&d).status.success());
  fs::write(&path, svg).unwrap();
  for file in ["figure_data.csv", "reference/rows.csv"] {
    let p = d.path().join(file);
    let text = fs::read_to_string(&p).unwrap();
    let mut lines = text.lines();
    let header = lines.next().unwrap();
    let forged = lines
      .enumerate()
      .map(|(i, l)| {
        if i == 0 {
          let mut fields: Vec<_> = l.split(',').collect();
          fields[11] = "999";
          fields.join(",")
        } else {
          l.to_string()
        }
      })
      .collect::<Vec<_>>()
      .join("\n");
    fs::write(p, format!("{header}\n{forged}\n")).unwrap();
  }
  let result = audit(&d);
  assert!(!result.status.success());
  assert!(String::from_utf8_lossy(&result.stderr).contains("equation"));
}

#[test]
fn lowess_and_plot_preserve_pooled_estimation_state() {
  fn exec(s: &mut Session, cmd: &str) {
    s.execute(parse_command(cmd).unwrap()).unwrap();
  }
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
  exec(&mut s, "sort id year");
  exec(&mut s, "regress lpat logr, cluster(id)");
  let fit = s.state().estimation.clone();
  exec(&mut s, "predict figure_linear, xb");
  exec(&mut s, "lowess lpat logr, gen(figure_lowess) bandwidth=0.8");
  exec(&mut s, "summarize figure_linear figure_lowess");
  exec(
    &mut s,
    &format!(
      "scatter lpat logr, lines(figure_lowess figure_linear) saving(\"{}\") noopen",
      d.path().join("state.svg").display()
    ),
  );
  assert_eq!(s.state().estimation, fit);
  exec(&mut s, "predict after_lowess, xb");
  assert_eq!(s.state().estimation, fit);
}

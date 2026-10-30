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
const INPUT: &str =
  include_str!("../crates/tabdat-stats/tests/fixtures/panel_fd_bootstrap/inputs.csv");
fn root() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
fn example(name: &str) -> PathBuf {
  root().join("examples/mma21p1panel").join(name)
}
fn ok(o: &Output) {
  assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
}
fn prepare(d: &TestDir) {
  fs::write(d.path().join("MOM.csv"), INPUT).unwrap();
  fs::create_dir(d.path().join("reference")).unwrap();
  for (name, text) in [
    (
      "source.csv",
      include_str!("fixtures/mma21_fd_figure/source.csv"),
    ),
    (
      "rows.csv",
      include_str!("fixtures/mma21_fd_figure/rows.csv"),
    ),
    (
      "meta.csv",
      include_str!("fixtures/mma21_fd_figure/meta.csv"),
    ),
  ] {
    fs::write(d.path().join("reference").join(name), text).unwrap();
  }
}
fn run(d: &TestDir, mode: &str) -> Output {
  tabdat_bin()
    .current_dir(d.path())
    .args([mode, "-f"])
    .arg(example("fd_figure.td"))
    .output()
    .unwrap()
}
fn audit(d: &TestDir) -> Output {
  Command::new("python3")
    .arg(example("check_fd_figure.py"))
    .arg(d.path())
    .arg(d.path().join("reference"))
    .arg(d.path().join("out.jsonl"))
    .output()
    .unwrap()
}
fn svg_audit(d: &TestDir) -> Output {
  Command::new("python3").args(["-c","import sys;sys.path.insert(0,sys.argv[1]);from check_fd_figure import check_svg,rows;check_svg(sys.argv[2],rows(sys.argv[3]))"]).arg(example("")).arg(d.path().join("fd_figure.svg")).arg(d.path().join("fd_figure.csv")).output().unwrap()
}
fn events(o: &Output) -> Vec<Value> {
  String::from_utf8_lossy(&o.stdout)
    .lines()
    .map(|s| serde_json::from_str(s).unwrap())
    .collect()
}
fn clear(d: &TestDir) {
  for f in ["fd_figure.csv", "fd_figure.svg"] {
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
fn independent_raw_pairs_cluster_ols_lowess_svg_order_and_no_clobber() {
  let d = TestDir::new();
  prepare(&d);
  let p = d.path().join("MOM.csv");
  let o = run(&d, "--json");
  ok(&o);
  fs::write(d.path().join("out.jsonl"), &o.stdout).unwrap();
  ok(&audit(&d));
  let csv = fs::read(d.path().join("fd_figure.csv")).unwrap();
  let svg = fs::read(d.path().join("fd_figure.svg")).unwrap();
  let text = std::str::from_utf8(&csv).unwrap();
  let records: Vec<Vec<&str>> = text
    .lines()
    .skip(1)
    .map(|l| l.split(',').collect())
    .collect();
  assert_eq!(records.len(), 14);
  assert!(!records.iter().any(|r| r[1] == "7"));
  assert!(!records.iter().any(|r| r[1] == "1" && r[2] == "1981"));
  assert!(!records.iter().any(|r| r[1] == "2" && r[2] == "1981"));
  assert_eq!(fs::read_to_string(&p).unwrap(), INPUT);
  let o = run(&d, "--json");
  assert!(!o.status.success());
  assert!(String::from_utf8_lossy(&o.stderr).contains("export target already exists"));
  assert_eq!(fs::read(d.path().join("fd_figure.csv")).unwrap(), csv);
  assert_eq!(fs::read(d.path().join("fd_figure.svg")).unwrap(), svg);
  clear(&d);
  ok(&run(&d, "--text"));
  assert_eq!(fs::read(d.path().join("fd_figure.csv")).unwrap(), csv);
  assert_eq!(fs::read(d.path().join("fd_figure.svg")).unwrap(), svg);
  clear(&d);
  let mut lines = INPUT.lines();
  let header = lines.next().unwrap();
  let mut rows: Vec<_> = lines.collect();
  rows.reverse();
  fs::write(&p, format!("{header}\n{}\n", rows.join("\n"))).unwrap();
  let o = run(&d, "--json");
  ok(&o);
  fs::write(d.path().join("out.jsonl"), &o.stdout).unwrap();
  ok(&audit(&d));
  assert_eq!(fs::read(d.path().join("fd_figure.csv")).unwrap(), csv);
  assert_eq!(fs::read(d.path().join("fd_figure.svg")).unwrap(), svg);
  clear(&d);
  fs::write(d.path().join("fd_figure.svg"), b"sentinel").unwrap();
  let o = run(&d, "--json");
  assert!(!o.status.success());
  assert_eq!(
    fs::read(d.path().join("fd_figure.svg")).unwrap(),
    b"sentinel"
  );
  assert_eq!(fs::read(d.path().join("fd_figure.csv")).unwrap(), csv);
}
#[test]
fn invalid_duplicate_keys_collision_and_singular_design_preserve_source_and_files() {
  let d = TestDir::new();
  prepare(&d);
  let p = d.path().join("MOM.csv");
  let mut lines = INPUT.lines();
  let header = lines.next().unwrap();
  let rows: Vec<_> = lines.collect();
  let svg = d.path().join("fd_figure.svg");
  fs::write(&svg, b"sentinel").unwrap();
  let cases = [
    format!("{header}\n{}\n{}\n", rows.join("\n"), rows[0]),
    rewrite(INPUT, |i, r| {
      if i == 0 {
        r[7] = "1.5".into()
      }
    }),
    rewrite(INPUT, |i, r| {
      if i == 0 {
        r[0] = "1.5".into()
      }
    }),
    format!(
      "{header},DLNWG\n{}\n",
      rows
        .iter()
        .map(|r| format!("{r},999"))
        .collect::<Vec<_>>()
        .join("\n")
    ),
    rewrite(INPUT, |_, r| r[2] = "1.00".into()),
  ];
  for case in cases {
    fs::write(&p, &case).unwrap();
    let o = run(&d, "--json");
    assert!(!o.status.success());
    assert!(
      !events(&o)
        .iter()
        .any(|e| e["result_type"] == "RegressionResult")
    );
    assert_eq!(fs::read_to_string(&p).unwrap(), case);
    assert!(!d.path().join("fd_figure.csv").exists());
    assert_eq!(fs::read(&svg).unwrap(), b"sentinel");
  }
}
#[test]
fn checker_rejects_shared_wrong_fit_and_svg_forgeries() {
  let d = TestDir::new();
  prepare(&d);
  let o = run(&d, "--json");
  ok(&o);
  fs::write(d.path().join("out.jsonl"), &o.stdout).unwrap();
  ok(&audit(&d));
  let p = d.path().join("fd_figure.svg");
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
  for f in ["fd_figure.csv", "reference/rows.csv"] {
    let p = d.path().join(f);
    let text = fs::read_to_string(&p).unwrap();
    let mut lines = text.lines();
    let header = lines.next().unwrap();
    let data = lines
      .enumerate()
      .map(|(i, l)| {
        if i == 0 {
          let mut v: Vec<_> = l.split(',').collect();
          v[10] = "999";
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
fn smoother_export_labels_and_plot_preserve_fd_ols_state() {
  fn exec(s: &mut Session, c: &str) {
    s.execute(parse_command(c).unwrap()).unwrap();
  }
  let d = TestDir::new();
  prepare(&d);
  let mut s = Session::new();
  exec(
    &mut s,
    &format!("use \"{}\"", d.path().join("MOM.csv").display()),
  );
  exec(
    &mut s,
    "sql \"\"\"WITH adjacent AS (SELECT source_row,id,year,lnhr,lnwg,LAG(source_row) OVER w AS previous_source_row,LAG(year) OVER w AS previous_year,lnhr-LAG(lnhr) OVER w AS dlnhr,lnwg-LAG(lnwg) OVER w AS dlnwg FROM active WINDOW w AS (PARTITION BY id ORDER BY year)) SELECT * FROM adjacent WHERE year=previous_year+1 AND isfinite(dlnhr) AND isfinite(dlnwg) ORDER BY id,year\"\"\" into fd_figure_data",
  );
  exec(&mut s, "regress dlnhr dlnwg, cluster(id)");
  let fit = s.state().estimation.clone();
  exec(&mut s, "predict figure_linear, xb");
  exec(
    &mut s,
    "lowess dlnhr dlnwg, gen(figure_lowess) bandwidth=0.8",
  );
  exec(&mut s, "summarize figure_linear figure_lowess");
  exec(
    &mut s,
    &format!("export \"{}\"", d.path().join("state.csv").display()),
  );
  exec(&mut s, "label variable dlnwg \"Change in log hourly wage\"");
  exec(
    &mut s,
    "label variable dlnhr \"Change in log annual hours\"",
  );
  exec(&mut s, "label variable figure_lowess \"Nonparametric fit\"");
  exec(
    &mut s,
    "label variable figure_linear \"First-difference linear fit\"",
  );
  exec(
    &mut s,
    &format!(
      "scatter dlnhr dlnwg, lines(figure_lowess figure_linear) saving(\"{}\") noopen",
      d.path().join("state.svg").display()
    ),
  );
  assert_eq!(s.state().estimation, fit);
  exec(&mut s, "predict after_lowess, xb");
  assert_eq!(s.state().estimation, fit);
}
